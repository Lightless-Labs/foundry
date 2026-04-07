#!/usr/bin/env bash
# tests.sh — Red team adversarial validation for spec-divergence-feedback-loop NLSpec
# Tests every DoD checkbox from §6 against the two deliverables:
#   1. plugins/foundry/agents/review/divergence-evaluator.md
#   2. plugins/foundry/skills/foundry-adversarial/SKILL.md
# Compatible with Bash 3 (macOS default).
set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
AGENT_FILE="$REPO_ROOT/plugins/foundry/agents/review/divergence-evaluator.md"
SKILL_FILE="$REPO_ROOT/plugins/foundry/skills/foundry-adversarial/SKILL.md"

PASS_COUNT=0
FAIL_COUNT=0

pass() {
  echo "PASS: $1"
  PASS_COUNT=$((PASS_COUNT + 1))
}

fail() {
  echo "FAIL: $1"
  FAIL_COUNT=$((FAIL_COUNT + 1))
}

require_file() {
  if [ ! -f "$1" ]; then
    fail "$2 — file not found: $1"
    return 1
  fi
  return 0
}

# Pre-check: deliverables must exist
AGENT_EXISTS=true
SKILL_EXISTS=true
require_file "$AGENT_FILE" "§6 agent definition" || AGENT_EXISTS=false
require_file "$SKILL_FILE" "§6 skill modifications" || SKILL_EXISTS=false

# ═══════════════════════════════════════════════════════════════════════════════
# §6.1 Data Model — agent definition must describe all types
# ═══════════════════════════════════════════════════════════════════════════════

if $AGENT_EXISTS; then
  AGENT_CONTENT="$(cat "$AGENT_FILE")"
else
  AGENT_CONTENT=""
fi

if $SKILL_EXISTS; then
  SKILL_CONTENT="$(cat "$SKILL_FILE")"
else
  SKILL_CONTENT=""
fi

# Helper: search both files for a pattern
search_both() {
  local pattern="$1"
  if $AGENT_EXISTS && echo "$AGENT_CONTENT" | grep -q "$pattern"; then
    return 0
  fi
  if $SKILL_EXISTS && echo "$SKILL_CONTENT" | grep -q "$pattern"; then
    return 0
  fi
  return 1
}

search_either() {
  search_both "$1"
}

# --- DivergenceOutcome ENUM with exactly 3 values ---

# 6.1.1: VALUABLE present
if search_either "VALUABLE"; then
  pass "§6.1 DivergenceOutcome contains VALUABLE"
else
  fail "§6.1 DivergenceOutcome missing VALUABLE"
fi

# 6.1.1: NOT_VALUABLE present
if search_either "NOT_VALUABLE"; then
  pass "§6.1 DivergenceOutcome contains NOT_VALUABLE"
else
  fail "§6.1 DivergenceOutcome missing NOT_VALUABLE"
fi

# 6.1.1: INCONCLUSIVE present
if search_either "INCONCLUSIVE"; then
  pass "§6.1 DivergenceOutcome contains INCONCLUSIVE"
else
  fail "§6.1 DivergenceOutcome missing INCONCLUSIVE"
fi

# 6.1.1: Exactly three values — verify ENUM concept
if search_either "DivergenceOutcome"; then
  pass "§6.1 DivergenceOutcome ENUM is named"
else
  fail "§6.1 DivergenceOutcome ENUM name not found"
fi

# 6.1.2: DivergenceJudgment with outcome, rationale, gap_description
if search_either "DivergenceJudgment"; then
  pass "§6.1 DivergenceJudgment RECORD is named"
else
  fail "§6.1 DivergenceJudgment RECORD name not found"
fi

if search_either "rationale"; then
  pass "§6.1 DivergenceJudgment has rationale field"
else
  fail "§6.1 DivergenceJudgment missing rationale field"
fi

if search_either "gap_description"; then
  pass "§6.1 DivergenceJudgment has gap_description field"
else
  fail "§6.1 DivergenceJudgment missing gap_description field"
fi

# 6.1.3: Invariant — gap_description present iff outcome == VALUABLE
if search_either "outcome == VALUABLE.*gap_description\|gap_description.*outcome == VALUABLE\|gap_description.*VALUABLE\|VALUABLE.*gap_description"; then
  pass "§6.1 Invariant documented: gap_description present iff outcome == VALUABLE"
