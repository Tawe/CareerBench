# Local Model Implementation Notes

## Current Status

✅ **Structure Complete**: The `LocalProvider` has all the scaffolding in place:
- Model path configuration
- Lazy loading pattern
- All four AI operations implemented
- Prompt formatting (matches cloud provider)
- JSON extraction helper
- Error handling

⏳ **Library Integration Pending**: Need to choose a stable Rust-compatible library

## Library Evaluation

### Attempted Options

1. **Candle** ❌
   - **Issue**: Dependency conflicts with `half` crate
   - **Status**: Not compatible with current dependency set

2. **llm_client** ❌
   - **Issue**: Requires nightly Rust (`#![feature]`)
   - **Status**: Not suitable for production Tauri app

3. **llama_cpp_rs** ❌
   - **Issue**: Build errors on some platforms
   - **Status**: Unreliable for cross-platform deployment

### Recommended Next Steps

**Option A: llama-cpp-sys-3** (Most Stable)
- Direct bindings to llama.cpp
- Works with stable Rust
- Requires C build tools (acceptable for Tauri)
- Well-tested and widely used
- **Action**: Add `llama-cpp-sys-3 = "0.5"` and implement wrapper

**Option B: Custom llama.cpp Wrapper**
- Use `llama-cpp-sys-3` as base
- Create simple async wrapper in our codebase
- More control, but more work
- **Action**: Create `src-tauri/src/ai/llama_wrapper.rs`

**Option C: Wait for Candle Fix**
- Candle is actively developed
- May fix dependency issues in future versions
- **Action**: Monitor Candle releases

## Implementation Path Forward

### Recommended: Option A (llama-cpp-sys-3)

1. **Add Dependency**
   ```toml
   llama-cpp-sys-3 = "0.5"
   ```

2. **Create Wrapper Module**
   - `src-tauri/src/ai/llama_wrapper.rs`
   - Simple async wrapper around llama.cpp C API
   - Handle model loading and inference

3. **Update LocalProvider**
   - Use wrapper instead of direct library
   - Implement `ensure_model_loaded()` and `run_inference()`

4. **Test with Phi-3-mini**
   - Download GGUF model
   - Test all four operations
   - Verify JSON output quality

## Model Recommendation Still Stands

**Phi-3-mini** remains the best choice:
- 2.3GB quantized (perfect size)
- Excellent instruction following
- Reliable JSON output
- Fast CPU inference

## Current Code Status

The `LocalProvider` code is **ready for integration**. Once we add a working library:

1. Replace `_model: Arc<Mutex<Option<()>>>` with actual model type
2. Implement `ensure_model_loaded()` to load GGUF model
3. Implement `run_inference()` to run inference and extract JSON
4. Test end-to-end

The structure, prompts, and error handling are all in place and match the cloud provider patterns.

---

**Next Action**: Implement Option A (llama-cpp-sys-3 wrapper) or wait for Candle dependency resolution.

