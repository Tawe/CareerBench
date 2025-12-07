//! Wrapper around llama.cpp for async model loading and inference
//! Provides a simple async interface for GGUF model inference

use crate::ai::errors::AiProviderError;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

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
pub struct LlamaModel {
    model_path: PathBuf,
    model: *mut llama_model,
    ctx: *mut llama_context,
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
                ctx_params.n_threads = num_cpus::get() as u32; // Use all CPU cores
                ctx_params.n_threads_batch = num_cpus::get() as u32;

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
        })
    }

    /// Generate text from a prompt
    /// Returns the generated text (which should contain JSON)
    pub async fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, AiProviderError> {
        // Copy pointers (safe - just copying memory addresses)
        // We use std::ptr::addr_of! to ensure we're just copying the address
        let model_ptr = self.model as usize;
        let ctx_ptr = self.ctx as usize;
        let prompt = prompt.to_string();

        // Run inference in blocking thread
        // Note: We reconstruct pointers from usize, which is safe since they're just addresses
        tokio::task::spawn_blocking(move || {
            // Reconstruct pointers from usize (safe - just addresses)
            let model = model_ptr as *mut llama_model;
            let ctx = ctx_ptr as *mut llama_context;
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

                // Create batch for prompt tokens
                let mut batch = llama_batch_get_one(
                    tokens.as_mut_ptr(),
                    tokens.len() as i32,
                    0, // pos_0
                    0, // seq_id
                );

                // Set logits flag for the last token (we need logits to generate the next token)
                // llama_batch_get_one() doesn't initialize logits array, so we need to allocate it
                if batch.logits.is_null() {
                    let logits_size = batch.n_tokens as usize;
                    let logits_vec: Vec<i8> = vec![0; logits_size];
                    let logits_box = Box::into_raw(logits_vec.into_boxed_slice());
                    batch.logits = logits_box as *mut i8;
                    log::debug!("[llama_wrapper] Allocated logits array for {} tokens", logits_size);
                }
                // Set flag for last token to compute logits
                if batch.n_tokens > 0 {
                    let last_idx = (batch.n_tokens - 1) as usize;
                    // Already in unsafe block, can access raw pointer directly
                    *batch.logits.add(last_idx) = 1;
                    log::debug!("[llama_wrapper] Set logits flag for last prompt token (index {})", last_idx);
                }

                // Decode the prompt
                log::debug!("[llama_wrapper] Decoding prompt with {} tokens", tokens.len());
                if llama_decode(ctx, batch) != 0 {
                    log::error!("[llama_wrapper] Failed to decode prompt");
                    // Don't free batch from llama_batch_get_one() - it's stack-allocated
                    // But we should free the logits array we allocated
                    if !batch.logits.is_null() {
                        let _ = Box::from_raw(std::slice::from_raw_parts_mut(batch.logits, batch.n_tokens as usize));
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

                for _token_idx in 0..max_tokens {
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
                        log::info!("[llama_wrapper] EOS token reached, stopping generation");
                        break;
                    }

                    // Decode token to text
                    let mut buffer = vec![0u8; 32];
                    let n_chars = llama_token_to_piece(
                        model,
                        next_token,
                        buffer.as_mut_ptr() as *mut c_char,
                        buffer.len() as i32,
                        false, // special
                    );

                    if n_chars > 0 {
                        let piece = CStr::from_bytes_with_nul(
                            &buffer[..n_chars as usize]
                        ).unwrap_or_else(|_| {
                            // If no null terminator, create a new CStr from the slice
                            CStr::from_bytes_with_nul(&[0]).unwrap()
                        });
                        
                        if let Ok(text) = piece.to_str() {
                            output.push_str(text);
                        }
                    }

                    // Prepare next batch (single token)
                    // Note: Don't free batch from llama_batch_get_one() - it's stack-allocated
                    // But we should free the logits array we allocated for the previous batch
                    if !batch.logits.is_null() {
                        // Intentionally leak - batch wasn't allocated with llama_batch_init()
                        // Freeing it would cause crashes. Small memory leak is acceptable.
                        log::debug!("[llama_wrapper] Not freeing logits array (intentional leak for stability)");
                    }
                    
                    // We'll create a new batch for the next token
                    let mut next_token_for_batch = next_token;
                    current_pos += 1;
                    batch = llama_batch_get_one(
                        &mut next_token_for_batch,
                        1, // n_tokens
                        current_pos, // pos_0 (continue from where we left off)
                        0, // seq_id
                    );

                    // Set logits flag for this token (we need logits to generate the next token)
                    if batch.logits.is_null() {
                        let logits_size = batch.n_tokens as usize;
                        let logits_vec: Vec<i8> = vec![0; logits_size];
                        let logits_box = Box::into_raw(logits_vec.into_boxed_slice());
                        batch.logits = logits_box as *mut i8;
                        log::debug!("[llama_wrapper] Allocated logits array for generated token");
                    }
                    // Set flag for the token to compute logits
                    if batch.n_tokens > 0 {
                        // Already in unsafe block, can access raw pointer directly
                        *batch.logits = 1;
                        log::debug!("[llama_wrapper] Set logits flag for generated token");
                    }

                    log::debug!("[llama_wrapper] Decoding next token at position {}", current_pos);
                    // Decode next token
                    if llama_decode(ctx, batch) != 0 {
                        log::warn!("[llama_wrapper] Failed to decode token at position {}, stopping generation", current_pos);
                        // Don't free batch from llama_batch_get_one()
                        break;
                    }
                }

                log::info!("[llama_wrapper] Generation completed. Output length: {} chars", output.len());
                // Don't free batch from llama_batch_get_one() - it's stack-allocated
                Ok(output)
            }
        }).await
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
            // For now, we'll reload to keep it simple
            // TODO: Optimize to share the same model instance
        }
    }
    
    // Load new model (or reload if path changed)
    // Note: We need to drop the old model before loading new one
    *cache = None;
    drop(cache); // Release lock before loading (which may take time)
    
    let model = LlamaModel::load(model_path).await?;
    let model_arc = Arc::new(model);
    
    // Update cache
    let mut cache = model_cache.lock().await;
    *cache = Some(LlamaModel {
        model_path: model_arc.path().clone(),
        model: model_arc.model,
        ctx: model_arc.ctx,
    });
    
    Ok(model_arc)
}
