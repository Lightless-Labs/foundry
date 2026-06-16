# Pi Live Kimi/MiniMax Chess Smoke

Run ID: `pi-live-kimi-minimax-chess-smoke`
Date: 2026-06-05

## What this run shows

This live Pi dispatch smoke repeats the provider-diverse MiniMax/Kimi lane pattern on a deeper worked example: the Chess engine.

It demonstrates that the distinct-lane plumbing is not limited to the small Sudoku fixture. The Chess example carries more convention pressure through golden perft-style expectations, making it a stronger smoke for model-lane reporting and replay validation.

## Result

Final replay result: `chess-engine` `44/44` tests passed.

Model lanes recorded in `behavioral-smoke.toon`:

- red-team planned/actual: `minimax/MiniMax-M3`
- green-team planned/actual: `kimi-coding/kimi-for-coding`
- orchestrator planned/actual: `openai-codex/gpt-5.5`

`requires_distinct_model_lanes: true` verifies that red and green ran on different provider-qualified model lanes.

The preserved cargo output includes Rust warnings in the generated green implementation, but all smoke-scope tests pass.

## Validate

```bash
tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-chess-smoke
```

## Preserved artifacts

- `behavioral-smoke.toon` — replay summary, distinct-lane requirement, and model lanes.
- `pi-foundry-team.jsonl` — live Pi `foundry_team` JSONL trace.
- `chess-engine-cargo-test.out` — final Chess cargo test output, including warnings and `44/44` pass result.

## Notes

The `behavioral-smoke.toon` file uses the generic live-dispatch run id from the smoke harness. Treat the directory name, `pi-live-kimi-minimax-chess-smoke`, as the preserved run identity.

This run served as the deeper provider-diverse bridge before the fuller from-scratch Kimi/MiniMax slugify red/green and reviewer fan-out smoke.
