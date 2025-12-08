# ADR-0001: Use Tauri for Desktop Application Framework

## Status

Accepted

## Context

CareerBench needs to be a desktop application that:
- Runs on macOS, Windows, and Linux
- Has access to local file system and system APIs
- Provides a modern, responsive UI
- Can bundle a local SQLite database
- Can integrate with local AI models (GGUF files)
- Has minimal resource footprint
- Provides native performance for data operations

We evaluated several options:
1. **Electron**: Mature, large ecosystem, but high memory usage (~100-200MB base)
2. **Tauri**: Lightweight, Rust backend, smaller bundle size (~5-10MB base)
3. **Native frameworks**: Qt, GTK, etc. - more complex, less web-friendly
4. **Web app only**: Doesn't meet desktop app requirements

## Decision

We chose **Tauri** as the desktop application framework.

### Rationale

1. **Lightweight**: Tauri uses the system webview, resulting in much smaller bundle sizes (~5-10MB vs Electron's ~100-200MB)
2. **Performance**: Rust backend provides native performance for database operations and AI inference
3. **Security**: Rust's memory safety and Tauri's security model reduce attack surface
4. **Local-first**: Easy integration with SQLite and local file system
5. **Modern UI**: Can use React/TypeScript for familiar web development experience
6. **Cross-platform**: Supports macOS, Windows, and Linux
7. **Active development**: Growing ecosystem with good documentation

### Architecture

```
┌─────────────────────────────────────┐
│         React Frontend               │
│    (TypeScript, React Router)       │
└──────────────┬──────────────────────┘
               │ Tauri IPC
┌──────────────▼──────────────────────┐
│         Rust Backend                 │
│  (Tauri Commands, SQLite, AI)       │
└──────────────────────────────────────┘
```

## Consequences

### Positive

- **Small bundle size**: Users download ~10-20MB instead of ~100-200MB
- **Fast startup**: Lower memory footprint means faster app launch
- **Native performance**: Rust backend handles heavy operations efficiently
- **Security**: Rust's safety guarantees reduce vulnerabilities
- **Developer experience**: Can use familiar React/TypeScript tooling

### Negative

- **Learning curve**: Team needs Rust knowledge for backend development
- **Ecosystem**: Smaller ecosystem than Electron (though growing)
- **Webview dependency**: Requires system webview (usually available, but not guaranteed on all Linux distros)
- **Build complexity**: Need to build both Rust and TypeScript components

### Mitigations

- **Documentation**: Comprehensive setup guides and examples
- **Type safety**: Strong TypeScript types for Tauri commands reduce errors
- **Testing**: Integration tests ensure Rust/TypeScript boundary works correctly
- **Fallback**: Webview requirements are documented in installation guide

## Alternatives Considered

### Electron

**Pros**: Larger ecosystem, more examples, easier for web developers
**Cons**: Much larger bundle size, higher memory usage, slower startup
**Why not chosen**: Bundle size and performance were critical for a desktop app that may run continuously

### Native Frameworks (Qt, GTK)

**Pros**: True native performance, no webview dependency
**Cons**: More complex UI development, less familiar to web developers, larger learning curve
**Why not chosen**: Team expertise in web technologies, need for rapid development

## References

- [Tauri Documentation](https://tauri.app/)
- [Tauri vs Electron Comparison](https://tauri.app/v1/guides/getting-started/comparison)
- Project setup: `docs/development/setup.md`

