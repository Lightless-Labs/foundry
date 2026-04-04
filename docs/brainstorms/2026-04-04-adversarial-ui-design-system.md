---
date: 2026-04-04
topic: adversarial-ui-design-system
---

# Adversarial Red/Green for UI: Design Systems as Testable Specifications

## The Problem

"Design a beautiful UI" is an open-ended problem subject to interpretation. The adversarial red/green process needs objective pass/fail signals, and aesthetic judgment isn't one. How do you apply adversarial development to frontend work?

## The Insight

A **design system** is a testable specification. It defines tokens (spacing, colors, typography), components (cards, buttons, inputs), layout rules (grids, breakpoints, gaps), and composition patterns. Once codified, a design system makes UI correctness as mechanically verifiable as a chess engine's move generation.

## Three-Level Testing Strategy

### Level 1: Mock Matching (Reference Comparison)

The spec includes Figma mocks for key screens. The red team writes tests that render the implemented components and compare screenshots against the reference mocks. Pass/fail is a visual diff (pixel-level or perceptual).

- **Green sees:** "login_screen: FAIL", "dashboard_card: PASS"
- **Green doesn't see:** the reference images, the diff, what specifically looks wrong
- **Golden test vectors:** the mock images themselves

This is the minimum viable adversarial UI test.

### Level 2: Held-Back Instances (Content/State Variations)

The red team gets design mocks for specific content and states that the green team never sees. Same components, different data — long titles, empty states, error states, RTL text, extreme aspect ratios, single item vs many items.

Green implements the components from the design system spec. Red tests whether those components, when filled with held-back content, match the reference mocks.

- **Green sees:** "card_with_long_title: FAIL", "empty_state_dashboard: PASS"
- **Green doesn't see:** the specific content used, the reference images for those states
- **Information barrier preserved:** green knows which states to handle from the design system spec's state documentation, but doesn't know which specific content the red team chose

This catches the common failure mode: components that look right with lorem ipsum but break with real content.

### Level 3: Generative Composition (Design System Correctness)

The red team knows the design system rules. It can compose novel screens that never existed as mocks but are fully specified by the system. "Three cards in a 2-column grid with 12px gap at 768px viewport" — the red team knows exactly what this should look like from the spacing tokens, grid rules, and responsive breakpoints.

The red team generates test compositions by:
1. Selecting components from the design system
2. Placing them in layouts defined by the grid system
3. Filling them with varied content
4. Computing the expected layout from the design system rules
5. Rendering and comparing against the expectation

The comparator at this level is a lightweight LLM that evaluates against the design system spec directly:
- Are paddings/margins correct per the spacing scale?
- Does the typography hierarchy hold (h1 > h2 > body)?
- Are all colors from the palette?
- Does the layout respond to viewport width as the responsive spec says?
- Are interactive states (hover, focus, disabled) visually distinct?

This is a **design system correctness verifier**, not a visual diff tool.

- **Green sees:** "novel_composition_3col_768px: FAIL", "typography_hierarchy_check: PASS"
- **Green doesn't see:** the composition, the LLM's evaluation, or the specific rules that failed
- **Green can't game it:** infinite novel compositions are possible from the system rules, so green must implement the actual design system, not pixel-match specific mocks

## Information Barrier for UI

| Entity | Sees | Never sees |
|--------|------|------------|
| Red team | Design system spec (full), reference mocks (all), held-back content | Implementation code, component source |
| Green team | Design system spec (full), reference mocks (subset — not held-back ones), test outcome labels | Reference mocks for held-back states, red's test compositions, LLM evaluation details |
| Green reviewer | Implementation + outcomes | Test compositions, reference mocks for held-back states |
| LLM comparator | Rendered screenshot + design system spec (for level 3) or reference image (for levels 1-2) | Implementation code |

## Key Decision: Design System as NLSpec

The design system document IS the NLSpec for UI work:
- **Why:** product goals, brand identity, user experience principles
- **What:** tokens (spacing scale, color palette, typography scale, breakpoints), component catalog, layout system
- **How:** component implementation guidance, responsive behavior rules, animation specs, accessibility requirements
- **Done:**
  - Level 1: screenshots match reference mocks
  - Level 2: held-back content/states render correctly
  - Level 3: novel compositions obey system rules (LLM-verified)

## Practical Implementation

### Red team tooling
- Screenshot capture: Playwright, Puppeteer, or `agent-browser screenshot`
- Visual comparison (levels 1-2): `pixelmatch`, `resemblejs`, or perceptual hash
- Design system evaluation (level 3): lightweight LLM (flash-tier) with the design system spec as system prompt, screenshot as input, structured output (pass/fail per rule)

### Green team tooling
- Standard frontend framework (React, SwiftUI, etc.)
- Design system spec as the implementation guide
- Test outcome labels only: `component_state: PASS/FAIL`

### Outcome filtering
Same as code adversarial: green sees only test names and pass/fail. "card_long_title_rtl: FAIL" — not "the right padding is 12px but should be 16px per the spacing scale."

## Open Questions

- **LLM comparator reliability:** How consistent is a lightweight LLM at evaluating design system compliance from screenshots? Needs experimentation. May need calibration examples in the system prompt.
- **Threshold for "close enough":** Pixel-perfect is too strict (browser rendering differences). Perceptual hash is too loose (misses spacing errors). The right threshold depends on the design system's tolerance spec.
- **Animation/interaction testing:** Levels 1-3 are static screenshots. Interactive states (hover, focus, transitions) need video or multi-frame comparison. Deferred to future work.
- **Accessibility as part of the adversarial process:** Screen reader output, tab order, contrast ratios — these are mechanically testable and could be red team tests. Natural extension.

## Relationship to Existing Foundry Skills

This would be a new skill: `foundry:adversarial-ui` that extends `foundry:adversarial` with:
- Screenshot-based test execution instead of Cucumber
- LLM comparator for level 3 evaluation
- Design system spec as the NLSpec format
- Image-based outcome labels ("PASS/FAIL" with no visual details to green)

The existing foundry:adversarial skill handles the orchestration. The UI-specific parts are: how tests are defined (compositions + screenshot comparison), how they're executed (render + capture + compare), and what the outcome labels contain.
