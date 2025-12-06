# Bundled Model Instructions

To bundle the Phi-3-mini GGUF model with the app:

1. Download the model from Hugging Face:
   ```
   https://huggingface.co/microsoft/Phi-3-mini-4k-instruct-gguf/resolve/main/Phi-3-mini-4k-instruct-q4_k_m.gguf
   ```

2. Place the downloaded file here:
   ```
   src-tauri/resources/models/Phi-3-mini-4k-instruct-q4_k_m.gguf
   ```

3. The model will be bundled with the app during build and extracted to the app data directory on first use.

**Note**: The model file is ~2.3GB, so it will significantly increase the app bundle size. It's recommended to:
- Use the download feature in Settings for development
- Only bundle for production releases if you want offline-first experience
- Consider making it an optional component that users can download post-install

