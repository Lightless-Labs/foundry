---
date: 2026-05-22
type: feat
status: completed
completed: 2026-05-22
---

# feat: Pi Live Behavioral Smoke Lane

**Completed:** 2026-05-22 — added a slow/manual Pi live dispatch smoke harness that creates real PromptEnvelope artifacts, invokes the public `foundry_team` extension through `pi -e`, captures child Pi model lanes from the tool result, writes `behavioral-smoke.toon`, and validates the resulting run directory with `tests/behavioral-smoke.sh`. Also tightened `foundry_team` model-lane reporting to include provider-qualified IDs when Pi exposes `provider` + `model`.

## Problem Frame

The replay-level behavioral smoke harness can validate `runs/<run_id>/` artifacts, and the public Pi extension can spawn child Pi processes from PromptEnvelope paths. The missing public-plugin lane was a repeatable live smoke that proves the extension can produce run artifacts with real child model dispatches, without depending on private engine/BuildKite infrastructure or committing generated run directories.

## Scope

### In scope

- Add a manual/slow script that:
  - creates a temporary Foundry run directory;
  - writes red-team and green-team `foundry.prompt-envelope.v1` artifacts;
  - invokes `pi -e ./extensions/pi-foundry-team/index.ts` and requires a `foundry_team` tool call;
  - confirms child outputs and actual model IDs;
  - runs the Sudoku worked example's red tests to record `30/30`;
  - emits `behavioral-smoke.toon` from actual tool-result details;
  - validates the run with `tests/behavioral-smoke.sh`.
- Keep generated artifacts out of git by default, with an opt-in `--run-dir`/`--keep` path for debugging.
- Preserve the information-barrier invariant: green sees only NLSpec How + PASS/FAIL labels.

### Out of scope

- Full autonomous regeneration of Sudoku red tests and green implementation from the NLSpec.
- Private engine/BuildKite dispatcher integration.
- CI-enabling the live lane by default; this script performs real model calls and should remain a slow/manual gate.

## Verification

```bash
tests/pi-live-dispatch-smoke.sh
tests/behavioral-smoke.sh
tests/validate-pi-extension.sh
tests/validate-agents.sh
```

## Follow-up

A stronger live lane can later run the full `foundry:adversarial` skill under Pi once Pi skill adapters are available. This slice proves the public extension and replayable artifact contract are executable today.
