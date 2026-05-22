---
name: foundry-nlspec
description: "Derive a Natural Language Specification from a spec document and review it. Use when a spec exists and you need a buildable spec for adversarial implementation."
---

# Foundry Nlspec — Pi Adapter

This is a thin Pi-compatible adapter for the canonical Foundry skill.

Canonical source of truth:

```text
../../plugins/foundry/skills/foundry-nlspec/SKILL.md
```

## Required Behavior

1. Read `../../plugins/foundry/skills/foundry-nlspec/SKILL.md` before doing the work.
2. Follow the canonical skill instructions exactly, including all gates, review phases, artifact paths, and information-barrier rules.
3. Treat this adapter as packaging glue only. If this file and the canonical skill disagree, the canonical skill wins.
4. Preserve the Foundry information barrier: red sees full spec/NLSpec but no implementation; green sees only NLSpec How plus PASS/FAIL outcome labels and never sees red test code, assertions, raw failures, or NLSpec Done criteria.

## Invocation Notes

Pi exposes this adapter as:

```text
/skill:foundry-nlspec <arguments>
```

The equivalent Claude plugin skill name is `foundry:nlspec`.
