# Local AI Model Setup Guide

This guide explains how to set up and configure local AI models for CareerBench.

## Overview

CareerBench supports local AI inference using GGUF format models. This allows you to:
- Generate resumes and cover letters offline
- Keep your data private (no API calls to external services)
- Avoid API costs
- Work without internet connection

## Recommended Models

### Phi-3-mini (Recommended for most users)

- **Size:** ~2.3GB (Q4 quantization)
- **Performance:** Good balance of quality and speed
- **Context:** 4K tokens
- **Download:** [Hugging Face - Phi-3-mini-4k-instruct-gguf](https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf)

**Recommended variant:** `Phi-3-mini-4k-instruct-q4_k_m.gguf`

### Other Options

- **Phi-3-medium:** Better quality, larger size (~7GB)
- **Llama 3.2 3B:** Alternative option with good performance
- **Qwen2.5:** Another high-quality option

## Installation Steps

### Step 1: Download the Model

**Option A: Manual Download**

1. Visit the model's Hugging Face page
2. Navigate to the GGUF files
3. Download the Q4 quantized version (recommended)
4. Save to: `src-tauri/.careerbench/models/`

**Option B: Using curl (macOS/Linux)**

```bash
# Create models directory
mkdir -p src-tauri/.careerbench/models

# Download Phi-3-mini Q4 model
curl -L -o src-tauri/.careerbench/models/Phi-3-mini-4k-instruct-q4_k_m.gguf \
  https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf
```

### Step 2: Configure in CareerBench

1. Open CareerBench
2. Go to **Settings** → **AI Settings**
3. Set **AI Mode** to "Local" or "Hybrid"
4. Enter the model path:
   ```
   /path/to/your/project/src-tauri/.careerbench/models/Phi-3-mini-4k-instruct-q4_k_m.gguf
   ```
   Or use relative path:
   ```
   src-tauri/.careerbench/models/Phi-3-mini-4k-instruct-q4_k_m.gguf
   ```
5. Click **Save**

### Step 3: Verify Setup

1. Check the AI Provider status in Settings
2. It should show "Local (Ready)" if configured correctly
3. Try generating a resume to test

## Bundling Models with the App

If you want to bundle a model with the app distribution:

### Steps

1. **Download the model** (~2.3GB):
   ```bash
   curl -L -o src-tauri/resources/models/Phi-3-mini-4k-instruct-q4_k_m.gguf \
     https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf
   ```

2. **Update tauri.conf.json** to include the resource:
   ```json
   {
     "bundle": {
       "active": true,
       "targets": "all",
       "resources": [
         "models/Phi-3-mini-4k-instruct-q4_k_m.gguf"
       ]
     }
   }
   ```

3. **Build the app** - the model will be bundled:
   ```bash
   npm run tauri build
   ```

### Considerations

- **App Size:** Bundling adds ~2.3GB to the app bundle
- **Build Time:** Initial build will be slower due to copying the large file
- **Distribution:** Consider making it optional or providing a separate installer

## Alternative: Post-Install Download

If you prefer not to bundle (to keep app size small):
- Users can use the "Download Model" button in Settings (if implemented)
- Or configure a custom model path manually

## Troubleshooting

### Model file not found

- Verify the path is correct and absolute
- Check that the file exists at the specified location
- Ensure you have read permissions

### Model crashes during inference

See [Local AI Troubleshooting](local-ai-troubleshooting.md) for detailed debugging steps.

### Slow performance

- Use Q4 or Q5 quantization for better speed
- Ensure sufficient RAM (models need 2-4GB+)
- Consider using a smaller model if speed is critical

## Related Documentation

- [Troubleshooting Guide](troubleshooting.md) - General troubleshooting
- [Local AI Troubleshooting](local-ai-troubleshooting.md) - AI-specific issues
- [AI Provider Documentation](../specs/ai-provider.md) - Technical details