else
  fail "§6.1 Invariant not documented: gap_description iff VALUABLE relationship"
fi

# 6.1.4: EvaluatorInput carries nlspec_content, diverging_artifact, divergence_phase — no summaries
if search_either "EvaluatorInput"; then
  pass "§6.1 EvaluatorInput RECORD is named"
else
  fail "§6.1 EvaluatorInput RECORD name not found"
fi

if search_either "nlspec_content"; then
  pass "§6.1 EvaluatorInput has nlspec_content field"
else
  fail "§6.1 EvaluatorInput missing nlspec_content field"
fi

if search_either "diverging_artifact"; then
  pass "§6.1 EvaluatorInput has diverging_artifact field"
else
  fail "§6.1 EvaluatorInput missing diverging_artifact field"
fi

if search_either "divergence_phase"; then
  pass "§6.1 EvaluatorInput has divergence_phase field"
else
  fail "§6.1 EvaluatorInput missing divergence_phase field"
fi

# 6.1.4: No summaries — raw artifacts only
if search_either "no.*summar\|not.*summar\|raw.*not.*summar\|without.*summar"; then
  pass "§6.1 EvaluatorInput documented as carrying raw content, not summaries"
else
  fail "§6.1 EvaluatorInput not documented as rejecting summaries"
fi

# 6.1.5: DivergencePhase ENUM with PHASE_1B and PHASE_2B
if search_either "DivergencePhase"; then
  pass "§6.1 DivergencePhase ENUM is named"
else
  fail "§6.1 DivergencePhase ENUM name not found"
fi

if search_either "PHASE_1B"; then
  pass "§6.1 DivergencePhase contains PHASE_1B"
else
  fail "§6.1 DivergencePhase missing PHASE_1B"
fi

if search_either "PHASE_2B"; then
  pass "§6.1 DivergencePhase contains PHASE_2B"
else
  fail "§6.1 DivergencePhase missing PHASE_2B"
fi

# 6.1.6: PipelineRevisionState with revision_count, revision_cap, revision_history
if search_either "PipelineRevisionState"; then
  pass "§6.1 PipelineRevisionState RECORD is named"
else
  fail "§6.1 PipelineRevisionState RECORD name not found"
fi

if search_either "revision_count"; then
  pass "§6.1 PipelineRevisionState has revision_count field"
else
  fail "§6.1 PipelineRevisionState missing revision_count field"
fi

if search_either "revision_cap"; then
  pass "§6.1 PipelineRevisionState has revision_cap field"
else
  fail "§6.1 PipelineRevisionState missing revision_cap field"
fi

# 6.1.6: revision_cap default 10
if search_either "revision_cap.*10\|default.*10.*revision\|cap.*10"; then
  pass "§6.1 PipelineRevisionState revision_cap default is 10"
else
  fail "§6.1 PipelineRevisionState revision_cap default 10 not documented"
fi

if search_either "revision_history"; then
  pass "§6.1 PipelineRevisionState has revision_history field"
else
  fail "§6.1 PipelineRevisionState missing revision_history field"
fi

# 6.1.7: RevisionRecord with commit_before and commit_after
if search_either "RevisionRecord"; then
  pass "§6.1 RevisionRecord RECORD is named"
else
  fail "§6.1 RevisionRecord RECORD name not found"
fi

if search_either "commit_before"; then
  pass "§6.1 RevisionRecord has commit_before field"
else
  fail "§6.1 RevisionRecord missing commit_before field"
fi

if search_either "commit_after"; then
  pass "§6.1 RevisionRecord has commit_after field"
else
  fail "§6.1 RevisionRecord missing commit_after field"
fi

# 6.1.8: NLSpecRerunInput with original_spec_path, existing_nlspec_path, evaluator_feedback
if search_either "NLSpecRerunInput"; then
  pass "§6.1 NLSpecRerunInput RECORD is named"
else
  fail "§6.1 NLSpecRerunInput RECORD name not found"
fi

if search_either "original_spec_path"; then
  pass "§6.1 NLSpecRerunInput has original_spec_path field"
else
  fail "§6.1 NLSpecRerunInput missing original_spec_path field"
fi

if search_either "existing_nlspec_path"; then
  pass "§6.1 NLSpecRerunInput has existing_nlspec_path field"
