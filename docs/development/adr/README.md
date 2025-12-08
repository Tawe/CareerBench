# Architecture Decision Records (ADRs)

This directory contains Architecture Decision Records (ADRs) for CareerBench. ADRs document important architectural decisions, the context in which they were made, and their consequences.

## What are ADRs?

Architecture Decision Records are documents that capture:
- **What** decision was made
- **Why** it was made (context and constraints)
- **Consequences** (positive and negative)
- **Alternatives** that were considered

ADRs help:
- New team members understand why things are built the way they are
- Future developers make informed decisions about changes
- Document the evolution of the architecture
- Avoid repeating past discussions

## ADR Format

Each ADR follows this structure:

```markdown
# ADR-XXXX: Title

## Status
[Proposed | Accepted | Deprecated | Superseded]

## Context
[Background and constraints that led to this decision]

## Decision
[What we decided to do]

## Consequences
### Positive
[Benefits of this decision]

### Negative
[Drawbacks and trade-offs]

### Mitigations
[How we address the negatives]

## Alternatives Considered
[Other options we evaluated and why we didn't choose them]

## References
[Links to related documentation, code, or discussions]
```

## ADR Index

### Accepted ADRs

- **[ADR-0001: Use Tauri for Desktop Application Framework](0001-use-tauri-for-desktop-app.md)**
  - Decision to use Tauri instead of Electron or native frameworks
  - Status: Accepted

- **[ADR-0002: Pluggable AI Provider Architecture](0002-pluggable-ai-provider-architecture.md)**
  - Decision to use trait-based architecture for AI providers
  - Status: Accepted

- **[ADR-0003: Multi-Level AI Caching Strategy](0003-multi-level-ai-caching.md)**
  - Decision to implement caching at multiple stages of AI pipeline
  - Status: Accepted

- **[ADR-0004: Hybrid Mode Routing with Automatic Fallback](0004-hybrid-mode-routing.md)**
  - Decision to implement automatic fallback between cloud and local providers
  - Status: Accepted

## Adding New ADRs

1. **Number**: Use next sequential number (e.g., `0005-...`)
2. **Title**: Descriptive, concise title
3. **Status**: Start with "Proposed", update to "Accepted" after review
4. **Content**: Follow the ADR format above
5. **Update Index**: Add new ADR to this README

## ADR Lifecycle

1. **Proposed**: Initial draft, open for discussion
2. **Accepted**: Decision finalized, implementation can proceed
3. **Deprecated**: Decision is no longer recommended but still in use
4. **Superseded**: Decision replaced by a newer ADR

## When to Create an ADR

Create an ADR when:
- Making a significant architectural decision
- Choosing between multiple viable alternatives
- The decision affects multiple parts of the system
- The decision is hard to reverse
- Future developers will benefit from understanding the rationale

Don't create an ADR for:
- Minor implementation details
- Obvious choices with no alternatives
- Temporary workarounds
- Decisions that are well-documented elsewhere

## References

- [ADR Template by Michael Nygard](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions)
- [ADR GitHub Organization](https://adr.github.io/)
- Project Architecture: `docs/development/architecture.md`
- Algorithms Documentation: `docs/development/algorithms-and-data-flows.md`

