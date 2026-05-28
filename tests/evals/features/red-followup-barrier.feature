Feature: Foundry red follow-up barrier evals
  Red follow-up prompts may carry spec/NLSpec context, red-side artifacts, and redacted reviewer feedback.
  They must never expose implementation code, counterpart paths, or counterpart reasoning.

  Background:
    Given PromptEnvelope schema "foundry.prompt-envelope.v1"
    And red follow-up receives spec or NLSpec context needed to fix tests
    And red follow-up never receives implementation code or counterpart workspace paths
    And red follow-up never receives counterpart reasoning or raw implementation failure context

  Scenario Outline: Red follow-up preserves the information barrier
    Given a red follow-up eval case "<case_id>"
    And the route source is "<route_source>"
    And the visible spec context is "<spec_context>"
    And the visible red artifact is "<red_artifact>"
    When the orchestrator creates a red follow-up PromptEnvelope
    Then the envelope should validate against barrier rules
    And withheld implementation samples should remain absent from the prompt

    Examples:
      | case_id                    | route_source              | spec_context                                                   | red_artifact                                             | redacted_feedback                                      | implementation_sample                         | counterpart_path_sample | counterpart_reasoning_sample                         |
      | arbiter_test_wrong          | arbiter TEST_WRONG        | Empty input is outside the accepted Roman numeral input scope   | Scenario: empty string converts to zero                  | The disputed test contradicts the NLSpec scope        | fn parse_roman(input: &str) -> Result<u32>   | green/src/lib.rs         | implementation currently returns Ok for empty input  |
      | divergence_not_valuable     | divergence NOT_VALUABLE   | Only I,V,X,L,C,D,M symbols are in scope                        | Scenario: accept emoji numeral tokens                    | Emoji tokens are outside the NLSpec                  | match symbol { 'Ⅰ' => 1, _ => unreachable!() } | green/tests/debug.rs     | green-team thought emoji symbols were unsupported    |
      | reviewer_red_test_fix       | red-team-test-reviewer    | Parser must reject malformed subtractive notation              | Scenario: reject VX as malformed subtractive notation    | Tighten the scenario to cite the malformed rule       | let value = permissive_subtractive(input);    | green/src/parser.rs      | counterpart traced VX through a permissive branch    |
