---
date: 2026-04-05
topic: spec-divergence-feedback-loop
---

# Research: Spec Divergence Feedback Loop

## Codebase Context

- Language: Skill/agent definitions (Markdown with YAML frontmatter), Rust examples
- Build system: Bazel
- Test framework: tests/validate-agents.sh (207/207 structural checks)

## Existing Work

- **Todo:** `todos/spec-divergence-feedback-loop.md` — the full framing; re-run nlspec agent with enriched input, never amend in place
- **Incident:** `docs/solutions/workflow-issues/orchestrator-reconciliation-breaks-provenance-20260401.md` — commit dbf64c8, orchestrator directly rewrote step definitions, correctness guarantee voided
- **Convention risk:** `docs/solutions/best-practices/golden-test-vectors-as-convention-anchors-20260404.md` — Rubik's (no vectors, 31/46) vs Chess (with vectors, 44/44)
- **Adversarial playbook:** `docs/solutions/workflow-issues/adversarial-orchestration-playbook-20260404.md`

## Insertion Points

**Phase 1b** (after red-team-test-reviewer + cucumber-reviewer + barrier-integrity-auditor pass):
- Trigger: red team test references behavior not in NLSpec DoD
- Action: classify divergence → if valuable, re-run nlspec agent; if not, send red back

**Phase 2b inner loop** (when same test fails N consecutive green iterations):
- Trigger: pattern suggests API contract issue vs implementation bug
- Action: classify divergence → if valuable, re-run nlspec agent; if not, send green back

## NLSpec Re-run Input

The nlspec agent expects: original spec path + existing NLSpec path + feedback description.
Key constraint: feedback must be high-level (no raw code/test snippets) to prevent prompt injection.

```
re-run nlspec agent with:
  - original_spec: docs/specs/YYYY-MM-DD-<topic>-spec.md
  - existing_nlspec: docs/nlspecs/YYYY-MM-DD-<topic>.nlspec.md
  - feedback: structured description of divergence (category + description + evidence summary)
```

After re-run: pipeline restarts from red team against new NLSpec.

## Relevant Code

- `plugins/foundry/skills/foundry-adversarial/SKILL.md` — Phase 1b lines ~92-131, Phase 2b lines ~172-195
- `plugins/foundry/skills/foundry-nlspec/SKILL.md` — Phase 1 (spec analysis), Phase 4 (handoff)
- `plugins/foundry/agents/review/spec-completeness-reviewer.md` — closest existing agent; hunts for ambiguous behaviors where red and green would diverge
- `plugins/foundry/agents/review/nlspec-fidelity-reviewer.md` — gap-detection logic (inverted: "should X be in spec?" vs "is X in spec?")
- `tests/validate-agents.sh` — 20 mandatory checks any new agent must pass

## Three Divergence Categories

1. **Convention mismatch** — both teams internally correct, mutually incompatible. Not fixable by re-running nlspec alone; needs golden vectors added to spec. Example: Rubik's cube facelet permutation.
2. **Spec gap** — team surfaced real missing behavior. nlspec agent can fix by re-deriving. Example: third-thoughts "State Emission Probabilities" vs "State Characteristics".
3. **Team error** — test/implementation exceeds spec scope. Send team back. Not a spec problem.

## Agent Validation Requirements

Any new divergence-evaluator agent must pass 20 checks in validate-agents.sh:
- YAML frontmatter: `name`, `description`, `model: inherit`, `tools: Read Grep Glob Bash`, `color`
- "## What you're hunting for" with specific gap categories
- "## Confidence calibration" with three tiers: high (0.80+), moderate (0.60-0.79), low (<0.60)
- "## What you don't flag" with territory boundary referencing at least one other agent by name
- "## Output format" with JSON schema: `{reviewer, findings[], residual_risks[], testing_gaps[]}`

## Key Constraints

- **No direct NLSpec amendment** — nlspec agent owns authorship; orchestrator never patches
- **Ephemeral evaluator** — no persistent context across invocations; scoped to one divergence at a time
- **Prompt injection risk** — teams could embed instructions in variable names, comments, error messages to influence evaluator judgment
- **Filesystem isolation** — agents suppress uncertainty 85.5% of the time; prompt-level barriers are unreliable
- **Convention mismatch is a separate escalation path** — requires golden vectors, not just nlspec re-run

## Open Questions

1. **Who decides "valuable"?** Agent alone, or human confirmation required for P1/P2 divergences?
2. **Wrong evaluation failure modes** — false negative (signal lost) vs false positive (wasted nlspec re-run cycle). Which is worse?
3. **Convention mismatch escalation** — separate path from spec gap, or same path with different resolution?
4. **Prompt injection hardening** — does evaluator see code at all, or only orchestrator-written descriptions?
5. **Handoff after nlspec re-run** — restart from Phase 1 (red rewrites tests) or Phase 1b (re-review existing red tests against new NLSpec)?
6. **Trigger threshold for Phase 2** — after how many consecutive failures on same test?