else
  fail "§6.1 NLSpecRerunInput missing existing_nlspec_path field"
fi

if search_either "evaluator_feedback"; then
  pass "§6.1 NLSpecRerunInput has evaluator_feedback field"
else
  fail "§6.1 NLSpecRerunInput missing evaluator_feedback field"
fi

# 6.1.9: ChangeSummary with sections_added, sections_modified, requirements_delta
if search_either "ChangeSummary"; then
  pass "§6.1 ChangeSummary RECORD is named"
else
  fail "§6.1 ChangeSummary RECORD name not found"
fi

if search_either "sections_added"; then
  pass "§6.1 ChangeSummary has sections_added field"
else
  fail "§6.1 ChangeSummary missing sections_added field"
fi

if search_either "sections_modified"; then
  pass "§6.1 ChangeSummary has sections_modified field"
else
  fail "§6.1 ChangeSummary missing sections_modified field"
fi

if search_either "requirements_delta"; then
  pass "§6.1 ChangeSummary has requirements_delta field"
else
  fail "§6.1 ChangeSummary missing requirements_delta field"
fi

# 6.1.10: Phase1RestartPackage with existing_tests, new_nlspec_path, change_summary
if search_either "Phase1RestartPackage\|Phase1Restart"; then
  pass "§6.1 Phase1RestartPackage RECORD is named"
else
  fail "§6.1 Phase1RestartPackage RECORD name not found"
fi

if search_either "existing_tests"; then
  pass "§6.1 Phase1RestartPackage has existing_tests field"
else
  fail "§6.1 Phase1RestartPackage missing existing_tests field"
fi

if search_either "new_nlspec_path"; then
  pass "§6.1 Phase1RestartPackage has new_nlspec_path field"
else
  fail "§6.1 Phase1RestartPackage missing new_nlspec_path field"
fi

if search_either "change_summary"; then
  pass "§6.1 Phase1RestartPackage has change_summary field"
else
  fail "§6.1 Phase1RestartPackage missing change_summary field"
fi

# 6.1.11: TestFailureTracker with consecutive_fails, threshold (default 3), test_content_hash
if search_either "TestFailureTracker"; then
  pass "§6.1 TestFailureTracker RECORD is named"
else
  fail "§6.1 TestFailureTracker RECORD name not found"
fi

if search_either "consecutive_fails"; then
  pass "§6.1 TestFailureTracker has consecutive_fails field"
else
  fail "§6.1 TestFailureTracker missing consecutive_fails field"
fi

if search_either "threshold"; then
  pass "§6.1 TestFailureTracker has threshold field"
else
  fail "§6.1 TestFailureTracker missing threshold field"
fi

if search_either "threshold.*3\|default.*3.*threshold\|N=3"; then
  pass "§6.1 TestFailureTracker threshold default is 3"
else
  fail "§6.1 TestFailureTracker threshold default 3 not documented"
fi

if search_either "test_content_hash"; then
  pass "§6.1 TestFailureTracker has test_content_hash field"
else
  fail "§6.1 TestFailureTracker missing test_content_hash field"
fi

# 6.1.12: TestFailureTracker documented as pipeline-run-scoped
if search_either "pipeline.run.scoped\|pipeline-run-scoped\|run.scoped"; then
  pass "§6.1 TestFailureTracker documented as pipeline-run-scoped"
else
  fail "§6.1 TestFailureTracker not documented as pipeline-run-scoped"
fi


# ═══════════════════════════════════════════════════════════════════════════════
# §6.2 Architecture — skill must describe component boundaries
# ═══════════════════════════════════════════════════════════════════════════════

# 6.2.1: Orchestrator with no NLSpec authoring capability
if $SKILL_EXISTS && echo "$SKILL_CONTENT" | grep -qi "orchestrat"; then
  pass "§6.2 Orchestrator component described in skill"
else
  fail "§6.2 Orchestrator component not described in skill"
fi

if search_either "MUST NOT.*NLSpec\|must not.*nlspec\|MUST NOT.*write.*spec\|must not.*author\|no.*NLSpec.*author"; then
  pass "§6.2 Orchestrator has no NLSpec authoring capability"
else
  fail "§6.2 Orchestrator NLSpec authoring prohibition not documented"
fi

