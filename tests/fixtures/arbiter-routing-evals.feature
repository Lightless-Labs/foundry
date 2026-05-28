Feature: Foundry arbiter routing evals
  The arbiter is a controlled single-test breach in the adversarial workflow.
  These eval cases mock agent outputs and verify routing/provenance without live model calls.

  Background:
    Given PromptEnvelope schema "foundry.prompt-envelope.v1"
    And arbiter dispatch is scoped to exactly one disputed test
    And green follow-up must receive only NLSpec How plus PASS/FAIL labels
    And red follow-up must never receive implementation code

  Scenario Outline: Mocked arbiter output routes one disputed test
    Given an arbiter dispute eval case "<case_id>"
    And the NLSpec rule is "<nlspec_rule>"
    And the disputed test artifact is "<test_artifact>"
    And the relevant implementation snippet is "<implementation_snippet>"
    And the runner outcome is "<runner_outcome>"
    When the mocked arbiter returns outcome "<mock_outcome>" routed to "<mock_route_to>"
    Then the orchestrator route should be "<mock_route_to>"
    And downstream follow-up should preserve the information barrier

    Examples:
      | case_id                | nlspec_rule                                           | test_artifact                                             | implementation_snippet                                | runner_outcome | mock_outcome          | mock_route_to           |
      | test_wrong_out_of_scope | Only non-empty Roman numerals I,V,X are in scope       | asserts empty string converts to zero                     | rejects empty input before parsing                    | FAIL           | TEST_WRONG           | red-team                |
      | implementation_wrong    | Empty input must return InvalidInput                   | asserts empty input returns InvalidInput                  | returns Ok(0) for empty input                         | FAIL           | IMPLEMENTATION_WRONG | green-team              |
      | spec_incomplete         | Parser must handle subtractive notation                | asserts IC is rejected as invalid subtractive notation    | accepts any smaller numeral before larger numeral     | FAIL           | SPEC_INCOMPLETE      | spec_update_and_restart |
      | inconclusive_packet     | Parser should handle invalid input                     | asserts whitespace-only input is rejected                 | trims input before parsing                            | FAIL           | INCONCLUSIVE         | user                    |
