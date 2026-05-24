---
title: Modularize foundry-adversarial skill contracts
created: 2026-05-24
completed: 2026-05-24
status: completed
---

# Modularize Foundry Adversarial Skill Contracts Plan

## Goal

Reduce the obedience risk in `plugins/foundry/skills/foundry-adversarial/SKILL.md` by extracting self-contained procedures into referenced playbooks and adding executable structural checks for the extracted contracts.

## Constraints

- Preserve the existing adversarial workflow semantics.
- Preserve grep anchors relied on by existing validators, especially `Phase 2b` and `VALUABLE` routing text.
- Keep PromptEnvelope validation and behavioral-smoke requirements visible in the main skill.
- Green still sees only NLSpec How plus `PASS/FAIL` labels; red still sees no implementation code.

## Implementation Steps

1. Extract divergence routing into a dedicated playbook.
2. Extract `spec_update_and_restart` into a dedicated playbook.
3. Extract provider-specific troubleshooting into a dedicated playbook.
4. Replace bulky sections in the main skill with short mandatory references and invariant summaries.
5. Add `tests/validate-adversarial-modules.sh` to verify the main skill references the playbooks and that playbooks keep the required routing, barrier, and restart anchors.
6. Run existing skill/agent/barrier validators plus the new module validator.

## Acceptance

- [x] The main adversarial skill is shorter and points to durable playbooks for the extracted procedures.
- [x] Extracted playbooks preserve critical exact terms (`findings[0].outcome`, `Phase 2b` `VALUABLE`, `spec_update_and_restart`, revision history, PromptEnvelope barrier language).
- [x] Validation fails if the skill stops referencing a required module or if a module loses key contract text.
