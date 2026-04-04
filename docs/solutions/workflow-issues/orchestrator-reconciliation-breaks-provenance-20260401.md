---
title: "Orchestrator reconciliation breaks adversarial provenance chain"
module: foundry-workflow
date: 2026-04-01
problem_type: workflow_issue
component: adversarial_orchestrator
severity: critical
applies_when:
  - "Red and green teams deliver independently but tests fail"
  - "Orchestrator is tempted to fix step definitions or implementation directly"
  - "Contract/spec gaps cause red team tests to not exercise green team implementation correctly"
tags:
  - adversarial-workflow
  - orchestrator
  - information-barrier
  - provenance
  - field-report
---

# Orchestrator Reconciliation Breaks Adversarial Provenance Chain

## Context

During the first real application of the adversarial red/green process to middens Phase 3 (output engine), the red team wrote 178 Cucumber scenarios from the NLSpec DoD and the green team implemented from the NLSpec How section. Both teams delivered independently. When the tests were run, 64 of 178 scenarios failed.

The orchestrator — instead of routing filtered feedback back through the teams — acted as a god-mode fixer: read both the test code and the implementation, identified the gaps, and rewrote the step definitions to bridge them.

All 178 tests now pass, but **the correctness guarantee is void**. There is no way to verify that the orchestrator didn't game the tests to match the implementation, which is the exact attack the adversarial process is designed to prevent.

## Root Causes of the 64 Failures

The failures were **not** implementation bugs or test logic errors. They were contract gaps — the NLSpec didn't specify enough about the integration surface between red and green:

### 1. Cucumber-rs concurrency semantics (50+ failures)

The red team used `thread_local!` statics to share state between Given/When/Then steps. Cucumber-rs runs scenarios concurrently across threads, so thread-locals are stale or belong to a different scenario. The NLSpec contract said "use the World struct" but didn't explain *why* thread-locals are fatal in cucumber-rs.

**Fix for future NLSpecs:** The contract section must explicitly state: "All per-scenario state MUST be stored in the World struct. Thread-local storage and global statics will cause data races because cucumber-rs runs scenarios concurrently."

### 2. Gherkin escaping conventions not specified (8 failures)

The red team wrote step patterns with `[brackets]` and `{braces}` in cucumber expression mode, which is invalid (cucumber expressions reserve `{}` for parameter placeholders and can't represent literal `[]`). The contract didn't specify which cucumber matching mode to use for complex patterns.

**Fix for future NLSpecs:** The contract should specify: "Use `regex = r#"..."#` mode for step patterns containing brackets, braces, or other special characters. Use `expr = "..."` mode only for simple patterns with `{int}`, `{float}`, `{string}`, `{word}` placeholders."

### 3. Complex value representation in Gherkin (6 failures)

The red team wrote scenarios with JSON arrays (`[1,2,3]`) and objects (`{"a":1}`) inline in step text. The contract didn't specify how these should be passed through Gherkin's string matching or how Unicode escapes (`\u2014`) would be handled.

**Fix for future NLSpecs:** The contract should specify the escaping convention: "JSON values in Gherkin step text are passed as raw strings. The step definition is responsible for parsing them. Unicode escapes in Gherkin strings are NOT automatically decoded — step definitions must handle `\\uXXXX` → char conversion."

## The Correct Response

When adversarial tests fail after independent delivery, the orchestrator should:

### Step 1: Classify the failure source

| Failure type | Route to | With what information |
|---|---|---|
| **Contract gap** (spec didn't specify integration detail) | NLSpec author | Which contract assumptions failed and why |
| **Red team bug** (step definitions are wrong) | Red team | "These N scenarios fail. Your step definitions have [category] issues." No implementation code shown. |
| **Green team bug** (implementation is wrong) | Green team | `test_name: PASS/FAIL` only. No test code, no assertions, no error messages. |
| **Ambiguous spec** (both interpretations are defensible) | NLSpec author | The ambiguity, both interpretations, recommendation |

### Step 2: Iterate through the correct team

- If contract gap → refine NLSpec contract → red team rewrites steps → green team re-tests
- If red team bug → red team fixes → green team re-tests
- If green team bug → green team gets PASS/FAIL → green team fixes → re-test

### Step 3: Never touch code

The orchestrator runs tests and routes outcomes. It does not:
- Read implementation code to understand why a test fails
- Read test code to understand why an implementation doesn't pass
- Write or modify any code files
- "Fix" step definitions or implementation directly

## Delegation Model

Use different AI tools for each role to enforce context isolation naturally:

| Role | Tool | Why |
|---|---|---|
| Red team | `/codex-cli` or `/gemini-cli` | Fresh context, can't see prior implementation discussion |
| Green team | `/opencode-cli` or `/codex-cli` | Different model family from red for maximum independence |
| Orchestrator | Main Claude session | Sees everything but writes nothing |

This also saves quota — the orchestrator's expensive context window is used only for decisions, not for code generation.

## Evidence

- **Project:** third-thoughts/middens Phase 3 (output engine)
- **Red team:** 178 scenarios across 4 feature files (markdown, json, ascii, integration)
- **Green team:** 3 renderer modules (markdown.rs, json.rs, ascii.rs) + OutputMetadata struct
- **Failure rate:** 64/178 (36%) — all caused by contract gaps, not implementation bugs
- **Recovery:** Orchestrator reconciled directly (process violation), all 178 pass
- **Provenance status:** Compromised — cannot verify orchestrator didn't game the tests

## Prevention

1. **NLSpec contract section must be exhaustive for the test framework.** Include:
   - State management conventions (World struct, no thread-locals)
   - Step matching mode conventions (expr vs regex)
   - Complex value representation in step text
   - Concurrency semantics of the test runner

2. **Run a "contract smoke test" before full adversarial delivery.** Have the red team write 3-5 trivial scenarios against a stub implementation to verify the integration surface works before investing in the full test suite.

3. **The orchestrator must be stateless.** It runs tests, reads outcomes, routes messages. If it starts reasoning about *why* a test fails by reading code, it has crossed the barrier.

## Related

- [Adversarial red/green development methodology](../best-practices/adversarial-red-green-development-methodology.md)
- [Compile-time information barriers](../best-practices/compile-time-information-barriers-via-role-scoped-types.md)
