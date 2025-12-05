# Local Model Recommendation for CareerBench

## 🎯 Recommended Approach

### Library Choice: **Candle** (Hugging Face)

**Why Candle?**
- ✅ Actively maintained by Hugging Face
- ✅ Modern Rust framework, well-documented
- ✅ Supports CPU and GPU inference
- ✅ Good performance on desktop hardware
- ✅ Works well with GGUF models
- ✅ Cross-platform (macOS, Linux, Windows)
- ✅ No C dependencies (pure Rust)

**Alternative**: `llm_client` - Simpler API, but less actively maintained

### Model Choice: **Phi-3-mini** (3.8B parameters)

**Why Phi-3-mini?**
- ✅ **Small size**: ~2.3GB in Q4_K_M quantization (perfect for desktop app)
- ✅ **Excellent instruction following**: Trained specifically for structured tasks
- ✅ **JSON output**: Very reliable at producing valid JSON
- ✅ **Fast inference**: Optimized for CPU, runs well on modern laptops
- ✅ **Good quality**: Comparable to larger models for structured tasks
- ✅ **Microsoft-backed**: Well-maintained, good documentation

**Model Details**:
- **Full name**: `microsoft/Phi-3-mini-4k-instruct`
- **Size**: 3.8B parameters
- **Quantized (Q4_K_M)**: ~2.3GB
- **Context**: 4K tokens (sufficient for our use cases)
- **Format**: GGUF (works with candle/llama.cpp)

**Download**: Available on Hugging Face in GGUF format

### Alternative Models (if Phi-3-mini doesn't work)

1. **Llama 3.2 3B Instruct** (~2GB quantized)
   - Meta's latest small model
   - Very capable, good instruction following
   - Slightly larger than Phi-3-mini

2. **TinyLlama 1.1B** (~700MB quantized)
   - Smallest option
   - Quality may suffer for complex tasks
   - Good for testing/development

3. **Mistral 7B Instruct** (~4GB quantized)
   - Higher quality but larger
   - Better for production if size isn't a concern

## 📊 Comparison Table

| Model | Size (Q4) | Quality | JSON Reliability | Speed | Best For |
|-------|-----------|---------|-----------------|-------|----------|
| **Phi-3-mini** | 2.3GB | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | **Recommended** |
| Llama 3.2 3B | 2.0GB | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | Alternative |
| TinyLlama 1.1B | 700MB | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ | Testing only |
| Mistral 7B | 4.0GB | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | If size OK |

## 🚀 Implementation Plan

### Phase 1: Setup (Current)
- ✅ Structure in place
- ✅ Prompt formatting complete
- ✅ Error handling in place
- ⚠️ Library selection: Need stable Rust-compatible option
  - Candle has dependency conflicts
  - llm_client requires nightly Rust
  - Evaluating llama-cpp-sys-3 or custom wrapper

### Phase 2: Basic Integration
1. Add `candle-core` and `candle-transformers` dependencies
2. Implement model loading with GGUF support
3. Test with Phi-3-mini model file
4. Implement basic inference

### Phase 3: JSON Extraction
1. Implement prompt formatting for structured output
2. Add JSON extraction (handle markdown code blocks)
3. Add validation and error handling
4. Test all four operations

### Phase 4: Optimization
1. Add model caching (keep loaded in memory)
2. Optimize inference parameters
3. Add progress indicators
4. Performance testing

## 💡 Why This Combination Works

### For CareerBench's Use Cases:

1. **Job Parsing**: Phi-3-mini excels at extraction tasks
   - Can reliably extract skills, responsibilities, etc.
   - Good at following "only extract what's stated" rules

2. **Resume Generation**: Structured output is key
   - Phi-3-mini is trained for instruction following
   - Produces consistent JSON format

3. **Cover Letters**: Needs to be coherent and structured
   - 3.8B parameters is sufficient for quality text
   - Instruction tuning ensures it follows format rules

4. **Skill Suggestions**: Analysis and recommendations
   - Good reasoning capabilities for gap analysis
   - Reliable structured output

### Technical Benefits:

- **Size**: 2.3GB is reasonable for a desktop app
  - Can be downloaded on first use
  - Or bundled with app (if distribution allows)
  
- **Performance**: Fast enough for interactive use
  - ~5-10 seconds per inference on modern CPU
  - Acceptable for user-facing features
  
- **Reliability**: Instruction-tuned models are more reliable
  - Less likely to hallucinate
  - Better at following JSON schema rules

## 📦 Distribution Strategy

### Option 1: Download on First Use (Recommended)
- App checks for model on first local AI request
- Downloads from Hugging Face if missing
- Shows progress indicator
- Stores in app data directory

### Option 2: Bundle with App
- Include model in app bundle
- Larger download (~2.5GB)
- Works offline immediately
- May need platform-specific builds

### Option 3: User Provides Model
- User downloads model separately
- Configure path in settings
- Most flexible, but requires user setup

**Recommendation**: Start with Option 1, add Option 3 for power users.

## 🔧 Implementation Notes

### Candle Integration Example

```rust
use candle_core::{Device, Tensor};
use candle_transformers::models::llama::{Llama, LlamaConfig};

// Load model
let device = Device::Cpu; // or Device::Cuda(0) for GPU
let model = Llama::load_from_gguf(&model_path, &device)?;

// Run inference
let tokens = tokenize(prompt)?;
let logits = model.forward(&tokens, 0)?;
let response = decode(logits)?;
```

### Prompt Formatting

For structured JSON output, use clear instructions:
```
You are a job description parser. Extract structured information.

CRITICAL: Output ONLY valid JSON, no other text.

Job description:
{description}

Output JSON matching this schema:
{
  "requiredSkills": string[],
  "responsibilities": string[],
  ...
}
```

## ✅ Next Steps

1. **Add Candle Dependencies**
   ```toml
   candle-core = "0.4"
   candle-transformers = "0.4"
   ```

2. **Download Test Model**
   - Get Phi-3-mini GGUF from Hugging Face
   - Test loading and basic inference

3. **Implement Model Loading**
   - Update `ensure_model_loaded()` in LocalProvider
   - Handle model path configuration

4. **Implement Inference**
   - Update `run_inference()` in LocalProvider
   - Add JSON extraction logic

5. **Test End-to-End**
   - Test all four AI operations
   - Verify JSON output quality
   - Check performance

---

**Recommendation Summary**: Use **Candle** library with **Phi-3-mini** model for the best balance of size, quality, and reliability for CareerBench's structured output requirements.

