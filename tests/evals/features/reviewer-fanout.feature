Feature: Foundry reviewer fan-out workflow evals
  Phase 3 final review must dispatch the mandatory reviewers plus conditional language,
  Bazel, UniFFI, and reliability reviewers while preserving the red/green information barrier.
  These eval cases generate PromptEnvelope artifacts for every selected reviewer and validate
  that test-facing reviewers do not receive implementation material and implementation-facing
  reviewers do not receive red tests, raw failures, or NLSpec Done criteria.

  Background:
    Given PromptEnvelope schema "foundry.prompt-envelope.v1"
    And Phase 3 review always dispatches green-team-reviewer, red-team-test-reviewer, barrier-integrity-auditor, correctness-reviewer, and testing-reviewer
    And Phase 3 review dispatches exactly one language reviewer for the detected language
    And Bazel, UniFFi, and reliability reviewers are conditional on BUILD files, UDL files, and I/O behavior
    And implementation-facing reviewers never receive red test code, raw failures, or NLSpec Done
    And test-facing reviewers never receive implementation code or implementation paths

  Scenario Outline: Phase 3 reviewer fan-out is complete and barrier-safe
    Given a reviewer fan-out eval case "<case_id>"
    And the detected language is "<language>"
    And BUILD files are present "<has_build>"
    And UDL files are present "<has_udl>"
    And implementation touches I/O "<touches_io>"
    When the orchestrator creates Phase 3 reviewer PromptEnvelopes
    Then the selected reviewers should be "<expected_reviewers>"
    And every envelope should validate against barrier rules
    And reviewer prompts should preserve their territory boundaries

    Examples:
      | case_id               | language   | has_build | has_udl | touches_io | expected_reviewers                                                                                                                                                  |
      | rust_core             | rust       | false     | false   | false      | green-team-reviewer;red-team-test-reviewer;barrier-integrity-auditor;rust-reviewer;correctness-reviewer;testing-reviewer                                            |
      | swift_uniffi_bazel_io | swift      | true      | true    | true       | green-team-reviewer;red-team-test-reviewer;barrier-integrity-auditor;swift-reviewer;bazel-reviewer;uniffi-bridge-reviewer;correctness-reviewer;testing-reviewer;reliability-reviewer |
      | typescript_bazel      | typescript | true      | false   | false      | green-team-reviewer;red-team-test-reviewer;barrier-integrity-auditor;typescript-reviewer;bazel-reviewer;correctness-reviewer;testing-reviewer                       |
