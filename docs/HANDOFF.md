# Foundry Plugin Handoff

Read this at the start of every session. Update it before context compaction or at natural milestones.

**Last updated:** 2026-06-09

## What This Repo Is

The **public plugin** for Foundry — an adversarial red/green development workflow where isolated agent teams develop features from a shared NLSpec without seeing each other's work.

Installable via:
```bash
claude plugin marketplace add github:Lightless-Labs/foundry
claude plugin install foundry
```

This repo is the **skills + agents + examples** side. The Rust engine (state machine, concurrent dispatch, write lock coordination) lives in the private monorepo at `lightless-labs/lightless-labs/foundry/`.

## Current State

### Validation: 224/224 checks passing + 8 generic workflow eval suites + arbiter evals + replay/Pi/Codex/module self-tests + Pi adversarial + multi-lane smoke + Kimi/MiniMax live lanes + phase-artifact capture + fuller provider-diverse red/green smoke

`tests/validate-agents.sh` covers structural (YAML frontmatter, required sections, model: inherit, tools), attribution (12 adopted agents), language-specific coverage, adversarial process coverage including the scoped arbiter, and territory boundaries.

Additional validators:
- `tests/validate-barrier-envelopes.sh` — PromptEnvelope v1 replay/audit checks, including green PASS/FAIL-only gates and arbiter single-test scope gates.
- `tests/behavioral-smoke.sh` — replay-level run checks over PromptEnvelope artifacts plus `behavioral-smoke.toon` summaries (example pass rates, model lanes, divergence restart counts).
- `tests/foundry-evals.sh` — generic deterministic Gherkin-authored workflow eval runner. Current suites: `arbiter-routing` (mocked arbiter routes), `divergence-routing` (Phase 1b/2b `VALUABLE`/`NOT_VALUABLE`/`INCONCLUSIVE` routing), `green-followup-barrier` (green receives only NLSpec How plus PASS/FAIL labels), `phase-choreography` (full mocked run phase order, restart/reviewer-reject branches, final validators, and behavioral-smoke artifacts), `phase2-trigger-strategy` (adaptive/fixed Phase 2b trigger decisions), `red-followup-barrier` (red receives no implementation/counterpart context), `reviewer-fanout` (Phase 3 mandatory/conditional reviewer dispatch and reviewer territory boundaries), and `spec-update-restart` (NLSpec rerun/provenance/tracker-reset/revision-cap behavior).
- `tests/arbiter-routing-evals.sh` — compatibility wrapper for the generic `arbiter-routing` suite; still accepts the old fixture path.
- `tests/validate-behavioral-smoke-contract.sh` — ensures `foundry-adversarial` requires real runs to emit/validate `behavioral-smoke.toon`.
- `tests/validate-pi-extension.sh` — ensures the Pi package exposes the `foundry_team` child-dispatch extension and uses the PromptEnvelope contract, including runtime arbiter scope rejection.
- `tests/validate-codex-plugin.sh` — ensures the Codex CLI plugin manifest exposes the root Agent Skills adapters, canonical-source links, command wrappers, agent-card blocker notes, and barrier-language anchors.
- `tests/validate-adversarial-modules.sh` — ensures extracted adversarial playbooks preserve divergence routing, scoped arbiter routing/evals, reviewer fan-out and phase-choreography eval anchors, `spec_update_and_restart`, provider troubleshooting, and critical grep anchors.
- `tests/pi-live-dispatch-smoke.sh` — slow/manual live lane that performs real Pi model calls, runs a selectable worked example (`sudoku-solver` `30/30`, `rubiks-solver` `46/46`, or `chess-engine` `44/44`), dispatches red/green child Pi processes through `foundry_team`, supports explicit per-lane model overrides, supports `--phase-task artifact-sketch` for lightweight red/green phase artifacts beyond `RED_OK`/`GREEN_OK`, writes parsed artifacts under `phase-artifacts/` when the run is kept, writes `behavioral-smoke.toon`, and validates the resulting run directory.
- `runs/pi-autonomous-sudoku-smoke/` — smoke-scoped autonomous `/skill:foundry-adversarial` Pi run artifacts (red-team, green-team, barrier-integrity-auditor PromptEnvelopes + `behavioral-smoke.toon`).
- `runs/pi-live-divergence-arbiter-smoke/` — live Pi `foundry_team` dispute-route smoke. Real child dispatches ran `foundry:review:divergence-evaluator` (`findings[0].outcome=VALUABLE`) and `foundry:review:arbiter-agent` (`findings[0].outcome=TEST_WRONG`) over scoped slugify PromptEnvelopes; barrier and behavioral validators pass.
- `runs/pi-live-multilane-smoke/` — live Pi red/green distinct-lane smoke. Real child dispatches ran red on `openai-codex/gpt-5.5:xhigh` and green on `openai-codex/gpt-5.5:medium`; `requires_distinct_model_lanes: true`, barrier validation, and behavioral validation pass.
- `runs/pi-live-kimi-minimax-smoke/` — live Pi provider-diverse Sudoku lane. Real child dispatches ran red on `minimax/MiniMax-M3` and green on `kimi-coding/kimi-for-coding`; `requires_distinct_model_lanes: true`, barrier validation, and behavioral validation pass.
- `runs/pi-live-kimi-minimax-chess-smoke/` — live Pi provider-diverse Chess lane. Real child dispatches ran red on `minimax/MiniMax-M3` and green on `kimi-coding/kimi-for-coding`; `chess-engine` `44/44`, `requires_distinct_model_lanes: true`, barrier validation, and behavioral validation pass.
- `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/` — fuller live provider-diverse red/green smoke for a from-scratch Rust `slugify_smoke` library. Real child dispatches ran red on `minimax/MiniMax-M3` to write executable integration tests and green on `kimi-coding/kimi-for-coding` to implement from the How section only; red tests passed `11/11`, `requires_distinct_model_lanes: true`, barrier validation, and behavioral validation pass.

