# CareerBench

A self-hosted, AI-powered desktop application for managing your job search, generating tailored application materials, tracking interviews, and developing personalized learning plans.

## Features

- **User Profile Management**: Store your professional profile, experience, skills, education, and portfolio
- **Job Intake**: Capture and parse job descriptions with AI
- **Application Pipeline**: Track applications through stages (Saved → Applied → Interviewing → Offer)
- **Resume & Cover Letter Generator**: AI-powered generation tailored to specific jobs
- **Dashboard**: Visual overview of your job search metrics and activity
- **Local AI Caching**: Avoid repeated AI calls with intelligent caching

## Tech Stack

- **Frontend**: React + TypeScript + Vite
- **Backend**: Rust + Tauri
- **Database**: SQLite
- **AI**: Pluggable providers (OpenAI, Gemini, local LLM)

## Getting Started

### Quick Start

See the [Quick Start Guide](QUICKSTART.md) for detailed setup instructions.

### Prerequisites

- Node.js (v18 or later)
- Rust (latest stable)
- npm or yarn

### Installation

1. Install dependencies:
```bash
npm install
```

2. Build and run in development mode:
```bash
npm run tauri dev
```

3. Build for production:
```bash
npm run tauri build
```

For more details, see the [Development Setup Guide](docs/development/setup.md).

## Documentation

- **[Quick Start Guide](QUICKSTART.md)** - Get up and running quickly
- **[Documentation Index](docs/README.md)** - Complete documentation
- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute
- **[Troubleshooting](docs/guides/troubleshooting.md)** - Common issues and solutions

## Project Structure

```
CareerBench/
├── src/                    # React frontend
│   ├── components/         # React components
│   ├── pages/              # Page components
│   └── main.tsx            # Entry point
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── main.rs         # Tauri app entry
│   │   ├── db.rs           # Database setup and migrations
│   │   ├── ai_cache.rs     # AI caching layer
│   │   └── commands.rs     # Tauri commands
│   └── Cargo.toml          # Rust dependencies
├── docs/                   # Documentation
│   ├── guides/            # User guides
│   ├── development/       # Developer docs
│   └── specs/            # Feature specifications
└── scripts/                # Utility scripts
```

## Development Status

This is an MVP implementation. Current status:

- ✅ Project structure and basic setup
- ✅ Database schema and migrations
- ✅ AI caching layer
- ✅ Dashboard backend and frontend
- 🚧 User Profile module (backend ready, UI in progress)
- 🚧 Job Intake module (backend ready, UI in progress)
- 🚧 Application Pipeline (backend ready, UI in progress)
- 🚧 Resume & Cover Letter Generator (backend ready, UI in progress)

## Next Steps

1. Complete User Profile UI
2. Complete Job Intake UI
3. Implement AI provider integration
4. Complete Application Pipeline UI
5. Complete Resume Generator UI

## License

MIT

