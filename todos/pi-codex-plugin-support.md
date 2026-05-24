---
title: Add Pi extensions and Codex CLI plugin support
origin: 2026-05-01 user request during repo-identity cleanup
priority: medium
status: completed — Pi support and Codex CLI plugin packaging landed
updated: 2026-05-24
---

# Pi Extensions and Codex CLI Plugin Support

**Addendum:** 2026-05-21 — first Pi extension slice landed while closing the behavioral-smoke live-lane gap. Added a root `package.json` Pi manifest and `extensions/pi-foundry-team/index.ts`. The extension follows Pi's officially shipped `examples/extensions/subagent/` pattern: it registers a `foundry_team` tool, validates PromptEnvelope JSON artifacts, then spawns child `pi --mode json -p --no-session` processes with isolated context windows. This is intentionally not a fork of the canonical prompts: Foundry agent prompts are discovered from `plugins/foundry/agents/**/*.md`. Codex support remains pending.

**Addendum:** 2026-05-22 — Pi skill adapters and install docs landed. Root `package.json` exposes `./skills`; `skills/foundry-{research,brainstorm,nlspec,adversarial,forge}/SKILL.md` provide Agent Skills-compatible hyphenated names and direct Pi to read canonical `plugins/foundry/skills/**/SKILL.md` sources. `docs/pi-codex-support.md` documents install and invocation. `tests/validate-pi-extension.sh` now validates the package skill manifest, all adapters, canonical links, and `foundry_team` guidance.

**Addendum:** 2026-05-22 — smoke-scoped autonomous Pi adversarial run completed using `/skill:foundry-adversarial` + `foundry_team`. The run emitted `runs/pi-autonomous-sudoku-smoke/` artifacts and passed behavioral-smoke/barrier validation. Remaining Pi hardening from a from-scratch non-example feature is tracked separately in `todos/from-scratch-pi-adversarial-run.md`.

**Addendum:** 2026-05-22 — clarified that "Codex support" means **Codex CLI plugin support**, not generic Codex documentation. Local CLI exposes `codex plugin marketplace add|upgrade|remove`; cached marketplace examples under `~/.codex/.tmp/plugins/plugins/*` use `.codex-plugin/plugin.json` plus optional `skills/`, `agents/`, `commands/`, `hooks.json`, `.app.json`, `.mcp.json`, and `assets/`. Foundry should add a Codex CLI plugin bundle that exposes the existing `skills/foundry-*` adapters and, where supported, thin agent/command wrappers around canonical `plugins/foundry/agents/**/*.md` without forking prompts.

**Addendum:** 2026-05-24 — Codex CLI plugin packaging landed. Added `.codex-plugin/plugin.json`, `assets/foundry-codex.svg`, `agents/openai.yaml`, and thin `commands/foundry-{adversarial,forge}.md` wrappers. The manifest exposes the existing root `skills/foundry-*` Agent Skills adapters (`"skills": "./skills/"`); adapters remain packaging glue and point back to canonical `plugins/foundry/skills/**/SKILL.md`. `agents/openai.yaml` documents the current Codex blocker: local examples expose agent cards, but the installed CLI does not document a Claude-style dispatchable subagent API, so canonical reviewers stay under `plugins/foundry/agents/**/*.md` until a PromptEnvelope-safe Codex dispatch primitive is confirmed. Added `tests/validate-codex-plugin.sh` and smoke-loaded the repo with `HOME=$(mktemp -d) codex plugin marketplace add "$PWD"`.

Foundry currently ships as a Claude plugin with skills, agents, examples, and validation aimed at Claude's plugin surface. To make the workflow portable across agent harnesses, add first-class support for Pi extensions and Codex CLI plugin packaging.

## What to explore

- **Pi extension packaging:** map Foundry skills/agents into Pi's extension model without weakening the red/green information barrier.
- **Codex CLI plugin packaging:** determine the Codex CLI `.codex-plugin/plugin.json` surface for skills, agents, commands, apps/MCP, and how much can be generated from the existing Claude plugin artifacts.
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
- [x] Run a real public-plugin adversarial session under Pi using `foundry_team`.
- [x] Research Codex CLI plugin conventions and document support/blockers.
- [x] Add `.codex-plugin/plugin.json` or `plugins/foundry/.codex-plugin/plugin.json` for Foundry.
- [x] Expose existing `skills/foundry-*` adapters through the Codex CLI plugin manifest.
- [x] Decide whether Codex plugin `agents/` can wrap canonical Foundry agents without prompt forks; document blockers if not.
- [x] Add `tests/validate-codex-plugin.sh` for manifest shape, skill exposure, canonical-source links, and barrier-language anchors.
- [x] Smoke-load/install the local Codex CLI plugin if the CLI supports local marketplace/plugin sources.

## Acceptance criteria

- [x] A documented packaging strategy exists for both Pi extensions and Codex CLI plugin support.
- [x] At least one Foundry skill and one Foundry agent can be exposed through each target harness, or explicit blockers are documented.
- [x] Existing Claude plugin behavior and validation remain unchanged.
- [x] Information-barrier guarantees are preserved or stricter in the landed Pi child-dispatch slice.
