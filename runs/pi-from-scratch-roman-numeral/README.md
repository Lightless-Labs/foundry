# Pi From-Scratch Roman Numeral Adversarial Run

Run ID: `pi-from-scratch-roman-numeral`
Date: 2026-05-24

## Feature

Fresh Rust `roman-numeral` library generated from a minimal reviewed NLSpec.

- Project seed: `/tmp/foundry-pi-roman-numeral`
- NLSpec fixture preserved at `project/docs/nlspecs/roman-numeral.nlspec.md`
- Red tests preserved at `red/tests/roman_numeral.rs`
- Final green implementation preserved at `green/src/lib.rs`

## Result

Final red tests: `8/8` pass.

```bash
tests/validate-barrier-envelopes.sh runs/pi-from-scratch-roman-numeral/dispatch
tests/behavioral-smoke.sh runs/pi-from-scratch-roman-numeral
```

Both validators pass.

## Pi / Foundry Workflow Notes

The initial orchestration was invoked through Pi with:

- `--skill ./skills`
- `-e ./extensions/pi-foundry-team/index.ts`
- `foundry_team` enabled
- `/skill:foundry-adversarial` requested against the reviewed NLSpec

Pi generated fresh red tests and green implementation through PromptEnvelope-backed `foundry_team` dispatches.

The first long-running orchestration hit the external 900s shell timeout during Phase 3 reviewer fan-out after the Rust reviewer reported an overflow-risk finding. The fix was routed back through a new green-team PromptEnvelope (`dispatch/phase3/green-team-reviewer-fix.json`) and dispatched through `foundry_team`; the orchestrator did not edit implementation code directly.

A first manual continuation envelope incorrectly listed an allowed test outcome label as a withheld red-test sample. `foundry_team` rejected it before dispatch with a withheld-sample leak error. The envelope was corrected by using assertion/body snippets as withheld samples instead.

A follow-up `rust-reviewer-after-fix` dispatch approved with no findings.

## Preserved Artifacts

- `behavioral-smoke.toon` — final replay summary.
- `dispatch/` — PromptEnvelope artifacts for phase0, phase1, phase1b, phase2, phase2b, and phase3.
- `logs/` — selected cargo test outputs and PASS/FAIL outcome files.
- `project/` — minimal spec/NLSpec inputs.
- `red/` — fresh generated tests.
- `green/` — final generated implementation.

Generated Cargo `target/` directories and large Pi JSONL traces were intentionally removed from the committed fixture.
