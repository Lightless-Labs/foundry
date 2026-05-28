# Foundry Plugin Handoff

Read this at the start of every session. Update it before context compaction or at natural milestones.

**Last updated:** 2026-05-28

## What This Repo Is

The **public plugin** for Foundry — an adversarial red/green development workflow where isolated agent teams develop features from a shared NLSpec without seeing each other's work.

Installable via:
```bash
claude plugin marketplace add github:Lightless-Labs/foundry
claude plugin install foundry
```

This repo is the **skills + agents + examples** side. The Rust engine (state machine, concurrent dispatch, write lock coordination) lives in the private monorepo at `lightless-labs/lightless-labs/foundry/`.

## Current State

### Validation: 224/224 checks passing + generic workflow evals + arbiter evals + replay/Pi/Codex/module self-tests + Pi adversarial smoke

`tests/validate-agents.sh` covers structural (YAML frontmatter, required sections, model: inherit, tools), attribution (12 adopted agents), language-specific coverage, adversarial process coverage including the scoped arbiter, and territory boundaries.

Additional validators:
- `tests/validate-barrier-envelopes.sh` — PromptEnvelope v1 replay/audit checks, including green PASS/FAIL-only gates and arbiter single-test scope gates.
- `tests/behavioral-smoke.sh` — replay-level run checks over PromptEnvelope artifacts plus `behavioral-smoke.toon` summaries (example pass rates, model lanes, divergence restart counts).
- `tests/foundry-evals.sh` — generic deterministic Gherkin-authored workflow eval runner. Current suites: `arbiter-routing` (mocked arbiter routes) and `green-followup-barrier` (green receives only NLSpec How plus PASS/FAIL labels).
- `tests/arbiter-routing-evals.sh` — compatibility wrapper for the generic `arbiter-routing` suite; still accepts the old fixture path.
- `tests/validate-behavioral-smoke-contract.sh` — ensures `foundry-adversarial` requires real runs to emit/validate `behavioral-smoke.toon`.
- `tests/validate-pi-extension.sh` — ensures the Pi package exposes the `foundry_team` child-dispatch extension and uses the PromptEnvelope contract, including runtime arbiter scope rejection.
- `tests/validate-codex-plugin.sh` — ensures the Codex CLI plugin manifest exposes the root Agent Skills adapters, canonical-source links, command wrappers, agent-card blocker notes, and barrier-language anchors.
- `tests/validate-adversarial-modules.sh` — ensures extracted adversarial playbooks preserve divergence routing, scoped arbiter routing/evals, `spec_update_and_restart`, provider troubleshooting, and critical grep anchors.
- `tests/pi-live-dispatch-smoke.sh` — slow/manual live lane that performs real Pi model calls, runs Sudoku `30/30`, dispatches red/green child Pi processes through `foundry_team`, writes `behavioral-smoke.toon`, and validates the resulting run directory.
- `runs/pi-autonomous-sudoku-smoke/` — smoke-scoped autonomous `/skill:foundry-adversarial` Pi run artifacts (red-team, green-team, barrier-integrity-auditor PromptEnvelopes + `behavioral-smoke.toon`).