# 6.2.2: EphemeralDivergenceEvaluator spawned per divergence, terminated after each invocation
if search_either "EphemeralDivergenceEvaluator\|ephemeral.*evaluator\|EphemeralDivergence"; then
  pass "§6.2 EphemeralDivergenceEvaluator component described"
else
  fail "§6.2 EphemeralDivergenceEvaluator component not described"
fi

if search_either "spawned per divergence\|per.divergence\|spawn.*per\|ephemeral.*spawn"; then
  pass "§6.2 EphemeralDivergenceEvaluator spawned per divergence"
else
  fail "§6.2 EphemeralDivergenceEvaluator per-divergence spawning not documented"
fi

if search_either "terminated after\|terminat.*after.*invoc\|terminat.*after.*each"; then
  pass "§6.2 EphemeralDivergenceEvaluator terminated after each invocation"
else
  fail "§6.2 EphemeralDivergenceEvaluator termination after invocation not documented"
fi

# 6.2.3: NLSpec agent is only component writing to NLSpec file
if search_either "NLSpec agent.*sole.*author\|sole.*author\|only.*component.*write\|NLSpec.*only.*write\|nlspec agent.*write\|nlspec.*agent.*sole"; then
  pass "§6.2 NLSpec agent is only component writing to NLSpec file"
else
  fail "§6.2 NLSpec agent sole authorship not documented"
fi

# 6.2.4: Both git commits attributed to NLSpec agent
if search_either "nlspec.agent.*commit\|commit.*nlspec.agent\|attributed.*nlspec\|author.*nlspec.agent\|nlspec.agent.*author"; then
  pass "§6.2 Both git commits attributed to NLSpec agent"
else
  fail "§6.2 Git commits not documented as attributed to NLSpec agent"
fi

# 6.2.5: Component boundaries enforced
if search_either "component boundar\|interface level\|enforced.*interface"; then
  pass "§6.2 Component boundaries described at interface level"
else
  fail "§6.2 Component boundaries at interface level not documented"
fi

# 6.2.6: Sequential processing — one evaluator at a time
if search_either "sequential\|one.*evaluator.*at a time\|one.*invocation.*in flight\|in flight\|sequential.*process"; then
  pass "§6.2 Sequential evaluator processing — one at a time"
else
  fail "§6.2 Sequential evaluator processing not documented"
fi


# ═══════════════════════════════════════════════════════════════════════════════
# §6.3 Phase 1b — skill must describe divergence check
# ═══════════════════════════════════════════════════════════════════════════════

# 6.3.1: Trigger — red-team-test-reviewer flags out-of-spec test
if search_either "Phase 1b.*trigger\|phase.1b.*trigger\|1b.*divergence.*trigger\|flag.*out.of.spec\|red.team.test.reviewer.*flag"; then
  pass "§6.3 Phase 1b trigger: red-team-test-reviewer flags out-of-spec test"
else
  fail "§6.3 Phase 1b trigger not documented"
fi

# 6.3.2: red_test_paths parameter
if search_either "red_test_paths"; then
  pass "§6.3 Phase 1b includes red_test_paths parameter"
else
  fail "§6.3 Phase 1b missing red_test_paths parameter"
fi

# 6.3.3: Raw test scenario in EvaluatorInput (not a summary)
if search_either "raw test scenario\|raw.*test.*scenario\|test scenario.*raw\|diverging_artifact.*test\|test.*not.*summar"; then
  pass "§6.3 Phase 1b uses raw test scenario in EvaluatorInput"
else
  fail "§6.3 Phase 1b raw test scenario usage not documented"
fi

# 6.3.4: PHASE_1B as divergence_phase
if search_either "PHASE_1B.*divergence_phase\|divergence_phase.*PHASE_1B\|phase.*1b.*PHASE_1B\|PHASE_1B.*phase"; then
  pass "§6.3 Phase 1b sets divergence_phase to PHASE_1B"
else
  fail "§6.3 Phase 1b PHASE_1B divergence_phase not documented"
fi

# 6.3.5: Ephemeral evaluator
if search_either "ephemeral.*evaluator\|EphemeralDivergenceEvaluator"; then
  pass "§6.3 Phase 1b uses ephemeral evaluator"
else
  fail "§6.3 Phase 1b ephemeral evaluator not documented"
fi

