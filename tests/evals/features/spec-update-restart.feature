Feature: Foundry spec update and restart evals
  Valuable divergence routes must preserve provenance and route NLSpec changes through the NLSpec agent.
  The orchestrator must pass findings[0].gap_description verbatim, reset tracker state on restart,
  and pause when the revision cap is reached.

  Background:
    Given PromptEnvelope schema "foundry.prompt-envelope.v1"
    And the NLSpec agent is the sole author of revised NLSpec content
    And evaluator_feedback is findings[0].gap_description verbatim
    And TestFailureTracker resets when Phase 1 restarts
    And revision cap exhaustion escalates to the user without rewriting specs

  Scenario Outline: Spec update and restart preserves provenance
    Given a spec restart eval case "<case_id>"
    And the revision count is "<revision_count>" with cap "<revision_cap>"
    And the evaluator feedback is "<gap_description>"
    When spec_update_and_restart runs
    Then the route should be "<expected_route>"
    And provenance and restart state should match the expectation

    Examples:
      | case_id                  | original_spec_path                    | existing_nlspec_path                          | gap_description                                          | red_test_paths                                      | revision_count | revision_cap | expected_route          | expected_restart |
      | phase1b_restart          | docs/specs/roman-spec.md              | docs/nlspecs/roman.nlspec.md                  | Clarify whether lowercase input is in scope              | red/features/roman.feature                         | 1              | 10           | phase1_restart          | true             |
      | phase2b_restart          | docs/specs/roman-spec.md              | docs/nlspecs/roman.nlspec.md                  | Clarify maximum repetition rules for tens symbols        | red/features/roman.feature;red/features/errors.feature | 2              | 10           | phase1_restart          | true             |
      | revision_cap_escalation  | docs/specs/rubiks-spec.md             | docs/nlspecs/rubiks.nlspec.md                 | Clarify cube coordinate convention with golden vectors   | red/features/rubiks.feature                        | 10             | 10           | user                    | false            |
