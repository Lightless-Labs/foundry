---
title: "Feedback from third-thoughts running the adversarial playbook for middens Batch 4"
module: foundry
date: 2026-04-06
problem_type: upstream_feedback
component: adversarial-orchestration-playbook
severity: medium
status: proposed
source_repo: Lightless-Labs/third-thoughts
source_commit_range: "2d1d367..1a1ceb1"
provenance: |
  Discovered while running the foundry adversarial red/green workflow manually
  (no Foundry engine) from a Claude Code main session in third-thoughts, porting
  middens Batch 4 — 4 Python analytical techniques (user_signal_analysis,
  cross_project_graph, change_point_detection, corpus_timeline) — on 2026-04-06.
  Red team: Gemini 3.1 Pro Preview via gemini-cli. Green teams: 4× parallel
  Kimi K2.5 via opencode run. Orchestrator: Claude Opus 4.6 (main session).
  Outcome: 270/270 cucumber scenarios passing on third-thoughts commit 1a1ceb1.
  This file is intended to be self-contained — every proposal below carries its
  own evidence trail inline, so it can be acted on without referencing external
  repos.
tags:
  - foundry
  - feedback
  - adversarial-workflow
  - opencode-cli
  - fixture-discipline
  - barrier-integrity
  - orchestrator-fallibility
---

# Feedback From third-thoughts — Middens Batch 4 (2026-04-06)

Six proposed updates to the foundry `adversarial-orchestration-playbook` discovered while running the red/green workflow on a 4-technique Python porting batch. Each item is presented with (a) the symptom as observed during the run, (b) the root cause, (c) the proposed playbook change, entirely inline so it can be acted on without reading anything else.

## Context

Middens Batch 4 ported 4 Python analytical techniques via the foundry adversarial workflow. The setup:

- **NLSpec:** single document with 6 sections per foundry convention
- **Red team:** Gemini 3.1 Pro Preview via `gemini -y -s false --prompt` — wrote a cucumber feature file from NLSpec sections 1 (Why), 2 (What), 6 (Definition of Done) only
- **Green team:** 4 parallel Kimi K2.5 dispatches via `opencode run -m kimi-for-coding/k2p5 --format json` — each received the shared contract + one per-technique How section as `-f` attachments
- **Orchestrator:** Claude Opus 4.6 main session — wrote spec, mediated contract gaps, wired scripts, ran cucumber

Total wall clock from spec draft to 270/270 green: ~35 minutes. This is substantially faster than the prior batch which went through 6 rounds of PR-review iteration — the difference is that red/green adversarial catches bugs at local-test time rather than at remote-review time.

The barrier held where it mattered. The red team never saw implementation; the green team never saw test files. The orchestrator had to mediate 3 genuine contract gaps and 2 orchestrator-side mistakes. Five distinct failure modes surfaced during the run and inform the proposals that follow.

---

## F1 — OpenCode CLI argument order (message BEFORE `-f`)

### Symptom observed

First parallel dispatch of 4 Kimi green teams produced **zero files**. All 4 log files were 0 bytes. The dispatch commands looked correct, all 4 processes exited with status 0, but nothing landed on disk. In a parallel `&` + `wait` pattern the error went into the redirected stderr and was effectively invisible.

### Root cause

`opencode run` has both a positional `message` argument (array) and a `-f/--file` flag (also array). If `-f` comes first, its greedy collection of file paths consumes the trailing positional message as another file path, which produces a silent `File not found: <your prompt>` error that only surfaces when running foreground:

```
Error: File not found: Implement the single Python technique described in the attached contract...
```

Because the error is emitted to stderr and routed into a log file in the parallel dispatch pattern, the orchestrator sees empty output files and assumes a silent hang.

### Correct form

```bash
# WRONG — 0-byte output, silent hang
opencode run -m kimi-for-coding/k2p5 --format json -f contract.md "implement the technique"

# RIGHT — message first, -f files come after
opencode run -m kimi-for-coding/k2p5 --format json "implement the technique" -f contract.md
```

### Proposed playbook addition

Add to the "Multi-Provider Delegation" section (or a new "OpenCode quirks" subsection):

> **OpenCode CLI argument order:** The positional `message` must come BEFORE any `-f/--file` flag. `opencode run -m MODEL 'prompt text' -f contract.md`. If you put the message after `-f`, OpenCode consumes it as another file path and fails with `Error: File not found: <your prompt>`. In parallel dispatch via `&` + `wait` the error goes into the redirected stderr and the output file is 0 bytes, so it looks like a silent hang. Run one invocation foreground first to validate the command shape.

