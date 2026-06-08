---
date: 2026-06-08
topic: durable phase-artifact capture for Pi live dispatch smoke
---

# Research: Durable Phase-Artifact Capture for Pi Live Dispatch Smoke

## Codebase Context

- This repo is the public Foundry plugin layer: Markdown/YAML skills and agents, shell/Python validators, Pi extension, Codex packaging, and worked examples.
- Relevant implementation surface:
  - `tests/pi-live-dispatch-smoke.sh` — slow/manual Pi live dispatch harness.
  - `tests/behavioral-smoke.sh` — validates `dispatch/**/*.json` PromptEnvelope artifacts and `behavioral-smoke.toon` run summaries.
  - `tests/validate-barrier-envelopes.sh` — validates PromptEnvelope barrier invariants.
  - `extensions/pi-foundry-team/index.ts` — runtime child-dispatch primitive that returns child outputs and model lanes.
- Current `artifact-sketch` mode validates red/green JSON artifacts from `pi-foundry-team.jsonl`, but does not persist parsed artifacts separately from the raw JSONL.

## Existing Work

- `docs/plans/2026-06-07-001-test-provider-diverse-phase-artifact-smoke-plan.md` completed opt-in `--phase-task artifact-sketch`.
- `todos/multi-provider-delegation.md` records the temporary-artifact validation result for MiniMax red vs Kimi green.
- `docs/HANDOFF.md` explicitly distinguishes preserved run directories (`runs/pi-live-*`) from temporary phase-artifact smoke evidence.
- Earlier preserved live runs store raw `pi-foundry-team.jsonl`, PromptEnvelopes under `dispatch/`, and `behavioral-smoke.toon`, but no parsed child-output artifact directory.

## Relevant Code

- `tests/pi-live-dispatch-smoke.sh` parses child results in an embedded Python script after `foundry_team` completes.
- In `plumbing` mode the script requires exact `RED_OK` / `GREEN_OK` output.
- In `artifact-sketch` mode the script:
  - parses red output as a JSON object;
  - requires `artifact_type=red_test_plan`, matching `example`, `implementation_visible=false`, at least three `test_categories`, and non-empty `oracle_strategy`;
  - parses green output as a JSON object;
  - requires `artifact_type=green_implementation_plan`, matching `example`, `saw_red_tests=false`, `permitted_feedback=PASS_FAIL_ONLY`, and at least three `implementation_steps`;
  - scans outputs for known withheld samples.
- `tests/behavioral-smoke.sh` does not currently know about phase artifacts; adding optional files under a new directory will not affect existing validation.

## External References

- Skipped. Strong local patterns exist and this is a harness/auditability extension, not a new library or framework integration.

## Test Landscape

- Fast/static checks:
  - `bash -n tests/pi-live-dispatch-smoke.sh`
  - `tests/pi-live-dispatch-smoke.sh --help`
  - `tests/validate-behavioral-smoke-contract.sh`
  - `tests/validate-pi-extension.sh`
  - `tests/validate-agents.sh`
- Slow/live checks if runtime budget allows:
  - `tests/pi-live-dispatch-smoke.sh --phase-task artifact-sketch --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --run-dir runs/<manual-run>`
  - `tests/pi-live-dispatch-smoke.sh --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes`

## Open Questions

- Should `behavioral-smoke.sh` require optional phase artifacts when a run declares an artifact phase-task? Current `behavioral-smoke.toon` has no `phase_task` scalar, so a minimal slice can persist artifacts without changing the manifest schema.
- Should parsed artifacts be committed as a new preserved run? Not necessary for the minimal harness hardening; `--run-dir` users can preserve them when desired.
