# Pi Live Divergence/Arbiter Smoke

Controlled live smoke for Foundry dispute routing. It dispatches `divergence-evaluator` and `arbiter-agent` through Pi `foundry_team` from replayable PromptEnvelope artifacts, then validates the dispatch directory and behavioral smoke manifest.

Feature domain: a tiny `slugify` contract with intentionally ambiguous Unicode handling.

## Result

- `foundry:review:divergence-evaluator` ran through Pi `foundry_team` and returned `findings[0].outcome = VALUABLE` for the Unicode transliteration ambiguity.
- `foundry:review:arbiter-agent` ran through Pi `foundry_team` and returned `findings[0].outcome = TEST_WRONG` for a single emoji-preservation test outside the ASCII slug contract.
- Both child lanes reported `actualModel = openai-codex/gpt-5.5`.
- Replay validators pass:
  - `tests/validate-barrier-envelopes.sh runs/pi-live-divergence-arbiter-smoke/dispatch`
  - `tests/behavioral-smoke.sh runs/pi-live-divergence-arbiter-smoke`

Note: the divergence child also emitted a noncanonical helper `route_to = NLSPEC_REDERIVATION`; this run still routes from `findings[0].outcome` per the Foundry invariant.
