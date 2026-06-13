# Lightless Labs Foundry Public Plugin

Foundry is the public plugin layer for an adversarial red/green development workflow. This checkout contains Claude plugin metadata, skills, agents, examples, validation scripts, durable docs, and todos.

The Rust execution engine/state machine lives in the private engine repo at `lightless-labs/lightless-labs/foundry/`. Do not assume Rust+Bazel product work belongs in this public repo unless you are explicitly working on an example or documentation that references the engine.

## Repository Identity

This repo is the installable Foundry plugin:

```bash
claude plugin marketplace add github:Lightless-Labs/foundry
claude plugin install foundry
```

It provides:

- `plugins/foundry/skills/` — user-facing workflow skills (`research`, `brainstorm`, `nlspec`, `adversarial`, `forge`)
- `plugins/foundry/agents/` — reviewer, research, language, and adversarial-process agents
- `.claude-plugin/` and `plugins/foundry/.claude-plugin/` — marketplace/plugin metadata
- `examples/` — worked examples that demonstrate the method and preserve artifacts
- `tests/validate-public-plugin.sh` — fast aggregate validation for public plugin checks
- `tests/validate-agents.sh` — structural validation for agent prompt files
- `docs/` and `todos/` — plans, learnings, handoff state, and pending work

## Current Operating Context

Read `docs/HANDOFF.md` at the start of each session. It is the source of truth for:

- current skill/agent counts
- validation status
- worked example results
- open todos and suggested next steps
- information-barrier invariants
- links to private engine docs when engine work is required

## Technical Stack

This repo is primarily Markdown/YAML plugin content plus shell validation.

### Plugin surface

- Skills are Markdown files with executable orchestration instructions.
- Agents are Markdown files with YAML frontmatter and strict output schemas.
- Plugin metadata is JSON under `.claude-plugin/` and `plugins/foundry/.claude-plugin/`.
- Validation is shell-based via `tests/validate-public-plugin.sh` for the fast aggregate suite and targeted `tests/validate-*.sh` scripts for focused checks.

### Examples

Worked examples may contain Rust crates and generated build outputs. Keep build artifacts out of git. Do not treat examples as the main product surface; they exist to teach and regression-test the workflow.

### Engine split

Engine/state-machine changes belong in the private engine repo, not here. Public plugin changes may document engine expectations, but should not invent engine APIs without cross-checking the engine handoff.

## Quick Commands

```bash
# Fast aggregate validation (excludes slow/live model lanes)
tests/validate-public-plugin.sh

# Structural validation for plugin agents
tests/validate-agents.sh

# Replayable PromptEnvelope barrier validation
tests/validate-barrier-envelopes.sh

# Inspect current repo state
git status --short --branch

# Compare local branch with remote
git rev-list --left-right --count main...origin/main
```

## Key Directories

- `plugins/foundry/skills/` — Foundry workflow skills
- `plugins/foundry/agents/` — dispatchable review/research agents
- `tests/` — validation scripts
- `examples/` — worked examples and case studies
- `docs/HANDOFF.md` — session handoff and current state
- `docs/solutions/` — documented learnings
- `docs/plans/` — implementation plans
- `docs/research/`, `docs/specs/`, `docs/nlspecs/` — durable phase artifacts
- `todos/` — pending work items

## Core Invariant

The information barrier is the central property of Foundry:

- Red team sees the NLSpec/spec and writes tests; it must not see implementation code.
- Green team sees only the NLSpec How section and test outcome labels; it must not see red test code, assertions, error messages, or NLSpec Done criteria.
- The orchestrator may see everything but must preserve provenance and route fixes through the proper team.

When editing skills or agents, preserve this invariant explicitly. Prefer mechanical checks and replayable artifacts over prose-only discipline.

## Process

- Use sub-agents for each task when available. Parallelize independent research/validation work.
- Favor small, atomic commits.
- Use a TDD/check-driven approach: run `tests/validate-agents.sh` after skill/agent changes.
- When picking up a milestone from a roadmap or general plan, create a dedicated plan if one does not already exist.
- When a plan is deepened, reviewed, completed, or amended, update its header with the date and reason.
- When a gap is discovered during execution, update the relevant plan or todo with an addendum.
- Update `docs/HANDOFF.md` before context compaction or at natural milestones.

## Conventions

- Favor `ast-grep` over grep when researching code structures. For Markdown/plugin content, text search is fine.
- Keep generated artifacts, build outputs, and local logs out of the working tree.
- Keep repo identity clear: public plugin work here; engine implementation work in the private engine repo.
- Preserve attribution comments for adopted agents.
- Keep agent frontmatter fields and output schemas compatible with `tests/validate-agents.sh`.

### Git

Set `SSH_AUTH_SOCK` before any git operation requiring authentication (push, pull, fetch, clone):

```bash
export SSH_AUTH_SOCK=~/.ssh/agent.sock
```
