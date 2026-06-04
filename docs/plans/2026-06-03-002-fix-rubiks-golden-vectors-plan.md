---
title: Fix Rubik's cube convention mismatch with golden vectors
created: 2026-06-03
status: completed
completed: 2026-06-03
---

# Fix Rubik's Cube Golden Vectors Plan

## Goal

Turn the Rubik's cube worked example from a preserved convention-mismatch case (`31/46`) into a regression-improved example by anchoring the cube-state convention with authoritative golden vectors and aligning implementation/tests/docs to that convention.

## Scope

- Inspect the current red test failures and green implementation constants.
- Add or correct golden move vectors for Kociemba facelet order (`URFDLB`, 3x3 row-major faces).
- Update the NLSpec/README to document the convention anchors.
- Fix only the public example artifacts; no private engine changes.
- Preserve the learning that missing golden vectors caused the original deadlock.

## Non-goals

- No full rewrite into an optimal solver.
- No broad workflow/prompt changes unless the example exposes a concrete issue.
- No weakening of red/green provenance in docs; fixes are now example maintenance after the case study.

## Acceptance

- [x] Rubik's example tests improve beyond the existing `31/46` baseline, ideally to the documented target near `44/46`.
- [x] Golden vectors are present in the NLSpec/README or red tests.
- [x] `cargo test` in `examples/rubiks-solver` is run and the final pass count is recorded.
- [x] Handoff is updated with the new result and any residual failures.

## Validation Log

2026-06-03:

- `cd examples/rubiks-solver && cargo test --quiet` — 46/46 passed.
- `tests/validate-agents.sh` — 224/224 passed.

## Notes

The repair aligned the example to Kociemba's Python reference facelet convention. The important fixes were:

- replace the ambiguous/mistaken corner facelet mapping in the NLSpec with Kociemba's `FaceCube.cornerFacelet` mapping;
- add Kociemba-derived golden vectors for `R`, `U`, `R U R' U'`, and the hard 20-move scramble;
- update red-test simulator permutation tables and scramble constants to those vectors;
- update green move cycles/orientation deltas to the same pull-cycle convention;
- parse corner orientation using Kociemba's clockwise non-U/D color match rather than unordered color-set matching;
- enforce canonical ordering for commuting opposite-face moves in the solver output.
