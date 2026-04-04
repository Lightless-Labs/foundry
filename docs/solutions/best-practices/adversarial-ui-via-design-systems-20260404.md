---
title: "Adversarial UI testing via design systems as testable specifications"
module: foundry-workflow
date: 2026-04-04
problem_type: best_practice
component: development_workflow
severity: medium
applies_when:
  - "Applying adversarial workflow to frontend or UI work"
  - "Design system exists or can be codified"
  - "UI correctness needs mechanical verification"
tags:
  - adversarial-ui
  - design-system
  - visual-testing
  - llm-comparator
  - generative-composition
---

# Adversarial UI Testing via Design Systems as Testable Specifications

## Three-Level Testing

**Level 1: Mock matching.** Screenshot vs reference image comparison. Straightforward, deterministic, but only covers known states.

**Level 2: Held-back instances.** Content and states the green team never sees during development. Tests generalization beyond the examples provided.

**Level 3: Generative composition.** Red team invents novel layouts from design system rules — impossible to game because the test cases don't exist until test time. LLM-as-visual-comparator evaluates whether the rendered output conforms to the design system.

## Key Insight

The design system IS the NLSpec for UI. It defines the constraints, tokens, and composition rules that both teams work from. This makes adversarial UI testing a natural fit for the red/green workflow.

## Status

This is a brainstorm/investigation, not implemented yet. The three-level framework and LLM-as-comparator approach need validation through a real project.
