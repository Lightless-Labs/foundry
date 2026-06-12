---
title: Investigate adversarial UI via a tiny design-system spike
created: 2026-06-12
status: completed
completed: 2026-06-12
todo: todos/adversarial-ui-investigation.md
research: docs/research/2026-06-12-adversarial-ui-investigation-research.md
---

# Investigate Adversarial UI via a Tiny Design-System Spike

## Goal

Turn the existing adversarial UI brainstorm into a bounded, evidence-producing spike that tests whether design systems can make UI red/green work mechanically verifiable without weakening Foundry's information barrier.

## Scope

- Define a tiny design system fixture:
  - tokens: spacing, color, typography,
  - components: card, button, grid,
  - states/content: normal, long text, empty/error, responsive layout.
- Exercise the three proposed levels manually or with lightweight fixtures:
  1. **Mock matching** — public reference mock/screenshot comparison.
  2. **Held-back instances** — hidden content/state reference comparison with PASS/FAIL-only green labels.
  3. **Generative composition** — one red-generated layout from design-system rules, evaluated with a structured visual/design-system comparator prompt.
- Preserve PromptEnvelope-style barrier reasoning for all red/green/comparator artifacts.
- Document comparator reliability, threshold choices, flake risks, and whether a dedicated `foundry:adversarial-ui` skill is justified.

## Non-goals

- Do not implement private engine/state-machine support.
- Do not add heavyweight browser or image dependencies to the plugin package until the method is proven.
- Do not claim Level 3 is deterministic based on a single LLM judgment.
- Do not expose hidden reference images, visual diffs, hidden content, comparator rationales, or NLSpec Done-style criteria to green.

## Proposed Experiment Shape

- Keep the first fixture under `examples/adversarial-ui-design-system/` if it becomes durable.
- Use static HTML/CSS and/or serialized mock metadata before adding Playwright/pixelmatch dependencies.
- Use opaque green-visible labels (`T-001: PASS/FAIL`) for held-back and generated cases to avoid semantic leaks from names such as `card_long_title_rtl`.
- Store human-readable mappings and comparator rationales as orchestrator/red-only artifacts.
- If a vision-capable comparator is used, record the model, prompt, screenshot/input artifact, structured output, and any disagreement on rerun.

## Acceptance

- [x] A tiny design-system fixture exists or the spike records why no fixture was created.
- [x] Level 1 and Level 2 produce PASS/FAIL-only outcome examples without exposing hidden references/diffs to green.
- [x] One Level 3 generative-composition trial is documented with comparator prompt/output and reliability caveats.
- [x] Barrier risks specific to visual artifacts are documented.
- [x] The related todo and `docs/HANDOFF.md` are updated with the result.
- [x] Existing public-plugin validators still pass after any plugin/skill/agent changes.

## Risk Controls

- Prefer documentation and fixture artifacts over runtime dependencies for the first slice.
- Treat screenshots, diffs, hidden content, and comparator explanations as withheld red/orchestrator evidence.
- Use explicit font/viewport/DPR assumptions if screenshots are introduced.
- Calibrate thresholds from intentionally small visual differences before treating a comparator as a gate.
- Preserve failures and comparator disagreement as evidence rather than tightening prompts until the result looks clean.

## Validation Log

2026-06-12:

- Read current handoff and confirmed `main` was aligned with `origin/main` before starting.
- Pushed existing branch state; remote was already up to date.
- Created local research context in `docs/research/2026-06-12-adversarial-ui-investigation-research.md`.
- Dispatched learnings and feasibility child agents through PromptEnvelope-backed `foundry_team`; both recommended a tiny manual design-system spike before adding tooling.
- Added `examples/adversarial-ui-design-system/` with a tiny design system, public Level 1 fixtures, hidden Level 2 red cases, a Level 3 generated composition, and PASS/FAIL-only outcome artifacts.
- Validated handwritten JSON fixtures with `python3 -m json.tool`.
- Validated the Level 3 comparator PromptEnvelope with `tests/validate-barrier-envelopes.sh examples/adversarial-ui-design-system/dispatch`.
- Dispatched the Level 3 comparator envelope through `foundry_team`; result persisted at `examples/adversarial-ui-design-system/artifacts/level3-comparator-output.json` with `outcome=PASS` and residual risks noting that this was a text measurement-snapshot trial, not a real screenshot/vision run.
- `tests/validate-agents.sh` — passed 224/224.
