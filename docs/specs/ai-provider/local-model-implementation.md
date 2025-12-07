# Local Model Implementation Guide

## ✅ Current Status

The structure for local model integration is **complete and ready**. All the scaffolding is in place:

- ✅ `LocalProvider` with async support
- ✅ `llama_wrapper` module for model abstraction
- ✅ Model caching and lazy loading
- ✅ Prompt formatting (matches cloud provider)
- ✅ JSON extraction helper
- ✅ Error handling
- ✅ All four AI operations wired up
- ✅ `llama-cpp-sys-3` dependency added
- ✅ Code compiles successfully

## ⏳ Remaining Work

The final step is implementing the actual llama.cpp C API calls in `llama_wrapper.rs`. The structure is ready, but the actual C API integration needs to be completed:

1. **Model Loading** (`LlamaModel::load`)
   - ✅ Structure in place
   - ⏳ Initialize llama backend (`llama_backend_init`)
   - ⏳ Load GGUF model file (`llama_load_model_from_file`)
   - ⏳ Create inference context (`llama_new_context_with_model`)
   - ⏳ Store handles in struct

2. **Inference** (`LlamaModel::generate`)
   - ✅ Structure in place
   - ⏳ Tokenize prompt (`llama_tokenize`)
   - ⏳ Create batch (`llama_batch_get_one` or `llama_batch_init`)
   - ⏳ Run inference loop (`llama_decode`, `llama_get_logits_ith`)
   - ⏳ Sample tokens (`llama_sample_token_greedy` or `llama_sample_token`)
   - ⏳ Decode to text (`llama_token_to_piece`)

## 📚 Resources for Implementation

### llama-cpp-sys-3 API

The crate provides bindings to llama.cpp. Key functions:

- `llama_backend_init()` - Initialize backend
- `llama_load_model_from_file()` - Load GGUF model
- `llama_new_context_with_model()` - Create context
- `llama_tokenize()` - Tokenize text
- `llama_decode()` - Run inference step
- `llama_sample_*()` - Sample next token
- `llama_token_to_piece()` - Decode token to text

### Example Implementation Pattern

```rust
use llama_cpp_sys_3::*;

impl LlamaModel {
    pub async fn load(path: PathBuf) -> Result<Self, AiProviderError> {
        // Run in blocking thread pool since llama.cpp is synchronous
        let model = tokio::task::spawn_blocking(move || {
            unsafe {
                // Initialize backend
                llama_backend_init(false);
                
                // Load model
                let model_params = llama_model_default_params();
                let model = llama_load_model_from_file(
                    path.to_str().unwrap().as_ptr() as *const i8,
                    model_params
                );
                
                if model.is_null() {
                    return Err("Failed to load model");
                }
                
                // Create context
                let ctx_params = llama_context_default_params();
                let ctx = llama_new_context_with_model(model, ctx_params);
                
                Ok((model, ctx))
            }
        }).await??;
        
        Ok(Self { model_path: path, model, ctx })
    }
    
    pub async fn generate(&self, prompt: &str, max_tokens: usize) -> Result<String, AiProviderError> {
        tokio::task::spawn_blocking({
            let prompt = prompt.to_string();
            let model = self.model;
            let ctx = self.ctx;
            move || {
                unsafe {
                    // Tokenize
                    let tokens = llama_tokenize(model, prompt.as_ptr(), ...);
                    
                    // Inference loop
                    for _ in 0..max_tokens {
                        llama_decode(ctx, ...);
                        let next_token = llama_sample(...);
                        // Decode and append to output
                    }
                    
                    Ok(output)
                }
            }
        }).await?
    }
}
```

## 🧪 Testing Steps

Once implementation is complete:

1. **Download Phi-3-mini GGUF model**
   - From Hugging Face: `microsoft/Phi-3-mini-4k-instruct-gguf`
   - Use Q4_K_M quantization (~2.3GB)

2. **Configure model path**
   - Add to Settings UI or set via `LocalProvider::with_model_path()`

3. **Test each operation**
   - Job parsing
   - Resume generation
   - Cover letter generation
   - Skill suggestions

4. **Verify JSON output**
   - Check that responses are valid JSON
   - Verify schema matches expected types
   - Test error handling for malformed responses

## 📝 Notes

- llama.cpp is **synchronous**, so use `tokio::task::spawn_blocking` for async wrapper
- Model loading can take 5-10 seconds (acceptable for first use)
- Inference speed: ~5-10 seconds per request on modern CPU
- Memory: Model stays in memory once loaded (good for performance)

## 🎯 Next Action

Implement the actual llama.cpp calls in `llama_wrapper.rs` following the pattern above. The structure is ready - just need to fill in the C API calls.

---

**Status**: Structure complete, ready for final implementation step.