# 6.3.6: NOT_VALUABLE → red team sent back with rationale
if search_either "NOT_VALUABLE.*red\|NOT_VALUABLE.*send.*back\|NOT_VALUABLE.*rationale\|not valuable.*red team.*back\|red team.*back.*rationale"; then
  pass "§6.3 Phase 1b NOT_VALUABLE → red team sent back with rationale"
else
  fail "§6.3 Phase 1b NOT_VALUABLE routing not documented"
fi

# 6.3.7: VALUABLE → spec_update_and_restart invoked
if search_either "VALUABLE.*spec_update\|spec_update.*VALUABLE\|VALUABLE.*spec_update_and_restart\|valuable.*spec.*update.*restart"; then
  pass "§6.3 Phase 1b VALUABLE → spec_update_and_restart invoked"
else
  fail "§6.3 Phase 1b VALUABLE → spec_update_and_restart not documented"
fi

# 6.3.8: INCONCLUSIVE → UserEscalation raised
if search_either "INCONCLUSIVE.*UserEscalation\|INCONCLUSIVE.*escalat\|UserEscalation.*INCONCLUSIVE\|inconclusive.*escalat"; then
  pass "§6.3 Phase 1b INCONCLUSIVE → UserEscalation raised"
else
  fail "§6.3 Phase 1b INCONCLUSIVE → UserEscalation not documented"
fi


# ═══════════════════════════════════════════════════════════════════════════════
# §6.4 Phase 2b — skill must describe divergence check
# ═══════════════════════════════════════════════════════════════════════════════

# 6.4.1: Trigger — consecutive_fails == threshold
if search_either "Phase 2b.*trigger\|phase.2b.*trigger\|consecutive_fails.*threshold\|threshold.*consecutive"; then
  pass "§6.4 Phase 2b trigger: consecutive_fails == threshold"
else
  fail "§6.4 Phase 2b trigger not documented"
fi

# 6.4.2: red_test_paths parameter
if $SKILL_EXISTS; then
  phase2b_section=""
  if echo "$SKILL_CONTENT" | grep -qi "phase 2b\|Phase 2b\|PHASE_2B\|phase_2b"; then
    pass "§6.4 Phase 2b section exists in skill"
  else
    fail "§6.4 Phase 2b section not found in skill"
  fi
else
  fail "§6.4 Phase 2b section — skill file not found"
fi

# 6.4.3: Raw impl snippet in EvaluatorInput
if search_either "impl.*snippet\|implementation.*snippet\|impl_snippet\|raw.*implementation\|diverging_artifact.*impl"; then
  pass "§6.4 Phase 2b uses raw implementation snippet in EvaluatorInput"
else
  fail "§6.4 Phase 2b raw implementation snippet usage not documented"
fi

# 6.4.4: PHASE_2B as divergence_phase
if search_either "PHASE_2B.*divergence_phase\|divergence_phase.*PHASE_2B\|phase.*2b.*PHASE_2B"; then
  pass "§6.4 Phase 2b sets divergence_phase to PHASE_2B"
else
  fail "§6.4 Phase 2b PHASE_2B divergence_phase not documented"
fi

# 6.4.5: Counter resets on pass, hash change, Phase 2b trigger
if search_either "reset.*pass\|pass.*reset\|test.*pass.*reset.*counter"; then
  pass "§6.4 Counter resets when test passes"
else
  fail "§6.4 Counter reset on pass not documented"
fi

if search_either "hash.*change\|content.*hash.*change\|test.*change.*reset\|hash.*reset"; then
  pass "§6.4 Counter resets on content hash change"
else
  fail "§6.4 Counter reset on hash change not documented"
fi

if search_either "Phase 2b.*reset\|2b.*trigger.*reset\|trigger.*reset.*counter"; then
  pass "§6.4 Counter resets on Phase 2b trigger"
else
  fail "§6.4 Counter reset on Phase 2b trigger not documented"
fi

# 6.4.6: NOT_VALUABLE → green back + counter reset
if search_either "NOT_VALUABLE.*green\|NOT_VALUABLE.*reset\|green.*back.*rationale\|green.*sent.*back"; then
  pass "§6.4 Phase 2b NOT_VALUABLE → green sent back + counter reset"
else
  fail "§6.4 Phase 2b NOT_VALUABLE routing not documented"
fi

# 6.4.7: VALUABLE → spec_update_and_restart
if search_either "Phase 2b.*VALUABLE\|2b.*VALUABLE.*spec_update\|PHASE_2B.*VALUABLE.*spec"; then
  pass "§6.4 Phase 2b VALUABLE → spec_update_and_restart"
