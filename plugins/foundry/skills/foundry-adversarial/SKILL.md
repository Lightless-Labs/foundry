---
name: foundry:adversarial
description: "Run the adversarial red/green implementation process. A red team writes tests from the NLSpec's Definition of Done, a green team implements from the NLSpec's How section, and the orchestrator mediates with strict information barriers. Use when you have a reviewed NLSpec and want adversarial implementation. Triggers on 'adversarial', 'red green', 'forge it', 'implement adversarially'."
argument-hint: "[path to NLSpec document]"
---

# Foundry Adversarial

Run the adversarial red/green implementation process. You (the orchestrating agent) mediate between a red team (test writers) and a green team (implementers) with strict information barriers.

## Prerequisites

A reviewed NLSpec document (from `foundry:nlspec`). If not provided, search `docs/nlspecs/` for the most recent reviewed NLSpec.

## The Information Barrier

This is the core invariant. Violating it defeats the purpose.

| Entity | Sees | Never sees |
|--------|------|------------|
| Red team | NLSpec (full), spec | Implementation code |
| Red reviewer | NLSpec, test code | Implementation code |
| Green team | NLSpec How section only, test outcome labels (pass/fail) | Test code, assertions, error messages, NLSpec Done section |
| Green reviewer | NLSpec How section, implementation, test outcomes | Test code, NLSpec Done section |
| Arbiter agent | Full spec/NLSpec, one disputed test artifact, relevant implementation snippet, one runner result | Full red suite, full implementation, broad red/green conversation history |
| You (orchestrator) | Everything | — |

**You are the only entity that crosses the barrier. You enforce it by controlling what each subagent receives in its prompt. Every dispatch MUST pass through the PromptEnvelope gate below before `Agent(...)` is invoked, or before the Pi `foundry_team` tool is called.**

## Workflow

### Phase 0: Setup

1. Read the NLSpec. Extract:
   - The **How section** (implementation guidance for green team)
   - The **Definition of Done** (test criteria for red team)
   - The **Data Model** (shared types for both teams)
   - The **Integration Smoke Test** (end-to-end test for red team)

2. Detect the project language (inspect manifests) and determine Cucumber/Gherkin bindings.

3. Initialize replayable run auditing:
   - Create `runs/<run_id>/dispatch/`
   - Treat `<run_id>` as stable for this adversarial run
   - No subagent dispatch may happen until its prompt has a serialized `PromptEnvelope`
   - Initialize `runs/<run_id>/behavioral-smoke.toon` metadata as described in the Behavioral Smoke Run Summary section; update it as the run progresses

4. **Search institutional knowledge** — spawn the learnings-researcher to check `docs/solutions/` for past solutions relevant to this feature. If relevant learnings exist, factor them into the red team's test design and the green team's implementation guidance.

```
Agent(
    subagent_type="foundry:research:learnings-researcher",
    prompt="Search docs/solutions/ for learnings relevant to: [feature topic from NLSpec].
    PromptEnvelope: runs/<run_id>/dispatch/phase0/learnings-researcher.json
    Return any past bugs, patterns, or best practices that should inform the implementation."
)
```

5. Create the workspace structure (or use git worktrees for real filesystem isolation):
   - `shared/` — NLSpec data model, types, interfaces
   - `red/` — test workspace (features/, step_definitions/)
   - `green/` — implementation workspace (src/)

### Mechanical Barrier Gate: PromptEnvelope v1

Before every `Agent(...)` call, assemble a `PromptEnvelope` and validate it. The skill consumes envelopes; it does not hand-send prompt strings.

Envelope path:

```text
runs/<run_id>/dispatch/<phase>/<recipient>.json
```

Envelope shape:

```json
{
  "schema_version": "foundry.prompt-envelope.v1",
  "run_id": "<stable run id>",
  "phase": "phase0|phase1|phase1b|phase2|phase2b|phase3",
  "recipient": "red-team|red-reviewer|green-team|green-reviewer|barrier-integrity-auditor|...",
  "prompt": "<exact prompt sent to the subagent>",
  "visible_context": [
    {"label": "nlspec_how", "kind": "nlspec_how", "sha256": "...", "content": "..."}
  ],
  "withheld_context": [
    {"label": "red_feature_files", "kind": "red_test_code", "sha256": "...", "samples": ["<high-entropy withheld substring>"]}
  ],
  "redactions": [
    {"source": "raw_test_output", "action": "pass_fail_labels_only", "removed": ["assertion_text", "stack_trace", "line_numbers"]}
  ]
}
```

