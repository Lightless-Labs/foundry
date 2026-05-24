# /foundry-forge

Run the full Foundry pipeline: research, brainstorm, NLSpec, then adversarial red/green implementation.

## Arguments

- `feature_request`: the feature, problem, or product idea to forge.

## Workflow

1. Read the Agent Skills adapter at `skills/foundry-forge/SKILL.md`.
2. Follow its canonical-source pointer to `plugins/foundry/skills/foundry-forge/SKILL.md`.
3. Execute the canonical Foundry pipeline with gates between phases.
4. When the pipeline reaches adversarial implementation, preserve the PromptEnvelope red/green information barrier exactly.

## Guardrails

- Treat this command as Codex packaging glue only; do not fork workflow steps here.
- Red/green dispatch must remain replayable from PromptEnvelope artifacts.
- Green receives only NLSpec How plus PASS/FAIL outcome labels.