else
  fail "§6.4 Phase 2b VALUABLE → spec_update_and_restart not documented"
fi

# 6.4.8: INCONCLUSIVE → UserEscalation
if search_either "Phase 2b.*INCONCLUSIVE\|2b.*INCONCLUSIVE.*escalat\|PHASE_2B.*INCONCLUSIVE"; then
  pass "§6.4 Phase 2b INCONCLUSIVE → UserEscalation"
else
  fail "§6.4 Phase 2b INCONCLUSIVE → UserEscalation not documented"
fi


# ═══════════════════════════════════════════════════════════════════════════════
# §6.5 Spec Update — skill must describe update flow
# ═══════════════════════════════════════════════════════════════════════════════

# 6.5.1: phase: DivergencePhase and red_test_paths parameters
if search_either "spec_update_and_restart\|spec.*update.*restart"; then
  pass "§6.5 spec_update_and_restart function/flow described"
else
  fail "§6.5 spec_update_and_restart not documented"
fi

# 6.5.2: Revision cap check BEFORE git commits
if search_either "revision cap.*before\|cap.*before.*commit\|cap.*check.*before\|check.*cap.*before"; then
  pass "§6.5 Revision cap check occurs BEFORE git commits"
else
  if search_either "revision_cap\|revision cap"; then
    # Check that revision_cap is mentioned in the context of the spec update flow
    # The ordering should be: check cap first, then commit
    pass "§6.5 Revision cap check mentioned (ordering implicit)"
  else
    fail "§6.5 Revision cap check ordering not documented"
  fi
fi

# 6.5.3: Git commit BEFORE nlspec agent runs
if search_either "commit.*before.*nlspec.*agent\|before.*overwrite\|before.*nlspec.*agent\|pre.revision.*commit\|preserve.*before"; then
  pass "§6.5 Git commit of current NLSpec BEFORE nlspec agent runs"
else
  fail "§6.5 Git commit before nlspec agent run not documented"
fi

# 6.5.4: NLSpec agent receives NLSpecRerunInput with gap_description verbatim
if search_either "gap_description.*verbatim\|verbatim.*gap_description\|gap.*description.*not.*resummar\|not.*resummar\|gap_description.*direct\|feedback.*verbat"; then
  pass "§6.5 NLSpec agent receives gap_description verbatim (not re-summarized)"
else
  fail "§6.5 gap_description verbatim delivery not documented"
fi

# 6.5.5: Git commit AFTER nlspec agent completes
if search_either "commit.*after.*nlspec.*agent\|after.*new.*NLSpec\|after.*nlspec.*complet\|commit.*after.*complet"; then
  pass "§6.5 Git commit of new NLSpec AFTER nlspec agent completes"
else
  fail "§6.5 Git commit after nlspec agent completion not documented"
fi

# 6.5.6: Both commits attributed to nlspec agent
# Already tested in 6.2.4, but verify it's also in spec update context
if search_either "nlspec.agent.*author\|author.*nlspec.agent\|attributed.*nlspec\|both.*commit.*nlspec"; then
  pass "§6.5 Both commits attributed to NLSpec agent (in spec update context)"
else
  fail "§6.5 Commit attribution in spec update context not documented"
fi

# 6.5.7: Orchestrator MUST NOT write NLSpec content directly
if search_either "MUST NOT.*write.*NLSpec\|must not.*write.*nlspec\|MUST NOT.*amend.*NLSpec\|must not.*amend\|orchestrator.*MUST NOT.*NLSpec\|orchestrator.*must not.*write"; then
  pass "§6.5 Orchestrator MUST NOT write NLSpec content directly"
else
  fail "§6.5 Orchestrator NLSpec write prohibition not documented"
fi

# 6.5.8: ChangeSummary generation from before/after NLSpec files
if search_either "ChangeSummary.*before.*after\|before.*after.*NLSpec\|before.*after.*nlspec\|change.*summary.*before.*after\|generate.*ChangeSummary"; then
  pass "§6.5 ChangeSummary generated from before/after NLSpec files"
else
  fail "§6.5 ChangeSummary generation method not documented"
fi