Last local validation (2026-05-28): `tests/foundry-evals.sh` passed 2 generic suites / 6 cases (`arbiter-routing`, `green-followup-barrier`); `tests/arbiter-routing-evals.sh tests/fixtures/arbiter-routing-evals.feature` passed the compatibility path; `tests/foundry-evals.sh --suite green-followup-barrier` passed 2/2; `tests/validate-adversarial-modules.sh` passed 62/62 with generic eval anchors; `tests/validate-agents.sh` passed 224/224. Previously on 2026-05-26: `tests/validate-codex-plugin.sh` passed 44/44; `tests/validate-pi-extension.sh` passed 43/43; `tests/validate-behavioral-smoke-contract.sh` passed 7/7; `tests/validate-barrier-envelopes.sh` self-tests passed including outcome-label withheld-sample regression plus good/bad arbiter scope cases; `tests/validate-barrier-envelopes.sh runs/pi-autonomous-sudoku-smoke/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-autonomous-sudoku-smoke` passed; `tests/validate-barrier-envelopes.sh runs/pi-from-scratch-roman-numeral/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-from-scratch-roman-numeral` passed. Last slow live lane remains 2026-05-22: `tests/pi-live-dispatch-smoke.sh --keep` passed with a real `foundry_team` Pi tool call; `/skill:foundry-adversarial` under Pi produced `runs/pi-autonomous-sudoku-smoke/` with Sudoku `30/30` and provider-qualified `openai-codex/gpt-5.5` model lanes. New from-scratch live Pi run (2026-05-24) produced `runs/pi-from-scratch-roman-numeral/` with fresh red tests, fresh green implementation, and Roman numeral `8/8`.

### 5 Skills (composable pipeline)

| Skill | Triggers | Output |
|-------|----------|--------|
| `foundry:research` | "research this", "investigate" | Research context doc |
| `foundry:brainstorm` | "brainstorm", "spec this out" | Spec document (via user dialogue) |
| `foundry:nlspec` | "nlspec", "derive nlspec" | NLSpec (Why/What/How/Done) |
| `foundry:adversarial` | "adversarial", "red green" | Implementation + tests with information barrier |
| `foundry:forge` | "forge", "full pipeline" | All 4 above in sequence, gated |

Each skill can be invoked independently. Forge composes them with gates between phases and skip logic (existing artifacts skip their phase).

### Pi and Codex package support

Root `package.json` makes this repo installable as a Pi package (`pi-package` keyword) and exposes `extensions/pi-foundry-team/` plus Agent Skills-compatible adapters under `skills/`.

`foundry_team` is the Pi-side team/subagent primitive. Pi intentionally has no built-in subagents; this extension follows Pi's officially shipped `examples/extensions/subagent/` pattern by spawning child `pi --mode json -p --no-session` processes. It dispatches from PromptEnvelope paths, validates withheld samples first, disables child sessions/extensions/skills/prompt-templates/context-files by default, reports provider-qualified actual model lane IDs when Pi exposes provider+model, and reuses canonical agent prompts from `plugins/foundry/agents/**/*.md`.

Agent Skills adapters expose `/skill:foundry-research`, `/skill:foundry-brainstorm`, `/skill:foundry-nlspec`, `/skill:foundry-adversarial`, and `/skill:foundry-forge`. They are thin wrappers that instruct Pi/Codex to read the canonical Claude plugin skill files under `plugins/foundry/skills/**/SKILL.md`; do not fork workflow prompts into the adapters.

Codex CLI packaging lives at `.codex-plugin/plugin.json` and exposes `"skills": "./skills/"`, `commands/foundry-{adversarial,forge}.md`, `agents/openai.yaml`, and `assets/foundry-codex.svg`. Local smoke-load with a temporary HOME succeeded via `codex plugin marketplace add "$PWD"` on 2026-05-24. Current Codex blocker: examples expose `agents/openai.yaml` as an agent card, but the installed CLI does not document a Claude-style dispatchable subagent API; canonical Foundry reviewers remain under `plugins/foundry/agents/**/*.md` until a PromptEnvelope-safe Codex dispatch primitive is confirmed.

### 25 Agents