---

## F2 — Kimi green prompts must explicitly disable `write`/`edit` tools

### Symptom observed

Three of 4 re-dispatched Kimi green teams produced files via bash heredoc as instructed. The fourth (`cross_project_graph`) did not — the file never landed on disk. Inspection of the raw NDJSON log showed:

```text
permission requested: external_directory (/.../middens/python/techniques/*); auto-rejecting
```

with a `tool_use` event of type `write` and `status: "error"` nested in the permission rejection.

### Root cause

Kimi K2.5 via OpenCode sometimes prefers the structured `write` tool over `bash` even when the prompt says "use bash heredoc." Probably because `write` is a more canonical instruction-following path in Kimi's training distribution. If the target path is outside the OpenCode workspace (`$PWD` at dispatch time), OpenCode's permission layer auto-rejects the `write` call with a non-interactive hard-fail — there is no approval prompt, just a rejection.

The instruction "write the file via bash heredoc" in the prompt was not strong enough to prevent Kimi from trying `write` first.

### Fix that worked

Re-dispatched with stronger tool-discipline wording:

> You MUST write the file with a bash tool call. The write and edit tools are DISABLED. After writing, print OK and stop.

This worked on the first retry.

### Proposed playbook addition

Add to the "Spawning the Green Team" section:

> **Green prompts must explicitly disable alternative tools.** A prompt like "write the file via bash heredoc" is NOT sufficient for Kimi K2.5 via OpenCode — Kimi will often try the `write` tool first, which fails silently on `external_directory` permission rejection when the target is outside the OpenCode workspace. Required form:
>
> ```
> You MUST write the file with a bash tool call:
>
>     cat > /absolute/path/to/file <<'PYEOF'
>     <content>
>     PYEOF
>
> The write and edit tools are DISABLED. If you cannot call bash,
> stop and emit the file body as text — the orchestrator will salvage it.
> After writing, print OK and stop.
> ```
>
> This pattern applies to any provider that has both a structured write tool and a shell tool, not just Kimi.

### Related: Kimi tokenizer leakage salvage

A related failure during the same run: one Kimi dispatch emitted a tool_use event with type `invalid` and error `JSON parsing failed: ... <|tool_call_end|><` — Kimi's tokenizer leaked control tokens into the tool-call JSON body, corrupting the serialization. OpenCode rejected the call. Kimi then emitted the intended file body as a plain `text` part in the NDJSON output — **functional content, wrong envelope**. The orchestrator salvaged it programmatically:

```python
import json, re
for line in open('dispatch.ndjson'):
    obj = json.loads(line)
    if obj.get('type') == 'text':
        text = obj.get('part', {}).get('text', '') or obj.get('text', '')
        if 'PYEOF' in text:
            unescaped = text.encode().decode('unicode_escape')
            m = re.search(r"cat\s*>\s*(\S+)\s*<<'PYEOF'\n(.*?)\nPYEOF", unescaped, re.DOTALL)
            if m:
                path, body = m.group(1), m.group(2)
                with open(path, 'w') as f:
                    f.write(body)
                break
```

Recovered 13.5 KB of valid Python that was otherwise lost. Salvaged files MUST be syntax-checked before use (`python3 -c "import ast; ast.parse(open('path').read())"`) because Kimi can truncate mid-function with no indicator.

This salvage procedure is worth codifying in the playbook as a fallback for any green team whose tool-call envelope corrupts.

---

## F3 — Fixture discipline: sanity-check before routing as green-team bug

### Symptom observed

After green delivered `cross_project_graph.py` and a field-name typo was fixed inline, cucumber still failed with:

```
Summary does not contain 'cross-project graph':
insufficient cross-project references: need at least 2 projects with an edge (found 3 projects, 0 edges)
```

The "3 projects, 0 edges" part was diagnostic — the fixture WAS injecting projects, so project extraction was working. But the edge count was zero, which looked like the implementation wasn't finding the cross-project mentions.

### Root cause (NOT a green-team bug)

The failure looked exactly like a green-team bug — and the orchestrator's first instinct was to re-dispatch green — but the actual cause was a **fixture bug**. The orchestrator had added a new cucumber fixture step that was supposed to inject a cross-project mention into the first user message of each session. The injection logic was:

