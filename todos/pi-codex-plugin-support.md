---
title: Add Pi extensions and Codex plugin support
origin: 2026-05-01 user request during repo-identity cleanup
priority: medium
status: partial — Pi extension package, skill adapters, and install docs landed; full adversarial Pi run and Codex support pending
updated: 2026-05-22
---

# Pi Extensions and Codex Plugin Support

**Addendum:** 2026-05-21 — first Pi extension slice landed while closing the behavioral-smoke live-lane gap. Added a root `package.json` Pi manifest and `extensions/pi-foundry-team/index.ts`. The extension follows Pi's officially shipped `examples/extensions/subagent/` pattern: it registers a `foundry_team` tool, validates PromptEnvelope JSON artifacts, then spawns child `pi --mode json -p --no-session` processes with isolated context windows. This is intentionally not a fork of the canonical prompts: Foundry agent prompts are discovered from `plugins/foundry/agents/**/*.md`. Codex support remains pending.

**Addendum:** 2026-05-22 — Pi skill adapters and install docs landed. Root `package.json` exposes `./skills`; `skills/foundry-{research,brainstorm,nlspec,adversarial,forge}/SKILL.md` provide Agent Skills-compatible hyphenated names and direct Pi to read canonical `plugins/foundry/skills/**/SKILL.md` sources. `docs/pi-codex-support.md` documents install and invocation. `tests/validate-pi-extension.sh` now validates the package skill manifest, all adapters, canonical links, and `foundry_team` guidance. Remaining Pi work: run the full autonomous adversarial session under Pi and harden any discovered workflow gaps. Codex support remains pending.

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

## Progress

- [x] Research Pi extension/package docs and official examples.
- [x] Confirm Pi has no built-in subagent/team/swarm primitive; official guidance is to build/install extensions.
- [x] Prototype a Pi extension package slice: `package.json` + `extensions/pi-foundry-team/index.ts`.
- [x] Reuse canonical Foundry agent prompts from `plugins/foundry/agents/**/*.md` instead of forking prompt copies.
- [x] Add validation for the Pi extension package contract (`tests/validate-pi-extension.sh`).
- [x] Add Pi skill adapters/install docs for the canonical Foundry skills.
- [ ] Run a real public-plugin adversarial session under Pi using `foundry_team`.
- [ ] Research Codex plugin conventions and document support/blockers.

## Acceptance criteria

- [ ] A documented packaging strategy exists for both Pi extensions and Codex plugin support.
- [ ] At least one Foundry skill and one Foundry agent can be exposed through each target harness, or explicit blockers are documented.
- [x] Existing Claude plugin behavior and validation remain unchanged.
- [x] Information-barrier guarantees are preserved or stricter in the landed Pi child-dispatch slice.
