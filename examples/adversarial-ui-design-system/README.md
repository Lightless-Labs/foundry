# Adversarial UI Design System Spike

This example is a tiny documentation-first spike for `todos/adversarial-ui-investigation.md`.

It tests the core idea from `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md`: a design system can act as a UI NLSpec, allowing red/green UI work to produce mechanical PASS/FAIL outcomes while preserving Foundry's information barrier.

## Contents

- `design-system.md` — tiny token/component/layout rules.
- `fixtures/public-mocks.json` — green-visible public examples for Level 1 mock matching.
- `fixtures/hidden-red-cases.json` — red/orchestrator-only held-back Level 2 cases with opaque green labels.
- `fixtures/generative-composition.json` — red/orchestrator/comparator-only Level 3 generated layout.
- `artifacts/level1-level2-outcomes.toon` — example PASS/FAIL-only outcomes.
- `dispatch/level3/ui-comparator.json` — PromptEnvelope for the Level 3 comparator trial.
- `artifacts/level3-comparator-output.json` — persisted comparator output.

## Current Result

- Level 1/2 are represented as fixture artifacts and PASS/FAIL-only outcome examples; no screenshot runtime has been added yet.
- Level 3 was trialed as a text measurement-snapshot comparator dispatch through `foundry_team`; it returned `PASS` with residual risks.
- The trial intentionally does **not** claim screenshot or vision reliability yet.

## Barrier Pattern

Green-visible hidden/generated outcomes should use opaque labels:

```text
T-101: FAIL
T-102: PASS
```

The human-readable mapping (`T-101 -> card_long_title_rtl`) stays red/orchestrator-only. Hidden content, reference screenshots, visual diffs, comparator prompts, and comparator rationales must not be sent to green.

## Validated Locally

```bash
python3 -m json.tool examples/adversarial-ui-design-system/fixtures/public-mocks.json >/dev/null
python3 -m json.tool examples/adversarial-ui-design-system/fixtures/hidden-red-cases.json >/dev/null
python3 -m json.tool examples/adversarial-ui-design-system/fixtures/generative-composition.json >/dev/null
python3 -m json.tool examples/adversarial-ui-design-system/dispatch/level3/ui-comparator.json >/dev/null
tests/validate-barrier-envelopes.sh examples/adversarial-ui-design-system/dispatch
```
