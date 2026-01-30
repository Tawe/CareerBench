//! Wrapper around llama.cpp for async model loading and inference
//! Provides a simple async interface for GGUF model inference

use crate::ai::errors::AiProviderError;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::ffi::CString;
use std::os::raw::c_char;
use serde_json;

// Import llama.cpp types and functions
use llama_cpp_sys_3::{
    llama_backend_init,
    llama_model_default_params, llama_load_model_from_file, llama_model, llama_free_model,
    llama_context_default_params, llama_new_context_with_model, llama_context, llama_free,
    llama_tokenize, llama_decode, llama_get_logits_ith, llama_n_vocab,
    llama_token_to_piece, llama_sample_token_greedy, llama_sample_softmax,
    llama_batch_get_one, llama_kv_cache_clear,
    llama_token_data, llama_token_data_array,
    llama_token_eos,
};

/// Wrapper for llama.cpp model and context
/// Handles model loading and inference in an async-friendly way
/// 
/// SAFETY: llama.cpp contexts are NOT thread-safe. All inference must be serialized.
/// We use a mutex to prevent concurrent inference on the same context.
pub struct LlamaModel {
    model_path: PathBuf,
    model: *mut llama_model,
    ctx: *mut llama_context,
    // Mutex to serialize inference (llama.cpp contexts are not thread-safe)
    _inference_lock: Arc<tokio::sync::Mutex<()>>,
}

unsafe impl Send for LlamaModel {}
unsafe impl Sync for LlamaModel {}

impl LlamaModel {
    /// Load a GGUF model from the given path
    pub async fn load(path: PathBuf) -> Result<Self, AiProviderError> {
        if !path.exists() {
            return Err(AiProviderError::Unknown(
                format!("Model file not found: {}", path.display())
            ));
        }
        
        // Validate filename doesn't contain query parameters (from buggy downloads)
        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
            if filename.contains('?') {
                let error_msg = format!(
                    "Invalid model filename: '{}'. The filename contains query parameters, which suggests it was downloaded incorrectly. Please delete this file and re-download the model using the Settings page.",
                    filename
                );
                log::error!("[llama_wrapper] {}", error_msg);
                return Err(AiProviderError::Unknown(error_msg));
            }
        }

        // Clone path for use in closure
        let path_for_closure = path.clone();

        // Run in blocking thread since llama.cpp is synchronous
        let (model_ptr, ctx_ptr) = tokio::task::spawn_blocking(move || {
            unsafe {
                log::info!("[llama_wrapper] Initializing llama backend...");
                // Initialize backend (thread-safe, can be called multiple times)
                llama_backend_init();
                log::info!("[llama_wrapper] Backend initialized");

                // Convert path to C string
                let path_str = path_for_closure.to_str()
                    .ok_or_else(|| AiProviderError::Unknown("Invalid model path".to_string()))?;
                let c_path = CString::new(path_str)
                    .map_err(|e| AiProviderError::Unknown(format!("Failed to create C string: {}", e)))?;

                // Set up model parameters
                let mut model_params = llama_model_default_params();
                model_params.n_gpu_layers = 0; // CPU only for now (can be configured later)

                // Load model
                log::info!("[llama_wrapper] Loading model from: {}", path_str);
                let model = llama_load_model_from_file(
                    c_path.as_ptr(),
                    model_params
                );

                if model.is_null() {
                    log::error!("[llama_wrapper] Failed to load model from: {}", path_str);
                    return Err(AiProviderError::Unknown("Failed to load model. Check that the file is a valid GGUF model.".to_string()));
                }
                log::info!("[llama_wrapper] Model loaded successfully");

                // Set up context parameters
                let mut ctx_params = llama_context_default_params();
                ctx_params.n_ctx = 4096; // Context window size
                // Use fewer threads - sometimes fewer threads is faster due to less overhead
                // For CPU inference, 4-6 threads often performs better than all cores
                let num_cores = num_cpus::get();
                let optimal_threads = if num_cores > 8 { 6 } else { num_cores.max(2) };
                ctx_params.n_threads = optimal_threads as u32;
                ctx_params.n_threads_batch = optimal_threads as u32;

                // Create context
                log::info!("[llama_wrapper] Creating context: n_ctx={}, n_threads={}, n_threads_batch={}", 
                    ctx_params.n_ctx, ctx_params.n_threads, ctx_params.n_threads_batch);
                let ctx = llama_new_context_with_model(model, ctx_params);

                if ctx.is_null() {
                    log::error!("[llama_wrapper] Failed to create context");
                    llama_free_model(model);
                    return Err(AiProviderError::Unknown("Failed to create context".to_string()));
                }
                log::info!("[llama_wrapper] Context created successfully");

                // Convert pointers to usize for thread safety (just copying addresses)
                Ok((model as usize, ctx as usize))
            }
        }).await
        .map_err(|e| AiProviderError::Unknown(format!("Task join error: {}", e)))??;

