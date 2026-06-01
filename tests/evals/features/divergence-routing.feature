Feature: Foundry divergence routing evals
  The divergence evaluator is an ephemeral process gate for possible spec gaps.
  These eval cases mock evaluator outputs and verify route/provenance/barrier behavior
  for Phase 1b red-test divergence and Phase 2b repeated green failures.

  Background:
    Given PromptEnvelope schema "foundry.prompt-envelope.v1"
    And divergence evaluator dispatch is scoped to one divergence at a time
    And routing uses findings[0].outcome rather than top-level outcome or route_to fields
    And VALUABLE invokes spec_update_and_restart
    And NOT_VALUABLE routes back to the responsible team with barrier-preserving feedback
    And INCONCLUSIVE pauses for user judgment

  Scenario Outline: Mocked divergence output routes one divergence
    Given a divergence eval case "<case_id>"
    And the divergence phase is "<divergence_phase>"
    And the diverging artifact is "<diverging_artifact>"
    And the implementation snippet is "<implementation_snippet>"
    And the test id is "<test_id>"
    When the mocked divergence evaluator returns outcome "<mock_outcome>" routed to "<expected_route>"
    Then the orchestrator route should be "<expected_route>"
    And downstream follow-up should preserve provenance and information barriers

    Examples:
      | case_id                    | divergence_phase | nlspec_content                                      | nlspec_how                                           | diverging_artifact                                      | implementation_snippet                           | test_id              | test_result | red_test_paths                         | mock_outcome  | expected_route          | gap_description                                             | rationale                                           |
      | phase1b_valuable_gap        | PHASE_1B         | Roman numerals accept canonical subtractives only    | Parse by validating symbols then summing values      | Scenario: accept lowercase roman numerals              | none                                              | none                 | none        | red/features/roman.feature             | VALUABLE      | spec_update_and_restart | Clarify whether lowercase input is in scope                 | Lowercase behavior is absent from current NLSpec    |
      | phase1b_not_valuable_scope   | PHASE_1B         | Roman numerals accept only I,V,X,L,C,D,M             | Parse by validating symbols then summing values      | Scenario: accept emoji numeral tokens                  | none                                              | none                 | none        | red/features/roman.feature             | NOT_VALUABLE  | red-team                 | none                                                        | Emoji tokens are outside the NLSpec                 |
      | phase1b_inconclusive         | PHASE_1B         | Roman numerals must reject malformed subtractives    | Parse by validating symbols then summing values      | Scenario: reject VX as malformed subtractive           | none                                              | none                 | none        | red/features/roman.feature             | INCONCLUSIVE  | user                     | none                                                        | Need user judgment on convention                    |
      | phase2b_valuable_gap         | PHASE_2B         | Parser handles canonical uppercase numerals          | Parse by validating symbols then summing values      | repeated failure threshold reached                     | rejects any numeral containing repeated X        | roman_repeated_x     | FAIL        | red/features/roman.feature             | VALUABLE      | spec_update_and_restart | Clarify maximum repetition rules for tens symbols           | Repetition limit is underspecified                  |
      | phase2b_not_valuable_impl    | PHASE_2B         | Parser rejects empty input before conversion         | Parse by validating input before conversion          | repeated failure threshold reached                     | returns Ok(0) for empty input                    | roman_empty_input    | FAIL        | red/features/roman.feature             | NOT_VALUABLE  | green-team               | none                                                        | Implementation should reject empty input            |
      | phase2b_inconclusive         | PHASE_2B         | Parser trims or rejects whitespace consistently      | Validate normalized input before conversion          | repeated failure threshold reached                     | trims whitespace before validation                | roman_whitespace     | FAIL        | red/features/roman.feature             | INCONCLUSIVE  | user                     | none                                                        | Whitespace convention needs user judgment           |
