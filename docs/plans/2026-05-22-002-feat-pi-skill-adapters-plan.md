---
date: 2026-05-22
type: feat
status: completed
completed: 2026-05-22
---

# feat: Pi Skill Adapters

**Completed:** 2026-05-22 — added Pi-compatible skill adapters for all five Foundry skills, exposed them through the Pi package manifest, documented Pi install/use commands, and expanded Pi validation to lock adapter names, canonical-source links, and `foundry_team` guidance.

## Problem Frame

Foundry's canonical skills live under `plugins/foundry/skills/**/SKILL.md` and use Claude plugin names like `foundry:adversarial`. Pi implements Agent Skills and warns on names containing `:`, so the canonical skill directories are not a clean Pi package surface. To run a full autonomous `foundry:adversarial` session under Pi, Foundry needs thin Pi-compatible skill adapters that load the canonical skill instructions without forking them.

## Scope

### In scope

- Add Pi-compatible skill adapters with lowercase/hyphen names:
  - `foundry-research`
  - `foundry-brainstorm`
  - `foundry-nlspec`
  - `foundry-adversarial`
  - `foundry-forge`
- Update the Pi package manifest so Pi loads the adapters.
- Add validation that catches missing adapters, invalid adapter names, missing canonical-source links, and missing `foundry_team` guidance for the adversarial/forge adapters.
- Add install/use docs for Pi commands.

### Out of scope

- Forking canonical Foundry skill prompts.
- Codex packaging beyond preserving the todo as pending.
- Running the full autonomous adversarial workflow; this slice removes the skill-loading prerequisite.

## Verification

```bash
tests/validate-pi-extension.sh
tests/validate-agents.sh
tests/behavioral-smoke.sh
```