Required gate sequence for every dispatch:

1. `build_prompt_envelope(recipient, phase, visible_context, withheld_context)` — construct the prompt only from `visible_context`.
2. `redact_and_validate_prompt(envelope)` — compare `prompt` against every `withheld_context[].samples[]`; if any withheld sample appears, STOP before dispatch.
3. Serialize the envelope to `runs/<run_id>/dispatch/<phase>/<recipient>.json`.
4. Dispatch the agent with exactly `envelope.prompt`; do not append extra context after validation.
   - Claude Code: use `Agent(...)` with exactly `envelope.prompt`.
   - Pi: use the Foundry package extension tool `foundry_team` with `envelopePath`; never paste the prompt or hidden context directly into a normal message.
5. Include the envelope path in any barrier-integrity-auditor request.

Minimum withheld samples by recipient:

| Recipient | Withheld samples MUST include |
|-----------|-------------------------------|
| Red team / red reviewer | Green workspace paths, implementation file names, representative implementation snippets |
| Green team / green reviewer | Red workspace paths, `.feature` scenario text, step-definition/assertion snippets, raw failure output, NLSpec Done section snippets |
| Language/correctness/reliability reviewers | Red test files and NLSpec Done snippets unless the reviewer explicitly audits tests |
| Arbiter agent | Unrelated red tests, unrelated green files, broad conversation history; scope the prompt to one disputed test |
| Barrier-integrity-auditor | Nothing withheld from the auditor; auditor receives envelope paths and may inspect all serialized artifacts |

Run `tests/validate-barrier-envelopes.sh runs/<run_id>/dispatch` whenever envelopes exist. This script is the public-plugin validator for replayable barrier artifacts.

### Pi Team Dispatch: `foundry_team`

Pi intentionally has no built-in subagent/team/swarm primitive. When this package is installed as a Pi package, it provides a `foundry_team` extension tool that rolls the missing primitive locally using the officially documented Pi extension pattern from `examples/extensions/subagent/`: spawn child `pi --mode json -p --no-session` processes with isolated context windows.

Use `foundry_team` only after writing PromptEnvelope artifacts. The tool reads and validates the envelope, then sends exactly `envelope.prompt` to the child pi process.

Single dispatch:

```json
{
  "envelopePath": "runs/<run_id>/dispatch/phase1b/red-team-test-reviewer.json",
  "agent": "foundry:review:red-team-test-reviewer",
  "model": "<optional provider/model lane>",
  "tools": ["read", "grep", "find", "ls"]
}
```

Parallel dispatch:

```json
{
  "dispatches": [
    {
      "envelopePath": "runs/<run_id>/dispatch/phase1/red-team.json",
      "model": "<red lane, e.g. openai-codex/gpt-5.5:xhigh>",
      "tools": ["read", "write", "edit", "bash", "grep", "find", "ls"]
    },
    {
      "envelopePath": "runs/<run_id>/dispatch/phase2/green-team.json",
      "model": "<green lane, e.g. kimi-coding/kimi-for-coding or openai-codex/gpt-5.5:medium>",
      "tools": ["read", "write", "edit", "bash", "grep", "find", "ls"]
    }
  ]
}
```

Notes:
- `foundry_team` discovers Foundry agent prompts from `plugins/foundry/agents/**/*.md`.
- It disables child extensions, skills, prompt templates, sessions, and context files by default to keep child context explicit.
- It reports `actualModel` in tool details; copy those values into `behavioral-smoke.toon` `model_lanes` rows.
- For multi-provider or distinct-lane exercises, pass explicit per-dispatch `model` values (for example red on `openai-codex/gpt-5.5:xhigh` and green on `kimi-coding/kimi-for-coding`, or green on `openai-codex/gpt-5.5:medium` as a weaker fallback) and set `requires_distinct_model_lanes: true` in `behavioral-smoke.toon`.
- If the tool is unavailable, stop and tell the user to install or enable the Foundry Pi package; do not fake subagent isolation in the main Pi session.

