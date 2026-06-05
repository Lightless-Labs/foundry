---
date: 2026-06-05
topic: provider-diverse worked-example live smoke
---

# Research: Provider-Diverse Worked-Example Live Smoke

## Codebase Context
- Repository is the public Foundry plugin: Markdown/YAML skills and agents, shell/Python validators, Pi extension, and Rust worked examples.
- Build/test surface for this topic:
  - `tests/pi-live-dispatch-smoke.sh` — slow/manual live Pi dispatch harness.
  - `tests/behavioral-smoke.sh` — replay validator for run manifests and PromptEnvelope artifacts.
  - `tests/validate-barrier-envelopes.sh` — PromptEnvelope barrier validator.
  - `extensions/pi-foundry-team/index.ts` — Pi `foundry_team` PromptEnvelope dispatch primitive.
  - `examples/{sudoku-solver,rubiks-solver,chess-engine}` — worked examples with split green implementation and red integration tests.

## Existing Work
- `docs/HANDOFF.md` says the next step is to broaden provider-diverse exercises across deeper lanes after `runs/pi-live-kimi-minimax-smoke/` validated MiniMax red vs Kimi green on the Sudoku smoke.
- `docs/plans/2026-06-03-001-test-multi-provider-delegation-plan.md` completed per-lane model overrides and distinct-lane validation, but predates the successful Kimi/MiniMax run.
- `docs/pi-codex-support.md` documents `foundry_team` exact `envelope.prompt` dispatch, provider-qualified model lanes, and the existing Sudoku-oriented smoke invocation.
- `docs/HANDOFF.md` now records that PromptEnvelope child prompts must be self-contained because `visible_context` is replay/audit metadata, not automatically injected.

## Relevant Code
- `tests/pi-live-dispatch-smoke.sh` currently hardcodes:
  - `SUDOKU_DIR="$ROOT_DIR/examples/sudoku-solver"`
  - two Sudoku PromptEnvelope JSON files
  - a `sudoku-solver,30,30,30,30` behavioral-smoke row
  - `RED_OK` / `GREEN_OK` child output checks
- `tests/behavioral-smoke.sh` accepts any `test_results` example rows as long as `passed == expected_passed` and `total == expected_total`. It already self-tests multiple examples, including `chess-engine,44,44,44,44`.
- Example manifests:
  - `examples/sudoku-solver/Cargo.toml`: binary from `green/src/main.rs`, test from `red/tests/solver_tests.rs`, final count `30/30`.
  - `examples/rubiks-solver/Cargo.toml`: binary from `green/src/main.rs`, test from `red/tests/solver_tests.rs`, final count `46/46`.
  - `examples/chess-engine/Cargo.toml`: binary from `green/src/main.rs`, test from `red/tests/engine_tests.rs`, final count `44/44`.

## External References
- No external research needed. Strong local patterns exist and this is a harness extension, not a new framework or library integration.

## Test Landscape
- Local example health checks run on 2026-06-05:
  - `cd examples/rubiks-solver && cargo test --quiet` — `46/46` passed after an initial binary `0 tests` phase.
  - `cd examples/chess-engine && cargo test --quiet` — `44/44` passed after warnings and an initial binary `0 tests` phase.
- Focused validators for new live run artifacts should be:
  - `tests/validate-barrier-envelopes.sh runs/<run>/dispatch`
  - `tests/behavioral-smoke.sh runs/<run>`
  - `tests/validate-behavioral-smoke-contract.sh`
  - `tests/validate-pi-extension.sh`
  - `tests/validate-agents.sh`

## Open Questions
- Which heavier worked example should be the first provider-diverse live lane?
  - Chess is faster locally (`44/44` in ~3s) and still exercises a substantially deeper golden-vector example.
  - Rubik’s is a stronger convention-mismatch case study but slower (`46/46` in ~20s).
- Should the live smoke remain a lightweight dispatch plumbing check (`RED_OK`/`GREEN_OK`) or eventually dispatch richer example-specific reviewer/test-writing prompts? For the next slice, keep it lightweight and selectable so it remains practical as a slow/manual smoke.
