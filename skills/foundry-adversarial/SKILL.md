---
name: foundry-adversarial
description: "Run Foundry's adversarial red/green implementation process under Pi. Use when you have a reviewed NLSpec and need isolated red test writing plus green implementation with PromptEnvelope barriers."
---

# Foundry Adversarial — Pi Adapter

This is a thin Pi-compatible adapter for the canonical Foundry skill.

Canonical source of truth:

```text
../../plugins/foundry/skills/foundry-adversarial/SKILL.md
```

## Required Behavior

1. Read `../../plugins/foundry/skills/foundry-adversarial/SKILL.md` before doing the work.
2. Follow the canonical skill instructions exactly, including all gates, review phases, artifact paths, and information-barrier rules.
3. Treat this adapter as packaging glue only. If this file and the canonical skill disagree, the canonical skill wins.
4. Preserve the Foundry information barrier: red sees full spec/NLSpec but no implementation; green sees only NLSpec How plus PASS/FAIL outcome labels and never sees red test code, assertions, raw failures, or NLSpec Done criteria.

## Pi Dispatch Requirement

When the canonical skill calls for subagents, teams, `Agent(...)`, or parallel review dispatch, use the Foundry Pi package tool `foundry_team` after writing PromptEnvelope JSON artifacts. Pi has no native subagent primitive; never simulate the barrier by pasting hidden context into the main Pi conversation.

## Invocation Notes

Pi exposes this adapter as:

```text
/skill:foundry-adversarial <arguments>
```

The equivalent Claude plugin skill name is `foundry:adversarial`.
