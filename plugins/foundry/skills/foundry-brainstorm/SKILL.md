---
name: foundry:brainstorm
description: "Collaboratively define a specification through structured dialogue with the user. Use when starting a new feature, exploring requirements, or when the user says 'brainstorm', 'let's think about', 'what should we build', 'spec this out'. Produces a specification document ready for NLSpec derivation."
argument-hint: "[feature description or research context path]"
---

# Foundry Brainstorm

Collaboratively define a specification through structured dialogue. Produces a spec document that feeds into `foundry:nlspec`.

## Interaction Method

Use the platform's question tool when available. Ask one question at a time. Prefer concise single-select choices when natural options exist.

## Input

Either:
- A feature description from the user
- A path to a research context document (from `foundry:research`)
- Both

If a research context exists, read it first and use it to inform the dialogue.

## Workflow

### Phase 1: Understand

1. **Read context** — If research doc exists, load it. If the codebase hasn't been investigated, recommend `foundry:research` first (but don't block — the user may know enough).

2. **Frame the problem** — In 2-3 sentences, reflect back what you understand the user wants. Ask: "Is this right, or should I adjust?"

3. **Ask clarifying questions** — One at a time. Focus on:
   - What behavior is expected (not how to implement it)
   - Who/what are the actors
   - What are the boundaries (what's in scope, what's not)
   - What are the success criteria
   - What constraints exist

4. **Pressure-test assumptions** — Challenge anything that seems assumed but not stated. Surface hidden complexity. Ask "what happens when X fails?" for each critical path.

### Phase 2: Explore Approaches

1. **Propose 2-3 concrete approaches** — Each with trade-offs. Don't present a "correct" answer; present genuine alternatives.

2. **Let the user choose** — Or combine elements from multiple approaches.

3. **Resolve key decisions** — For each decision point, capture:
   - The decision made
   - The rationale
   - What was rejected and why

### Phase 3: Define the Specification

Write a specification document to `docs/specs/YYYY-MM-DD-<topic>-spec.md`:

```markdown
---
date: YYYY-MM-DD
topic: <topic>
status: active
research: docs/research/YYYY-MM-DD-<topic>-research.md  # if exists
---

# Specification: <Topic>

## Problem Statement
[Three-part: status quo, pain, solution]

## Actors and Boundaries
[Who/what interacts, where the system boundary lies]

## Requirements
- R1. [Concrete, testable requirement]
- R2. [...]

## Behaviors
[For each major behavior:]
### Behavior: <Name>
- **Trigger:** What initiates this behavior
- **Input:** What it receives
- **Process:** What happens (high-level, not implementation)
- **Output:** What it produces
- **Errors:** What can go wrong and how it's handled

## Key Decisions
- **Decision:** [What was decided]
  - **Rationale:** [Why]
  - **Rejected:** [What was rejected and why]

## Scope Boundaries
- **In scope:** [...]
- **Out of scope:** [...]
- **Future:** [Deferred items with known extension points]

## Success Criteria
- [Observable, testable criterion]

## Open Questions
### Resolved
- [Question → Answer]

### Deferred
- [Question → Why deferred, when to resolve]
```

### Phase 4: Review

Before finalizing, re-read the spec and verify:
- Every requirement is testable (could you write a checkbox for it?)
- Every decision has rationale
- Scope boundaries are explicit
- No hidden assumptions

If the spec has 5+ requirements or touches high-risk areas, spawn a reviewer subagent:

```
Agent(subagent_type="general-purpose", prompt="Review this specification for completeness,
ambiguity, and missing edge cases. The spec is at [path]. Report gaps as concrete questions
the author should answer, not vague suggestions.")
```

Present the reviewer's findings to the user. Iterate if needed.

### Phase 5: Handoff

Present options:
1. **Derive NLSpec** — `/foundry:nlspec` from this spec
2. **Revise** — continue refining
3. **Done** — spec stands alone
