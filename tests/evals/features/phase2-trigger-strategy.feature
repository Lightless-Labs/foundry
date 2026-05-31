Feature: Phase 2b trigger strategy evals
  The Phase 2b divergence trigger should stay simple enough to audit while
  escalating stable failing tests earlier when green has made distinct attempts
  that do not move the outcome.

  Background:
    Given TestFailureTracker is pipeline-run-scoped
    And the default trigger strategy is adaptive_with_fixed_floor
    And the fixed fallback threshold remains N=3
    And adaptive early trigger requires an unchanged test and at least two distinct implementation hashes

  Scenario Outline: Phase 2b trigger decision is deterministic
    Given a tracker case "<case_id>"
    And the same test has failed "<consecutive_fails>" consecutive times
    And the test content hash status is "<test_hash_status>"
    And the green implementation hashes are "<implementation_hashes>"
    When the orchestrator evaluates the trigger strategy "<strategy>" with threshold "<threshold>"
    Then the decision should be "<expected_decision>"
    And the trigger reason should be "<expected_reason>"

    Examples:
      | case_id                          | consecutive_fails | test_hash_status | implementation_hashes | strategy                  | threshold | expected_decision | expected_reason          |
      | fixed_threshold_third_fail        | 3                 | unchanged        | impl_a                 | adaptive_with_fixed_floor | 3         | trigger           | fixed_threshold          |
      | adaptive_two_distinct_attempts    | 2                 | unchanged        | impl_a,impl_b          | adaptive_with_fixed_floor | 3         | trigger           | adaptive_impl_changed    |
      | no_early_when_impl_unchanged      | 2                 | unchanged        | impl_a,impl_a          | adaptive_with_fixed_floor | 3         | continue_green    | waiting_for_more_signal  |
      | reset_when_test_hash_changes      | 1                 | changed          | impl_b                 | adaptive_with_fixed_floor | 3         | continue_green    | test_changed_reset       |
