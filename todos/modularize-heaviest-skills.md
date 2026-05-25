---
title: Modularize heaviest skills — smaller contracts, more executable checks
origin: 2026-04-17 ilia-feedback-foundry-plugin (item 4)
priority: medium
status: completed — first modularization slice landed
updated: 2026-05-24
---

# Modularize Heaviest Skills

**Addendum:** 2026-05-24 — first modularization slice landed. Extracted Phase 1b/2b divergence routing, `spec_update_and_restart`, and provider troubleshooting into `docs/playbooks/foundry-adversarial-{divergence-routing,spec-update-and-restart,provider-troubleshooting}.md`. The main adversarial skill now keeps short mandatory summaries and links to the playbooks. Added `tests/validate-adversarial-modules.sh` to preserve critical executable anchors (`findings[0].outcome`, Phase 2b `VALUABLE`, revision-history count, green PASS/FAIL-only barrier language). Existing behavioral smoke and Pi extension validators still pass.

**Addendum:** 2026-05-24 — from-scratch Pi Roman numeral run produced two concrete obedience/workflow observations. First, the full Pi `/skill:foundry-adversarial` orchestration completed red/green generation and reached Phase 3, but the outer shell timed out during reviewer fan-out; future live runs should either use a longer timeout/resumable Pi session or narrower phase-level invocations. Second, a manual continuation envelope mistakenly put an allowed test outcome label in `withheld_context.samples`; `foundry_team` correctly rejected it before dispatch. This suggests a future executable helper should derive withheld samples from bodies/assertions/raw output only and explicitly exclude allowed PASS/FAIL test-name labels.

**Addendum:** 2026-05-24 — hardening landed for both observations. `tests/validate-barrier-envelopes.sh` now rejects green withheld samples that duplicate allowed PASS/FAIL outcome labels (including terminal names from namespaced labels), and the self-tests include that regression. `extensions/pi-foundry-team/index.ts` now rejects the same mistake before child dispatch with a clearer error. Added `docs/playbooks/foundry-adversarial-pi-continuation.md` and linked it from the adversarial skill for timeout/resume continuation from serialized PromptEnvelope artifacts.

`foundry-adversarial/SKILL.md` was the heaviest skill in the plugin. It carries a lot of nuance — information barrier, Phase 1b/2b divergence handling, spec-update-and-restart, test-fix inner loop, troubleshooting. That nuance is why the skill works, but it's also a reliability risk: long instruction blocks are easier for models to partially obey.

## What to fix

- Break `foundry-adversarial` into a dispatch shell + per-phase sub-skills or referenced playbooks.
- Push structural constraints (barrier matrix, dispatch envelope shape, divergence outcome routing) into executable checks the skill can call, not prose the model must re-derive.
- Keep the method intact — the goal is the same behavior with tighter, enforceable contracts, not a different workflow.

## Candidates for extraction

- Phase 1b / 2b divergence evaluator dispatch + outcome routing (already relatively self-contained)
- `spec_update_and_restart` — currently 30+ lines of prose; a good candidate for a named sub-skill
- Troubleshooting entries (OpenCode/Kimi, tokenizer salvage) — move to a provider-specific reference doc the skill links to

## Suggested approach

Profile obedience first: instrument a few runs to see where the skill is being partially obeyed (e.g., a step skipped, an envelope field omitted). Extract those hot spots first. Don't modularize speculatively.

## Progress

- [x] Extract Phase 1b / 2b divergence evaluator dispatch + outcome routing.
- [x] Extract `spec_update_and_restart`.
- [x] Extract provider-specific troubleshooting entries.
- [x] Add executable structural checks for extracted contracts.
- [ ] Continue profiling obedience on future real runs; extract additional modules only when runs show a concrete obedience gap.

See: `docs/solutions/workflow-issues/ilia-feedback-foundry-plugin-20260417.md` (item 4).
