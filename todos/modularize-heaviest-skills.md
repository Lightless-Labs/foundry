---
title: Modularize heaviest skills — smaller contracts, more executable checks
origin: 2026-04-17 ilia-feedback-foundry-plugin (item 4)
priority: medium
status: ready
---

# Modularize Heaviest Skills

`foundry-adversarial/SKILL.md` is the heaviest skill in the plugin (~375 lines). It carries a lot of nuance — information barrier, Phase 1b/2b divergence handling, spec-update-and-restart, test-fix inner loop, troubleshooting. That nuance is why the skill works, but it's also a reliability risk: long instruction blocks are easier for models to partially obey.

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

See: `docs/solutions/workflow-issues/ilia-feedback-foundry-plugin-20260417.md` (item 4).
