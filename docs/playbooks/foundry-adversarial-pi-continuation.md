# Foundry Adversarial Pi Continuation Playbook

Use this playbook when a live Pi adversarial run is interrupted after valid PromptEnvelope artifacts have already been written, for example because an outer shell timeout killed a long Phase 3 reviewer fan-out.

## Core Rule

Continue from serialized PromptEnvelope artifacts. Do not reconstruct red/green context in the main Pi conversation, and do not fix implementation code directly as the orchestrator.

## Safe Continuation Steps

1. **Inspect the last complete artifact state**
   - List `runs/<run_id>/dispatch/**.json`.
   - Run `tests/validate-barrier-envelopes.sh runs/<run_id>/dispatch` before continuing.
   - Read logs/outcome-label files only as orchestrator; do not paste raw failures to green.

2. **Identify the next recipient**
   - If a reviewer rejected implementation, route feedback to green.
   - If a red reviewer rejected tests, route feedback to red.
   - If the barrier auditor rejected, stop and fix the envelope/context leak before any team dispatch.

3. **Write a new PromptEnvelope**
   - Use a new path such as `runs/<run_id>/dispatch/phase3/green-team-reviewer-fix.json`.
   - Keep `run_id` stable.
   - Preserve `schema_version: foundry.prompt-envelope.v1`.
   - For green fixes, visible context may include only:
     - NLSpec How section.
     - Review feedback that does not reveal test code/assertions/raw failures/NLSpec Done.
     - PASS/FAIL outcome labels.
   - Put follow-up instructions after a `## Task` header so `Test results:` parsing has a hard terminator.

4. **Choose withheld samples carefully**
   - Use assertion/body/raw-output snippets as poison samples.
   - Do **not** use PASS/FAIL test outcome labels or terminal test names as withheld samples. Green is allowed to see those labels, so they are bad poison samples and should fail validation.
   - Include NLSpec Done snippets for green envelopes.

5. **Validate before dispatch**
   ```bash
   tests/validate-barrier-envelopes.sh runs/<run_id>/dispatch
   ```

6. **Dispatch through `foundry_team`**
   ```bash
   pi \
     -e ./extensions/pi-foundry-team/index.ts \
     --mode json \
     -p \
     --no-session \
     --no-context-files \
     --no-skills \
     --tools foundry_team \
     'Call foundry_team exactly once for envelopePath runs/<run_id>/dispatch/<phase>/<recipient>.json ...'
   ```

7. **Run tests and finalize replay artifacts**
   - Assemble runner workspace as orchestrator.
   - Run tests.
   - Send only PASS/FAIL labels back to green if further implementation work is needed.
   - Update `runs/<run_id>/behavioral-smoke.toon` with final counts and model lanes.

8. **Final validation**
   ```bash
   tests/validate-barrier-envelopes.sh runs/<run_id>/dispatch
   tests/behavioral-smoke.sh runs/<run_id>
   ```

## Observed Example

`runs/pi-from-scratch-roman-numeral/` used this pattern after an outer 900-second shell timeout interrupted Phase 3 reviewer fan-out. A Rust reviewer finding was routed back to green through `dispatch/phase3/green-team-reviewer-fix.json`; green fixed the implementation through `foundry_team`; tests passed `8/8`; a follow-up Rust reviewer approved; barrier and behavioral validators passed.
