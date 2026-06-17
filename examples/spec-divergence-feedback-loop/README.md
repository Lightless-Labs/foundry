# Spec Divergence Feedback Loop Example

This example preserves the first adversarial red-team validation fixture for Foundry's spec-divergence feedback loop.

The feature asks Foundry to treat red/green divergence from an NLSpec as potential product signal instead of immediate error. If an isolated team surfaces behavior outside the current spec, an ephemeral divergence evaluator decides whether the divergence is valuable. Valuable divergences rerun NLSpec derivation with enriched input; non-valuable divergences route back to the responsible team.

## What this example shows

- A red-team shell test suite can validate workflow/documentation deliverables, not only product code.
- Divergence routing must be explicit in the public skill and agent prompts.
- The `divergence-evaluator` must return structured outcomes:
  - `VALUABLE`
  - `NOT_VALUABLE`
  - `INCONCLUSIVE`
- Valuable divergence restarts from NLSpec derivation instead of patching the existing NLSpec in place.
- The orchestrator must preserve provenance and route fixes through the right phase.

## Preserved artifacts

- `red/tests.sh` — Bash 3-compatible red-team validation script.
- `green/` — placeholder for implementation-side artifacts from the original adversarial example.
- `shared/` — placeholder for shared fixture inputs from the original adversarial example.

The canonical deliverables validated by this example now live in the main plugin surface:

- `plugins/foundry/agents/review/divergence-evaluator.md`
- `plugins/foundry/skills/foundry-adversarial/SKILL.md`
- `docs/playbooks/foundry-adversarial-divergence-routing.md`

## Validate current behavior

Use the current production gates from the repository root:

```bash
tests/validate-adversarial-modules.sh
tests/foundry-evals.sh --suite divergence-routing
tests/foundry-evals.sh --suite spec-update-restart
tests/foundry-evals.sh --suite phase-choreography
```

These validators cover the productionized divergence evaluator, Phase 1b/2b routing, `spec_update_and_restart`, tracker reset, revision-cap behavior, PromptEnvelope barrier checks, and phase choreography.

## Legacy red script

The original red-team script is preserved at:

```bash
bash examples/spec-divergence-feedback-loop/red/tests.sh
```

It is a historical fixture, not a current passing gate. Later modularization moved many anchors into playbooks and generic eval suites, so the script now reports expected drift against the current docs. Prefer the validators above for current regression coverage.

## Related docs

- `todos/spec-divergence-feedback-loop.md`
- `docs/research/2026-04-05-spec-divergence-feedback-loop-research.md`
- `docs/specs/2026-04-06-spec-divergence-feedback-loop-spec.md`
- `docs/nlspecs/2026-04-07-spec-divergence-feedback-loop.nlspec.md`
- `docs/playbooks/foundry-adversarial-divergence-routing.md`

## Status

Completed and merged in April 2026. Later generic workflow evals, PromptEnvelope barrier validators, provider-diverse restart smokes, and `tests/validate-adversarial-modules.sh` now cover the productionized divergence/restart behavior more thoroughly. This example remains as the original red-team fixture and a compact regression reference.
