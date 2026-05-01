---
title: Fix repo identity — public plugin/skills/agents, not Rust+Bazel product
origin: 2026-04-17 ilia-feedback-foundry-plugin (item 1)
priority: high
status: completed
completed: 2026-05-01
---

# Repo Identity: Public Plugin, Not Product

**Completed:** 2026-05-01 — root `AGENTS.md` and `CLAUDE.md` now identify this checkout as the public plugin/skills/agents repo, with the Rust engine split called out explicitly.

Root `AGENTS.md` and `CLAUDE.md` used to read as a Rust + Bazel product repo. But this checkout IS the public Claude plugin / skills / agents layer — the Rust engine lives in the private engine repo at `lightless-labs/lightless-labs/foundry/`.

## What to fix

- Root `AGENTS.md` / `CLAUDE.md` should state plainly what this repo is: the installable Foundry plugin (skills, agents, examples). Rust-specific language should move to the engine repo's instructions.
- Stack section currently says "Rust + Bazel" — misleading here. Should describe the plugin surface: YAML frontmatter conventions, skill/agent file layout, validation harness.
- Make the dual-repo split explicit at the top: "public plugin (this repo)" ↔ "private engine (lightless-labs/lightless-labs/foundry)".

## Suggested approach

Rewrite root `CLAUDE.md` first. Mirror the relevant parts into `AGENTS.md`. Cross-link to the engine repo's HANDOFF and CLAUDE.md so operators know where Rust work lives.

See: `docs/solutions/workflow-issues/ilia-feedback-foundry-plugin-20260417.md` (item 1).