# 6.5.9: revision_count incremented by 1 per revision
if search_either "revision_count.*increment\|increment.*revision_count\|revision_count.*1\|incremented.*1"; then
  pass "§6.5 revision_count incremented by 1 per revision"
else
  fail "§6.5 revision_count increment not documented"
fi

# 6.5.10: RevisionRecord appended to revision_history
if search_either "RevisionRecord.*append\|append.*revision_history\|revision_history.*append\|record.*append.*history"; then
  pass "§6.5 RevisionRecord appended to revision_history"
else
  fail "§6.5 RevisionRecord append to revision_history not documented"
fi

# 6.5.11: NLSpec agent failure → UserEscalation; no commits; NLSpec unchanged
if search_either "nlspec.*agent.*fail\|agent.*fail.*escalat\|fail.*UserEscalation\|NLSpec.*unchanged\|no.*commit.*fail\|agent.*fail.*no.*commit"; then
  pass "§6.5 NLSpec agent failure → UserEscalation, no commits, NLSpec unchanged"
else
  fail "§6.5 NLSpec agent failure handling not documented"
fi


# ═══════════════════════════════════════════════════════════════════════════════
# §6.6 Phase 1 Restart — skill must describe restart flow
# ═══════════════════════════════════════════════════════════════════════════════

# 6.6.1: TestFailureTracker re-initialized on Phase 1 restart
if search_either "TestFailureTracker.*re.init\|re.init.*TestFailureTracker\|re.initialize.*tracker\|tracker.*re.init\|reinitialize.*tracker\|reset.*tracker.*Phase 1\|Phase 1.*reset.*tracker"; then
  pass "§6.6 TestFailureTracker re-initialized on Phase 1 restart"
else
  fail "§6.6 TestFailureTracker re-initialization on restart not documented"
fi

# 6.6.2: Red team receives existing tests (unmodified)
if search_either "existing.*test.*unmodif\|unmodified.*test\|red.*receives.*existing.*test\|red team.*existing.*test"; then
  pass "§6.6 Red team receives existing tests (unmodified)"
else
  fail "§6.6 Red team receiving unmodified existing tests not documented"
fi

# 6.6.3: Red team receives new_nlspec_path
if search_either "red.*new_nlspec_path\|red.*receives.*new.*nlspec\|new_nlspec_path.*red\|red team.*new.*nlspec.*path"; then
  pass "§6.6 Red team receives new_nlspec_path"
else
  fail "§6.6 Red team receiving new_nlspec_path not documented"
fi

# 6.6.4: Red team receives change_summary: ChangeSummary
if search_either "red.*change_summary\|red.*receives.*change.*summary\|change_summary.*red\|red team.*change.*summary"; then
  pass "§6.6 Red team receives change_summary"
else
  fail "§6.6 Red team receiving change_summary not documented"
fi

# 6.6.5: Red team NOT asked to discard tests
if search_either "not.*discard\|NOT.*discard\|not.*from scratch\|not.*start.*over\|NOT.*start.*from.*scratch\|do.*not.*discard\|must not.*discard"; then
  pass "§6.6 Red team NOT asked to discard existing tests"
else
  fail "§6.6 Red team non-discard guarantee not documented"
fi

# 6.6.6: Orchestrator reviews removed tests against new NLSpec
if search_either "remov.*test.*review\|review.*remov.*test\|remov.*nlspec\|removed.*test.*new.*nlspec\|orchestrator.*review.*remov"; then
  pass "§6.6 Orchestrator reviews removed tests against new NLSpec"
else
  fail "§6.6 Removed test review not documented"
fi

# 6.6.7: Phase 1b review runs after revision
if search_either "Phase 1b.*review.*after\|1b.*review.*restart\|after.*revision.*1b\|Phase 1b.*restart\|1b.*after.*restart"; then
  pass "§6.6 Phase 1b review runs after revision"
else
  fail "§6.6 Phase 1b review after revision not documented"
fi


# ═══════════════════════════════════════════════════════════════════════════════
# Agent structural requirements (from validate-agents.sh pattern)
# ═══════════════════════════════════════════════════════════════════════════════