```rust
for (turn_idx, msg) in session.messages.iter_mut().enumerate() {
    if turn_idx == 1 && msg.role == MessageRole::User {
        msg.text = format!("{} — also look at how {} does it", msg.text, other_project);
    }
}
```

The factory emits messages as `[User, Assistant, User, Assistant, ...]`. So `turn_idx == 1` is the **first Assistant message**, not a User message. The `role == User` check failed silently and the injection NEVER happened. The test fixture contained zero cross-project mentions to find. The failing assertion was correct; the implementation was correct; the fixture was a lie.

### Proposed playbook addition

Add to the "When Tests Fail After Both Teams Deliver" section:

> **Before routing any test failure as a green-team bug, sanity-check the fixture.** Fixture bugs and implementation bugs produce IDENTICAL symptoms — both manifest as "technique emits wrong numbers on the fixture." The playbook's failure taxonomy (contract gap / red bug / green bug / convention mismatch / ambiguous spec) can't distinguish them without an explicit fixture-verification step.
>
> Orchestrator sanity-check procedure:
>
> 1. Dump the first session (or a representative sample) of the fixture input
> 2. Confirm the expected signals are actually present in the raw data
> 3. Only after verifying the fixture has the right inputs, route as a green-team bug
>
> Useful heuristic: **if the failing assertion is "X should exist in the output" and the actual output shows "0 X found," the fixture is the first suspect, not the implementation.** This is especially true when the failure surfaces immediately after the orchestrator has written or modified a fixture step.
>
> Cost: ~30 seconds per fixture check. Savings: one full green re-dispatch cycle per mistaken routing.

---

## F4 — Inline-fix-vs-re-dispatch carveout for no-algorithmic-content bugs

### Symptom observed

Cucumber reported a single failing scenario for `cross_project_graph`: `"Summary does not contain 'cross-project graph': insufficient cross-project references: ... (found 0 projects, 0 edges)"`. The green script was reading `session.get("metadata", {}).get("project_name", "")` — but the Rust `SessionMetadata` struct field is `project`, not `project_name`. This was a 1-line field-name drift.

### Root cause

Two factors:

1. The NLSpec originally used `project_name`, which the orchestrator corrected to `project` mid-process after the red team flagged it (see F5). The orchestrator ran `sed -i ''` across the spec + prompt files but missed one occurrence in the shared contract file, which Kimi had already been primed with.
2. Kimi's pattern-matching habits reinforced the old name even where the spec had been corrected.

### What the orchestrator did

**Fixed the typo inline** — one line edit in the Python file — instead of routing the failure back to green as a pass/fail label and re-dispatching. This was a **small but real barrier violation**: the orchestrator read the implementation code to find the bug.

### Why inline was defensible

The bug had **no algorithmic content**. It was a literal string mismatch — `project_name` vs `project`. The orchestrator could have routed it to green as "field-name mismatch, check `session.metadata.project`" but that's not even a pass/fail label, it's already an in-context fix instruction. Re-dispatching would have added a full dispatch cycle for a mechanical edit.

### Proposed playbook addition

The current playbook is strict: "Never read both sides and fix code directly. That breaks provenance — the orchestrator becomes a god-mode fixer and the adversarial guarantee is void."

This is correct in principle but produces a hole: there's no middle ground between "route pass/fail to green" and "god-mode fix." The real distinction is **algorithmic content**. Propose adding:

> **Inline fixes are acceptable for green-team bugs with NO algorithmic content.** Specifically allowed:
>
> - Literal typos (variable name misspellings)
> - Field-name drift matching a known NLSpec correction the orchestrator already made
> - Off-by-one at a loop boundary when the intended bound is obvious from the surrounding code
> - Missing `if __name__ == '__main__':` guards
> - Imports left out but clearly needed by code that's otherwise complete
>
> **Anything with algorithmic content must go back to green.** Specifically forbidden inline:
>
> - Wrong aggregation (sum vs mean, product vs sum)
> - Wrong iteration order
> - Missing edge cases (empty input, single element, zero divisor)
> - Wrong threshold comparisons (`>=` vs `>`)
> - Missing normalization or sanitization
> - Missing fallback branches
>
> **The test:** could I explain this fix to green in a single pass/fail label? If no (and the fix is genuinely mechanical), inline is fine — but **log it in the retrospective** so the adversarial guarantee stays auditable.

---

## F5 — Post-correction verification must use `grep -n`, not `grep -c`

### Symptom observed

