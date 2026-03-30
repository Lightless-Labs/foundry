# Ligthless Labs Foundry

An agentic software dark factory 

## Technical Stack

### Build system

This project relies on Bazel for building and dependency management.

### Programming language

The programming language of choice is Rust.

### Infrastructure

Strictly adhere to Infrastructure-as-Code.
The infrastructure should allow for ephemeral feature-based environments spun up and torn down at will (eg for a PR's lifetime).

## Tools

// TBD

## Environment Setup

// TBC

## Quick Commands

// TBC

## Key Directories

- `todos/` - Pending work items
- `docs/plans/` - Implementation plans
- `docs/solutions/` - Documented learnings

## Process

Use sub-agents for each task. Parallelize tasks that can be parallelized.
When picking up a milestone from a roadmap or general plan, if the milestone does not have a dedicated plan, a dedicated plan should be created.
When a plan is deepened, the plan should be updated to reflect it (eg **Enhanced:** 2026-01-29 (via `/deepen-plan`) in the header).
When a plan is reviewed, the plan should be updated to reflect it (eg **Reviewed:** 2026-01-29 (via `/$SKILL / $COMMAND`) in the header).
When a plan is completed, the plan should be updated to reflect it (eg **Completed:** 2026-01-29 in the header).
When a gap is discovered during execution, the plan should be updated with an addendum (eg **Addendum:** 2026-02-07 — description of what was added and why).

## Conventions

Favour ast-grep over grep when researching and operating over code.
Commit early and eagerly. Favour atomic commits.
Use a TDD approach.
Run checks and gates (tests, linting,...) regularly to tighten your feedback loop.

### Git

Set `SSH_AUTH_SOCK` before any git operation requiring authentication (push, pull, fetch, clone):

```bash
export SSH_AUTH_SOCK=~/.ssh/agent.sock
```
