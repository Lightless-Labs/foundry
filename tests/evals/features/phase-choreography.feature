Feature: Foundry full phase choreography workflow evals
  A mocked adversarial run should move through the expected phases, preserve PromptEnvelope
  barriers at every dispatch, reset restart-scoped state when specs change, route reviewer
  rejections back to the correct team, and emit a valid behavioral smoke summary.

  Background:
    Given PromptEnvelope schema "foundry.prompt-envelope.v1"
    And every subagent dispatch is serialized before use
    And green sees only NLSpec How plus PASS/FAIL labels
    And red sees NLSpec/spec test criteria but no implementation material
    And VALUABLE divergence restarts reset the TestFailureTracker
    And finalization requires barrier and behavioral-smoke validators

  Scenario Outline: Mocked adversarial run follows the expected phase choreography
    Given a phase choreography eval case "<case_id>"
    And the scripted route is "<route_script>"
    When the orchestrator runs the mocked adversarial choreography
    Then the phase sequence should be "<expected_phase_sequence>"
    And the final route should be "<expected_final_route>"
    And requires_divergence_restart should be "<requires_divergence_restart>"
    And final test results should be "<expected_passed>/<expected_total>"
    And every dispatch envelope and behavioral smoke artifact should validate

    Examples:
      | case_id              | route_script                    | requires_divergence_restart | expected_phase_sequence                                                                         | expected_final_route       | expected_passed | expected_total |
      | happy_path           | all_pass                        | false                       | phase0>phase1>phase1b>phase2>phase2b>phase3>phase4                                             | finalized                  | 8               | 8              |
      | valuable_restart     | phase1b_valuable_restart        | true                        | phase0>phase1>phase1b>spec_update_and_restart>phase1>phase1b>phase2>phase2b>phase3>phase4      | finalized                  | 8               | 8              |
      | reviewer_reject_green | phase3_green_reject_then_fix    | false                       | phase0>phase1>phase1b>phase2>phase2b>phase3>phase2b>phase3>phase4                             | finalized_after_green_fix  | 8               | 8              |
