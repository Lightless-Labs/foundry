---
title: "Plugin architecture for Claude Code marketplace workflow skills"
module: foundry-plugin
date: 2026-04-04
problem_type: best_practice
component: tooling
severity: medium
applies_when:
  - "Building a Claude Code plugin for the marketplace"
  - "Packaging multi-agent workflow as distributable skill"
  - "Creating agent-based review systems with quality gates"
tags:
  - plugin-architecture
  - claude-code
  - marketplace
  - agent-validation
---

# Plugin Architecture for Claude Code Marketplace Workflow Skills

## Directory Structure

- `marketplace.json` at root
- `plugin.json` per plugin
- `SKILL.md` per skill
- Agents in `agents/<category>/`

## Validation

Validate via `claude plugin validate`. Agent quality gate: `validate-agents.sh` with 207 checks across 23 agents (structural, attribution, coverage, territory boundaries).

## Why This Matters

Without a validation script, agents drift — refactoring removes sections, new agents omit fields. The validation script acts as a regression gate that catches structural decay before it reaches users. At 23 agents, manual review is no longer feasible; mechanical validation is the only way to maintain consistency.
