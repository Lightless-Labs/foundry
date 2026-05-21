---
date: 2026-05-21
type: feat
status: completed
completed: 2026-05-21
---

# feat: Behavioral Smoke Tests

**Completed:** 2026-05-21 — added `tests/behavioral-smoke.sh`, a replay-level run harness that validates PromptEnvelope artifacts, TOON run summaries, example pass rates, model lanes, and divergence restart counts. Enhanced the public adversarial skill so real plugin runs emit `runs/<run_id>/behavioral-smoke.toon` and validate it before claiming completion.

**Addendum:** 2026-05-21 — clarified that this repo owns the public plugin/extension live lane, not private runtime or BuildKite dispatcher changes. Added `tests/validate-behavioral-smoke-contract.sh` to lock the adversarial skill's run-summary contract.

**Plan for:** adding replayable behavioral smoke coverage for Foundry's adversarial workflow, including barrier invariants, divergence restarts, model-lane assertions, worked-example pass-rate checks, and a TOON run-summary investigation.

## Problem Frame

`tests/validate-agents.sh` verifies plugin structure. `tests/validate-barrier-envelopes.sh` verifies individual PromptEnvelope artifacts. The missing layer is a run-level behavioral smoke harness that can answer: did the workflow preserve the red/green barrier across a run, did divergence routing restart when expected, did provider/model lanes match the plan, and did worked examples retain their expected outcomes?

The public plugin repo cannot own external infrastructure, so the useful slice is to make the **public skill itself** emit replayable run artifacts and validate those artifacts. That gives CI and humans a deterministic gate now, and any live Claude/Pi/Codex execution of the public plugin can produce the same `runs/<run_id>/` contract.

## TOON Research Note

TOON (Token-Oriented Object Notation, <https://github.com/toon-format/toon>) is a compact, human-readable encoding of the JSON data model for LLM input. It combines YAML-like indentation with CSV-style tabular arrays:

```toon
model_lanes[2]{recipient,planned_model,actual_model}:
  red-team,gemini,gemini
  green-team,codex,codex
```

Findings relevant to Foundry:

- Good fit for LLM-facing run summaries with uniform arrays: model lanes, test outcomes, divergence restarts, artifact index rows.
- The `[N]` row count and `{fields}` header are useful behavioral smoke guardrails because they make truncation/shape drift visible.
- Not a good replacement for canonical `PromptEnvelope` artifacts yet: JSON is already easy to mechanically validate, broadly supported, and preserves exact prompt strings without adding a custom parser dependency.
- Recommended use in this slice: support a tiny, dependency-free TOON subset for `foundry.behavioral-smoke.v1` run manifests, while keeping envelopes in JSON.

## Scope

### In scope

- Add `tests/behavioral-smoke.sh`.
- Validate run directories containing:
  - `dispatch/**/*.json` PromptEnvelope artifacts;
  - `behavioral-smoke.toon` run summary manifests.
- Provide a no-argument self-test fixture so CI can exercise the harness without model calls.
- Assert:
  - barrier validator passes for all dispatch envelopes;
  - worked-example pass rates equal expected values;
  - Phase 1b/2b `VALUABLE` divergence restarts have exactly one revision-history entry;
  - planned and actual model lanes match.
- Require `foundry-adversarial` to emit the TOON summary during real public-plugin runs.
- Document the remaining gap: a real run still needs to be exercised manually or by a harness that invokes the public plugin, but the artifact contract is now in the public skill.

### Out of scope

- Implementing Claude/Pi/Codex/Gemini provider dispatch machinery in this repo.
- Replacing PromptEnvelope JSON with TOON.
- Parsing the complete TOON spec; only the narrow top-level scalar + uniform tabular array subset needed for smoke manifests.

## Requirements

| ID | Description |
|----|-------------|
| R1 | `tests/behavioral-smoke.sh` runs with no arguments and exercises a representative replay fixture. |
| R2 | The script delegates dispatch envelope validation to `tests/validate-barrier-envelopes.sh`. |
| R3 | The script fails when any worked-example result row differs from expected pass/total counts. |
| R4 | The script fails when a `VALUABLE` divergence restart has revision history count other than 1. |
| R5 | The script fails when any model-lane row has `planned_model != actual_model`. |
| R6 | The script accepts a TOON run summary using a small documented subset. |
| R7 | Existing structural and barrier-envelope checks remain green. |

## Implementation Units

### Unit 1 — Behavioral smoke script

**File:** `tests/behavioral-smoke.sh`

- Bash wrapper with embedded Python validator.
- No external dependencies beyond Python 3 and existing barrier-envelope validator.
- Directory mode validates each passed run directory.
- No-argument mode creates a temp Foundry run fixture and validates it.

### Unit 2 — Public skill live-lane contract

**Files:**

- `plugins/foundry/skills/foundry-adversarial/SKILL.md`
- `tests/validate-behavioral-smoke-contract.sh`

Require real adversarial runs to create `runs/<run_id>/behavioral-smoke.toon`, document the TOON subset, and validate the skill contains the required anchors.

### Unit 3 — Documentation and todo state

**Files:**

- `todos/behavioral-smoke-tests.md`
- `docs/HANDOFF.md`

Record that replay-level behavioral smoke coverage and the public skill live-lane artifact contract landed.

## Verification

```bash
tests/behavioral-smoke.sh
tests/validate-behavioral-smoke-contract.sh
tests/validate-barrier-envelopes.sh
tests/validate-agents.sh
```
