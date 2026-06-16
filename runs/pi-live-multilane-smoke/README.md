# Pi Live Multilane Smoke

Run ID: `pi-live-multilane-smoke`
Date: 2026-06-03

## What this run shows

This live Pi dispatch smoke verifies that Foundry can run red and green on distinct model lanes through `foundry_team`, even when both lanes come from the same provider/model family.

The target is the Sudoku worked example. The run is smoke-scoped: it validates child dispatch plumbing, model-lane reporting, and final test replay, not a new from-scratch feature.

## Result

Final replay result: `sudoku-solver` `30/30` tests passed.

Model lanes recorded in `behavioral-smoke.toon`:

- red-team planned/actual: `openai-codex/gpt-5.5:xhigh`
- green-team planned/actual: `openai-codex/gpt-5.5:medium`
- orchestrator planned/actual: `openai-codex/gpt-5.5`

`requires_distinct_model_lanes: true` verifies that the red and green lanes did not collapse to the same provider-qualified model identifier. This specifically preserves the thinking suffixes (`:xhigh`, `:medium`) in actual lane reporting.

## Validate

```bash
tests/behavioral-smoke.sh runs/pi-live-multilane-smoke
```

## Preserved artifacts

- `behavioral-smoke.toon` — replay summary, distinct-lane requirement, and model lanes.
- `pi-foundry-team.jsonl` — live Pi `foundry_team` JSONL trace.
- `sudoku-cargo-test.out` — final Sudoku cargo test output.

## Notes

The `behavioral-smoke.toon` file uses the generic live-dispatch run id from the smoke harness. Treat the directory name, `pi-live-multilane-smoke`, as the preserved run identity.

This run is the bridge between single-model smoke plumbing and later provider-diverse MiniMax/Kimi lanes.
