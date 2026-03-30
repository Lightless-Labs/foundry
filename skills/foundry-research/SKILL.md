---
name: foundry:research
description: "Investigate a codebase, its documentation, and relevant external resources before specifying work. Use when starting a new feature, exploring unfamiliar code, or preparing context for foundry:brainstorm. Triggers on 'research this', 'investigate', 'explore the codebase', 'what do we have', 'understand the project'."
argument-hint: "[topic or area to investigate]"
---

# Foundry Research

Produce a comprehensive research context for a topic or area of a codebase. This context feeds into `foundry:brainstorm` or can stand alone as a codebase exploration artifact.

## Interaction Method

Use the platform's question tool when available (AskUserQuestion in Claude Code). Ask one question at a time.

## Input

The user provides a topic, area, or question. If empty, ask: "What area or topic should I investigate?"

## Workflow

### Phase 1: Local Codebase Investigation

Run these in parallel:

1. **Structure scan** — Glob for key files: manifests (Cargo.toml, package.json, go.mod, etc.), README, CLAUDE.md, AGENTS.md, WORKFLOW.md. Map the directory tree at depth 3. Identify language, build system, test framework.

2. **Relevant code search** — Use ast-grep or grep for patterns, types, functions, and modules related to the topic. Read the most relevant files. Note architectural patterns, conventions, and existing implementations to follow.

3. **Documentation scan** — Read all docs/ subdirectories (plans, brainstorms, solutions). Read any project-specific documentation files. Extract relevant decisions, learnings, and prior art.

4. **Test scan** — Find test files related to the topic. Understand existing test patterns and coverage.

### Phase 2: External Research

Decide whether external research adds value:

**Research when:**
- The topic involves unfamiliar libraries or frameworks
- The codebase has thin local patterns (fewer than 3 examples)
- The topic is high-risk (security, payments, data migrations)
- The user is exploring new territory

**Skip when:**
- Strong local patterns exist
- The user already knows the approach
- The topic is well-covered by local docs

When researching externally:
- Use web search for current best practices and community patterns
- Use context7 (if available) for library/framework documentation
- Fetch specific URLs the user provides

### Phase 3: Consolidate

Write a research context document to `docs/research/YYYY-MM-DD-<topic>-research.md` containing:

```markdown
---
date: YYYY-MM-DD
topic: <topic>
---

# Research: <Topic>

## Codebase Context
- Language, build system, key frameworks
- Relevant modules, files, and patterns
- Existing conventions to follow

## Existing Work
- Related plans, brainstorms, solutions
- Prior decisions and their rationale

## Relevant Code
- Key types, traits, functions (with file paths)
- Patterns to follow or extend

## External References
- Library docs, best practices, community patterns
- Relevant articles or specifications

## Test Landscape
- Existing test patterns and coverage
- Testing frameworks and conventions

## Open Questions
- Gaps discovered during research
- Areas needing clarification
```

### Phase 4: Handoff

Present options:
1. **Start brainstorm** — `/foundry:brainstorm` with this research context
2. **Continue investigating** — dig deeper into a specific area
3. **Done** — research stands alone
