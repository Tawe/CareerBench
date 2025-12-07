# Documentation Standards

This document defines the standards and conventions for all documentation in the CareerBench project.

## File Organization

### Structure

```
docs/
├── README.md                    # Documentation index
├── documentation-standards.md   # This file
├── security.md                  # Security documentation
├── todo.md                      # Project TODO list
├── guides/                      # User-facing guides
│   ├── troubleshooting.md
│   ├── model-setup.md
│   └── local-ai-troubleshooting.md
├── development/                 # Developer documentation
│   ├── setup.md
│   ├── testing.md
│   └── architecture.md
├── specs/                       # Feature specifications
│   ├── overview.md
│   ├── features/
│   ├── ai-provider.md
│   └── design/
└── reference/                   # API reference (future)
```

### Root Level Files

- `README.md` - Main project README
- `QUICKSTART.md` - Quick start guide
- `CONTRIBUTING.md` - Contribution guidelines

## Format Standards

### File Naming

**Standard:** All documentation files use **lowercase with hyphens**

- ✅ Correct: `troubleshooting.md`, `model-setup.md`, `local-ai-troubleshooting.md`
- ❌ Incorrect: `TROUBLESHOOTING.md`, `Model_Setup.md`, `localAITroubleshooting.md`

**Exceptions:**
- `README.md` - Standard convention (capitalized)
- Special files like `CHANGELOG.md` or `LICENSE.md` if needed

**Guidelines:**
- Be descriptive and clear
- Use hyphens to separate words
- Avoid abbreviations unless widely understood
- Keep names concise but meaningful

### Headers

- Use `#` for main title (H1)
- Use `##` for major sections (H2)
- Use `###` for subsections (H3)
- Use `####` for sub-subsections (H4)
- Maintain consistent hierarchy

### Structure Template

```markdown
# Document Title

Brief one-sentence description of what this document covers.

## Table of Contents

- [Section 1](#section-1)
- [Section 2](#section-2)

---

## Section 1

Content here...

### Subsection

More content...

---

## Section 2

Content here...

---

## Related Documentation

- [Link to related doc](path/to/doc.md)
- [Another related doc](path/to/another.md)
```

### Code Blocks

- Always specify language: ` ```bash `, ` ```rust `, ` ```typescript `
- Include context and comments where helpful
- Show both correct and incorrect examples when relevant

### Cross-References

- Use relative paths: `[Link Text](../path/to/doc.md)`
- Keep links up to date when moving files
- Test links periodically

### Status Indicators

Use emoji for status (optional but consistent):
- ✅ Complete
- 🚧 In Progress
- ❌ Not Started / Blocked
- ⚠️ Warning / Important Note

## Content Standards

### Clarity

- Write clearly and concisely
- Use active voice when possible
- Avoid jargon unless necessary
- Define acronyms on first use

### Completeness

- Include all necessary information
- Provide examples where helpful
- Link to related documentation
- Include troubleshooting when relevant

### Accuracy

- Keep documentation up to date
- Update when code changes
- Verify examples work
- Test commands before documenting

## Style Guide

### Tone

- Professional but approachable
- Direct and helpful
- Avoid overly casual language
- Be inclusive and welcoming

### Formatting

- Use lists for multiple items
- Use tables for structured data
- Use code blocks for commands/code
- Use blockquotes for important notes

### Examples

**Good:**
```markdown
## Installation

To install CareerBench:

1. Clone the repository
2. Install dependencies: `npm install`
3. Run the app: `npm run tauri dev`
```

**Bad:**
```markdown
## Installation

Clone it, install stuff, run it.
```

## Maintenance

### When to Update

- When adding new features
- When changing existing functionality
- When fixing bugs that affect usage
- When receiving feedback about unclear docs

### Review Process

- Review documentation in PRs
- Update related docs when making changes
- Keep cross-references accurate
- Remove outdated information

## Tools

### Markdown Linting

Consider using markdown linters:
- `markdownlint` for VS Code
- `remark` for command-line checking

### Link Checking

Periodically check for broken links:
- Use tools like `markdown-link-check`
- Test links manually during reviews

## Examples

See these files for good examples:
- `docs/guides/troubleshooting.md` - Well-structured guide
- `docs/development/setup.md` - Clear setup instructions
- `QUICKSTART.md` - Concise quick start

## Questions?

If you're unsure about documentation standards:
1. Check existing documentation for examples
2. Follow the structure of similar documents
3. Ask in PR reviews
4. Update this guide if you find better patterns

