# Pi Live Kimi/MiniMax Divergence Restart Smoke

Controlled live Foundry run for provider-diverse divergence restart routing.

- Run id: `pi-live-kimi-minimax-divergence-restart-smoke`
- Red lane: MiniMax (`minimax/MiniMax-M3`)
- Green lane: Kimi (`kimi-coding/kimi-for-coding`)
- Scenario: slugify v1 intentionally omits accented Latin transliteration, producing a Phase 2b stable failure.
- Expected route: divergence evaluator returns `findings[0].outcome = VALUABLE`, then `spec_update_and_restart` records one restart event.

Artifacts are replayable PromptEnvelope v1 JSON under `dispatch/`.

## Result

- Red live dispatch completed on `minimax/MiniMax-M3`.
- Green live dispatch completed on `kimi-coding/kimi-for-coding`.
- Divergence evaluator r1 completed on `minimax/MiniMax-M3` and returned `NOT_VALUABLE` because the initial packet explicitly excluded accented Latin transliteration. This is preserved as a prompt-authoring anomaly.
- Divergence evaluator r2 completed on `minimax/MiniMax-M3` and returned `findings[0].outcome = VALUABLE` with a non-null `gap_description`.
- `spec-update-and-restart.json` and `phase1-restart-package.json` record the restart with `revision_history_count: 1`.
- `behavioral-smoke.toon` validates with `requires_divergence_restart: true` and `requires_distinct_model_lanes: true`.
- Post-restart red dispatch completed on `minimax/MiniMax-M3` from the revised NLSpec and change summary.
- Post-restart green dispatch completed on `kimi-coding/kimi-for-coding` from the revised How section plus opaque `T-###` PASS/FAIL labels only.
- Green plan r1 referenced an older fuller-smoke path; r2 produced a self-contained implementation artifact under `resumed/green/src/lib.rs`.
- `cd resumed/green && cargo test --quiet` passed `4/4` post-restart policy tests.

## Boundary Note

This run now proves the provider-diverse route/restart path plus a resumed post-restart convergence smoke. Pre-restart red/green artifacts remain tied to the original NLSpec; post-restart artifacts under `dispatch/post_restart/` and `resumed/` are tied to the revised NLSpec. The green post-restart prompts use opaque test IDs and explicitly withhold post-restart red output.
