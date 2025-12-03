# Troubleshooting Guide

## Error: "Cannot read properties of undefined (reading 'invoke')"

This error occurs when the Tauri API is not available. This typically happens when:

### Cause 1: Running `npm run dev` instead of `npm run tauri dev`

**Solution:** Always use `npm run tauri dev` to run the app. The Tauri API is only available when running through the Tauri application context.

```bash
# ❌ Wrong - This runs only the Vite dev server in a browser
npm run dev

# ✅ Correct - This runs the full Tauri app
npm run tauri dev
```

### Cause 2: Tauri API not initialized

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

### Cause 3: Import path issue

The correct import for Tauri 2.0 is:
```typescript
import { invoke } from "@tauri-apps/api/core";
```

If you see import errors, try:
```bash
rm -rf node_modules
npm install
```

## Other Common Issues

### Database not found

If you see database errors:
- The database is created automatically on first run
- Location: `.careerbench/careerbench.db` in project root
- To reset: Delete `.careerbench` folder and restart app

### Rust compilation errors

If Rust fails to compile:
```bash
cd src-tauri
cargo clean
cargo update
cargo build
```

### Frontend build errors

If frontend fails to build:
```bash
rm -rf node_modules dist
npm install
npm run build
```

