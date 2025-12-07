# Troubleshooting Guide

This guide covers common issues and their solutions when using CareerBench.

## Table of Contents

- [Tauri API Errors](#tauri-api-errors)
- [Database Issues](#database-issues)
- [Build and Compilation](#build-and-compilation)
- [Local AI Model Issues](#local-ai-model-issues)
- [Frontend Issues](#frontend-issues)

---

## Tauri API Errors

### Error: "Cannot read properties of undefined (reading 'invoke')"

This error occurs when the Tauri API is not available.

#### Cause 1: Running `npm run dev` instead of `npm run tauri dev`

**Solution:** Always use `npm run tauri dev` to run the app. The Tauri API is only available when running through the Tauri application context.

```bash
# ❌ Wrong - This runs only the Vite dev server in a browser
npm run dev

# ✅ Correct - This runs the full Tauri app
npm run tauri dev
```

#### Cause 2: Tauri API not initialized

If you're running `npm run tauri dev` and still getting this error:

1. **Check that Tauri is properly installed:**
   ```bash
   npm list @tauri-apps/api
   npm list @tauri-apps/cli
   ```

2. **Rebuild the Rust backend:**
   ```bash
   cd src-tauri
   cargo clean
   cargo build
   cd ..
   npm run tauri dev
   ```

3. **Check the Tauri configuration:**
   - Verify `src-tauri/tauri.conf.json` exists and is valid
   - Check that `src-tauri/Cargo.toml` has the correct dependencies

#### Cause 3: Import path issue

The correct import for Tauri 2.0 is:
```typescript
import { invoke } from "@tauri-apps/api/core";
```

If you see import errors, try:
```bash
rm -rf node_modules
npm install
```

---

## Database Issues

### Database not found

**Symptoms:** Database errors on startup or when accessing data.

**Solution:**
- The database is created automatically on first run
- Location: `src-tauri/.careerbench/careerbench.db` (development)
- To reset: Delete `.careerbench` folder and restart app

### Database corruption

**Symptoms:** SQLite errors, data not loading.

**Solution:**
```bash
# Backup first (if needed)
cp src-tauri/.careerbench/careerbench.db src-tauri/.careerbench/careerbench.db.backup

# Delete and let app recreate
rm -rf src-tauri/.careerbench
# Restart app - migrations will recreate the database
```

### Populating test data

Use the test data script:
```bash
./scripts/populate_test_data.sh
```

See [scripts/README.md](../scripts/README.md) for details.

---

## Build and Compilation

### Rust compilation errors

**Solution:**
```bash
cd src-tauri
cargo clean
cargo update
cargo build
```

### Frontend build errors

**Solution:**
```bash
rm -rf node_modules dist
npm install
npm run build
```

### TypeScript errors

**Solution:**
```bash
# Clear TypeScript cache
rm -rf node_modules/.cache
npm run build
```

---

## Local AI Model Issues

### Model file not found

**Symptoms:** "Local model path not configured" or "Model file not found" errors.

**Solution:**
1. Download a GGUF model (e.g., Phi-3-mini) from Hugging Face
2. Place it in `src-tauri/.careerbench/models/` or configure a custom path
3. Update the model path in Settings

See [Model Setup Guide](model-setup.md) for detailed instructions.

### Model crashes during inference

**Symptoms:** App crashes with GGML_ASSERT errors or segfaults during resume generation.

**Solution:**
- Ensure you're using a compatible GGUF model format
- Check that the model file is not corrupted
- Verify you have sufficient RAM (models require 2-4GB+)
- See [Local AI Troubleshooting](local-ai-troubleshooting.md) for detailed debugging

### Slow inference performance

**Symptoms:** Very slow token generation (seconds per token).

**Solutions:**
- Use a quantized model (Q4 or Q5) for better performance
- Ensure you're using CPU threads efficiently
- Consider using GPU acceleration if available
- Reduce context window size if not needed

---

## Frontend Issues

### UI not updating

**Symptoms:** Changes to React components not appearing.

**Solution:**
```bash
# Clear Vite cache
rm -rf node_modules/.vite
npm run tauri dev
```

### Styling issues

**Symptoms:** CSS not applying correctly.

**Solution:**
- Check browser console for CSS errors
- Verify CSS imports in component files
- Clear browser cache if testing in browser mode

### State management issues

**Symptoms:** Data not persisting or state not updating.

**Solution:**
- Check Tauri command responses
- Verify database operations are completing
- Check browser console for errors

---

## Getting More Help

If you encounter an issue not covered here:

1. Check the [GitHub Issues](https://github.com/your-repo/issues) for similar problems
2. Review the [Development Documentation](../development/) for technical details
3. See [Contributing Guide](../../CONTRIBUTING.md) for how to report bugs

---

## Related Documentation

- [Model Setup Guide](model-setup.md) - Setting up local AI models
- [Development Setup](../development/setup.md) - Development environment
- [Local AI Troubleshooting](local-ai-troubleshooting.md) - Detailed AI debugging

