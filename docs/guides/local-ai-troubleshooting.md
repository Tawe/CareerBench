# Local AI Troubleshooting Guide

Detailed troubleshooting for local AI model issues in CareerBench.

## Problem: llama.cpp Crashes During Resume Generation

### Symptoms

- App crashes during resume generation when using local Phi-3-mini GGUF model
- Crash occurs in the `llama_wrapper.rs` module during token generation
- GGML_ASSERT errors or segfaults
- Logs show successful prompt decoding, then sudden crash

### Root Cause

The crash was caused by calling `llama_batch_free()` on batches created with `llama_batch_get_one()` instead of `llama_batch_init()`.

**The Fix:** The code has been updated to not free batches from `llama_batch_get_one()`. These batches are stack-allocated and don't need freeing.

### Verification

If you're still experiencing crashes:

1. **Check the logs** (`src-tauri/.careerbench/careerbench.log`):
   ```bash
   tail -50 src-tauri/.careerbench/careerbench.log
   ```

2. **Look for error patterns:**
   - GGML_ASSERT errors
   - Segfault messages
   - "Failed to decode" messages

3. **Verify model file:**
   - Ensure the model file is not corrupted
   - Re-download if necessary
   - Check file size matches expected (~2.3GB for Phi-3-mini Q4)

### Solutions

#### Solution 1: Update to Latest Code

The crash fix is in the latest code. Ensure you have:
- Updated `llama_wrapper.rs` with proper batch handling
- No `llama_batch_free()` calls on `llama_batch_get_one()` batches

#### Solution 2: Check Model Compatibility

- Ensure you're using a compatible GGUF model
- Phi-3-mini 4k instruct Q4 is recommended
- Verify the model format is correct

#### Solution 3: Memory Issues

If crashes persist, check:
- **RAM:** Models require 2-4GB+ of free RAM
- **Context size:** Reduce `n_ctx` in context parameters if needed
- **System resources:** Close other memory-intensive applications

## Problem: Empty or Invalid Responses

### Symptoms

- Model generates empty responses
- JSON parsing errors
- "Generation completed. Output length: 0 chars" in logs

### Solutions

1. **Check prompt format:**
   - Verify system prompts are correctly formatted
   - Ensure JSON schema is clear in prompts

2. **Increase max_tokens:**
   - Default is 1000 tokens
   - Some responses may need more

3. **Check model output:**
   - Review logs for actual model responses
   - Verify JSON extraction logic

## Problem: Very Slow Inference

### Symptoms

- Takes 10+ seconds per token
- Overall generation takes minutes

### Solutions

1. **Use quantized models:**
   - Q4 quantization is recommended
   - Q8 is higher quality but slower

2. **Optimize context:**
   - Reduce context window if not needed
   - Shorter prompts = faster inference

3. **System optimization:**
   - Close other CPU-intensive applications
   - Ensure adequate cooling (CPU throttling slows inference)

4. **Consider GPU:**
   - GPU acceleration is not yet implemented
   - Future versions may support CUDA/Metal

## Problem: Model Not Loading

### Symptoms

- "Failed to load model" errors
- Model path not recognized

### Solutions

1. **Verify path:**
   - Use absolute paths for reliability
   - Check file permissions

2. **Check file:**
   - Ensure file exists
   - Verify file is not corrupted
   - Check file size matches expected

3. **Re-download:**
   - If file is corrupted, re-download
   - Verify download completed successfully

## Debugging Tips

### Enable Detailed Logging

The app logs detailed information about:
- Model loading attempts
- Inference requests
- Token generation progress
- Errors and warnings

Check logs at: `src-tauri/.careerbench/careerbench.log`

### Common Log Patterns

**Successful generation:**
```
[INFO] [LocalProvider] Model loaded successfully
[INFO] [LocalProvider] Starting inference request
[INFO] [llama_wrapper] Generation completed. Output length: X chars
```

**Errors to watch for:**
```
[ERROR] [LocalProvider] Failed to load model
[ERROR] [llama_wrapper] Failed to parse JSON
[ERROR] [LocalProvider] Model file not found
```

## Getting More Help

If issues persist:

1. Check the [main Troubleshooting Guide](troubleshooting.md)
2. Review [AI Provider Documentation](../specs/ai-provider.md)
3. Check GitHub issues for similar problems
4. Review logs for specific error messages

## Related Documentation

- [Model Setup Guide](model-setup.md) - Setting up models
- [Troubleshooting Guide](troubleshooting.md) - General troubleshooting
- [AI Provider Documentation](../specs/ai-provider.md) - Technical architecture