### Behavioral Smoke Run Summary: `behavioral-smoke.toon`

Every real adversarial run MUST produce a replayable run summary at:

```text
runs/<run_id>/behavioral-smoke.toon
```

This is the public-plugin live lane for behavioral smoke tests: the skill emits real run artifacts, and `tests/behavioral-smoke.sh runs/<run_id>` validates them without needing private infrastructure.

Use TOON (Token-Oriented Object Notation) for this summary because the data is small, row-oriented, and LLM-readable. Keep PromptEnvelope artifacts as JSON; TOON is only for the run-level summary.

Required TOON subset and fields:

```toon
schema_version: foundry.behavioral-smoke.v1
run_id: <stable run id>
requires_divergence_restart: false
requires_distinct_model_lanes: false

test_results[1]{example,passed,total,expected_passed,expected_total}:
  <feature-or-example-name>,<passed>,<total>,<expected_passed>,<expected_total>

model_lanes[3]{recipient,planned_model,actual_model}:
  red-team,<planned>,<actual>
  green-team,<planned>,<actual>
  orchestrator,<planned>,<actual>

divergence_restarts[0]{phase,outcome,revision_history_count}:
```

Rules:
- `test_results` records the final run result. For worked examples, `expected_*` MUST match the documented expected pass rate (Sudoku 30/30, Chess 44/44, Rubik's 31/46 until fixed). For new feature runs, set `expected_*` to the accepted final target for that run.
- `model_lanes` records every distinct team lane. If a model is inherited from the current session, write the inherited model ID as both planned and actual. If provider overrides are used, planned and actual must still match.
- Set `requires_distinct_model_lanes: true` only when the run is deliberately exercising red/green provider or model-lane separation; then red-team and green-team planned/actual model lanes MUST differ. Prefer different providers (for example Codex vs Kimi); if unavailable, use materially different lanes such as `openai-codex/gpt-5.5:xhigh` vs `openai-codex/gpt-5.5:medium` and call out the weaker isolation in the run notes.
- `divergence_restarts` records every Phase 1b/2b evaluator outcome that restarted the pipeline. For each `VALUABLE` restart, `revision_history_count` MUST be exactly `1` for that restart event.
- If the run is deliberately exercising the divergence loop, set `requires_divergence_restart: true`; otherwise `false`.
- Update this file after Phase 2b restarts and finalize it in Phase 4 before reporting success.
- Run `tests/behavioral-smoke.sh runs/<run_id>` before final user summary. A failure is a run artifact failure; fix the artifacts or the workflow before claiming completion.

### Phase 1: Red Team — Write Tests

Spawn the red team subagent:

```
Agent(
    name="red-team",
    mode="bypassPermissions",
    prompt="You are the RED TEAM. Your job is to write comprehensive tests
    that will validate an implementation you cannot see.

    ## What you see
    - The NLSpec Definition of Done: [paste DoD section]
    - The NLSpec Data Model: [paste data model section]
    - The project language and test framework: [language, Cucumber/Gherkin]

    ## What you CANNOT see
    - Any implementation code (it doesn't exist yet)
    - The NLSpec How section (that's for the green team)

    ## Your task
    For each Definition of Done item, write a Gherkin .feature file with
    concrete scenarios. Then write step definitions that implement the
    assertions.

    Rules:
    - Every DoD checkbox must have at least one scenario
    - The integration smoke test from the NLSpec must be a feature file
    - Include edge cases and error paths from the DoD
    - Tests must be runnable via [Cucumber command for language]
    - Write tests to [workspace path]

    When done, list all .feature files and the DoD items they cover."
)
```

### Phase 1b: Review Red Team Tests Against NLSpec

Spawn two reviewers in parallel:

```
Agent(
    subagent_type="foundry:review:red-team-test-reviewer",
    prompt="Review these test files against the NLSpec Definition of Done.

    NLSpec DoD: [paste DoD section]
    Test files: [paths to .feature files and step definitions]

    Return findings as JSON matching the findings schema."
)

Agent(
    subagent_type="foundry:review:cucumber-reviewer",
    prompt="Review these Gherkin feature files and step definitions for quality.

    Test files: [paths to .feature files and step definitions]

    Return findings as JSON matching the findings schema."
)
```

The red-team-test-reviewer checks DoD coverage, assertion specificity, trivially satisfiable tests, and scope creep. The cucumber-reviewer checks Gherkin quality (declarative style, scenario independence, step discipline).

Also spawn the barrier-integrity-auditor to verify no implementation code leaked into the red team's context. Give the auditor envelope paths, not ad-hoc pasted prompt fragments:

```
Agent(
    subagent_type="foundry:review:barrier-integrity-auditor",
    prompt="Audit these PromptEnvelope artifacts for barrier violations.
    Red team should see: NLSpec/spec test criteria and red workspace paths.
    Red team must NOT see: implementation code, green workspace paths.
    Envelope paths:
    - runs/<run_id>/dispatch/phase1/red-team.json
    - runs/<run_id>/dispatch/phase1b/red-team-test-reviewer.json
    Red workspace contents: [list files in red workspace]"
)
```

If there are UNCOVERED or WEAK items, send feedback to the red team (as a new message to the same agent) with the specific gaps. Iterate until reviewers pass.
#### Divergence Check (Phase 1b)

Use the mandatory routing module at `docs/playbooks/foundry-adversarial-divergence-routing.md`. Summary contract:

- Trigger only when red tests reference behavior not present in the NLSpec DoD.
- Assemble `EvaluatorInput` with the raw flagged scenario, full NLSpec, `divergence_phase=PHASE_1B`, and `red_test_paths`.
- Dispatch `foundry:review:divergence-evaluator` through a validated PromptEnvelope.
- Route on `findings[0].outcome` only; ignore any noncanonical `route_to`/`next_step` helper the evaluator may emit. Phase 1b `VALUABLE` → `spec_update_and_restart` and restart Phase 1; `NOT_VALUABLE` → red fixes using `findings[0].rationale`; `INCONCLUSIVE` → user escalation.
- Only one evaluator invocation may be in flight at a time.

### Phase 2: Green Team — Implement

**Before spawning green:** run the red team's tests to get the initial failure list. Capture ONLY the test names and pass/fail status. Store raw test output only in withheld context for the green envelope; never paste raw output into `prompt`.

Spawn the green team subagent:

```
Agent(
    name="green-team",
    mode="bypassPermissions",
    prompt="You are the GREEN TEAM. Your job is to implement a feature
    that passes tests you cannot see.

    ## What you see
    - The NLSpec How section: [paste How section ONLY — not the Done section]
    - The NLSpec Data Model: [paste data model section]
    - The project language: [language]
    - Test outcomes (pass/fail only):
      [test_name_1: FAIL]
      [test_name_2: FAIL]
      [...]

    ## What you CANNOT see
    - Test code, assertions, error messages, step definitions
    - The NLSpec Definition of Done section
    - The .feature files

    ## Your task
    Implement the feature following the NLSpec How section guidance.
    Write code to [workspace path].

    You know which tests exist by name and whether they pass or fail.
    Use the test NAMES as hints about what behavior is expected.
    You must NOT try to read test files or access the red workspace.

    When done, tell me you're ready for a test run."
)
```

### Phase 2b: Test-Fix Inner Loop

**State: TestFailureTracker** (pipeline-run-scoped — reset all counters on Phase 1 restart)

Maintain per failing test:
- `test_id`: test identifier
- `consecutive_fails`: consecutive green iterations where this test failed (Phase 2b trigger fires → resets to 0; test passes → resets to 0; test content changes → resets to 1)
- `threshold`: default 3 fixed fallback
- `test_content_hash`: hash of test file content; detects test changes between iterations
- `implementation_attempt_hashes`: ordered unique hashes of green implementation revisions observed while this unchanged test continues to fail
- `trigger_strategy`: default `adaptive_with_fixed_floor`

This is where you mediate. Loop:

1. **Assemble runner workspace** — Copy green's implementation + red's tests into a temporary directory
2. **Run tests** — Execute the test suite
3. **Filter outcomes** — Extract ONLY `test_name: PASS/FAIL`. Discard assertions, errors, stack traces.
4. **Update trackers** — PASS: reset `consecutive_fails=0` and clear `implementation_attempt_hashes`. FAIL: if test content hash changed, reset `consecutive_fails=1` and start `implementation_attempt_hashes` with the current implementation hash; else increment `consecutive_fails` and append the current implementation hash when it is distinct from prior attempts.
5. **Check divergence threshold** — For any test where `consecutive_fails >= threshold` (default 3), or where `trigger_strategy=adaptive_with_fixed_floor` and the unchanged test has `consecutive_fails >= 2` plus at least two distinct `implementation_attempt_hashes`, use `docs/playbooks/foundry-adversarial-divergence-routing.md`. This preserves the fixed N=3 floor while escalating clear pattern-based green/spec divergence one iteration earlier. Process one test at a time in ascending `test_id` order. Route on `findings[0].outcome` only; ignore any noncanonical `route_to`/`next_step` helper the evaluator may emit. Phase 2b `VALUABLE` → invoke `spec_update_and_restart`, then restart Phase 1; Phase 2b `NOT_VALUABLE` → send green back with `findings[0].rationale` and reset this test's tracker; Phase 2b `INCONCLUSIVE` → escalate to user and pause.
6. **Check arbitration threshold** — If normal divergence routing does not resolve a stable single-test dispute, or if a reviewer flags a suspicious pass/false-green signal, use `docs/playbooks/foundry-adversarial-arbiter-routing.md`. Dispatch `foundry:review:arbiter-agent` through a validated PromptEnvelope scoped to exactly one test. Route on `findings[0].outcome`: `TEST_WRONG` → red fixes the test without seeing implementation; `IMPLEMENTATION_WRONG` → green receives only redacted guidance plus `test_name: PASS/FAIL`; `SPEC_INCOMPLETE` → invoke `spec_update_and_restart`; `INCONCLUSIVE` → pause for user judgment.
7. **Check termination** — All pass → Phase 3. Any fail → send filtered outcomes to green.
8. **Check bounds** — If green has iterated more than the configured limit (default 20), pause and ask the user.

**Send to green team ONLY:**
```
Test results:
  test_name: PASS/FAIL
N tests total, X passed, Y failed.
```

Never include:
- Assertion text or expected vs actual values
- Stack traces
- Line numbers from test code
- Any content from .feature files or step definitions

### Spec Update and Pipeline Restart (`spec_update_and_restart`)

Use the mandatory restart module at `docs/playbooks/foundry-adversarial-spec-update-and-restart.md`. Summary contract:

- Trigger only when a divergence check returns `VALUABLE`.
- **You MUST NOT write NLSpec content directly. The NLSpec agent is the sole author.**
- Pass `findings[0].gap_description` verbatim as `evaluator_feedback`; do not paraphrase.
- Preserve the deferred commit pattern: commit the current NLSpec only after the NLSpec rerun succeeds, guard with `git diff --staged --quiet`, then commit the replacement NLSpec.
- Update `PipelineRevisionState`, generate `ChangeSummary`, pass `Phase1RestartPackage` with `red_test_paths`, reset `TestFailureTracker`, and restart Phase 1.
- For every Phase 1b/Phase 2b `VALUABLE` restart, update `runs/<run_id>/behavioral-smoke.toon` with `revision_history_count` exactly `1` for that restart event.

### Phase 3: Review

When all tests pass, spawn two reviewers in parallel:

Spawn these reviewers in parallel:

**Green team reviewer** (sees implementation, not tests):
```
Agent(
    subagent_type="foundry:review:green-team-reviewer",
    prompt="Review this implementation for code quality under information barrier constraints.

    NLSpec How section: [paste How section]
    Implementation: [read green workspace files]
    Test outcomes: [all passing]

    CRITICAL: You must NOT see test code, .feature files, step definitions, or the NLSpec Done section.

    PromptEnvelope: runs/<run_id>/dispatch/phase3/green-team-reviewer.json

    Return findings as JSON matching the findings schema."
)
```

**Red team test reviewer** (sees tests, not implementation):
```
Agent(
    subagent_type="foundry:review:red-team-test-reviewer",
    prompt="Final review of test suite thoroughness.

    NLSpec Definition of Done: [paste DoD]
    Test files: [read red workspace files]

    You do NOT see the implementation.

    Return findings as JSON matching the findings schema."
)
```

**Language-specific reviewer** (conditional — dispatch based on detected language):
```
# Dispatch the appropriate language reviewer based on the project:
# Rust  → foundry:review:rust-reviewer
# Swift → foundry:review:swift-reviewer
# TS    → foundry:review:typescript-reviewer
# Also dispatch foundry:review:bazel-reviewer if BUILD files exist
# Also dispatch foundry:review:uniffi-bridge-reviewer if .udl files exist

Agent(
    subagent_type="foundry:review:[language]-reviewer",
    prompt="Review the implementation for [language]-specific issues.
    Implementation: [read green workspace files]
    Return findings as JSON matching the findings schema."
)
```

**Barrier integrity auditor** (always — final barrier check):
```
Agent(
    subagent_type="foundry:review:barrier-integrity-auditor",
    prompt="Final barrier audit. Replay ALL PromptEnvelope artifacts under:
    runs/<run_id>/dispatch/

    For each envelope, compare prompt content against withheld_context samples and the barrier matrix.
    Verify that green saw only NLSpec How + test outcome labels, red saw no implementation material, and any arbiter-agent PromptEnvelope was scoped to exactly one disputed test with redacted follow-up.
    Report any leak as P0."
)
```

**Always-on reviewers** (dispatch in parallel with the above):
- `foundry:review:correctness-reviewer` — logic errors, edge cases, state bugs in the implementation
- `foundry:review:testing-reviewer` — coverage gaps, weak assertions in the test suite
- `foundry:review:reliability-reviewer` — error handling, timeouts, retry logic (if the implementation touches I/O)

Merge all reviewer findings. Deduplicate across reviewers (same file + line + issue = one finding, keep highest severity).

If any reviewer rejects:
- Green-team-reviewer rejects → send feedback to green team, re-enter test-fix loop
- Red-team-test-reviewer rejects → send feedback to red team, red rewrites tests, green re-tests against new suite
- Barrier-integrity-auditor finds violations → **STOP** — fix the barrier leak before continuing
- Language/correctness/reliability reviewers find P0/P1 → send to appropriate team for fixing

### Phase 4: Finalize

When all reviewers approve (zero P0/P1 findings, barrier audit clean):
1. Commit the implementation and tests
2. Update the NLSpec frontmatter: `status: implemented`
3. Finalize `runs/<run_id>/behavioral-smoke.toon` with final test counts, model lanes, and divergence restart rows
4. Run both replay validators:
   ```bash
   tests/validate-barrier-envelopes.sh runs/<run_id>/dispatch
   tests/behavioral-smoke.sh runs/<run_id>
   ```
5. Report summary: which DoD items are covered, test count, iteration count, PromptEnvelope validation status, and behavioral-smoke validation status

### Configuration

These can be set via the conversation or a config file:

| Setting | Default | Description |
|---------|---------|-------------|
| `inner_loop_limit` | 20 | Max green fix iterations before pausing |
| `divergence_trigger_strategy` | `adaptive_with_fixed_floor` | Phase 2b triggers at fixed N=3 or at N=2 when a stable failing test survives distinct green implementation attempts |
| `arbitration_threshold` | after normal divergence routing | Stable single-test dispute threshold before invoking `foundry:review:arbiter-agent` |
| `too_easily_threshold` | 3 | Consecutive passes before flagging "too easy" |
| `test_timeout` | 120s | Per-test-run timeout |
| `provider` | current model | Which model to use for subagents |

### Troubleshooting

Use `docs/playbooks/foundry-adversarial-provider-troubleshooting.md` for convergence and provider-specific troubleshooting, including OpenCode command-shape failures and Kimi K2.5 tokenizer/file-output salvage. If a live Pi run is interrupted after PromptEnvelope artifacts exist, use `docs/playbooks/foundry-adversarial-pi-continuation.md` to resume from serialized envelopes instead of reconstructing hidden context in the main conversation. Keep the barrier invariant intact while troubleshooting: green still receives only NLSpec How plus `test_name: PASS/FAIL` labels, never raw failures, assertions, test code, or NLSpec Done criteria.
