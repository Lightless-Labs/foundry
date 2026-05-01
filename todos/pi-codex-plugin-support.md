---
title: Add Pi extensions and Codex plugin support
origin: 2026-05-01 user request during repo-identity cleanup
priority: medium
status: ready
---

# Pi Extensions and Codex Plugin Support

Foundry currently ships as a Claude plugin with skills, agents, examples, and validation aimed at Claude's plugin surface. To make the workflow portable across agent harnesses, add first-class support for Pi extensions and Codex-compatible plugin packaging.

## What to explore

- **Pi extension packaging:** map Foundry skills/agents into Pi's extension model without weakening the red/green information barrier.
- **Codex plugin packaging:** determine the equivalent plugin/skill/agent surface for Codex-based workflows and how much can be generated from the existing Claude plugin artifacts.
- **Shared source of truth:** avoid maintaining divergent prompt copies across Claude, Pi, and Codex. Prefer generated packaging or thin adapters around canonical skill/agent docs.
- **Validation:** extend structural checks so every packaging target preserves required metadata, attribution, output schemas, and barrier-language anchors.
- **Install docs:** document install/use commands for each supported harness.

## Suggested approach

1. Research Pi extension APIs and Codex plugin conventions.
2. Create a compatibility matrix: Claude plugin feature → Pi equivalent → Codex equivalent → gap/workaround.
3. Prototype a minimal packaging adapter for one skill and one reviewer agent.
4. Add validation that catches drift between canonical Foundry docs and generated/adapted package artifacts.
5. Document supported and unsupported workflows.

## Acceptance criteria

- A documented packaging strategy exists for both Pi extensions and Codex plugin support.
- At least one Foundry skill and one Foundry agent can be exposed through each target harness, or explicit blockers are documented.
- Existing Claude plugin behavior and validation remain unchanged.
- Information-barrier guarantees are preserved or stricter in every target harness.