**Adversarial process (7):** arbiter-agent, barrier-integrity-auditor, divergence-evaluator, green-team-reviewer, red-team-test-reviewer, nlspec-fidelity-reviewer, spec-completeness-reviewer

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
| `todos/spec-divergence-feedback-loop.md` | P2 | **MERGED** — `divergence-evaluator` agent + adversarial skill Phase 1b/2b/restart extensions. 93/93 red team tests pass. Merged via PR #1 on 2026-04-08. |
| `todos/repo-identity-public-plugin.md` | High | **COMPLETED 2026-05-01** — root `AGENTS.md`/`CLAUDE.md` now identify this as the public plugin/skills/agents repo and call out the private Rust engine split |
| `todos/mechanical-barrier-enforcement.md` | High | **PUBLIC + PRIVATE DISPATCH CONTRACT LANDED** — public plugin `PromptEnvelope` v1/replayable artifact contract landed 2026-05-01; private BuildKite/pi dispatch runtime mirrors it with prompt-envelope artifacts and `test-prompt-envelope.sh` as of 2026-05-03 |
| `todos/behavioral-smoke-tests.md` | High | **COMPLETED 2026-05-22** — replay harness + Pi dispatch primitive + slow/manual Pi live dispatch smoke + smoke-scoped autonomous `/skill:foundry-adversarial` run landed. `runs/pi-autonomous-sudoku-smoke/` validates with behavioral-smoke and barrier validators |
| `todos/modularize-heaviest-skills.md` | Medium | **COMPLETED FIRST SLICE 2026-05-24** — extracted divergence routing, `spec_update_and_restart`, and provider troubleshooting playbooks; added `tests/validate-adversarial-modules.sh`; continue profiling future runs before further extraction |
| `todos/pi-codex-plugin-support.md` | Medium | **COMPLETED 2026-05-24** — Pi package manifest + `foundry_team` extension + Agent Skills adapters + Codex CLI `.codex-plugin/plugin.json`, command wrappers, agent card, validation, docs, and local smoke-load landed |
| `todos/from-scratch-pi-adversarial-run.md` | Medium | **COMPLETED 2026-05-24** — fresh Rust Roman numeral feature under Pi; red/green artifacts generated from scratch; 8/8 tests pass; barrier and behavioral validators pass |
| `todos/generalize-workflow-evals.md` | High | **COMPLETED 2026-05-28** — generic `tests/foundry-evals.sh` runner + adapter layout landed; arbiter routing ported; green follow-up barrier added as first non-arbiter suite; old arbiter command remains a compatibility wrapper |
| `todos/arbiter-agent.md` | Future | **COMPLETED 2026-05-26** — added `arbiter-agent`, scoped arbitration playbook, adversarial skill routing, barrier-auditor guidance, validator coverage, and arbiter PromptEnvelope scope hardening |
| `todos/phase2-trigger-strategy.md` | Future | Re-assess Phase 2 divergence trigger strategy (N=3 fixed vs pattern-based) |
| `todos/adversarial-ui-investigation.md` | Future | Three-level adversarial testing via design systems |

## Information Barrier (core invariant)

| Entity | Sees | Never sees |
|--------|------|------------|
| Red team | NLSpec (full), spec | Implementation code |
| Green team | NLSpec How section only, test outcome labels (PASS/FAIL) | Test code, assertions, error messages, NLSpec Done section |
| Arbiter agent | Full spec/NLSpec, one disputed test artifact, relevant implementation snippet, one runner result | Full red suite, full implementation, broad red/green conversation history |
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
| adversarial (arbitration, conditional) | arbiter-agent for one disputed test at a time after normal divergence routing or suspicious-pass evidence |
| adversarial (final review) | green-team-reviewer, red-team-test-reviewer, barrier-integrity-auditor, language-specific (auto), correctness, testing, reliability |

## Key Learnings