Last local validation (2026-06-09): fuller provider-diverse adversarial smoke passed. `foundry_team` red dispatch actual model `minimax/MiniMax-M3`; `foundry_team` green dispatch actual model `kimi-coding/kimi-for-coding`; `cd runs/pi-live-kimi-minimax-fuller-adversarial-smoke/green && cargo test --quiet` passed `11/11`; `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-fuller-adversarial-smoke` passed with distinct actual model lanes; preserved run artifact has no `target/` build output. Fast gates also passed: `tests/validate-behavioral-smoke-contract.sh` 9/9, `tests/validate-pi-extension.sh` 45/45, `tests/validate-codex-plugin.sh` 44/44, and `tests/validate-agents.sh` 224/224. Previous 2026-06-08: provider-diverse phase-artifact capture passed with `tests/pi-live-dispatch-smoke.sh --phase-task artifact-sketch --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --run-dir <tmp>/phase-artifact-capture`; PromptEnvelope validation and `behavioral-smoke.toon` validation passed, and post-run assertions confirmed `phase-artifacts/red-team-test-plan.json` and `phase-artifacts/green-team-implementation-plan.json` were written with the expected JSON shapes. Default plumbing compatibility also passed via `tests/pi-live-dispatch-smoke.sh --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --run-dir <tmp>/plumbing-no-artifacts`; explicit check confirmed plumbing mode does not create `phase-artifacts/`. `bash -n tests/pi-live-dispatch-smoke.sh` passed; `tests/validate-codex-plugin.sh` passed 44/44; `tests/validate-behavioral-smoke-contract.sh` passed 9/9; `tests/validate-pi-extension.sh` passed 45/45; `tests/validate-agents.sh` passed 224/224. Previous 2026-06-07: provider-diverse phase-artifact Pi smoke passed with `tests/pi-live-dispatch-smoke.sh --phase-task artifact-sketch --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes`; red returned a JSON `red_test_plan`, green returned a JSON `green_implementation_plan`, PromptEnvelope validation and `behavioral-smoke.toon` validation passed using temporary artifacts. Default plumbing compatibility also passed via `tests/pi-live-dispatch-smoke.sh --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes`; `bash -n tests/pi-live-dispatch-smoke.sh` passed; `tests/validate-behavioral-smoke-contract.sh` passed 9/9; `tests/validate-pi-extension.sh` passed 45/45; `tests/validate-agents.sh` passed 224/224. Previous 2026-06-05: deeper provider-diverse Pi smoke passed with `tests/pi-live-dispatch-smoke.sh --example chess-engine --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --run-dir runs/pi-live-kimi-minimax-chess-smoke`; `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-chess-smoke/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-chess-smoke` passed; default Sudoku compatibility also passed via `tests/pi-live-dispatch-smoke.sh --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes`; `tests/behavioral-smoke.sh` self-tests passed; `tests/validate-behavioral-smoke-contract.sh` passed 9/9; `tests/validate-pi-extension.sh` passed 45/45; `tests/validate-agents.sh` passed 224/224. Earlier same day: provider-diverse Pi smoke passed with `tests/pi-live-dispatch-smoke.sh --red-model minimax/MiniMax-M3 --green-model kimi-coding/kimi-for-coding --require-distinct-model-lanes --run-dir runs/pi-live-kimi-minimax-smoke`; `tests/validate-barrier-envelopes.sh runs/pi-live-kimi-minimax-smoke/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-live-kimi-minimax-smoke` passed. This also hardened `tests/pi-live-dispatch-smoke.sh` to use `--no-extensions` with the explicit local extension so installed package extensions do not conflict. Previous 2026-06-03: Rubik's golden-vector repair passed `cd examples/rubiks-solver && cargo test --quiet` (46/46) and `tests/validate-agents.sh` (224/224). Earlier same day: multi-lane Pi smoke passed with `tests/pi-live-dispatch-smoke.sh --red-model openai-codex/gpt-5.5:xhigh --green-model openai-codex/gpt-5.5:medium --require-distinct-model-lanes --run-dir runs/pi-live-multilane-smoke`; `tests/validate-barrier-envelopes.sh runs/pi-live-multilane-smoke/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-live-multilane-smoke` passed; `tests/behavioral-smoke.sh` self-tests passed with the collapsed-lane failure case; `tests/validate-behavioral-smoke-contract.sh` passed 9/9; `tests/validate-pi-extension.sh` passed 45/45; `tests/validate-adversarial-modules.sh` passed 106/106; `tests/validate-agents.sh` passed 224/224. Previous 2026-05-31: divergence evaluator schema hardening passed `tests/foundry-evals.sh --suite divergence-routing` (6/6, including route-helper schema-drift self-checks), `tests/foundry-evals.sh --suite phase-choreography` (3/3), `tests/foundry-evals.sh` (8 generic suites / 28 cases), `tests/validate-adversarial-modules.sh` (106/106), and `tests/validate-agents.sh` (224/224). Earlier same day: live Pi `foundry_team` dispatch for `runs/pi-live-divergence-arbiter-smoke/` succeeded for divergence evaluator and arbiter lanes; `tests/validate-barrier-envelopes.sh runs/pi-live-divergence-arbiter-smoke/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-live-divergence-arbiter-smoke` passed; `tests/foundry-evals.sh --suite phase2-trigger-strategy` passed 4/4 cases. Previous 2026-05-30: `tests/foundry-evals.sh --suite phase-choreography` passed 3/3 cases; `tests/foundry-evals.sh` passed 7 generic suites / 24 cases; `tests/validate-adversarial-modules.sh` passed 94/94 with phase-choreography anchors; `tests/validate-agents.sh` passed 224/224. Earlier 2026-05-29 focused suite runs passed for `reviewer-fanout` (3/3), `divergence-routing` (6/6), `red-followup-barrier` (3/3), and `spec-update-restart` (3/3); `tests/arbiter-routing-evals.sh tests/fixtures/arbiter-routing-evals.feature` passed the compatibility path; `tests/validate-codex-plugin.sh` passed 44/44; `tests/validate-pi-extension.sh` passed 43/43; `tests/validate-behavioral-smoke-contract.sh` passed 7/7; `tests/validate-barrier-envelopes.sh` self-tests passed. Previously on 2026-05-26: `tests/validate-barrier-envelopes.sh runs/pi-autonomous-sudoku-smoke/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-autonomous-sudoku-smoke` passed; `tests/validate-barrier-envelopes.sh runs/pi-from-scratch-roman-numeral/dispatch` passed; `tests/behavioral-smoke.sh runs/pi-from-scratch-roman-numeral` passed. Earlier slow live lane on 2026-05-22: `tests/pi-live-dispatch-smoke.sh --keep` passed with a real `foundry_team` Pi tool call; `/skill:foundry-adversarial` under Pi produced `runs/pi-autonomous-sudoku-smoke/` with Sudoku `30/30` and provider-qualified `openai-codex/gpt-5.5` model lanes. New from-scratch live Pi run (2026-05-24) produced `runs/pi-from-scratch-roman-numeral/` with fresh red tests, fresh green implementation, and Roman numeral `8/8`.

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
| Rubik's cube | 46/46 | Golden-vector repair — Kociemba reference vectors resolved the convention mismatch case study. |
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
| `todos/generalize-workflow-evals.md` | High | **COMPLETED 2026-05-28; EXTENDED 2026-05-29/30/31** — generic `tests/foundry-evals.sh` runner + adapter layout landed; current suites cover arbiter routing, divergence routing, green/red follow-up barriers, reviewer fan-out, phase choreography, Phase 2 trigger strategy, and spec update/restart; old arbiter command remains a compatibility wrapper |
| `todos/arbiter-agent.md` | Future | **COMPLETED 2026-05-26** — added `arbiter-agent`, scoped arbitration playbook, adversarial skill routing, barrier-auditor guidance, validator coverage, and arbiter PromptEnvelope scope hardening |
| `todos/phase2-trigger-strategy.md` | Future | **COMPLETED 2026-05-31** — adopted `adaptive_with_fixed_floor`: fixed N=3 fallback plus N=2 early trigger for unchanged tests with distinct green implementation attempts; covered by `phase2-trigger-strategy` evals |
| `todos/multi-provider-delegation.md` | High | **COMPLETED 2026-06-03; EXTENDED 2026-06-05/07/08/09** — Pi live dispatch smoke now supports per-lane model overrides, selectable worked examples, opt-in `--phase-task artifact-sketch`, and persisted parsed `phase-artifacts/` files when artifact-sketch runs are kept; behavioral smoke can require distinct red/green lanes; `runs/pi-live-multilane-smoke/` validates with Codex 5.5 xhigh red vs Codex 5.5 medium green; `runs/pi-live-kimi-minimax-smoke/` validates Sudoku with MiniMax M3 red vs Kimi for Coding green; `runs/pi-live-kimi-minimax-chess-smoke/` validates Chess `44/44` with MiniMax M3 red vs Kimi for Coding green; `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/` validates a from-scratch Rust slugify red/green phase with MiniMax red vs Kimi green and `11/11` tests |
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

