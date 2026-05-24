# Foundry Adversarial Spec Update and Restart Playbook

This playbook is the authoritative `spec_update_and_restart` module for the adversarial skill. It is triggered only when the divergence evaluator returns `findings[0].outcome == "VALUABLE"`.

## Hard Rule

The orchestrator MUST NOT write NLSpec content directly. The NLSpec agent is the sole author of revised NLSpec content.

## Inputs

- `original_spec_path`: path to the original spec document.
- `existing_nlspec_path`: path to the current NLSpec.
- `evaluator_feedback`: `findings[0].gap_description` verbatim; do not paraphrase.
- `red_test_paths`: current red team test paths when available, especially for Phase 1b restarts.
- `PipelineRevisionState`: includes `revision_count`, `revision_cap`, and `revision_history`.

## Procedure

1. **Check revision cap** — Read `PipelineRevisionState.revision_count`. If `revision_count >= revision_cap` (default `10`), pause and present full `revision_history` to the user before continuing.
2. **Re-run NLSpec agent** with `NLSpecRerunInput`:
   - `original_spec_path`
   - `existing_nlspec_path`
   - `evaluator_feedback`: exact `findings[0].gap_description`
3. **If NLSpec agent fails** — pause; present `findings[0].gap_description` to the user; do NOT commit; leave the NLSpec unchanged.
4. **Commit current NLSpec (`commit_before`)** — attribute to the NLSpec agent and guard the empty-staged case:
   ```bash
   git add <nlspec_path>
   git diff --staged --quiet || git commit --author="nlspec-agent <nlspec-agent@foundry>" -m "nlspec: preserve pre-revision NLSpec before divergence update"
   ```
5. **Write new NLSpec** — overwrite `<nlspec_path>` with the NLSpec agent's output.
6. **Commit new NLSpec (`commit_after`)** — attribute to the NLSpec agent.
7. **Generate `ChangeSummary`** from before/after NLSpec files:
   - `sections_added`: list of new section headings.
   - `sections_modified`: list of changed section headings.
   - `requirements_delta`: list of added/removed requirements.
8. **Update revision state** — increment `revision_count`; append `RevisionRecord(commit_before, commit_after)` to `revision_history`.
9. **Restart Phase 1** with a `Phase1RestartPackage`:
   - `existing_tests`: current red team test files, unmodified.
   - `new_nlspec_path`: path to the revised NLSpec.
   - `change_summary`: the generated `ChangeSummary`.
   - `red_test_paths`: paths to current red team test files.

## Restart Rules

- Re-initialize `TestFailureTracker`; pipeline-run-scoped state is cleared.
- Red team receives existing tests unchanged, the revised NLSpec path, the change summary, and current `red_test_paths`.
- Red team reviews existing tests against the new NLSpec and change summary, then revises or extends as needed.
- Red team MUST NOT discard previously passing tests without flagging the removal.
- Orchestrator reviews removed tests against the new NLSpec before continuing.
- Phase 1b review runs after revision.

## Behavioral-Smoke Contract

After each `VALUABLE` restart, update `runs/<run_id>/behavioral-smoke.toon` so the restart row has `revision_history_count` exactly `1` for that event.
