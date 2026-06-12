---
title: Exercise multi-provider red/green delegation
origin: 2026-06-03 user request after Gemini became unavailable
priority: high
status: completed
updated: 2026-06-11
---

# Exercise Multi-Provider Red/Green Delegation

Foundry's information barrier should be strengthened by sending isolated red and green lanes to different model/provider choices. Gemini is no longer available, so use Codex 5.5 high-reasoning/xhigh for red and either Kimi or a lower-effort Codex 5.5 lane for green.

## Work items

- [x] Add explicit per-lane model override support to the public Pi live dispatch smoke.
- [x] Add replay validation for runs that intentionally require distinct red/green model lanes.
- [x] Document the recommended invocation patterns and fallback when Kimi is unavailable.
- [x] Run a slow/manual live smoke with distinct lanes and preserve/validate the run artifacts.
- [x] Update `docs/HANDOFF.md` with final model lanes, validation status, and learnings.

## Completion

Completed 2026-06-03. Live run: `runs/pi-live-multilane-smoke/` with red `openai-codex/gpt-5.5:xhigh`, green `openai-codex/gpt-5.5:medium`, orchestrator `openai-codex/gpt-5.5`; barrier and behavioral validators passed. Correction: Pi has `kimi-coding` auth configured, but Kimi spot-check calls hung without stdout/stderr, so Codex medium was used as the operational fallback.

**Addendum:** 2026-06-05 — Kimi and MiniMax are operational in Pi. Added selectable worked-example support to `tests/pi-live-dispatch-smoke.sh` (`sudoku-solver` 30/30, `rubiks-solver` 46/46, `chess-engine` 44/44). Preserved validated provider-diverse runs for Sudoku (`runs/pi-live-kimi-minimax-smoke/`) and Chess (`runs/pi-live-kimi-minimax-chess-smoke/`) with red `minimax/MiniMax-M3` and green `kimi-coding/kimi-for-coding`.

**Addendum:** 2026-06-07 — Added opt-in `--phase-task artifact-sketch` support to `tests/pi-live-dispatch-smoke.sh`. The default plumbing mode still requires exact `RED_OK` / `GREEN_OK`; artifact-sketch asks red for a JSON `red_test_plan` and green for a JSON `green_implementation_plan` from How + PASS/FAIL labels only, then validates output shape and withheld-sample non-leakage. Both artifact-sketch and default plumbing live paths passed with red `minimax/MiniMax-M3` and green `kimi-coding/kimi-for-coding` using temporary artifacts.

**Addendum:** 2026-06-08 — Artifact-sketch mode now persists parsed child outputs when the run directory is kept: `phase-artifacts/red-team-test-plan.json` and `phase-artifacts/green-team-implementation-plan.json`. A live MiniMax/Kimi artifact-sketch run with an explicit temporary `--run-dir` wrote and validated both files; a separate default plumbing live run confirmed `phase-artifacts/` is not created outside artifact-sketch mode.

**Addendum:** 2026-06-09 — Completed a fuller provider-diverse red/green adversarial smoke under `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/`. Red used MiniMax M3 and generated executable Rust slugify integration tests; green used Kimi for Coding and generated the implementation from the NLSpec How section only. The orchestrator copied red tests into the green crate for execution; `11/11` tests passed, PromptEnvelope validation passed, and `behavioral-smoke.toon` validated with distinct actual model lanes.

**Addendum:** 2026-06-09 — Extended the same slugify run with provider-diverse Phase 3 reviewer fan-out. Green-team-reviewer and rust-reviewer ran on Kimi; red-team-test-reviewer and barrier-integrity-auditor ran on MiniMax. Reviewer outputs are preserved under `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/reviews/`; `behavioral-smoke.toon` now records reviewer lanes in `model_lanes[8]`. MiniMax returned no output on the first red-team-test-reviewer attempt and wrapped the r2 JSON in Markdown fences, recorded as a structured-output obedience anomaly. Barrier and behavioral validators pass.

**Addendum:** 2026-06-11 — Added `runs/pi-live-kimi-minimax-divergence-restart-smoke/`, a live provider-diverse Phase 2b divergence/restart smoke. Red and divergence-evaluator ran on MiniMax; green ran on Kimi. Divergence evaluator r2 returned `findings[0].outcome=VALUABLE` for an accented-Latin slugify policy gap; `spec_update_and_restart` artifacts record `revision_history_count: 1`; barrier and behavioral validators pass with `requires_divergence_restart: true` and distinct model lanes.

**Addendum:** 2026-06-11 — Extended the divergence/restart smoke with a post-restart resumed red/green lane. Post-restart green uses opaque `T-###` PASS/FAIL labels and explicitly redacts `post_restart_red_output`. Kimi r1 referenced an older run path, so r2 requested a self-contained Rust implementation artifact under `resumed/green/`; `cargo test --quiet` passed `4/4`. Barrier, behavioral, and agent validators pass.

## Plan

See `docs/plans/2026-06-03-001-test-multi-provider-delegation-plan.md`, `docs/plans/2026-06-05-001-test-provider-diverse-worked-example-smoke-plan.md`, `docs/plans/2026-06-07-001-test-provider-diverse-phase-artifact-smoke-plan.md`, `docs/plans/2026-06-08-001-test-phase-artifact-capture-plan.md`, `docs/plans/2026-06-09-001-test-provider-diverse-fuller-adversarial-phase-plan.md`, `docs/plans/2026-06-09-002-test-provider-diverse-reviewer-fanout-plan.md`, `docs/plans/2026-06-11-001-test-provider-diverse-divergence-restart-plan.md`, and `docs/plans/2026-06-11-002-test-post-restart-resume-smoke-plan.md`.
