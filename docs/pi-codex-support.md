# Foundry Pi and Codex Support

**Updated:** 2026-05-22

This document records the public-plugin packaging strategy for non-Claude harnesses.

## Pi support

Foundry is installable as a Pi package from this repository. The package manifest is `package.json` and exposes:

- `extensions/pi-foundry-team/` — the `foundry_team` child-dispatch tool.
- `skills/` — Pi-compatible thin adapters for the canonical Foundry skills.

### Install locally while developing

From this repository:

```bash
pi install ./
```

Or test without installing:

```bash
pi -e ./extensions/pi-foundry-team/index.ts -p --no-session 'Reply with OK'
```

### Install from git

Once published/available to the target machine:

```bash
pi install git:github.com/Lightless-Labs/foundry
```

Project-scoped installs can use Pi's `-l` flag so `.pi/settings.json` records the package for the current project.

### Skills

Pi skill names must be lowercase/hyphen Agent Skills names, so Foundry exposes adapters instead of the canonical Claude-style `foundry:*` names:

| Pi skill | Canonical skill |
|---|---|
| `/skill:foundry-research` | `foundry:research` |
| `/skill:foundry-brainstorm` | `foundry:brainstorm` |
| `/skill:foundry-nlspec` | `foundry:nlspec` |
| `/skill:foundry-adversarial` | `foundry:adversarial` |
| `/skill:foundry-forge` | `foundry:forge` |

The adapters live under `skills/` and intentionally do not fork the workflow prompts. Each adapter instructs Pi to read the canonical source under `plugins/foundry/skills/**/SKILL.md` and follow it exactly.

### Red/green dispatch under Pi

Pi has no built-in Claude-style `Agent(...)` or subagent primitive. Foundry's Pi package provides `foundry_team`, which:

1. Reads `foundry.prompt-envelope.v1` JSON artifacts.
2. Validates withheld-context poison samples before dispatch.
3. Spawns isolated child `pi --mode json -p --no-session` processes.
4. Sends exactly `envelope.prompt` to the child process.
5. Reports provider-qualified `actualModel` lanes for `behavioral-smoke.toon`.

For adversarial runs, never paste red/green hidden context directly into the main Pi conversation to simulate teams. Write PromptEnvelope artifacts and dispatch through `foundry_team`.

### Validation

Fast structural checks:

```bash
tests/validate-pi-extension.sh
tests/validate-agents.sh
```

Replay-level behavioral checks:

```bash
tests/behavioral-smoke.sh
```

Slow/manual live Pi lane with real model calls:

```bash
tests/pi-live-dispatch-smoke.sh --keep
```

Smoke-scoped autonomous Pi adversarial run artifacts:

```bash
tests/validate-barrier-envelopes.sh runs/pi-autonomous-sudoku-smoke/dispatch
tests/behavioral-smoke.sh runs/pi-autonomous-sudoku-smoke
```

The `runs/pi-autonomous-sudoku-smoke/` fixture was produced by invoking `/skill:foundry-adversarial` under Pi with `foundry_team`, copying the Sudoku worked example to `/tmp`, running the red tests (`30/30`), and dispatching red-team, green-team, and barrier-integrity-auditor from PromptEnvelope artifacts.

## Codex support

Codex support remains pending. Current known strategy:

- Keep canonical prompts in `plugins/foundry/skills/` and `plugins/foundry/agents/`.
- Add only thin adapters or generated packaging for Codex once Codex's current plugin/skill conventions are confirmed.
- Preserve the same PromptEnvelope boundary for any Codex child-dispatch mechanism.
- Do not invent a Codex-specific red/green API without validating it against the active Codex harness docs/runtime.

The tracking todo is `todos/pi-codex-plugin-support.md`.
