---
title: Codex CLI plugin support
created: 2026-05-24
completed: 2026-05-24
status: completed
---

# Codex CLI Plugin Support Plan

## Goal

Add a Codex CLI plugin packaging surface for Foundry without forking the canonical Foundry skill or agent prompts.

## Constraints

- Canonical workflow prompts remain in `plugins/foundry/skills/**/SKILL.md`.
- Agent/reviewer prompts remain in `plugins/foundry/agents/**/*.md`.
- Root `skills/foundry-*` adapters remain thin Agent Skills-compatible wrappers.
- The red/green information barrier and PromptEnvelope dispatch boundary must not be weakened.
- Do not invent a Codex-native subagent API unless the installed Codex CLI/runtime documents one.

## Implementation Steps

1. Research local Codex plugin examples and CLI help.
2. Add `.codex-plugin/plugin.json` that exposes the existing `skills/` adapters.
3. Add minimal Codex metadata (`agents/openai.yaml`, optional command wrappers/assets) that does not duplicate canonical prompts.
4. Update `docs/pi-codex-support.md` with current Codex support, limitations, and install notes.
5. Add `tests/validate-codex-plugin.sh` to catch manifest/adapter drift and barrier-language regressions.
6. Run existing validation plus the new Codex validator.

## Acceptance

- [x] Codex plugin manifest exists and points at the existing `skills/foundry-*` adapters.
- [x] The docs explain that Codex skill support is packaged, while Codex-native dispatchable subagent support is blocked pending a documented API.
- [x] Validation checks the manifest, all five adapters, canonical-source links, agent metadata, command wrappers, and key barrier language.