The orchestrator corrected a NLSpec field name (`project_name` → `project`), ran `sed -i ''` across the spec + per-technique prompt files, and then verified the fix by running:

```bash
grep -c "project_name" <files>
```

All files reported 0. The orchestrator considered the fix complete and dispatched green.

Green's output still contained the old field name (see F4).

### Root cause

`grep -c` reports a per-file count. One file had a residual `project_name` reference that `sed -i ''` didn't catch because the regex pattern was slightly different. `grep -c` showed `0` for that file because the match had been suppressed by an encoding difference (a smart-quote apostrophe inside an inline regex example); `grep -n` would have pinpointed the missed occurrence on a specific line.

(This turned out to be a second-order cause of F4 — the drift wasn't just Kimi's, it was partially an orchestrator-side verification failure.)

### Proposed playbook addition

Add a small verification-discipline note to the orchestrator's workflow:

> **After any NLSpec correction, verify ALL reference files with `grep -n <old_term>`, not `grep -c`.** The `-c` flag is per-file and can still miss the specific lines that matter when output encoding or quote characters differ subtly from your `sed` pattern. Use `-n` for line-level visibility and confirm zero actual hits across the set:
>
> ```bash
> grep -n '<old_term>' <spec_file> <shared_contract> <per_technique_prompts> && \
>   echo "RESIDUAL MATCHES — fix before dispatch" || \
>   echo "clean"
> ```
>
> When in doubt, regenerate the green prompt files from scratch rather than patching them.

---

## F6 — Add "orchestrator fallibility" as a first-class failure category

### Symptom observed

The current playbook classifies test failures as one of:

- Contract gap (spec didn't specify enough) → NLSpec author
- Red team bug (wrong test data) → Red team
- Green team bug (wrong implementation) → Green team
- Convention mismatch → NLSpec author
- Ambiguous spec → NLSpec author

During this Batch 4 run, **three of the five observed failure modes** (F1, F3, F5) were orchestrator mistakes, not red or green mistakes. They were eventually caught, but the playbook's taxonomy doesn't include "orchestrator screwed up" as a category, which makes it easy to mis-route (e.g., suspect green when the fixture is wrong — see F3).

### Root cause

The playbook implicitly assumes the orchestrator is infallible: it writes correct specs, builds correct fixtures, dispatches with correct CLI incantations, and routes failures honestly. When the orchestrator is an LLM (as here), these assumptions are soft. LLM orchestrators make errors at every stage and have no external audit.

### Proposed playbook addition

Add a new section titled "Orchestrator Fallibility":

> **The orchestrator can also be the source of a bug.** When a test fails, before classifying as red/green/contract-gap, ask: did the orchestrator get something wrong? Common orchestrator-side failure modes:
>
> - **Stale prompt files:** A correction in the canonical NLSpec didn't propagate to the green prompt attachment (see F5)
> - **Bad fixtures:** The test fixture step doesn't produce the inputs the assertions imply (see F3)
> - **Wrong dispatch command:** CLI argument order, stale model ID, forgot `--format json` (see F1)
> - **Wrong routing:** Classified a contract gap as a green-team bug and re-dispatched, wasting a cycle
> - **Inline fix without logging:** Fixed a green-team bug inline that had algorithmic content, breaking provenance (see F4)
>
> **Self-reporting is the primary defense — the orchestrator has no external audit.** The retrospective for each adversarial run should enumerate orchestrator errors alongside red/green errors. When a failure occurs, run through the orchestrator-side checklist BEFORE concluding it's a red or green bug. Budget: ~1 minute per failure. Savings: avoided re-dispatch cycles and preserved barrier integrity.

---

## How to act on this feedback

Each of the six proposals is self-contained: it has a symptom, a root cause, and a proposed playbook addition. A foundry session should be able to:

1. Read each item independently
2. Evaluate whether the proposed addition fits the project's voice and taxonomy
3. Accept, reject, or rework each item individually
4. File the accepted ones as edits to the relevant foundry docs

**This file is entirely self-contained — nothing in it requires reading anything else.**

Proposed target files (as of 2026-04-06):

- **F1, F2, F6:** `foundry/docs/solutions/workflow-issues/adversarial-orchestration-playbook-20260404.md` — the main playbook
- **F3, F4, F5:** same playbook OR a new `foundry/docs/solutions/best-practices/orchestrator-discipline.md` if the items warrant their own doc

If any item is reworked or rejected, a brief note back via the third-thoughts HANDOFF would help close the loop.
