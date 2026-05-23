---
date: 2026-05-22
type: test
status: completed
completed: 2026-05-22
---

# test: Full Pi Adversarial Session

**Completed:** 2026-05-22 — ran `/skill:foundry-adversarial` under Pi against the Sudoku worked example in a smoke-scoped mode. Pi copied the example to `/tmp/foundry-pi-autonomous-sudoku-smoke/sudoku-solver`, ran `cargo test -- --nocapture` with `30/30` passing, wrote PromptEnvelope artifacts for red-team, green-team, and barrier-integrity-auditor under `runs/pi-autonomous-sudoku-smoke/`, dispatched all three through `foundry_team`, wrote `behavioral-smoke.toon`, and passed both `tests/validate-barrier-envelopes.sh runs/pi-autonomous-sudoku-smoke/dispatch` and `tests/behavioral-smoke.sh runs/pi-autonomous-sudoku-smoke`.

## Problem Frame

The public plugin now has Pi skill adapters plus `foundry_team` live dispatch. The remaining behavioral-smoke gap is to run a full public `foundry-adversarial` session under Pi and validate the emitted `runs/<run_id>/` artifacts.

## Scope

- Run Pi with the `foundry-adversarial` adapter and `foundry_team` extension enabled.
- Use the Sudoku worked example NLSpec as the cheapest target.
- Prefer a temporary copy/workspace so generated implementation/test artifacts do not dirty the repository.
- Validate emitted run artifacts with `tests/behavioral-smoke.sh`.
- Record blockers or workflow gaps if Pi cannot complete the autonomous run.

## Verification

```bash
pi -e ./extensions/pi-foundry-team/index.ts --skill skills/foundry-adversarial/SKILL.md ...
tests/behavioral-smoke.sh <run_dir>
```
