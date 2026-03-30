---
title: feat: Build Foundry dark factory v1
type: feat
status: active
date: 2026-03-12
origin: docs/brainstorms/2026-03-12-foundry-dark-factory-brainstorm.md
---

# feat: Build Foundry dark factory v1

## Overview
Build Foundry v1 as a reusable control plane for agentic software execution across repositories. Foundry, not Linear or Buildkite, should be the canonical system for run state, policy, evidence, review gates, and audit history (see brainstorm: `docs/brainstorms/2026-03-12-foundry-dark-factory-brainstorm.md`).

V1 should deliver one trustworthy end-to-end workflow: `spec -> code -> PR`. The public operator model stays simple, while the runtime underneath uses a graph-based execution engine that can later support review-factory, test-factory, and self-improvement loops without changing the external mental model.

## Problem Statement
The project vision already defines Foundry as an agentic software dark factory, with Rust, Bazel, IaC, sub-agents, and plan-driven execution as core constraints ([`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L1), [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L9), [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L13), [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L17), [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L40), [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L51)). What does not exist yet is the control plane that makes those constraints operational and reliable across many repositories.

Without a canonical orchestration layer, task state fragments across Linear, CI, pull requests, agent transcripts, and human memory. That makes it hard to answer basic operational questions:

- Which task is currently running, paused, failed, sampled, or waiting on review?
- What evidence justifies the current state?
- Which repo-specific policy was in force for this run?
- Did a human review gate get added, and was it enforced immutably?
- Why did a run fail, retry, drift, or get rejected?

The first release should solve those operational questions before pursuing maximum autonomy. Quality and reliability are the primary goals in v1, with self-improvement deferred until the system can observe itself exhaustively (see brainstorm: `docs/brainstorms/2026-03-12-foundry-dark-factory-brainstorm.md`).

## Proposed Solution
Build an opinionated Foundry core with:

- A canonical Foundry domain model for work items, execution graphs, review gates, evidence, audits, and projections into external tools.
- A fixed external lifecycle for v1: `intake -> plan -> implement -> review -> PR`.
- An internal graph runtime that can express parallel clean-room agent teams, retries, pause/resume, and future workflow expansion.
- A repo-owned workflow contract, versioned with the codebase, that defines product-specific prompts, checks, policies, and handoff rules.
- A thin operator dashboard that exposes run status, gate state, evidence, and audit trails.
- Buildkite-backed execution on `tart` macOS VMs running on Mac minis.

The successful v1 run should end at a pull request and review handoff, not autonomous merge. This keeps the trust boundary narrow while still proving that Foundry can plan, implement, validate, and package work with strong evidence.

## Technical Approach

### Architecture
Foundry v1 should be organized into these major components:

1. `foundry-control-plane`
   - Owns canonical run state, dispatch decisions, retries, pause/resume, reconciliation, and policy enforcement.
   - Exposes the main API used by the dashboard and integration adapters.

2. `foundry-store`
   - Persists work items, runs, graph nodes, review gates, audit events, summaries, and artifact metadata.
   - Use a relational metadata store for durable canonical state.
   - Use object storage for large artifacts such as transcripts, logs, screenshots, videos, and diff bundles.

3. `foundry-repo-contract`
   - Loads repo-owned workflow definitions from a dedicated contract file such as `WORKFLOW.md`, plus any referenced policy files.
   - Separates Foundry runtime behavior from general repo guidance in `CLAUDE.md`.

4. `foundry-linear-projection`
   - Imports work into Foundry, receives webhook events, reconciles drift, and writes projections back into Linear.
   - Linear remains a window into Foundry, not the canonical execution ledger (see brainstorm: `docs/brainstorms/2026-03-12-foundry-dark-factory-brainstorm.md`).

5. `foundry-buildkite-dispatch`
   - Queues and schedules work on Buildkite.
   - Tracks remote job identity, worker heartbeats, completion status, and retryable infrastructure failures.

6. `foundry-worker-runtime`
   - Runs on Buildkite-backed `tart` VMs.
   - Prepares isolated workspaces, loads the repo contract, materializes plan and execution artifacts, and runs clean-room agent teams.

7. `foundry-dashboard`
   - Shows run status, graph progress, review gates, evidence summaries, audit selections, and operator actions.
   - Supports investigation and review, not rich workflow authoring in v1.

8. `foundry-observability`
   - Emits structured events, metrics, traces, and derived summaries.
   - Feeds v1 operations and v2 self-improvement loops.

### Core Domain Model
The canonical store should model at least:

- `WorkItem`
  - Foundry-owned task record with external identifiers for Linear, repository, branch, and PR.
- `Run`
  - One execution attempt for one workflow version and one repo revision.
- `GraphNode`
  - Planner, implementer, reviewer, validation, approval, projection, and audit nodes.
- `ReviewGate`
  - Immutable requirement for human review of plan, implementation review, or both.
- `AuditSelection`
  - Random or policy-driven sampling record.
- `EvidenceBundle`
  - Plan artifact, summaries, logs, test results, lint output, CI links, PR link, review findings, and media.
- `ProjectionState`
  - Last known external representation in Linear, Buildkite, and GitHub.
- `PolicyVersion`
  - The resolved repo workflow contract and Foundry runtime version used for the run.
- `FeedbackSignal`
  - Review rejection, bug, crash, timeout, rerun, human correction, or post-merge defect used later for v2.

### Clean-Room Team Model
Every implementation task should default to multiple bounded roles:

- `Planner/Spec Agent`
  - Reads the task and repo contract, generates the plan artifact, and may raise a plan-review gate.
- `Implementer`
  - Sees the task, plan, and bounded evidence needed to code; does not receive unrestricted reviewer context.
- `Reviewer`
  - Evaluates output independently and may require implementation review.
- `Optional Red/Green/Refactor Agents`
  - Enabled by repo policy or task class for stronger test and refactor discipline.

Boundaries matter more than role names. The worker runtime should explicitly encode which artifacts, summaries, and outputs each node is allowed to see.

### Evidence and Context Strategy
Foundry should treat evidence as a product primitive, not an afterthought:

- Write a versioned `docs/plans/...` artifact before implementation starts.
- Attach the plan artifact back to the task projection for operator visibility.
- Generate run summaries at multiple zoom levels so operators and downstream agents can inspect a run without replaying full transcripts.
- Preserve enough raw evidence to audit the system later, even if the UI relies on summaries by default.

Pyramid-style summaries should be a first-class pattern in the evidence model: the system should preserve short, medium, and expandable summaries for work items, runs, findings, and audits.

### Implementation Phases

#### Phase 1: Canonical Core
- Define the Foundry domain model in `src/domain/` and `src/control_plane/`.
- Stand up the durable store schema and artifact metadata model in `src/store/` and `docs/architecture/foundry-data-model.md`.
- Implement the lifecycle state machine, immutable review-gate semantics, and audit-selection model.
- Define the repo contract format in `WORKFLOW.md` and document it in `docs/contracts/workflow-contract.md`.
- Establish the event schema for runs, nodes, gates, failures, and feedback signals in `docs/architecture/foundry-events.md`.

Success criteria:
- A run, gate, and evidence bundle can be created, persisted, resumed, and inspected after restart.
- Review gates cannot be removed by any automated actor after being raised.
- A repo contract can be loaded deterministically and versioned with the run.

#### Phase 2: Intake, Dispatch, and the `spec -> code -> PR` Graph
- Build Foundry-native work ingestion and projection logic in `src/integrations/linear/`.
- Add webhook-first sync plus reconciliation polling for external drift.
- Build Buildkite dispatch and status tracking in `src/integrations/buildkite/`.
- Bootstrap `tart` worker execution in `src/worker/` with isolated per-run workspaces.
- Implement the first graph template in `src/workflows/spec_to_code_pr/`.
- Emit and attach `docs/plans/...` artifacts before coding begins.
- End successful runs in a PR-ready or PR-open state with evidence attached.

Success criteria:
- A Foundry work item can move from intake to PR handoff through a persisted graph run.
- Buildkite and worker failures produce deterministic retry or pause behavior.
- Foundry can project state to Linear without making Linear authoritative.
- Task cancellation or material task-state changes pause, stop, or requeue the active run deterministically without losing evidence.

#### Phase 3: Operator Surface, Audits, and Hardening
- Build the thin dashboard in `src/dashboard/`.
- Add audit sampling policy and operator workflows in `src/audits/`.
- Expose evidence summaries, graph progress, gate state, and failure reasons.
- Add operational runbooks in `docs/operations/foundry-v1.md`.
- Add IaC and deployment docs for service, storage, webhooks, and Buildkite/Tart worker setup in `infrastructure/` and `docs/deployment/foundry-v1.md`.
- Harden reconciliation, cancellation, duplicate-dispatch prevention, and restart recovery.

Success criteria:
- Operators can answer why a run is blocked, failed, retried, or sampled without opening raw logs first.
- Audit selections and immutable review gates are visible and enforceable end to end.
- Restart recovery and reconciliation work in staging without manual state repair.

## Alternative Approaches Considered

### Generic Workflow Engine First
Rejected for v1. It maximizes flexibility too early and weakens safety, observability, and consistency. The brainstorm explicitly chose a fixed external lifecycle with a graph runtime underneath instead (see brainstorm: `docs/brainstorms/2026-03-12-foundry-dark-factory-brainstorm.md`).

### Safety Shell Only
Rejected for v1. A thin audit wrapper around external tools would prove some operator workflows but would not establish Foundry as the canonical execution system or give a stable base for future review/test factories.

### Linear as Source of Truth
Rejected. The final brainstorm decision made Foundry canonical and redefined Linear as a window into the system. The plan should honor that rather than drift back to a tracker-centric model.

### Full Foundry-First UI
Rejected for v1. The chosen scope is a thin dashboard for status, evidence, gates, and audits, not a full authoring surface.

## System-Wide Impact

### Interaction Graph
At minimum, the v1 happy path should look like:

1. A task is created or updated in Foundry.
2. `foundry-linear-projection` mirrors task context into Linear and ingests Linear-originated changes.
3. `foundry-control-plane` evaluates eligibility and creates a `Run`.
4. `foundry-buildkite-dispatch` schedules a worker job.
5. `foundry-worker-runtime` provisions an isolated workspace on a `tart` VM.
6. The planner node writes `docs/plans/YYYY-MM-DD-NNN-...-plan.md`, records evidence, and may raise a plan gate.
7. If no blocking gate exists, implementer and reviewer nodes execute according to the graph.
8. Validation nodes collect tests, lint, CI, and repo-defined checks.
9. A PR projection is created or updated, and Foundry emits the handoff state.
10. Dashboard, audit, and projection services update their views from Foundry events.

### Error & Failure Propagation
- Webhook ingestion errors should never mutate canonical state partially. Invalid or unverifiable requests must be rejected before projection updates.
- Buildkite dispatch failures should leave the run in a retryable scheduling state, not a phantom running state.
- Worker crashes should transition the run to recoverable failure or timed-out status after heartbeat expiry.
- Repo contract parse failures should block the run before any agent work starts.
- Review-gate enforcement failures should fail closed: if Foundry cannot prove a gate is cleared by a human action, the run must remain blocked.

### State Lifecycle Risks
- Duplicate task claims could create competing runs for one work item.
- Projection drift could leave Linear or GitHub showing stale status while Foundry has moved on.
- Partial artifact writes could make summaries or review evidence misleading.
- Buildkite or VM termination could orphan external jobs unless heartbeats and reconciliation are explicit.
- External task changes could make an in-flight run invalid unless eligibility is continuously rechecked.
- Poor clean-room boundaries could collapse roles into a shared-context slop machine, defeating the point of multi-agent structure.

Mitigations:
- Canonical leasing per work item.
- Idempotent projection updates.
- Artifact manifests with completeness flags.
- Heartbeat-based lease expiry and reconciler sweeps.
- Explicit artifact visibility rules per graph node.

### API Surface Parity
The same canonical state should drive:

- Dashboard status pages
- Linear projections
- Buildkite job tracking
- PR annotations and links
- Audit and sampling workflows
- Future CLI or automation hooks

No surface should invent its own run state outside Foundry.

### Integration Test Scenarios
- Intake from Linear, run scheduling in Buildkite, worker crash, and successful resume without duplicate dispatch.
- Planner raises a mandatory human plan-review gate; no automated actor can clear it; a human clears it and execution resumes.
- Reviewer raises implementation review; PR is created with evidence bundle; dashboard and Linear both show the blocked state.
- Random audit selects a run with no prior gate; the sample appears in dashboard and projections before merge handoff.
- External drift occurs because a Buildkite job finishes but the callback is missed; reconciliation repairs the canonical state.
- A task is canceled or materially re-scoped while implementation is running; Foundry pauses or terminates the run, preserves evidence, and prevents stale completion from projecting as current truth.

## Acceptance Criteria

### Functional Requirements
- [ ] Foundry persists canonical work, run, graph, gate, evidence, audit, and projection state for multiple repositories.
- [ ] V1 supports the `spec -> code -> PR` lifecycle selected in the brainstorm (see brainstorm: `docs/brainstorms/2026-03-12-foundry-dark-factory-brainstorm.md`).
- [ ] Every implementation run begins with a versioned `docs/plans/...` artifact before coding starts.
- [ ] Foundry attaches or projects the plan artifact back to the issue surface used by operators.
- [ ] Clean-room subteams are the default execution model for implementation tasks.
- [ ] Any agent can raise a human review gate for the plan, the implementation review, or both.
- [ ] Once raised, a review gate is immutable unless cleared by an explicit human action recorded by Foundry.
- [ ] Random human audit sampling exists independently from agent-raised review gates.
- [ ] Buildkite is used as the queue and scheduler for worker execution on `tart` macOS VMs.
- [ ] Operators can inspect run state, evidence, gates, and audits through a thin Foundry dashboard.
- [ ] Foundry projects task state into Linear and execution outcomes into PR-related surfaces without surrendering canonical authority.
- [ ] Foundry pauses, cancels, or invalidates active runs when the work item becomes ineligible or materially changes.

### Non-Functional Requirements
- [ ] Restarting the control plane does not lose canonical run state or allow duplicate active claims.
- [ ] External integrations are idempotent and reconcile drift after missed webhook or callback events.
- [ ] All stage transitions, gate changes, audit selections, retries, and human actions emit structured events.
- [ ] Artifact and summary storage are durable enough to support later audits and v2 self-improvement analysis.
- [ ] Worker execution remains isolated per run and per repository.
- [ ] Security boundaries keep credentials, repo policy, and worker execution separable enough to reduce prompt-injection and overreach risk.

### Quality Gates
- [ ] Core state machine, gate semantics, and reconciliation logic are covered by automated tests.
- [ ] End-to-end staging validates intake, dispatch, planning, gating, PR handoff, and recovery paths.
- [ ] Documentation exists for repo contracts, operator workflows, deployment, and failure recovery.
- [ ] V1 does not enable autonomous merge as part of the happy path.

## Success Metrics
- Foundry records `100%` of run state transitions, gate actions, and audit selections in canonical storage.
- Foundry records `0` successful automated removals of irreversible review gates.
- Foundry records `0` duplicate simultaneous active claims for the same work item in staging and pilot use.
- `100%` of sampled runs expose a complete audit packet: plan, summaries, evidence manifest, and external links.
- Operators can answer the current state, blocking reason, and evidence location for any active run from the dashboard within one minute.
- Quality and reliability baselines are instrumented for future improvement:
  - PR rejection and rework rate
  - human-requested reruns
  - post-merge bugs attributable to Foundry runs
  - worker crash and resume rate
  - average time spent in each gate state

## Dependencies & Prerequisites
- Rust and Bazel project scaffolding aligned with repo conventions ([`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L9), [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L13)).
- IaC for control-plane deployment, storage, webhooks, and worker infrastructure ([`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L17)).
- Linear API and webhook credentials.
- Buildkite API credentials, pipelines, and worker registration.
- Mac mini hosts capable of running `tart` images.
- Git provider credentials for PR creation and metadata updates.
- A pilot repository contract defining workflow prompts, checks, and handoff rules.
- Institutional learnings baseline: none yet. `docs/solutions/` does not currently exist, so v1 should create the structures future work will depend on.

## Risk Analysis & Mitigation
- Reward hacking through shallow tests or tautological review:
  - Mitigate with clean-room roles, bounded context, explicit validation nodes, random sampling, and evidence review.
- Ambiguous task definitions:
  - Mitigate with mandatory plan generation and optional plan-review gates before implementation.
- Canonical/projection drift:
  - Mitigate with idempotent writes, reconciliation loops, and projection state tracking.
- Infrastructure flakiness on Buildkite or Mac minis:
  - Mitigate with heartbeats, retries, resumable runs, and explicit worker-health visibility.
- Context overload:
  - Mitigate with pyramid summaries and evidence manifests instead of raw transcript-first UX.
- Security and secrets exposure:
  - Mitigate with least-privilege tokens, worker isolation, distinct trust zones, and provider-signature verification for inbound events.

## Resource Requirements
- One control-plane service with a durable relational store.
- Object storage for artifact retention.
- Buildkite pipelines and agent capacity.
- Mac mini fleet with `tart` image lifecycle management.
- A minimal dashboard service or integrated web surface.
- Time to define repo contract schema, operator runbooks, and deployment/IaC.

## Future Considerations
- Add code review factory after the core workflow is trustworthy.
- Add test factory after review-factory patterns stabilize.
- Promote `FeedbackSignal` data into evaluation and self-improvement loops.
- Add richer policy experiments, routing, and adaptive sampling only after the observability model is complete.
- Consider multi-backend execution beyond Buildkite once the canonical control plane is stable.

## Documentation Plan
- Add `docs/contracts/workflow-contract.md` for the repo-owned Foundry contract.
- Add `docs/architecture/foundry-data-model.md` and `docs/architecture/foundry-events.md`.
- Add `docs/operations/foundry-v1.md` for operators.
- Add `docs/deployment/foundry-v1.md` for infrastructure and environment setup.
- Update [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md) as implementation lands so quick commands, tools, and environment setup stop being placeholders.

## Sources & References

### Origin
- **Brainstorm document:** [`docs/brainstorms/2026-03-12-foundry-dark-factory-brainstorm.md`](/Users/thomas/Projects/lightless-labs/foundry/docs/brainstorms/2026-03-12-foundry-dark-factory-brainstorm.md)
  - Carried-forward decisions: Foundry is canonical, v1 uses a fixed external lifecycle with an internal graph runtime, clean-room subteams are default, review gates are irreversible, random audit sampling exists, and v1 exposes a thin dashboard.

### Internal References
- Project direction and conventions: [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L1)
- Rust/Bazel/IaC stack: [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L9), [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L13), [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L17)
- Plan-driven and sub-agent process: [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L40)
- TDD and regular gates: [`CLAUDE.md`](/Users/thomas/Projects/lightless-labs/foundry/CLAUDE.md#L51)
- Institutional learnings: no relevant files found in `docs/solutions/`

### External References
- OpenAI Symphony README: <https://github.com/openai/symphony/>
- OpenAI Symphony specification: <https://raw.githubusercontent.com/openai/symphony/main/SPEC.md>
- OpenAI Harness Engineering: <https://openai.com/index/harness-engineering/>
- Linear GraphQL docs: <https://linear.app/developers/graphql>
- Linear webhook docs: <https://linear.app/developers/webhooks>
- Buildkite REST API overview: <https://buildkite.com/docs/apis/rest-api>
- Buildkite pipelines docs: <https://buildkite.com/docs/pipelines>
- Tart README: <https://github.com/cirruslabs/tart>
- StrongDM Pyramid Summaries: <https://factory.strongdm.ai/techniques/pyramid-summaries>
- StrongDM Attractor: <https://factory.strongdm.ai/products/attractor>
- Bassim El Eddath, “The 8 Levels of Agentic Engineering” (March 10, 2026; updated March 12, 2026): <https://www.bassimeledath.com/blog/levels-of-agentic-engineering>
- Jido: <https://jido.run>

### Related Work
- No local PRs, issues, or solution docs exist yet in this repository.
