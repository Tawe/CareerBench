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

- **Frontend**: React 18 + TypeScript + Vite
- **Backend**: Rust + Tauri
- **Database**: SQLite (local-first, no server required)
- **AI**: Pluggable providers (OpenAI, Anthropic, Local LLM via llama.cpp)
- **Styling**: CSS Modules with custom design system
- **Testing**: Vitest (frontend), Rust built-in (backend)

## Getting Started

### Prerequisites

Before you begin, ensure you have the following installed:

- **Node.js** (v18 or later) - [Download](https://nodejs.org/)
- **Rust** (latest stable) - [Install via rustup](https://rustup.rs/)
- **npm** or **yarn** (comes with Node.js)
- **Git** - [Download](https://git-scm.com/)

**Platform-specific requirements:**
- **macOS**: Xcode Command Line Tools (`xcode-select --install`)
- **Windows**: Microsoft Visual C++ Build Tools
- **Linux**: `libwebkit2gtk-4.0-dev`, `build-essential`, `curl`, `wget`, `libssl-dev`, `libgtk-3-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`

### Quick Installation

1. **Clone the repository:**
   ```bash
   git clone <repository-url>
   cd CareerBench
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode:**
   ```bash
   npm run tauri dev
   ```

   This will:
   - Install Rust dependencies (first time only)
   - Build the Rust backend
   - Start the Vite dev server
   - Launch the Tauri application

4. **Build for production:**
   ```bash
   npm run tauri build
   ```

   The built application will be in `src-tauri/target/release/`

### First Run

On first launch, CareerBench will:
- Create a local SQLite database in `src-tauri/.careerbench/careerbench.db`
- Initialize the database schema
- Set up default settings

### Next Steps

- **New to CareerBench?** → See the [Quick Start Guide](QUICKSTART.md)
- **Setting up AI?** → See [Model Setup Guide](docs/guides/model-setup.md)
- **Running into issues?** → See [Troubleshooting](docs/guides/troubleshooting.md)
- **Want to contribute?** → See [Contributing Guide](CONTRIBUTING.md)

## Documentation

### For Users

- **[Quick Start Guide](QUICKSTART.md)** - Get up and running quickly
- **[Troubleshooting](docs/guides/troubleshooting.md)** - Common issues and solutions
- **[Model Setup Guide](docs/guides/model-setup.md)** - Setting up local AI models

### For Developers

- **[Documentation Index](docs/README.md)** - Complete documentation
- **[Development Setup](docs/development/setup.md)** - Development environment setup
- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute
- **[Testing Guide](docs/development/testing.md)** - Testing standards
- **[Architecture Decision Records](docs/development/adr/)** - Key architectural decisions

### For Contributors

- **[Feature Specifications](docs/specs/features/)** - Detailed feature specs
- **[AI Provider Documentation](docs/specs/ai-provider.md)** - AI integration architecture
- **[Design Specifications](docs/specs/design/)** - UI/UX design documents

## Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:

- Code style and standards
- Testing requirements
- Pull request process
- Development workflow

## License

MIT License - see [LICENSE](LICENSE) file for details

## Development

### Development Workflow

1. **Start development server:**
   ```bash
   npm run tauri dev
   ```

2. **Run tests:**
   ```bash
   # Rust tests
   cd src-tauri && cargo test

   # TypeScript tests
   npm test
   ```

3. **Lint and format:**
   ```bash
   # TypeScript/React
   npm run lint
   npm run format

   # Rust
   cd src-tauri && cargo fmt && cargo clippy
   ```

4. **Populate test data:**
   ```bash
   ./scripts/populate_test_data.sh
   ```

### Project Structure

```
CareerBench/
├── src/                          # React frontend
│   ├── components/               # Reusable React components
│   ├── pages/                    # Page components (Dashboard, Jobs, etc.)
│   ├── hooks/                    # Custom React hooks
│   ├── utils/                    # Utility functions
│   └── main.tsx                  # Frontend entry point
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri app entry point
│   │   ├── commands.rs           # Tauri commands (API endpoints)
│   │   ├── db.rs                 # Database setup and migrations
│   │   ├── ai/                   # AI provider system
│   │   │   ├── provider.rs       # AI provider trait
│   │   │   ├── local_provider.rs # Local model integration
│   │   │   ├── cloud_provider.rs # Cloud API integration
│   │   │   └── hybrid_provider.rs # Hybrid mode routing
│   │   ├── ai_cache.rs           # AI response caching
│   │   └── resume_generator.rs   # Resume generation pipeline
│   └── Cargo.toml                # Rust dependencies
├── docs/                         # Documentation
│   ├── guides/                   # User guides
│   ├── development/              # Developer documentation
│   │   ├── setup.md             # Development setup
│   │   ├── testing.md           # Testing guide
│   │   ├── algorithms-and-data-flows.md # Complex algorithms
│   │   └── adr/                 # Architecture Decision Records
│   └── specs/                   # Feature specifications
└── scripts/                      # Utility scripts
```

### Key Technologies

- **Frontend**: React 18, TypeScript, Vite, React Router
- **Backend**: Rust, Tauri, SQLite (via rusqlite)
- **AI**: Pluggable providers (OpenAI, Anthropic, Local via llama.cpp)
- **Styling**: CSS Modules, custom design system
- **Testing**: Vitest (frontend), Rust built-in testing (backend)

### Development Resources

- **[Development Setup](docs/development/setup.md)** - Detailed setup instructions
- **[Testing Guide](docs/development/testing.md)** - Testing standards and practices
- **[Architecture Overview](docs/development/architecture.md)** - System architecture
- **[Algorithms & Data Flows](docs/development/algorithms-and-data-flows.md)** - Complex algorithms documentation
- **[Architecture Decision Records](docs/development/adr/)** - Key architectural decisions

## Project Status

CareerBench is in active development. Current status:

### ✅ Completed

- Core architecture and project setup
- Database schema and migrations
- AI provider system (Local, Cloud, Hybrid modes)
- Multi-level AI caching system
- Resume generation pipeline
- Cover letter generation
- Job parsing with AI
- Dashboard with visualizations
- User profile management
- Application tracking
- Testing infrastructure

### 🚧 In Progress

- Additional AI providers (as needed)
- Performance optimizations
- Enhanced UI/UX features

### 📋 Planned

- Learning plan generator
- Email integration
- Calendar integration
- Advanced analytics

See the [TODO List](docs/todo.md) for detailed task tracking.

