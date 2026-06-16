# Pi Live Kimi/MiniMax Sudoku Smoke

Run ID: `pi-live-kimi-minimax-smoke`
Date: 2026-06-05

## What this run shows

This live Pi dispatch smoke verifies provider-diverse red/green lanes through `foundry_team` on the Sudoku worked example.

It demonstrates that MiniMax can serve as the red lane, Kimi can serve as the green lane, and Foundry's behavioral validator can require provider-qualified lane separation while preserving the standard PASS/FAIL replay summary.

## Result

Final replay result: `sudoku-solver` `30/30` tests passed.

Model lanes recorded in `behavioral-smoke.toon`:

- red-team planned/actual: `minimax/MiniMax-M3`
- green-team planned/actual: `kimi-coding/kimi-for-coding`
- orchestrator planned/actual: `openai-codex/gpt-5.5`

`requires_distinct_model_lanes: true` verifies that red and green ran on different provider-qualified model lanes.

## Validate

```bash
tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-smoke
```

## Preserved artifacts

- `behavioral-smoke.toon` — replay summary, distinct-lane requirement, and model lanes.
- `pi-foundry-team.jsonl` — live Pi `foundry_team` JSONL trace.
- `sudoku-cargo-test.out` — final Sudoku cargo test output.

## Notes

The `behavioral-smoke.toon` file uses the generic live-dispatch run id from the smoke harness. Treat the directory name, `pi-live-kimi-minimax-smoke`, as the preserved run identity.

This run established that Kimi/MiniMax were operational in Pi after earlier uncertainty about Kimi responsiveness, and it motivated deeper provider-diverse smokes on Chess and from-scratch slugify.
