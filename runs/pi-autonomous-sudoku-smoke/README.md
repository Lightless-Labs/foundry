# Pi Autonomous Sudoku Smoke

Run ID: `pi-autonomous-sudoku-smoke`
Date: 2026-05-22

## What this run shows

This is the first smoke-scoped autonomous `/skill:foundry-adversarial` run preserved from Pi.

It demonstrates that Pi can execute the public Foundry skill adapter with the local `foundry_team` extension, dispatch PromptEnvelope-backed red/green/reviewer children, and preserve replayable artifacts for a known-good worked example.

The feature target is the existing Sudoku solver worked example, reused as a smoke fixture rather than generated from scratch.

## Result

Final replay result: `sudoku-solver` `30/30` tests passed.

Model lanes recorded in `behavioral-smoke.toon`:

- red-team: `openai-codex/gpt-5.5`
- green-team: `openai-codex/gpt-5.5`
- barrier-integrity-auditor: `openai-codex/gpt-5.5`
- orchestrator: `openai-codex/gpt-5.5`

This run does **not** require distinct red/green model lanes and does not exercise divergence restart routing.

## Validate

```bash
tests/validate-barrier-envelopes.sh runs/pi-autonomous-sudoku-smoke/dispatch
tests/behavioral-smoke.sh runs/pi-autonomous-sudoku-smoke
```

## Preserved artifacts

- `behavioral-smoke.toon` — replay summary and model lanes.
- `dispatch/phase1/red-team.json` — red-team PromptEnvelope.
- `dispatch/phase2/green-team.json` — green-team PromptEnvelope.
- `dispatch/phase3/barrier-integrity-auditor.json` — barrier reviewer PromptEnvelope.
- `dispatch-results.md` — actual child dispatch outcomes.
- `test-results.txt` — final Sudoku test summary.

## Notes

This run is intentionally small. Its value is proving Pi skill/extension wiring and PromptEnvelope replay validation before spending time on from-scratch or provider-diverse runs.
