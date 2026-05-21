---
title: Strengthen mechanical barrier enforcement and replayable audits
origin: 2026-04-17 ilia-feedback-foundry-plugin (item 2)
priority: high
status: landed
updated: 2026-05-21
---

# Mechanical Barrier Enforcement

**Addendum:** 2026-05-01 — public-plugin enforcement contract landed: `foundry-adversarial` now requires `PromptEnvelope` v1 artifacts for every dispatch, `tests/validate-barrier-envelopes.sh` mechanically checks withheld samples against prompts, and `barrier-integrity-auditor` audits replayable envelopes.

**Addendum:** 2026-05-21 — private dispatch enforcement is no longer pending for the active BuildKite/pi runtime. The private monorepo's `foundry/buildkite/scripts/run-agent-session.sh` writes and validates `foundry.prompt-envelope.v1` artifacts before invoking pi, uploads `runs/<agent_session_id>/dispatch/agent-session/<turn>.pi-agent.json` artifacts, and covers the path with `buildkite/scripts/test-prompt-envelope.sh` (17/17 in the private handoff). Full red/green engine integration remains future work, but the previously requested public/private PromptEnvelope contract mirror is landed.

The information barrier (red sees NLSpec + tests / green sees NLSpec How + pass/fail only) is the sharpest idea in the repo. But enforcement currently leans on careful orchestration prose in `foundry-adversarial/SKILL.md`. That makes the guarantee prompt-discipline, not mechanical.

## What to fix

- Mechanical checks around prompt shaping: the prompt sent to green must provably not contain any substring from red's test files, assertions, `.feature` files, or NLSpec Done section. Same in reverse for red.
- Redaction pipeline: prompt construction should pass through a named redaction step, not string-concat sections by hand.
- Replayable audits: every dispatch should leave an artifact (prompt sent + files visible + files withheld) that the barrier-integrity-auditor (and humans) can replay and diff against the invariant.

## Suggested approach

1. ✅ Add a PromptEnvelope type (or equivalent) to the engine side that owns "what entity sees what" — active private runtime uses `foundry.prompt-envelope.v1` in the BuildKite/pi dispatch layer.
2. ✅ Serialize each envelope to `runs/<run_id>/dispatch/<phase>/<agent>.json` so post-hoc audits work.
3. ✅ Extend validation with a sibling script that fuzzes prompt construction with known-poison inputs and asserts they are redacted (`buildkite/scripts/test-prompt-envelope.sh` in the private repo).

See: `docs/solutions/workflow-issues/ilia-feedback-foundry-plugin-20260417.md` (item 2).
