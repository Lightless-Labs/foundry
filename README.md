# Foundry

Foundry is an adversarial red/green development workflow for AI-assisted software engineering.

It helps you move from an idea to an implemented feature by composing five agent skills:

1. **Research** the codebase, domain, and relevant constraints.
2. **Brainstorm** requirements with the user.
3. **Derive an NLSpec** with explicit Why / What / How / Done sections.
4. **Write tests adversarially** with a red team that sees the full specification.
5. **Implement behind a barrier** with a green team that sees only implementation guidance and opaque PASS/FAIL labels.

The plugin is designed for workflows where agents can be useful without becoming an unstructured conversation blob. Foundry preserves durable artifacts, role separation, review checkpoints, and replayable dispatch boundaries.

## Key opinions

Foundry is built around a few strong opinions:

- **Specs should become executable pressure.** Requirements are not done when they sound good; they should drive tests, review, and implementation behavior.
- **Test authors and implementers should be isolated.** The red team writes tests from the spec. The green team implements from the NLSpec How section and opaque outcomes only.
- **Information barriers are a feature, not ceremony.** Green should not see red assertions, expected values, stack traces, hidden screenshots, diffs, or comparator rationale.
- **The orchestrator must preserve provenance.** If something fails, route fixes through the right role instead of silently patching across the barrier.
- **Artifacts should be replayable.** PromptEnvelope records make dispatches auditable across Claude, Pi, and other harnesses.
- **Golden vectors beat convention arguments.** For stateful or convention-heavy domains, concrete reference inputs/outputs are mandatory.
- **Optional heavy lanes should stay optional.** Live model dispatches and browser/device capture smokes are valuable, but fast validation remains dependency-light.

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