- **Golden test vectors are non-negotiable** — NLSpecs for state transformations MUST include concrete reference outputs. Without them, red and green diverge at the convention level (Rubik's failure vs Chess success).
- **Orchestrator must be stateless** — reading both sides and fixing code directly breaks provenance, even if all tests pass. Route fixes through the proper team.
- **Red team test data quality** — wrong test data under the information barrier amplifies damage. Green team can't see the test code to identify bad inputs. Verify against authoritative sources.
- **NLSpec derivation errors** — agents mix inputs from one source with outputs from another (e.g., FEN from position A with perft numbers from position B). Cross-check vectors against research docs.
- **Multi-provider strengthens the adversarial property** — different model families have different blind spots. Red on Gemini, green on Codex, orchestrator on Claude. Infrastructure ready, not yet systematically exercised.
- **Research as reflex** — fire research after every user reply during brainstorm when unknowns surface, not as a one-shot phase before brainstorm.
- **Deferred commit pattern** — in a two-commit before/after sequence, the pre-operation commit must come AFTER the operation succeeds, not before. If the operation can fail, committing before ties your hands. Also guard with `git diff --staged --quiet` to handle nothing-staged edge case. See `docs/solutions/workflow-issues/deferred-commit-pattern-20260408.md`.
- **Grep anchors in skill docs** — test scripts that grep for "Phase 2b" near "VALUABLE" require both terms on the same line. Context labels in routing sections (e.g., "Phase 2b \`VALUABLE\`") serve double duty as documentation and grep anchors. Remove them and tests silently regress.
- **Ephemeral evaluator output shape** — evaluators that follow the reviewer schema return `findings[0].outcome`, not a top-level `outcome` field. Routing logic and all prose references must use `findings[0].*`, not `DivergenceJudgment.*`. The two names diverge unless explicitly kept in sync.
- **Pi has no native subagents** — do not write Pi instructions that assume Claude-style `Agent(...)`, teams, or swarms. Roll the primitive as an extension. The endorsed pattern is Pi's own `examples/extensions/subagent/`: spawn child `pi --mode json -p --no-session` processes, bound concurrency, stream/capture JSON events, and keep child contexts explicit.
- **PromptEnvelope is the cross-harness dispatch boundary** — Claude can pass `envelope.prompt` to `Agent(...)`; Pi must call `foundry_team` with `envelopePath`. Never paste hidden context into normal Pi messages to simulate a subagent.
- **Green test-result blocks need a hard section terminator** — `validate-barrier-envelopes.sh` treats lines after `Test results:` as outcome labels until the next `#`/`##` header. Put follow-up instructions under `## Task`; otherwise ordinary prose such as `Reply exactly: GREEN_OK` is correctly rejected as a non-PASS/FAIL result leak.
- **Codex plugin support is packaging, not subagents (yet)** — local Codex examples use `.codex-plugin/plugin.json`, `skills/`, optional `commands/`, and `agents/openai.yaml` agent cards. The installed CLI can smoke-load this repo as a local marketplace, but does not document a Claude-style dispatchable subagent API. Keep canonical reviewer prompts under `plugins/foundry/agents/**/*.md` until a PromptEnvelope-safe Codex dispatch primitive exists.
- **Module extraction needs validators for old grep anchors** — moving bulky adversarial instructions into playbooks is safe only if tests preserve anchor strings such as `findings[0].outcome`, Phase 2b `VALUABLE`, `spec_update_and_restart`, and PASS/FAIL-only barrier language.
- **From-scratch Pi works but needs resumable/longer live orchestration** — Roman numeral run generated fresh red tests and green code and reached Phase 3, but the outer 900s shell timeout interrupted reviewer fan-out. Continue via PromptEnvelope/foundry_team worked, but future live lanes should use longer timeouts, sessions, or phase-level resumability.
- **Withheld samples must exclude allowed outcome labels** — a continuation envelope used a test name as a withheld red-test sample while the same name was allowed in `Test results:`. `foundry_team` correctly rejected it. This is now mechanically checked in `tests/validate-barrier-envelopes.sh` and `extensions/pi-foundry-team/index.ts`; samples should come from assertion/body/raw-output snippets, not PASS/FAIL label names.
- **Scoped arbitration is a controlled breach, not a new normal** — `arbiter-agent` may see one disputed test, the relevant implementation snippet, full spec/NLSpec, and one runner result. Its raw context/output goes only to the orchestrator; red/green follow-up must be redacted back to normal barrier rules. `tests/validate-barrier-envelopes.sh` and `foundry_team` now require `ArbiterInput`, exactly one `disputed_test`/`test_artifact`, scoped visible-context categories, withheld samples, and `single_test_scope` redaction metadata.
- **Workflow evals can be Gherkin-authored mocks** — `tests/foundry-evals.sh` runs adapter-backed `.feature` suites under `tests/evals/features/` without live model calls. Current suites mock arbiter JSON outputs and generate green follow-up PromptEnvelopes, then delegate mechanical leak checks to `validate-barrier-envelopes.sh`. Keep live model evals as a separate slow lane that can reuse the same scenarios later.

## What's Next

Ilia feedback (2026-04-17, `docs/solutions/workflow-issues/ilia-feedback-foundry-plugin-20260417.md`) raised four structural items. Repo identity is complete, the private dispatch runtime mirrors the public `PromptEnvelope` v1 contract, and a replay-level behavioral smoke harness now exists. The remaining suggested order is:

1. **Extend workflow eval coverage** — generic Gherkin/mock evals now exist; next useful suites are divergence routing, red follow-up barrier, spec update/restart, reviewer fan-out, and full phase choreography.
2. **Codex dispatch follow-up** (`todos/pi-codex-plugin-support.md`) — packaging is done; revisit only when Codex documents a PromptEnvelope-safe dispatchable subagent/team primitive.
3. **Continue modularization only from evidence** (`todos/modularize-heaviest-skills.md`) — first slice, arbiter routing/scope validation/evals, and Roman-run hardening are done; profile future real runs before extracting more modules.

Also still open from before:

4. **Multi-provider delegation** — systematically exercise red-on-Gemini, green-on-Codex across examples
5. **Adversarial UI** — brainstorm at `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md`; three-level testing via design systems
6. **Rubik's cube fix** — add golden vectors from Kociemba's Python reference (31/46 -> ~44/46)
7. **Phase 2 trigger strategy** — re-assess N=3 vs pattern-based (`todos/phase2-trigger-strategy.md`)
8. **Exercise spec-divergence/arbiter loops live** — deterministic arbiter evals now exist; next step is a real end-to-end example that triggers divergence evaluator and scoped arbiter behavior in practice

## Repo Layout

```
public/foundry/
├── .codex-plugin/plugin.json            (Codex CLI plugin manifest)
├── agents/openai.yaml                   (Codex agent card metadata)
├── assets/foundry-codex.svg             (Codex icon)
├── commands/                            (Codex thin command wrappers)
├── package.json                         (Pi package manifest)
├── runs/
│   ├── pi-autonomous-sudoku-smoke/      (validated Pi adversarial smoke artifacts)
│   └── pi-from-scratch-roman-numeral/   (fresh Pi adversarial Roman numeral run, 8/8)
├── .claude-plugin/marketplace.json
├── extensions/
│   └── pi-foundry-team/                 (Pi `foundry_team` dispatch extension)
├── skills/                              (Pi/Codex Agent Skills adapters)
├── plugins/foundry/
│   ├── .claude-plugin/plugin.json
│   ├── agents/
│   │   ├── document-review/     (2 agents)
│   │   ├── research/            (1 agent)
│   │   └── review/              (22 agents)
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
├── tests/
│   ├── behavioral-smoke.sh
│   ├── pi-live-dispatch-smoke.sh        (slow/manual, real Pi model calls)
│   ├── foundry-evals.sh                    (generic Gherkin/mock workflow evals)
│   ├── evals/                              (generic eval runner, features, adapters)
│   ├── arbiter-routing-evals.sh            (compatibility wrapper)
│   ├── fixtures/arbiter-routing-evals.feature
│   ├── validate-adversarial-modules.sh
│   ├── validate-agents.sh
│   ├── validate-barrier-envelopes.sh
│   ├── validate-behavioral-smoke-contract.sh
│   ├── validate-codex-plugin.sh
│   └── validate-pi-extension.sh
├── docs/
│   ├── brainstorms/
│   ├── plans/
│   ├── playbooks/                       (extracted adversarial workflow modules + Pi continuation)
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
