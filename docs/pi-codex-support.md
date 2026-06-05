# Foundry Pi and Codex Support

**Updated:** 2026-06-05

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

### Multi-provider / distinct-lane delegation

`foundry_team` accepts an optional `model` field on each dispatch item and passes it to the child `pi --model ...` invocation. Use this for red/green separation experiments while keeping the same PromptEnvelope barrier.

Preferred provider-diverse live pattern when Kimi/MiniMax are available:

```bash
tests/pi-live-dispatch-smoke.sh \
  --example chess-engine \
  --red-model minimax/MiniMax-M3 \
  --green-model kimi-coding/kimi-for-coding \
  --require-distinct-model-lanes \
  --run-dir runs/pi-live-kimi-minimax-chess-smoke
```

The live dispatch smoke accepts `--example sudoku-solver` (default, `30/30`), `--example rubiks-solver` (`46/46`), and `--example chess-engine` (`44/44`) so provider-diverse lanes can be exercised against deeper worked examples without running a full autonomous adversarial session.

Codex/Kimi pattern:

```bash
tests/pi-live-dispatch-smoke.sh \
  --red-model openai-codex/gpt-5.5:xhigh \
  --green-model kimi-coding/kimi-for-coding \
  --require-distinct-model-lanes \
  --run-dir runs/pi-live-multilane-smoke
```

Fallback if only Codex lanes are available:

```bash
tests/pi-live-dispatch-smoke.sh \
  --red-model openai-codex/gpt-5.5:xhigh \
  --green-model openai-codex/gpt-5.5:medium \
  --require-distinct-model-lanes \
  --run-dir runs/pi-live-multilane-smoke
```

Use the exact provider-qualified model IDs accepted by the local Pi installation. The script records the actual model lanes in `behavioral-smoke.toon`; when `requires_distinct_model_lanes: true`, `tests/behavioral-smoke.sh` rejects a run whose red and green lanes collapse to the same actual model.

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

## Codex CLI plugin support

Foundry now includes a Codex CLI plugin bundle at `.codex-plugin/plugin.json`.

Local Codex marketplace examples use:

- `.codex-plugin/plugin.json` as the required manifest.
- `skills/` as Agent Skills-compatible skill folders.
- Optional `agents/openai.yaml` agent cards.
- Optional `commands/*.md` command wrappers.
- Optional assets such as SVG/PNG icons.

Foundry's Codex manifest exposes the existing root `skills/foundry-*` adapters via:

```json
"skills": "./skills/"
```

Those adapters are still packaging glue only. They point back to the canonical Claude plugin prompts under `plugins/foundry/skills/**/SKILL.md`; do not fork workflow instructions into Codex-specific files.

### Codex commands and agent metadata

The Codex bundle also includes:

- `commands/foundry-adversarial.md` — thin command wrapper that routes to `skills/foundry-adversarial/SKILL.md` and then to the canonical adversarial skill.
- `commands/foundry-forge.md` — thin command wrapper for the full pipeline.
- `agents/openai.yaml` — plugin-level agent card metadata.

Current blocker: cached Codex marketplace examples expose `agents/openai.yaml` as an agent card, but the installed Codex CLI help does not document a Claude-style dispatchable subagent API. Therefore Foundry does **not** expose the 24 canonical reviewer prompts as Codex-native subagents yet. The canonical reviewer prompts remain under `plugins/foundry/agents/**/*.md` for orchestrators/tools that can enforce PromptEnvelope isolation.

### Codex install notes

The installed CLI exposes:

```bash
codex plugin marketplace add <SOURCE>
codex plugin marketplace upgrade <SOURCE>
codex plugin marketplace remove <SOURCE>
```

`add` accepts owner/repo refs, Git URLs, SSH URLs, or local marketplace root directories. It does not currently expose a separate documented local `plugin install ./` command in `codex plugin --help`.

Local development smoke-load (validated with a temporary `HOME` on 2026-05-24):

```bash
codex plugin marketplace add /path/to/foundry
```

Then validate the package shape locally with:

```bash
tests/validate-codex-plugin.sh
```

### Codex barrier rule

Codex skills must preserve the same PromptEnvelope boundary as Claude and Pi. If a Codex runtime does not provide an isolated subagent/team primitive, do **not** simulate red/green isolation by pasting hidden context into the main conversation. Use a harness/tool that consumes PromptEnvelope artifacts, or stop and report the blocker.

The tracking todo is `todos/pi-codex-plugin-support.md`.
