# Live Dispatch Results

Run: `pi-live-kimi-minimax-divergence-restart-smoke`

## Summary

| Recipient | Phase | Planned model | Actual model | Result |
|---|---|---|---|---|
| red-team | phase1 | `minimax/MiniMax-M3` | `minimax/MiniMax-M3` | Completed; produced NLSpec-derived test plan and controlled gap probe |
| green-team | phase2 | `kimi-coding/kimi-for-coding` | `kimi-coding/kimi-for-coding` | Completed; produced implementation plan; semantic label leakage risk noted |
| foundry:review:divergence-evaluator | phase2b r1 | `minimax/MiniMax-M3` | `minimax/MiniMax-M3` | Completed; `findings[0].outcome = NOT_VALUABLE` because prompt explicitly excluded transliteration |
| foundry:review:divergence-evaluator | phase2b r2 | `minimax/MiniMax-M3` | `minimax/MiniMax-M3` | Completed; `findings[0].outcome = VALUABLE` |

## Route Evidence

The accepted restart route is based on `divergence-evaluator-r2-output.json`:

```json
{
  "reviewer": "divergence-evaluator",
  "findings": [
    {
      "outcome": "VALUABLE",
      "rationale": "The NLSpec scopes normalization to ASCII and has no rule for non-ASCII input such as accented Latin letters. A strict implementation that maps non-ASCII characters to separators is faithful to the written How section, while the stable failing case expects readable ASCII transliteration for real content titles. This is a missing-behavior gap, not an implementation bug.",
      "gap_description": "NLSpec is silent on how slugify handles non-ASCII characters in the input, particularly accented Latin letters common in content titles. The spec scopes normalization to ASCII letters/digits/whitespace/punctuation and lists only ASCII-focused DoD cases, but it never specifies whether non-ASCII letters should be (a) transliterated to ASCII equivalents (e.g., 'Crème brûlée' → 'creme-brulee'), (b) dropped, (c) treated as separators, or (d) cause a fallback to 'untitled'. The Definition of Done must be extended to cover Unicode/extended-Latin input and to state the chosen transliteration (or drop) policy explicitly, so that a reader of the NLSpec alone can derive the expected output for titles like 'Crème brûlée', 'naïve approach', or 'São Paulo guide'."
    }
  ]
}
```

The orchestrator route is therefore `spec_update_and_restart`; see `spec-update-and-restart.json`, `phase1-restart-package.json`, and `behavioral-smoke.toon`.

## Post-Restart Resume

| Recipient | Phase | Planned model | Actual model | Result |
|---|---|---|---|---|
| red-team | post_restart_phase1 | `minimax/MiniMax-M3` | `minimax/MiniMax-M3` | Completed; returned opaque-ID test update plan |
| green-team | post_restart_phase2 | `kimi-coding/kimi-for-coding` | `kimi-coding/kimi-for-coding` | Completed; plan referenced older fuller-smoke path, so r2 requested self-contained artifact |
| green-team | post_restart_phase2 r2 | `kimi-coding/kimi-for-coding` | `kimi-coding/kimi-for-coding` | Completed; returned `resumed/green/src/lib.rs` implementation artifact |

Post-restart convergence is recorded in `resumed/convergence-record.json`; `cd resumed/green && cargo test --quiet` passed `4/4` policy tests.

## Barrier Observation

The initial green envelope was mechanically valid (How section plus PASS/FAIL labels only), but the label `slugify_unicode_transliteration: FAIL` was enough for green to infer transliteration. Treat green's pre-restart output as a provider/barrier observation, not as independent restart evidence. The post-restart green envelopes use opaque `T-###` labels, and r2 explicitly redacts `post_restart_red_output`.
