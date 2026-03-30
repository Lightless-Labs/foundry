---
date: 2026-03-12
topic: foundry-dark-factory
---

# Foundry Dark Factory

## What We're Building
Foundry is a reusable software dark factory that coordinates agent work across repositories. In v1, Foundry is the canonical system of record for execution state, policy, evidence, and audit history. Linear is a window into the system for humans, and Buildkite is the current queue and scheduler for work dispatched onto `tart` macOS VMs running on Mac minis.

The first end-to-end workflow is `spec -> code -> PR`. Foundry accepts work, creates a versioned repo plan artifact in `docs/plans/...`, attaches that plan back to the corresponding issue, runs implementation and review stages, and produces a pull request with supporting evidence. Externally, v1 exposes a simple, fixed lifecycle. Internally, it uses a graph-shaped runtime so later workflows like review-factory and test-factory can reuse the same substrate without changing the operator model.

Clean-room subteams are the default behavior for implementation work. A planner/spec agent, implementer, reviewer, and optional red/green/refactor agents collaborate through bounded interfaces rather than a shared full context. This is meant to improve quality, reduce reward hacking, and preserve a clearer audit trail of who concluded what and why.

## Why This Approach
Foundry should optimize for quality and reliability before throughput. That rules out both a loose “let every repo invent its own workflow engine” model and a narrow one-off automation demo. The chosen shape is an opinionated core: one canonical lifecycle, one evidence model, and one policy surface across repositories, with repository-owned rules for product-specific behavior.

This direction is consistent with the strongest signals in the reference material. OpenAI's harness-engineering guidance emphasizes repo-owned contracts, durable plans, and feedback loops. Symphony reinforces the need for explicit orchestration boundaries and isolated task execution. StrongDM's factory material highlights graph runtimes, resumability, checkpointing, and pyramid summaries for managing context. Bassim El Eddath's maturity framing supports designing toward higher-order orchestration rather than single-agent task execution. Foundry diverges from the strongest autonomy claims by making human review a first-class safety primitive instead of an edge case.

The result is a platform that can be trusted before it is maximally aggressive. Standardized telemetry and evidence are also the prerequisite for v2 self-improvement, where the system experiments on itself and learns from failures, delays, review rejections, bugs, and operator feedback.

## Key Decisions
- Foundry is the canonical system of record. Linear is a projection and control surface into Foundry, not the source of truth.
- Buildkite is an execution backend for polling, queuing, and scheduling work, not the product boundary.
- V1 is a reusable platform for multiple repositories, not an internal-only workflow for this repository.
- V1 exposes a fixed external lifecycle: intake, plan, implement, review, and PR. Internally, Foundry uses a graph runtime.
- The first factory workflow is `spec -> code -> PR`; code review factory comes next, followed by a test factory.
- Every implementation task uses clean-room subteams by default. Planner/spec, implementer, and reviewer roles are standard; red/green/refactor roles are available within the same model.
- Repository-specific behavior lives in the repository. Foundry defines cross-repo execution, safety, observability, policy, and integration primitives, while also owning its own repo-specific behavior in the same way any managed repository does.
- Foundry must generate a versioned `docs/plans/...` artifact before coding and attach that artifact back to the corresponding issue for usability and traceability.
- Any agent at any stage may require human review of the plan, the implementation review, or both. Once set for a task or stage, that review requirement is irreversible for the remainder of that run.
- Agent-triggered human review is a review gate, not a fatal error. Execution pauses at the gate in the same way as any other required review.
- Random human sampling is a separate audit mechanism and should exist even when no agent has requested human review.
- V1 should expose a thin Foundry dashboard for run state, evidence, irreversible gates, and audits while continuing to integrate with Linear, Buildkite, and GitHub.
- V2 should add self-improvement: exhaustive observability and feedback loops that allow the system to evaluate, monitor, and improve itself over both short-term and long-term metrics.

## Resolved Questions
- Canonical authority: Foundry is authoritative; external systems are views and integrations.
- Workflow style: simple lifecycle outside, graph runtime inside.
- Human oversight: irreversible review gates plus random audit sampling.
- Product surface: thin Foundry dashboard in v1 rather than no UI or a full Foundry-first application.
- Priorities: quality and reliability in v1; self-improvement in v2.

## Open Questions
None at the product-definition level. Remaining choices belong in implementation planning.

## Next Steps
Move to planning to define the core lifecycle, execution graph model, evidence model, review-gate semantics, observability requirements, and the contract between Foundry and repo-owned workflow definitions.
