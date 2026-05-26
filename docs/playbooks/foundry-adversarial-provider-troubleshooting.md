# Foundry Adversarial Provider Troubleshooting Playbook

This playbook contains provider/runtime-specific troubleshooting for `foundry:adversarial`. Keep vendor quirks here so the main skill stays focused on the workflow contract.

## General Convergence Issues

### Green is stuck on the same test

- Check if the test name gives enough information under the information barrier.
- If normal Phase 2b divergence routing does not resolve the stalemate, use `docs/playbooks/foundry-adversarial-arbiter-routing.md` for a scoped `arbiter-agent` invocation over exactly one disputed test.
- Do not reveal assertions, stack traces, raw outputs, `.feature` text, step definitions, NLSpec Done criteria, or arbiter raw context to green.

### Red tests are trivially satisfiable

- The red-team-test-reviewer should catch this.
- If it persists, the `too_easily_threshold` triggers red iteration. If a reviewer finds a concrete false-green/suspicious-pass signal, use `docs/playbooks/foundry-adversarial-arbiter-routing.md` before declaring success.

### Both teams iterate without convergence

- Pause after the configured `inner_loop_limit`.
- Use scoped arbiter routing for one disputed test at a time when evidence is narrow enough; otherwise ask the user to inspect both sides and arbitrate.

## OpenCode / Kimi Dispatch Issues

### Green team output file is missing or 0 bytes

- If using OpenCode, check that the dispatch command puts the message BEFORE any `-f` flags: `opencode run -m MODEL 'prompt' -f file.md`.
- If `-f` comes first, OpenCode consumes the message as a file path and exits `0` with no output.
- Run one invocation foreground first to validate the command shape before parallel dispatch.
- **Kimi K2.5 specifically:** add explicit tool discipline to the green prompt:
  ```text
  You MUST write files via a bash tool call. The write and edit tools are DISABLED. If bash is unavailable, emit the file body as plain text — the orchestrator will salvage it. After writing, print OK and stop.
  ```
- A softer instruction such as "use bash heredoc" is not sufficient; Kimi may prefer a structured write tool and fail silently on external_directory permission rejection.

### Green team output is garbled / tokenizer leakage

- Kimi K2.5 can emit control tokens such as `<|tool_call_end|>` into tool-call JSON, corrupting the envelope.
- OpenCode rejects the call; Kimi may then fall back to emitting the file body as a plain text part in the NDJSON output.
- Salvage procedure: scan the NDJSON log for `"type": "text"` parts containing a heredoc sentinel (`EOF` / `PYEOF`), extract the body, write it to the expected path, then syntax-check before use.
- Salvaged files must be syntax-checked before use because Kimi can truncate mid-function with no indicator.
