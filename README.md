# Foundry

Foundry is Lightless Labs' public plugin layer for an adversarial red/green development workflow.

It packages research, specification, NLSpec derivation, and implementation workflows as installable agent skills, with strict information barriers between test-writing and implementation agents.

## What this repository contains

This repository is the **public plugin** surface:

- `plugins/foundry/skills/` — canonical Foundry workflow skills.
- `plugins/foundry/agents/` — reviewer, research, language, and adversarial-process agents.
- `skills/` — thin Agent Skills adapters for Pi/Codex-style loaders.
- `extensions/pi-foundry-team/` — Pi `foundry_team` PromptEnvelope child-dispatch extension.
- `.claude-plugin/`, `.codex-plugin/` — plugin metadata.
- `examples/` — worked examples and adversarial UI spike fixtures.
- `tests/` — structural, barrier, package, eval, and fixture validators.
- `docs/`, `todos/` — plans, handoff state, learnings, and pending work.

The Rust execution engine/state machine lives in the private Lightless Labs monorepo, not here.

## Install

### Claude plugin

```bash
claude plugin marketplace add github:Lightless-Labs/foundry
claude plugin install foundry
```

### Pi package

This repo is installable as a Pi package and exposes:

- the `foundry_team` extension,
- Agent Skills adapters under `skills/`.

The Pi extension dispatches replayable PromptEnvelope artifacts to isolated child `pi` processes; Pi has no built-in subagent primitive.

### Codex plugin

Codex metadata lives in `.codex-plugin/plugin.json`. The Codex integration exposes the same root skill adapters and command wrappers, while canonical agent prompts remain under `plugins/foundry/agents/` until Codex has a PromptEnvelope-safe dispatch primitive.

## Skills

Foundry provides five composable skills:

| Skill | Use when | Output |
| --- | --- | --- |
| `foundry-research` | Investigating a codebase or domain | Research context |
| `foundry-brainstorm` | Shaping requirements through dialogue | Spec document |
| `foundry-nlspec` | Deriving an implementation-ready NLSpec | Why/What/How/Done NLSpec |
| `foundry-adversarial` | Building from a reviewed NLSpec | Isolated red tests + green implementation |
| `foundry-forge` | Running the full pipeline | Research → spec → NLSpec → adversarial implementation |

## Core invariant: the information barrier

Foundry's central property is the separation between adversarial roles:

| Entity | Sees | Must not see |
| --- | --- | --- |
| Red team | Full spec/NLSpec | Implementation code |
| Green team | NLSpec How section and opaque PASS/FAIL labels | Red test code, assertions, error messages, NLSpec Done criteria |
| Arbiter | One scoped disputed test, relevant snippet, spec/NLSpec, one runner result | Full red suite, broad implementation, broad conversations |
| Orchestrator | Everything | Must preserve provenance and route fixes through the right team |

Green receives only labels such as:

```text
T-042: FAIL
T-043: PASS
```

No hidden assertions, expected values, stack traces, screenshot paths, screenshot hashes, visual diffs, or comparator rationale should be sent to green.

## Validate

Fast aggregate validation:

```bash
tests/validate-public-plugin.sh
# or
npm run validate
```

Useful targeted checks:

```bash
tests/validate-agents.sh
tests/validate-barrier-envelopes.sh
tests/foundry-evals.sh
tests/validate-pi-extension.sh
tests/validate-codex-plugin.sh
```

Optional/manual lanes are intentionally excluded from fast validation, including live Pi dispatches and Playwright browser screenshot capture.

## Worked examples

- `examples/sudoku-solver/` — constraint-solving baseline.
- `examples/rubiks-solver/` — golden-vector convention repair case study.
- `examples/chess-engine/` — perft/golden-vector convention anchoring.
- `examples/adversarial-ui-design-system/` — UI design-system spike with capture contracts, visual controls, WebKit thumbnails, and optional Playwright viewport smoke.

## Current state

Read `docs/HANDOFF.md` for the latest validation status, live-run notes, open todos, and suggested next steps.