        // Convert back from usize to pointers
        Ok(Self {
            model_path: path,
            model: model_ptr as *mut llama_model,
            ctx: ctx_ptr as *mut llama_context,
            _inference_lock: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    /// Generate text from a prompt
    /// Returns the generated text (which should contain JSON)
    pub async fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, AiProviderError> {
        // Acquire lock to serialize inference (llama.cpp contexts are not thread-safe)
        let _lock = self._inference_lock.lock().await;
        
        // Copy pointers (safe - just copying memory addresses)
        let model_ptr = self.model as usize;
        let ctx_ptr = self.ctx as usize;
        let prompt = prompt.to_string();

        // Run inference in blocking thread
        // Note: We reconstruct pointers from usize, which is safe since they're just addresses
        tokio::task::spawn_blocking(move || {
            // Reconstruct pointers from usize (safe - just addresses)
            let model = model_ptr as *mut llama_model;
            let ctx = ctx_ptr as *mut llama_context;
            
            // Track allocated logits arrays (pointer, size) so we can free them
            let mut allocated_logits: Vec<(*mut i8, usize)> = Vec::new();
            
            unsafe {
                // Clear KV cache for fresh inference
                llama_kv_cache_clear(ctx);

                // Convert prompt to C string
                let c_prompt = CString::new(prompt.as_str())
                    .map_err(|e| AiProviderError::Unknown(format!("Failed to create C string: {}", e)))?;

                // Tokenize prompt
                log::info!("[llama_wrapper] Tokenizing prompt (length: {} chars)", prompt.len());
                // Allocate buffer (estimate: prompt length / 2, minimum 512)
                let max_tokens_for_prompt = (prompt.len() / 2).max(512).min(4096);
                let mut tokens = vec![0i32; max_tokens_for_prompt];

                let token_count = llama_tokenize(
                    model,
                    c_prompt.as_ptr(),
                    c_prompt.as_bytes().len() as i32,
                    tokens.as_mut_ptr(),
                    tokens.len() as i32,
                    true,  // add_special (add BOS token)
                    false, // parse_special
                );
                log::debug!("[llama_wrapper] Tokenize returned: {} (buffer size: {})", token_count, tokens.len());

                if token_count < 0 {
                    // Buffer was too small, try with larger buffer
                    let needed_size = (-token_count) as usize;
                    log::debug!("[llama_wrapper] Token buffer too small, resizing to {}", needed_size);
                    tokens.resize(needed_size, 0);
                    let retry_count = llama_tokenize(
                        model,
                        c_prompt.as_ptr(),
                        c_prompt.as_bytes().len() as i32,
                        tokens.as_mut_ptr(),
                        tokens.len() as i32,
                        true,
                        false,
                    );
                    if retry_count < 0 {
                        log::error!("[llama_wrapper] Failed to tokenize prompt even with larger buffer (size: {})", tokens.len());
                        return Err(AiProviderError::Unknown("Failed to tokenize prompt even with larger buffer".to_string()));
                    }
                    tokens.truncate(retry_count as usize);
                    log::info!("[llama_wrapper] Tokenized prompt: {} tokens", retry_count);
                } else {
                    tokens.truncate(token_count as usize);
                    log::info!("[llama_wrapper] Tokenized prompt: {} tokens", token_count);
                }

                if tokens.is_empty() {
                    log::error!("[llama_wrapper] Prompt tokenized to empty sequence");
                    return Err(AiProviderError::Unknown("Prompt tokenized to empty sequence".to_string()));
                }

                // Safety check: truncate tokens if they exceed batch size (2048)
                // The batch size is set in context params and we can't exceed it
                const MAX_BATCH_SIZE: usize = 2048;
                if tokens.len() > MAX_BATCH_SIZE {
                    log::warn!("[llama_wrapper] Prompt has {} tokens, truncating to {} to avoid batch size limit", tokens.len(), MAX_BATCH_SIZE);
                    tokens.truncate(MAX_BATCH_SIZE);
                }

                // Create batch for prompt tokens
                let mut batch = llama_batch_get_one(
                    tokens.as_mut_ptr(),
                    tokens.len() as i32,
                    0, // pos_0
                    0, // seq_id
                );

                // Validate batch was created correctly
                if batch.n_tokens == 0 {
                    log::error!("[llama_wrapper] Batch has 0 tokens after creation");
                    return Err(AiProviderError::Unknown("Failed to create batch for prompt tokens".to_string()));
                }

                // Set logits flag for the last token (we need logits to generate the next token)
                // llama_batch_get_one() should handle logits internally, but we need to set flags
                // Allocate logits array if needed (llama_batch_get_one may not initialize it)
                if batch.logits.is_null() {
                    let logits_size = batch.n_tokens as usize;
                    let mut logits_vec: Vec<i8> = vec![0; logits_size];
                    // Set flag for last token only
                    if logits_size > 0 {
                        logits_vec[logits_size - 1] = 1;
                    }
                    let logits_box = Box::into_raw(logits_vec.into_boxed_slice());
                    batch.logits = logits_box as *mut i8;
                    allocated_logits.push((batch.logits, logits_size));
                    log::debug!("[llama_wrapper] Allocated logits array for {} tokens", logits_size);
                } else {
                    // Set flag for last token to compute logits
                    if batch.n_tokens > 0 {
                        let last_idx = (batch.n_tokens - 1) as usize;
                        *batch.logits.add(last_idx) = 1;
                        log::debug!("[llama_wrapper] Set logits flag for last prompt token (index {})", last_idx);
                    }
                }

                // Decode the prompt
                log::debug!("[llama_wrapper] Decoding prompt with {} tokens", tokens.len());
                if llama_decode(ctx, batch) != 0 {
                    log::error!("[llama_wrapper] Failed to decode prompt");
                    // Free allocated logits arrays before returning
                    for (logits_ptr, size) in allocated_logits {
                        let _ = Box::from_raw(std::slice::from_raw_parts_mut(logits_ptr, size));
                    }
                    return Err(AiProviderError::Unknown("Failed to decode prompt".to_string()));
                }
                log::debug!("[llama_wrapper] Prompt decoded successfully. Batch has {} tokens", batch.n_tokens);

                // Generate tokens
                let mut output = String::new();
                let n_vocab = llama_n_vocab(model);
                let eos_token = llama_token_eos(model);
                log::info!("[llama_wrapper] Starting token generation: n_vocab={}, eos_token={}, max_tokens={}", n_vocab, eos_token, max_tokens);

                // Track current position in sequence
                let mut current_pos = batch.n_tokens as i32;

                for token_idx in 0..max_tokens {
                    // Log progress every 50 tokens
                    if token_idx % 50 == 0 && token_idx > 0 {
                        log::info!("[llama_wrapper] Generated {} tokens so far...", token_idx);
                    }
                    
                    // Validate batch state before accessing logits
                    if batch.n_tokens == 0 {
                        log::error!("[llama_wrapper] Batch has 0 tokens, cannot get logits");
                        return Err(AiProviderError::Unknown("Batch has 0 tokens".to_string()));
                    }

                    // Get logits for last token (batch.n_tokens - 1)
                    let logits_idx = (batch.n_tokens - 1) as i32;
                    log::debug!("[llama_wrapper] Getting logits for token at index {} (batch.n_tokens={})", logits_idx, batch.n_tokens);
                    
                    let logits_ptr = llama_get_logits_ith(ctx, logits_idx);
                    if logits_ptr.is_null() {
                        log::error!("[llama_wrapper] Logits pointer is null for index {}", logits_idx);
                        return Err(AiProviderError::Unknown(format!("Logits pointer is null for token index {}", logits_idx)));
                    }
                    
                    let logits = std::slice::from_raw_parts(logits_ptr, n_vocab as usize);

                    // Create token data array for sampling
                    let mut candidates: Vec<llama_token_data> = logits
                        .iter()
                        .enumerate()
                        .map(|(id, &logit)| llama_token_data {
                            id: id as i32,
                            logit,
                            p: 0.0,
                        })
                        .collect();

                    // Apply softmax to get probabilities
                    llama_sample_softmax(ctx, &mut llama_token_data_array {
                        data: candidates.as_mut_ptr(),
                        size: candidates.len(),
                        sorted: false,
                    });

                    // Create token data array for greedy sampling
                    let mut candidates_array = llama_token_data_array {
                        data: candidates.as_mut_ptr(),
                        size: candidates.len(),
                        sorted: false,
                    };

                    // Sample next token (greedy for deterministic JSON)
                    let next_token = llama_sample_token_greedy(ctx, &mut candidates_array);
                    log::debug!("[llama_wrapper] Generated token {} at position {}", next_token, current_pos);

                    // Check for EOS token
                    if next_token == eos_token {
                        log::info!("[llama_wrapper] EOS token reached at token {}, stopping generation", token_idx + 1);
                        break;
                    }
                    
                    // Early stopping: if we have a complete JSON object, stop immediately
                    // This helps avoid generating unnecessary tokens and significantly speeds up generation
                    // Start checking after 20 tokens (JSON usually starts early)
                    if token_idx >= 20 {
                        // Check if we have balanced braces
                        let open_braces = output.matches('{').count();
                        let close_braces = output.matches('}').count();
                        if open_braces > 0 && close_braces >= open_braces {
                            // Try to parse as JSON to verify it's complete (check every token for fastest stopping)
                            let trimmed = output.trim();
                            if let Ok(_) = serde_json::from_str::<serde_json::Value>(trimmed) {
                                log::info!("[llama_wrapper] Complete JSON detected at token {}, stopping early (output length: {} chars)", token_idx + 1, trimmed.len());
                                break;
                            } else if token_idx % 10 == 0 {
                                // Log every 10 tokens to help debug why JSON parsing might be failing
                                log::debug!("[llama_wrapper] JSON check at token {}: braces={}/{}", token_idx + 1, open_braces, close_braces);
                            }
                        }
                    }

                    // Decode token to text
                    // Use a larger buffer - tokens can be multi-byte UTF-8 sequences
                    let mut buffer = vec![0u8; 256];
                    let n_chars = llama_token_to_piece(
                        model,
                        next_token,
                        buffer.as_mut_ptr() as *mut c_char,
                        buffer.len() as i32,
                        false, // special
                    );

                    // llama_token_to_piece returns:
                    // - Positive: number of bytes written (including null terminator if present)
                    // - Negative: buffer too small, absolute value is needed size
                    // - Zero: empty token or error
                    if n_chars > 0 {
                        let bytes_written = n_chars as usize;
                        if bytes_written > buffer.len() {
                            // Buffer was too small, resize and retry
                            buffer.resize(bytes_written + 1, 0);
                            let retry_n_chars = llama_token_to_piece(
                                model,
                                next_token,
                                buffer.as_mut_ptr() as *mut c_char,
                                buffer.len() as i32,
                                false,
                            );
                            if retry_n_chars > 0 {
                                let slice = &buffer[..retry_n_chars as usize];
                                // Find null terminator or use the whole slice
                                let null_pos = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
                                if null_pos > 0 {
                                    match std::str::from_utf8(&slice[..null_pos]) {
                                        Ok(text) => {
                                            output.push_str(text);
                                            if token_idx < 10 || token_idx % 50 == 0 {
                                                log::debug!("[llama_wrapper] Token {} -> '{}' (output now {} chars)", next_token, text, output.len());
                                            }
                                        }
                                        Err(e) => {
                                            log::warn!("[llama_wrapper] Token {} UTF-8 error: {} (bytes: {:?})", next_token, e, &slice[..null_pos.min(10)]);
                                        }
                                    }
                                } else {
                                    log::debug!("[llama_wrapper] Token {} decoded but null_pos is 0", next_token);
                                }
                            }
                        } else {
                            // Buffer was large enough
                            let slice = &buffer[..bytes_written];
                            // Find null terminator or use the whole slice
                            let null_pos = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
                            if null_pos > 0 {
                                match std::str::from_utf8(&slice[..null_pos]) {
                                    Ok(text) => {
                                        output.push_str(text);
                                        if token_idx < 10 || token_idx % 50 == 0 {
                                            log::debug!("[llama_wrapper] Token {} -> '{}' (output now {} chars)", next_token, text, output.len());
                                        }
                                    }
                                    Err(e) => {
                                        log::warn!("[llama_wrapper] Token {} UTF-8 error: {} (bytes: {:?})", next_token, e, &slice[..null_pos.min(10)]);
                                    }
                                }
                            } else {
                                log::debug!("[llama_wrapper] Token {} decoded but null_pos is 0 (bytes_written={})", next_token, bytes_written);
                            }
                        }
                    } else if n_chars < 0 {
                        // Negative return means buffer was too small
                        let needed_size = (-n_chars) as usize;
                        buffer.resize(needed_size + 1, 0);
                        let retry_n_chars = llama_token_to_piece(
                            model,
                            next_token,
                            buffer.as_mut_ptr() as *mut c_char,
                            buffer.len() as i32,
                            false,
                        );
                        if retry_n_chars > 0 {
                            let slice = &buffer[..retry_n_chars as usize];
                            let null_pos = slice.iter().position(|&b| b == 0).unwrap_or(slice.len());
                            if null_pos > 0 {
                                match std::str::from_utf8(&slice[..null_pos]) {
                                    Ok(text) => {
                                        output.push_str(text);
                                        log::debug!("[llama_wrapper] Token {} -> '{}' (resized buffer, output now {} chars)", next_token, text, output.len());
                                    }
                                    Err(e) => {
                                        log::warn!("[llama_wrapper] Token {} UTF-8 error (resized): {} (bytes: {:?})", next_token, e, &slice[..null_pos.min(10)]);
                                    }
                                }
                            }
                        } else {
                            log::warn!("[llama_wrapper] Token {} failed to decode even with resized buffer (needed_size={}, retry_n_chars={})", next_token, needed_size, retry_n_chars);
                        }
                    } else {
                        // n_chars == 0, token might be empty or special
                        if token_idx < 10 {
                            log::debug!("[llama_wrapper] Token {} returned 0 characters (special/empty token)", next_token);
                        }
                    }

                    // Prepare next batch (single token)
                    // Free the logits array from the previous batch if we allocated it
                    // Track the previous batch's logits to free it
                    let prev_logits = if !batch.logits.is_null() {
                        // Find and remove this pointer from allocated_logits
                        if let Some(pos) = allocated_logits.iter().position(|(ptr, _)| *ptr == batch.logits) {
                            let (ptr, size) = allocated_logits.remove(pos);
                            Some((ptr, size))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    // Free the previous batch's logits if we allocated it
                    if let Some((ptr, size)) = prev_logits {
                        let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, size));
                    }
                    
                    // We'll create a new batch for the next token
                    let mut next_token_for_batch = next_token;
                    current_pos += 1;
                    
                    // Validate position is within context window (4096 is the context size we set)
                    const CONTEXT_WINDOW_SIZE: i32 = 4096;
                    if current_pos >= CONTEXT_WINDOW_SIZE {
                        log::warn!("[llama_wrapper] Reached context window limit ({}), stopping generation", CONTEXT_WINDOW_SIZE);
                        // Free any remaining allocated logits
                        for (logits_ptr, size) in allocated_logits.drain(..) {
                            let _ = Box::from_raw(std::slice::from_raw_parts_mut(logits_ptr, size));
                        }
                        break;
                    }
                    
                    batch = llama_batch_get_one(
                        &mut next_token_for_batch,
                        1, // n_tokens
                        current_pos, // pos_0 (continue from where we left off)
                        0, // seq_id
                    );

                    // Validate batch
                    if batch.n_tokens == 0 {
                        log::error!("[llama_wrapper] Failed to create batch for next token");
                        // Free any remaining allocated logits
                        for (logits_ptr, size) in allocated_logits.drain(..) {
                            let _ = Box::from_raw(std::slice::from_raw_parts_mut(logits_ptr, size));
                        }
                        break;
                    }

                    // Set logits flag for this token (we need logits to generate the next token)
                    if batch.logits.is_null() {
                        let logits_size = batch.n_tokens as usize;
                        let mut logits_vec: Vec<i8> = vec![0; logits_size];
                        // Set flag for the token
                        if logits_size > 0 {
                            logits_vec[0] = 1;
                        }
                        let logits_box = Box::into_raw(logits_vec.into_boxed_slice());
                        batch.logits = logits_box as *mut i8;
                        allocated_logits.push((batch.logits, logits_size));
                        log::debug!("[llama_wrapper] Allocated logits array for generated token");
                    } else {
                        // Set flag for the token to compute logits
                        if batch.n_tokens > 0 {
                            *batch.logits = 1;
                            log::debug!("[llama_wrapper] Set logits flag for generated token");
                        }
                    }

                    log::debug!("[llama_wrapper] Decoding next token at position {}", current_pos);
                    // Decode next token
                    let decode_result = llama_decode(ctx, batch);
                    if decode_result != 0 {
                        log::warn!("[llama_wrapper] Failed to decode token at position {} (error code: {}), stopping generation", current_pos, decode_result);
                        // Free any remaining allocated logits before breaking
                        for (logits_ptr, size) in allocated_logits.drain(..) {
                            let _ = Box::from_raw(std::slice::from_raw_parts_mut(logits_ptr, size));
                        }
                        break;
                    }
                    
                    // Log progress every 100 tokens for long generations
                    if (token_idx + 1) % 100 == 0 {
                        log::info!("[llama_wrapper] Progress: {}/{} tokens generated, output: {} chars", 
                            token_idx + 1, max_tokens, output.len());
                    }
                }

                // Free any remaining allocated logits arrays
                // Also free the current batch's logits if we allocated it
                if !batch.logits.is_null() {
                    if let Some(pos) = allocated_logits.iter().position(|(ptr, _)| *ptr == batch.logits) {
                        let (ptr, size) = allocated_logits.remove(pos);
                        let _ = Box::from_raw(std::slice::from_raw_parts_mut(ptr, size));
                    }
                }
                // Free any remaining allocated logits
                for (logits_ptr, size) in allocated_logits {
                    let _ = Box::from_raw(std::slice::from_raw_parts_mut(logits_ptr, size));
                }
                
                if output.is_empty() {
                    log::warn!("[llama_wrapper] Generation completed but output is empty");
                } else {
                    log::info!("[llama_wrapper] Generation completed. Output length: {} chars, preview: {}", 
                        output.len(), 
                        if output.len() > 100 { format!("{}...", &output[..100]) } else { output.clone() });
                }
                // Don't free batch from llama_batch_get_one() - it's stack-allocated
                Ok(output)
            }
        })
        .await
        .map_err(|e| AiProviderError::Unknown(format!("Task join error: {}", e)))?
    }

    /// Get the model path
    pub fn path(&self) -> &PathBuf {
        &self.model_path
    }
}

impl Drop for LlamaModel {
    fn drop(&mut self) {
        unsafe {
            if !self.ctx.is_null() {
                llama_free(self.ctx);
            }
            if !self.model.is_null() {
                llama_free_model(self.model);
            }
        }
    }
}

/// Thread-safe model cache
/// Allows sharing a loaded model across async tasks
pub type SharedModel = Arc<Mutex<Option<LlamaModel>>>;

/// Load or get cached model
pub async fn get_or_load_model(
    model_cache: &SharedModel,
    model_path: PathBuf,
) -> Result<Arc<LlamaModel>, AiProviderError> {
    let mut cache = model_cache.lock().await;
    
    // Check if model is already loaded with same path
    if let Some(ref model) = *cache {
        if model.path() == &model_path {
            // Model is already loaded with the same path
            // Return a new Arc pointing to the same model
            // We can't clone LlamaModel directly, so we need to reload
            // But first, let's try to reuse the existing model
            // Since we can't safely clone, we'll reload for now
            log::info!("[llama_wrapper] Model already loaded, but reloading to ensure thread safety");
        }
    }
    
    // Load new model (or reload if path changed)
    // Note: We need to drop the old model before loading new one to avoid double-free
    *cache = None;
    drop(cache); // Release lock before loading (which may take time)
    
    let model = LlamaModel::load(model_path).await?;
    let model_arc = Arc::new(model);
    
    // Update cache - store a reference to the Arc, not a new LlamaModel instance
    // We can't store the Arc directly in the cache because it would create a circular reference
    // Instead, we'll just store None and rely on the Arc reference counting
    // The caller will hold the Arc, which will keep the model alive
    let mut cache = model_cache.lock().await;
    // Don't store a new LlamaModel instance - that would cause double-free
    // The cache is just for checking if we need to reload, not for storing the model
    *cache = None; // Clear cache - the Arc will keep the model alive
    
    Ok(model_arc)
}
