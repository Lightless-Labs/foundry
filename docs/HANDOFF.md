# Foundry Plugin Handoff

Read this at the start of every session. Update it before context compaction or at natural milestones.

**Last updated:** 2026-04-07

## What This Repo Is

The **public plugin** for Foundry — an adversarial red/green development workflow where isolated agent teams develop features from a shared NLSpec without seeing each other's work.

Installable via:
```bash
claude plugin marketplace add github:Lightless-Labs/foundry
claude plugin install foundry
```

This repo is the **skills + agents + examples** side. The Rust engine (state machine, concurrent dispatch, write lock coordination) lives in the private monorepo at `lightless-labs/lightless-labs/foundry/`.

## Current State

### Validation: 215/215 checks passing

`tests/validate-agents.sh` covers structural (YAML frontmatter, required sections, model: inherit, tools), attribution (12 adopted agents), language-specific coverage, adversarial process coverage, and territory boundaries.

### 5 Skills (composable pipeline)

| Skill | Triggers | Output |
|-------|----------|--------|
| `foundry:research` | "research this", "investigate" | Research context doc |
| `foundry:brainstorm` | "brainstorm", "spec this out" | Spec document (via user dialogue) |
| `foundry:nlspec` | "nlspec", "derive nlspec" | NLSpec (Why/What/How/Done) |
| `foundry:adversarial` | "adversarial", "red green" | Implementation + tests with information barrier |
| `foundry:forge` | "forge", "full pipeline" | All 4 above in sequence, gated |

Each skill can be invoked independently. Forge composes them with gates between phases and skip logic (existing artifacts skip their phase).

### 24 Agents

**Adversarial process (6):** barrier-integrity-auditor, divergence-evaluator, green-team-reviewer, red-team-test-reviewer, nlspec-fidelity-reviewer, spec-completeness-reviewer

**Language-specific (6):** rust, swift, typescript, bazel, cucumber, uniffi-bridge

**Adopted from Compound Engineering (12, with attribution):** correctness, testing, reliability, maintainability, security-sentinel, api-contract, architecture-strategist, code-simplicity, data-migrations, learnings-researcher, feasibility, adversarial-document

All agents use `model: inherit`.

### 3 Worked Examples (progressive difficulty)

| Example | Result | Key Lesson |
|---------|--------|------------|
| Sudoku solver | 30/30 | Clean constraint problem, no convention ambiguity |
| Rubik's cube | 31/46 | Convention mismatch deadlock — spec lacked golden vectors. Intentional case study. |
| Chess engine | 44/44 | Golden test vectors (perft numbers) prevent convention mismatch; NLSpec derivation bug caught |

Each example preserves all artifacts: research doc, spec, NLSpec, red team tests, green team implementation, README walkthrough.

### Todos

| File | Priority | Status |
|------|----------|--------|
| `todos/spec-divergence-feedback-loop.md` | P2 | **IMPLEMENTED** — `divergence-evaluator` agent + adversarial skill Phase 1b/2b/restart extensions. 93/93 red team tests pass. Branch: `feedback/third-thoughts-batch4-20260406`. |
| `todos/phase2-trigger-strategy.md` | Future | Re-assess Phase 2 divergence trigger strategy (N=3 fixed vs pattern-based) |
| `todos/adversarial-ui-investigation.md` | Future | Three-level adversarial testing via design systems |

## Information Barrier (core invariant)

| Entity | Sees | Never sees |
|--------|------|------------|
| Red team | NLSpec (full), spec | Implementation code |
| Green team | NLSpec How section only, test outcome labels (PASS/FAIL) | Test code, assertions, error messages, NLSpec Done section |
| Orchestrator | Everything | — |
| Test runner | Both (execution only, no judgment) | — |

Green receives ONLY `test_name: PASS/FAIL` — no assertions, no expected values, no stack traces, no .feature file content.

## Agent Dispatch Map

