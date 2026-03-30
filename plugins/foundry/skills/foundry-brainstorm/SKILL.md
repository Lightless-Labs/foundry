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

## Mid-Dialogue Research

Research is not a phase — it's a reflex. After every user reply, evaluate whether new unknowns have surfaced that you can't answer from existing context.

**Trigger research when the user's reply:**
- Names a technology, library, pattern, or standard you don't have enough context on
- Reveals a constraint that changes the problem shape (e.g., "this runs on embedded" or "we need FIPS compliance")
- References existing code, files, or systems you haven't read
- Raises a question where the answer lives in the codebase, docs, or web — not in your training data
- Contradicts or complicates an assumption you were working with

**How to research mid-dialogue:**
1. Tell the user what you're checking and why (one line, not a speech)
2. Spawn a background research subagent or run targeted searches directly:
   - Codebase: grep/glob/read for the specific thing referenced
   - Docs: check docs/, README, CLAUDE.md for relevant context
   - Web: search for the specific technology, standard, or pattern
   - Context7: fetch library docs if a specific framework/library was named
3. Fold findings into your next question or response — don't dump raw research
4. If findings change your understanding, say so: "That changes things — [specific thing] means we need to reconsider [specific aspect]."

**Don't research when:**
- The user is making a preference/design choice (no fact to verify)
- You already have enough context to ask a good follow-up question
- The topic is purely about product behavior, not technical feasibility

The goal: never ask the user a question you could have answered yourself with a 10-second search. Never let a false assumption survive past the reply that contradicted it.

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

   After each user reply, run the mid-dialogue research check. If the reply surfaces something concrete to investigate, research it before asking your next question.

4. **Pressure-test assumptions** — Challenge anything that seems assumed but not stated. Surface hidden complexity. Ask "what happens when X fails?" for each critical path. Use targeted research to ground your challenges in facts (e.g., "I checked and the library doesn't support X" rather than "have you considered whether the library supports X?").

### Phase 2: Explore Approaches

1. **Propose 2-3 concrete approaches** — Each with trade-offs. Don't present a "correct" answer; present genuine alternatives. If an approach depends on a library or pattern you're unsure about, research it before presenting — don't propose something that turns out to be infeasible.

2. **Let the user choose** — Or combine elements from multiple approaches. If their choice raises new feasibility questions, research before proceeding.

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
