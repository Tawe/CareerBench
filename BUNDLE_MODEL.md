# Bundling the Phi-3-mini Model

To bundle the Phi-3-mini GGUF model with the app:

## Steps

1. **Download the model** (~2.3GB):
   ```bash
   # Download from Hugging Face
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

## How It Works

- The model is bundled with the app during build
- On first launch, the app extracts the model to `.careerbench/models/` in the app data directory
- The model path is automatically configured in settings
- Users can use Local mode immediately without downloading

## Considerations

- **App Size**: Bundling adds ~2.3GB to the app bundle
- **Build Time**: Initial build will be slower due to copying the large file
- **Distribution**: Consider making it optional or providing a separate installer

## Alternative: Post-Install Download

If you prefer not to bundle (to keep app size small):
- Users can use the "Download Model" button in Settings
- Or configure a custom model path manually

