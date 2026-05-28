Feature: Foundry green follow-up barrier evals
  Green follow-up prompts may carry only NLSpec How plus test_name: PASS/FAIL labels.
  These eval cases generate PromptEnvelope artifacts and validate that red tests, raw failures,
  assertions, and NLSpec Done criteria remain withheld.

  Background:
    Given PromptEnvelope schema "foundry.prompt-envelope.v1"
    And green follow-up receives only NLSpec How
    And green follow-up receives only PASS/FAIL outcome labels
    And green follow-up never receives red test code, assertions, raw failures, or NLSpec Done

  Scenario Outline: Green follow-up preserves the information barrier
    Given a green follow-up eval case "<case_id>"
    And the NLSpec How section is "<nlspec_how>"
    And the visible test outcomes are "<test_results>"
    When the orchestrator creates a green follow-up PromptEnvelope
    Then the envelope should validate against barrier rules
    And withheld samples should remain absent from the prompt

    Examples:
      | case_id                 | nlspec_how                                                  | test_results                                          | red_test_sample                                      | raw_failure_sample                                | nlspec_done_sample                                      |
      | single_failure_label     | Parse Roman numerals by validating input then summing symbols | roman_empty_input: FAIL                              | assert_eq!(parse(""), Err(InvalidInput))          | expected Err(InvalidInput), got Ok(0)             | Done: empty input returns InvalidInput                  |
      | mixed_pass_fail_labels   | Accept canonical numerals and reject malformed subtractives   | roman_valid_viii: PASS; roman_invalid_ic: FAIL       | Scenario: reject IC as invalid subtractive notation | assertion failed: left Invalid, right Ok(99)      | Done: invalid subtractive notation must be rejected     |
