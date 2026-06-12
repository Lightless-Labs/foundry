---
date: 2026-06-12
topic: adversarial-ui-investigation
---

# Research: Adversarial UI Investigation

## Codebase Context

Foundry's public plugin repo is primarily Markdown/YAML plugin content plus shell/Python validators. The core adversarial workflow lives in `plugins/foundry/skills/foundry-adversarial/SKILL.md`; Pi dispatch support lives in `extensions/pi-foundry-team/index.ts`; deterministic workflow evals live under `tests/evals/` and are launched by `tests/foundry-evals.sh`.

This repo does not currently contain frontend runtime dependencies, browser tooling, or a JavaScript package lock. The root `package.json` is a Pi package manifest, not an app/tooling manifest. Any UI-testing experiment should therefore avoid adding heavy dependencies until the method is proven.

## Existing Work

Relevant prior artifacts:

- `todos/adversarial-ui-investigation.md` tracks the remaining future item.
- `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md` proposes design systems as testable UI specifications.
- `docs/solutions/best-practices/adversarial-ui-via-design-systems-20260404.md` distills the core learning: the design system can act as the UI NLSpec.
- `docs/solutions/best-practices/adversarial-red-green-development-methodology.md` defines the generic red/green information barrier.

The brainstorm defines three levels:

1. **Mock matching** — compare rendered output against reference screenshots/mocks.
2. **Held-back instances** — test component generalization with hidden content/states.
3. **Generative composition** — red creates novel layouts from design-system rules and checks conformance.

## Relevant Code and Patterns

The existing adversarial workflow contributes reusable mechanics rather than UI-specific code:

- PromptEnvelope v1 artifacts before every dispatch.
- Red sees full spec/Done criteria and no implementation.
- Green sees implementation guidance plus PASS/FAIL labels only.
- Barrier validation is mechanical and replayable via `tests/validate-barrier-envelopes.sh`.
- Run evidence is summarized in `behavioral-smoke.toon` for live or replayed lanes.

For UI, the same barrier needs extra care because reference images, visual diffs, hidden content, and comparator rationales can all reveal red-side test details.

## External References

No external web research was needed for this first local slice. The key decision is not which browser library to choose yet, but how to keep the public-plugin experiment small enough to validate the method without bloating the package.

Candidate tools for a later implementation spike remain:

- Playwright or Puppeteer for browser screenshot capture.
- Pixelmatch, resemblejs, SSIM, or perceptual hash for Level 1/2 visual comparison.
- A vision-capable lightweight LLM for Level 3 structured conformance judgments.

## Test Landscape

Current validators cover plugin structure, PromptEnvelope barriers, Pi extension behavior, Codex packaging, and deterministic workflow evals. There are no screenshot or browser tests in the repo today.

A first UI investigation should therefore be documented as an experiment plan and, if implemented, keep checks lightweight:

- Prefer static fixtures and generated artifacts over adding browser dependencies initially.
- Treat PASS/FAIL-only labels as the green-visible contract.
- Keep hidden reference mocks, hidden content, visual diffs, and comparator rationales red/orchestrator-only.
- Run existing public-plugin validators after any skill/agent/package changes.

## Subagent Findings

A learnings-researcher child agent highlighted these risks:

- LLM visual comparator consistency is unproven.
- Pixel-perfect comparison may be too strict; perceptual hashes may be too loose.
- Browser, font, viewport, and device-pixel-ratio differences can create flake.
- Level 3 expected-output generation requires extremely explicit design-system rules.
- Visual artifacts increase barrier-leak risk.

A feasibility-reviewer child agent found the next slice feasible if kept small and manual:

- Create one tiny design system with card, button, grid, and minimal tokens.
- Prove Level 1 and Level 2 can emit PASS/FAIL-only outcomes.
- Try one Level 3 composition with a vision comparator and capture caveats.
- Avoid private engine changes and avoid adding heavy package dependencies until the experiment proves value.

## Open Questions

- What comparator threshold is stable across local and CI rendering?
- Can Level 3 be made deterministic enough for a Foundry gate, or should it remain reviewer evidence?
- Should green-visible UI test labels be fully opaque (`T-001`) to avoid semantic leaks from names like `card_long_title_rtl`?
- Which visual artifacts belong in `withheld_context.samples` for PromptEnvelope validation when the hidden content is binary/image-like?
- Is a dedicated `foundry:adversarial-ui` skill warranted, or should UI remain a playbook/module under `foundry:adversarial`?
