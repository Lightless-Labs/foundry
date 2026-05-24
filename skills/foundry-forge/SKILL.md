---
name: foundry-forge
description: "Run the full Foundry pipeline as an Agent Skills adapter: research, brainstorm, NLSpec, then adversarial red/green implementation. Use when starting a feature from scratch."
---

# Foundry Forge — Agent Skills Adapter

This is a thin Agent Skills-compatible adapter for the canonical Foundry skill.

Canonical source of truth:

```text
../../plugins/foundry/skills/foundry-forge/SKILL.md
```

## Required Behavior

1. Read `../../plugins/foundry/skills/foundry-forge/SKILL.md` before doing the work.
2. Follow the canonical skill instructions exactly, including all gates, review phases, artifact paths, and information-barrier rules.
3. Treat this adapter as packaging glue only. If this file and the canonical skill disagree, the canonical skill wins.
4. Preserve the Foundry information barrier: red sees full spec/NLSpec but no implementation; green sees only NLSpec How plus PASS/FAIL outcome labels and never sees red test code, assertions, raw failures, or NLSpec Done criteria.

## Pi Dispatch Requirement

When the canonical skill calls for subagents, teams, `Agent(...)`, or parallel review dispatch, use the Foundry Pi package tool `foundry_team` after writing PromptEnvelope JSON artifacts. Pi has no native subagent primitive; never simulate the barrier by pasting hidden context into the main Pi conversation.

## Invocation Notes

Pi and Codex expose this adapter as:

```text
/skill:foundry-forge <arguments>
```

The equivalent Claude plugin skill name is `foundry:forge`.
