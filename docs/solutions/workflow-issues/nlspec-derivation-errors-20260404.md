---
title: "NLSpec derivation errors from source document mix-and-match and prose ambiguity"
module: foundry-workflow
date: 2026-04-04
problem_type: workflow_issue
component: development_workflow
severity: high
applies_when:
  - "NLSpec derived by AI agent from research document"
  - "Research document contains multiple variant positions or configurations"
  - "NLSpec describes conventions in prose rather than concrete examples"
tags:
  - nlspec
  - derivation-error
  - fidelity
  - mix-and-match
---

# NLSpec Derivation Errors from Source Document Mix-and-Match and Prose Ambiguity

## Failure Modes

Two distinct failure modes observed.

**Mix-and-match (chess):** NLSpec agent used FEN from one position but perft numbers from another — internally contradictory. The spec looked complete and well-formed, but components came from different source entries, making it impossible for any implementation to satisfy all constraints simultaneously.

**Prose ambiguity (Rubik's):** Conventions described in text that admitted multiple valid interpretations. Both red and green teams derived valid but incompatible concrete implementations from the same prose description.

## Fix

The nlspec-fidelity-reviewer should cross-check golden vectors against the research document, confirming all components of each vector come from the same source entry. Prose conventions should be supplemented with concrete examples that eliminate ambiguity.
