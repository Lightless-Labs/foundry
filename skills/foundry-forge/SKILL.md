---
name: foundry:forge
description: "Full adversarial development pipeline: research, brainstorm, NLSpec, then red/green implementation. Use when starting a feature from scratch and you want the complete Foundry workflow. Triggers on 'forge', 'foundry forge', 'full pipeline', 'build this adversarially from scratch'."
argument-hint: "[feature description]"
---

# Foundry Forge

The complete Foundry pipeline: from idea to adversarially-implemented feature.

```
research -> brainstorm -> nlspec -> adversarial
```

Each phase produces an artifact that feeds the next. Each artifact is reviewed before advancing.

## Input

A feature description from the user. Can be vague ("add authentication") or specific ("implement JWT-based auth with refresh tokens and session invalidation").

## Pipeline

### Step 1: Research

Run `/foundry:research` with the feature description.

**Gate:** Research context document exists at `docs/research/`.

### Step 2: Brainstorm

Run `/foundry:brainstorm` with the research context and feature description.

**Gate:** Spec document exists at `docs/specs/` with status: active.

### Step 3: NLSpec

Run `/foundry:nlspec` with the spec document.

**Gate:** NLSpec document exists at `docs/nlspecs/` with status: reviewed.

### Step 4: Adversarial Implementation

Run `/foundry:adversarial` with the reviewed NLSpec.

**Gate:** All tests pass, both reviewers approve, NLSpec status: implemented.

### Step 5: Commit and Report

1. Commit the implementation and tests on the feature branch
2. Report:
   - Feature summary
   - NLSpec DoD coverage (which items are implemented)
   - Test count and pass rate
   - Iteration count (how many red/green cycles)
   - Files created/modified
3. Offer to create a PR

## Skipping Steps

Each step can be skipped if its artifact already exists:
- Research context exists → skip to brainstorm
- Spec exists → skip to nlspec
- NLSpec exists → skip to adversarial

The user can also enter at any point: `/foundry:adversarial path/to/nlspec.md`

## Failure Modes

| Failure | Response |
|---------|----------|
| Research finds nothing relevant | Ask user for more context, or proceed with brainstorm using user knowledge only |
| Brainstorm can't converge on spec | The spec has open questions — resolve them or defer and note in the NLSpec |
| NLSpec review fails repeatedly | The spec may be ambiguous — return to brainstorm to clarify |
| Green team can't pass tests after limit | Pause — ask user to arbitrate. May need to revise NLSpec or tests |
| Red reviewer rejects repeatedly | Tests may be too strict or wrong — ask user to inspect |
