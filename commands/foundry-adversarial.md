# /foundry-adversarial

Run the Foundry adversarial red/green implementation workflow from a reviewed NLSpec.

## Arguments

- `nlspec_path`: path to the reviewed NLSpec document.

## Workflow

1. Read the Agent Skills adapter at `skills/foundry-adversarial/SKILL.md`.
2. Follow its canonical-source pointer to `plugins/foundry/skills/foundry-adversarial/SKILL.md` and obey the canonical skill, not this command wrapper.
3. Preserve the PromptEnvelope boundary for every team/reviewer dispatch.
4. Red sees the full NLSpec/spec and never sees implementation code.
5. Green sees only NLSpec How plus `test_name: PASS/FAIL` labels and never sees red test code, assertions, raw failures, or NLSpec Done criteria.

## Guardrails

- Treat this command as Codex packaging glue only; do not fork workflow steps here.
- If Codex does not provide an isolated subagent primitive in the active runtime, do not simulate one by pasting hidden context into the main conversation. Use a harness/tool that consumes PromptEnvelope artifacts or stop and report the blocker.