| Skill Phase | Agents Dispatched |
|-------------|-------------------|
| brainstorm (review) | spec-completeness-reviewer; +adversarial-document/feasibility (conditional, 10+ reqs) |
| nlspec (review) | nlspec-fidelity-reviewer; +adversarial-document/spec-completeness (conditional, 10+ DoD) |
| adversarial (setup) | learnings-researcher |
| adversarial (red review) | red-team-test-reviewer, cucumber-reviewer, barrier-integrity-auditor |
| adversarial (final review) | green-team-reviewer, red-team-test-reviewer, barrier-integrity-auditor, language-specific (auto), correctness, testing, reliability |

## Key Learnings

- **Golden test vectors are non-negotiable** — NLSpecs for state transformations MUST include concrete reference outputs. Without them, red and green diverge at the convention level (Rubik's failure vs Chess success).
- **Orchestrator must be stateless** — reading both sides and fixing code directly breaks provenance, even if all tests pass. Route fixes through the proper team.
- **Red team test data quality** — wrong test data under the information barrier amplifies damage. Green team can't see the test code to identify bad inputs. Verify against authoritative sources.
- **NLSpec derivation errors** — agents mix inputs from one source with outputs from another (e.g., FEN from position A with perft numbers from position B). Cross-check vectors against research docs.
- **Multi-provider strengthens the adversarial property** — different model families have different blind spots. Red on Gemini, green on Codex, orchestrator on Claude. Infrastructure ready, not yet systematically exercised.
- **Research as reflex** — fire research after every user reply during brainstorm when unknowns surface, not as a one-shot phase before brainstorm.

## What's Next

1. **Merge spec-divergence branch** — PR `feedback/third-thoughts-batch4-20260406` → main; review `divergence-evaluator.md` + SKILL.md changes
2. **Multi-provider delegation** — systematically exercise red-on-Gemini, green-on-Codex across examples
3. **Adversarial UI** — brainstorm at `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md`; three-level testing via design systems
4. **Rubik's cube fix** — add golden vectors from Kociemba's Python reference (31/46 -> ~44/46)
5. **Phase 2 trigger strategy** — re-assess N=3 vs pattern-based (`todos/phase2-trigger-strategy.md`)

## Repo Layout

```
public/foundry/
├── .claude-plugin/marketplace.json
├── plugins/foundry/
│   ├── .claude-plugin/plugin.json
│   ├── agents/
│   │   ├── document-review/     (2 agents)
│   │   ├── research/            (1 agent)
│   │   └── review/              (20 agents)
│   └── skills/
│       ├── foundry-adversarial/SKILL.md
│       ├── foundry-brainstorm/SKILL.md
│       ├── foundry-forge/SKILL.md
│       ├── foundry-nlspec/SKILL.md
│       └── foundry-research/SKILL.md
├── examples/
│   ├── sudoku-solver/
│   ├── rubiks-solver/
│   └── chess-engine/
├── tests/validate-agents.sh
├── docs/
│   ├── brainstorms/
│   ├── plans/
│   └── solutions/
└── todos/
```

## Key Process Docs

- **Engine HANDOFF:** `lightless-labs/lightless-labs/foundry/docs/HANDOFF.md` — engine modules, 148 tests, state machine details
- **Adversarial playbook:** engine repo `docs/solutions/workflow-issues/adversarial-orchestration-playbook-20260404.md`
- **Golden vectors:** engine repo `docs/solutions/best-practices/golden-test-vectors-as-convention-anchors-20260404.md`
- **Orchestrator antipattern:** engine repo `docs/solutions/workflow-issues/orchestrator-reconciliation-breaks-provenance-20260401.md`

## How to Update This Doc

Before context compaction or when finishing a milestone:
1. Update validation count and agent/skill counts if changed
2. Update example results if reworked
3. Add new learnings
4. Update "What's Next" based on what was completed and what emerged
5. Update "Last updated" date
