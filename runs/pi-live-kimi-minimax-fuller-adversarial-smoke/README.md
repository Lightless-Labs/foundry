# Pi Live Kimi/MiniMax Fuller Adversarial Smoke

Run ID: `pi-live-kimi-minimax-fuller-adversarial-smoke`
Date: 2026-06-09

## Feature

Small from-scratch Rust `slugify_smoke` library generated from a reviewed NLSpec fixture preserved at:

- `project/docs/nlspecs/slugify.nlspec.md`

The library exposes:

```rust
pub fn slugify(input: &str) -> String
```

## Provider-Diverse Lanes

- Red team planned/actual model: `minimax/MiniMax-M3`
- Green team planned/actual model: `kimi-coding/kimi-for-coding`
- Orchestrator model: `openai-codex/gpt-5.5`

Red and green were dispatched through `foundry_team` from serialized PromptEnvelope artifacts. The actual child model lanes differ, so `behavioral-smoke.toon` sets `requires_distinct_model_lanes: true`.

## Result

Final red tests against green implementation: `11/11` pass.

```bash
tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch
tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke
```

Both validators pass.

## Barrier Notes

- Red saw the full NLSpec and wrote `red/tests/slugify_tests.rs`; implementation workspace samples were withheld.
- Green saw only the NLSpec How section and wrote `green/Cargo.toml` plus `green/src/lib.rs`; red test code, NLSpec Done snippets, and raw failure samples were withheld.
- The orchestrator copied red integration tests into `green/tests/` only for test execution after both child dispatches completed.
- No orchestrator implementation edits were made.
- Phase 3 reviewer fan-out was intentionally skipped for this bounded live smoke; the milestone targeted a fuller red/green phase, actual provider diversity, barrier replay, and behavioral-smoke validation.

## Preserved Artifacts

- `behavioral-smoke.toon` — final replay summary with distinct model lanes.
- `dispatch/` — PromptEnvelope artifacts for red and green dispatches.
- `project/` — NLSpec fixture.
- `red/` — generated red tests.
- `green/` — generated green implementation and copied integration tests used for execution.
- `logs/` — Cargo output and final PASS labels.

Cargo `target/` output was removed from the preserved fixture.
