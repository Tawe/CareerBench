# AI Provider Architecture Specification

## Overview

CareerBench supports **local AI inference** by default while also allowing users to optionally configure **their own API keys** for cloud-based LLMs (e.g., OpenAI, Anthropic). The system uses a unified abstraction (`AIProvider`) so the rest of the application does not depend on implementation details.

This document describes:
- Local model integration
- Cloud provider integration (user API keys)
- Provider resolution logic
- Tauri command design
- Frontend integration
- Validation and guardrails
- Testing strategy

---

## 1. AI Provider Interface

All AI functionality (resume suggestions, cover letter generation, skill suggestions) goes through a single abstraction.

### **AI Provider Interface**

interface AIProvider {
generateResumeSuggestions(input: ResumeInput): Promise<ResumeSuggestions>;
generateCoverLetter(input: CoverLetterInput): Promise<CoverLetter>;
generateSkillSuggestions(input: SkillSuggestionsInput): Promise<SkillSuggestions>;
}

### **Core Types**

These types live in `src/ai/types.ts`.
- `ResumeInput`
- `ResumeSuggestions`
- `CoverLetterInput`
- `CoverLetter`
- `SkillSuggestionsInput`
- `SkillSuggestions`

These form the contract for **both local and cloud providers**.

---

## 2. Local Model Architecture (Default)

CareerBench ships with a small local model (e.g., llama.cpp or candle-based). This model is:
- Bundled within the Tauri binary or as a local asset
- Called via Rust    
- Returns strictly validated JSON outputs

### **Local Provider (Rust)**

File: `src-tauri/src/ai/local_provider.rs`

Responsibilities:
- Load or reference the local model
- Format the structured prompt
- Run inference
- Parse JSON output
- Return typed results

Local provider is used when:
- User selects "Local" mode in settings (default)
- User has no cloud API key configured

---

## 3. Cloud Provider Architecture (Optional, User-Configured)

Users can bring their own API keys. Supported providers initially:
- OpenAI

### **Cloud Provider (Rust)**
File: `src-tauri/src/ai/cloud_provider.rs`

Responsibilities:
- Validate API key presence
- Call cloud LLM APIs
- Inject strict system prompts (no fabrication, JSON-only responses)
- Parse and validate outputs

### **User Settings**

Settings are stored securely using Tauri’s secure storage or encrypted file storage.
mode: "local" | "cloud" | "hybrid"
cloudProvider: "openai"
apiKey: string
modelName: string

The Settings UI allows users to:
- Switch between Local / Cloud modes
- Enter their own API key
- Select model
- Test connection

---

## 4. Provider Resolution

The Tauri backend decides which provider to use.

### **resolve_provider() Logic**

Pseudo-code:

```python
if settings.mode == "cloud" and apiKey exists:
return CloudProvider
else:
return LocalProvider
```

Hybrid mode may route specific tasks to local or cloud models.

---

## 5. Tauri Commands

Each AI task is exposed to the frontend via Tauri commands.
ai_resume_suggestions(input: ResumeInput) => ResumeSuggestions
ai_cover_letter(input: CoverLetterInput) => CoverLetter
ai_skill_suggestions(input: SkillSuggestionsInput) => SkillSuggestions

These commands:
- Call `resolve_provider()`
- Run the appropriate provider method
- Validate and return typed results

---

## 6. Frontend Integration

Frontend does **not** know if the model is local or cloud. It simply calls:

invoke("ai_resume_suggestions", { input })

### Key UI Principles

- AI never auto-edits the user’s resume
- Suggestions appear in a separate panel
- User explicitly chooses when to generate suggestions
- AI results must be manually applied

---

## 7. Guardrails & Validation

Regardless of provider:
- AI output must be valid JSON matching the defined schemas
- Outputs must not contain fabricated work experience    
- Errors must be clear (“The AI response was invalid. Try again.”)

Validation strategies:
- `serde_json` on Rust side
- Zod (or TS validation) on frontend if needed

---

## 8. Local + Cloud Prompt Strategy

Both providers use the same system-level guidelines:
- No invention of skills, companies, or dates
- Only reorganize or rephrase existing data
- JSON output only
- Deterministic format for each task
    
Prompts differ only in:
- Formatting for local model tokenization
- Cloud provider API format

---

## 9. Testing Strategy

Testing focuses on safety, not creative output.

### **Unit Tests**
- Schema validation tests
- Provider routing tests
- Tauri command tests

### **Contract Tests**
- Given known input → output is valid JSON
- No invented fields

### **Local Model Stability Tests**
- Snapshot tests if model is deterministic enough

---

## 10. Summary

This architecture:
- Provides a robust local-first AI experience
- Allows power users to plug in their own LLMs
- Maintains strict output contracts via shared types
- Ensures safe, predictable behavior through validation and prompts
- Cleanly separates frontend from provider details
    

CareerBench can now support both offline privacy-friendly workflows and advanced cloud-backed ones without changing any UI logic.