- **Golden test vectors are non-negotiable** — NLSpecs for state transformations MUST include concrete reference outputs. Without them, red and green diverge at the convention level. The Rubik's example originally failed at 31/46; adding Kociemba Python reference vectors for `R`, `U`, `R U R' U'`, and a hard scramble, then aligning both red simulator and green move tables, brought it to 46/46.
- **Orchestrator must be stateless** — reading both sides and fixing code directly breaks provenance, even if all tests pass. Route fixes through the proper team.
- **Red team test data quality** — wrong test data under the information barrier amplifies damage. Green team can't see the test code to identify bad inputs. Verify against authoritative sources.
- **NLSpec derivation errors** — agents mix inputs from one source with outputs from another (e.g., FEN from position A with perft numbers from position B). Cross-check vectors against research docs.
- **Distinct provider/model lanes strengthen the adversarial property** — different model families have different blind spots. Gemini is no longer available in this environment, so the first systematic live lane used Codex 5.5 xhigh red vs Codex 5.5 medium green. Correction from 2026-06-03: Pi does have `kimi-coding` auth configured, but minimal Kimi calls (`kimi-coding/kimi-for-coding`, `kimi-coding/k2.6-code-preview`) hung without stdout/stderr during spot checks, so the smoke used Codex medium as the operational fallback. Update 2026-06-05: Kimi and MiniMax are operational in Pi; `runs/pi-live-kimi-minimax-smoke/` validated Sudoku red `minimax/MiniMax-M3` vs green `kimi-coding/kimi-for-coding`, and `runs/pi-live-kimi-minimax-chess-smoke/` validated the deeper Chess worked example on the same provider-diverse lanes. `foundry_team` now preserves Pi thinking suffixes such as `:xhigh`/`:medium` in reported `actualModel` lanes so replay validation can detect collapsed red/green lanes.
- **PromptEnvelope child prompts must be self-contained** — Pi `foundry_team` sends exactly `envelope.prompt`; `visible_context` is replay/audit metadata, not automatically injected into the child prompt. MiniMax correctly refused a red smoke prompt that claimed it could see context not present in the prompt. Keep live smoke prompts explicit about the allowed visible context while preserving withheld-context metadata for barrier validation.
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
- **Workflow evals can be Gherkin-authored mocks** — `tests/foundry-evals.sh` runs adapter-backed `.feature` suites under `tests/evals/features/` without live model calls. Current suites mock arbiter and divergence evaluator JSON outputs, generate red/green follow-up, reviewer fan-out, full phase-choreography, and NLSpec-rerun PromptEnvelopes, verify spec restart/tracker-reset/revision-cap records, and delegate mechanical leak checks to `validate-barrier-envelopes.sh`. Keep live model evals as a separate slow lane that can reuse the same scenarios later.
- **Reviewer fan-out needs territory assertions beyond generic leak checks** — the barrier validator catches red/green sample leaks, but Phase 3 includes implementation-facing reviewers (`green-team-reviewer`, language/Bazel/UniFFI/correctness/reliability) and test-facing reviewers (`red-team-test-reviewer`, `testing-reviewer`) that require adapter-level assertions for complete reviewer selection and prompt territory boundaries.
- **Phase 2b trigger strategy is adaptive with a fixed floor** — keep N=3 as the auditable fallback, but escalate at N=2 when the red test is unchanged and at least two distinct green implementation hashes still fail. Do not early-trigger first failures, unchanged green attempts, or red test-content changes; green still receives only PASS/FAIL labels.
- **Live dispute-route smoke validates the hardest breach boundaries** — `runs/pi-live-divergence-arbiter-smoke/` proved real Pi child dispatch for both `divergence-evaluator` and scoped `arbiter-agent`. The arbiter correctly returned `TEST_WRONG` with exactly one disputed test. The divergence child returned `VALUABLE` but also added a noncanonical helper `route_to=NLSPEC_REDERIVATION`; the divergence-evaluator prompt and deterministic evals now forbid/reject `route_to`, and orchestration must continue routing only on `findings[0].outcome`.
- **Phase-artifact smoke is a safe bridge between plumbing and full autonomous runs** — `--phase-task artifact-sketch` keeps the same PromptEnvelope barrier and keep-or-cleanup run-directory ergonomics as the RED_OK/GREEN_OK live smoke while requiring red and green to produce structured, role-specific JSON artifacts. When runs are kept, parsed artifacts are available under `phase-artifacts/`. This catches provider/model obedience issues before spending runtime on a full adversarial phase.
- **Fuller provider-diverse red/green phases are now viable in Pi** — `runs/pi-live-kimi-minimax-fuller-adversarial-smoke/` proved MiniMax can author executable red tests and Kimi can implement from How-only context in a kept from-scratch Rust run. Actual model lanes must still be checked from `foundry_team` results and `behavioral-smoke.toon`; planned model divergence alone is not evidence.