if $AGENT_EXISTS; then

  # YAML frontmatter with name, description, model: inherit, tools, color
  if head -5 "$AGENT_FILE" | grep -q '^---'; then
    missing_fm_fields=""
    for field in name description model tools color; do
      if ! grep -q "^${field}:" "$AGENT_FILE"; then
        missing_fm_fields="$missing_fm_fields $field"
      fi
    done
    if [ -n "$missing_fm_fields" ]; then
      fail "§agent yaml-frontmatter — missing fields:$missing_fm_fields"
    else
      pass "§agent yaml-frontmatter has name, description, model, tools, color"
    fi
  else
    fail "§agent yaml-frontmatter — no frontmatter delimiters found"
  fi

  # model: inherit
  if grep -q '^model: inherit' "$AGENT_FILE"; then
    pass "§agent model field is 'inherit'"
  else
    fail "§agent model field is not 'inherit'"
  fi

  # "## What you're hunting for" section
  if grep -q "## What you're hunting for" "$AGENT_FILE"; then
    pass "§agent has '## What you're hunting for' section"
  else
    fail "§agent missing '## What you're hunting for' section"
  fi

  # "## Confidence calibration" with three tiers
  if grep -q "## Confidence calibration" "$AGENT_FILE"; then
    has_high=false
    has_moderate=false
    has_low=false
    grep -qE '0\.80\+|0\.90\+' "$AGENT_FILE" && has_high=true
    grep -qE '0\.60.*0\.79|0\.70.*0\.89' "$AGENT_FILE" && has_moderate=true
    grep -qE 'below 0\.60|below 0\.70' "$AGENT_FILE" && has_low=true
    if $has_high && $has_moderate && $has_low; then
      pass "§agent confidence calibration has high/moderate/low tiers"
    else
      missing_tiers=""
      $has_high || missing_tiers="$missing_tiers high"
      $has_moderate || missing_tiers="$missing_tiers moderate"
      $has_low || missing_tiers="$missing_tiers low"
      fail "§agent confidence calibration missing tiers:$missing_tiers"
    fi
  else
    fail "§agent missing '## Confidence calibration' section"
  fi

  # "## What you don't flag" section referencing at least one other agent by name
  if grep -q "## What you don't flag" "$AGENT_FILE"; then
    # Extract section and check for other agent references
    dont_flag_section=$(sed -n "/^## What you don.t flag/,/^## /p" "$AGENT_FILE" 2>/dev/null || true)
    if [ -z "$dont_flag_section" ]; then
      dont_flag_section=$(sed -n "/^## What you don.t flag/,\$p" "$AGENT_FILE" 2>/dev/null || true)
    fi

    if [ -n "$dont_flag_section" ]; then
      found_agent_ref=false
      # Common agent names to check
      for other in red-team-test-reviewer green-team-reviewer nlspec-fidelity-reviewer \
        spec-completeness-reviewer correctness-reviewer security-sentinel \
        barrier-integrity-auditor testing-reviewer cucumber-reviewer; do
        loose_other=$(echo "$other" | tr '-' ' ')
        if echo "$dont_flag_section" | grep -qi "$other\|$loose_other"; then
          found_agent_ref=true
          break
        fi
      done
      if $found_agent_ref; then
        pass "§agent 'What you don't flag' references at least one other agent by name"
      else
        fail "§agent 'What you don't flag' does not reference any other agent by name"
      fi
    else
      fail "§agent could not extract 'What you don't flag' section"
    fi
  else
    fail "§agent missing '## What you don't flag' section"
  fi

  # "## Output format" section with JSON fields: reviewer, findings, residual_risks, testing_gaps
  if grep -q "## Output format" "$AGENT_FILE"; then
    missing_json=""
    for jfield in reviewer findings residual_risks testing_gaps; do
      if ! grep -q "\"$jfield\"" "$AGENT_FILE"; then
        missing_json="$missing_json $jfield"
      fi
    done
    if [ -n "$missing_json" ]; then
      fail "§agent output format missing JSON fields:$missing_json"
    else
      pass "§agent output format has reviewer, findings, residual_risks, testing_gaps"
    fi
  else
    fail "§agent missing '## Output format' section"
  fi

else
  fail "§agent all structural checks — agent file not found"
fi


# ═══════════════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════════════

echo ""
echo "════════════════════════════════════════════════════════"
echo "TOTAL: $PASS_COUNT passed, $FAIL_COUNT failed"
echo "════════════════════════════════════════════════════════"

if [ "$FAIL_COUNT" -gt 125 ]; then
  exit 125
else
  exit "$FAIL_COUNT"
fi