## What's Next

Ilia feedback (2026-04-17, `docs/solutions/workflow-issues/ilia-feedback-foundry-plugin-20260417.md`) raised four structural items. Repo identity is complete, the private dispatch runtime mirrors the public `PromptEnvelope` v1 contract, and a replay-level behavioral smoke harness now exists. The remaining suggested order is:

1. **Extend fuller provider-diverse runs only where they add coverage** — MiniMax/Kimi now works for Sudoku, Chess, phase-artifact sketches, and a kept from-scratch Rust slugify red/green phase. The next useful live step is provider-diverse reviewer fan-out or a controlled divergence/restart run, not more plumbing.
2. **Continue modularization only from evidence** (`todos/modularize-heaviest-skills.md`) — first slice, arbiter routing/scope validation/evals, phase-choreography evals, Roman-run hardening, adaptive trigger evals, live dispute-route smoke, divergence schema hardening, multi-lane dispatch hardening, phase-artifact smoke, fuller provider-diverse red/green smoke, and Rubik's golden-vector repair are done; profile future real runs before extracting more modules.
3. **Codex dispatch follow-up** (`todos/pi-codex-plugin-support.md`) — packaging is done; revisit only when Codex documents a PromptEnvelope-safe dispatchable subagent/team primitive.
4. **Adversarial UI** — brainstorm at `docs/brainstorms/2026-04-04-adversarial-ui-design-system.md`; three-level testing via design systems

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
│   ├── pi-from-scratch-roman-numeral/   (fresh Pi adversarial Roman numeral run, 8/8)
│   ├── pi-live-divergence-arbiter-smoke/ (live Pi divergence + scoped arbiter route smoke)
│   ├── pi-live-multilane-smoke/         (live Pi distinct red/green model-lane smoke)
│   ├── pi-live-kimi-minimax-smoke/      (live Pi MiniMax red / Kimi green Sudoku smoke)
│   ├── pi-live-kimi-minimax-chess-smoke/ (live Pi MiniMax red / Kimi green Chess smoke)
│   └── pi-live-kimi-minimax-fuller-adversarial-smoke/ (live Pi MiniMax red / Kimi green slugify red/green phase)
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
