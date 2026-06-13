---
title: Wire adversarial UI controls into aggregate validation
created: 2026-06-13
status: completed
completed: 2026-06-13
todo: todos/adversarial-ui-investigation.md
---

# Wire Adversarial UI Controls into Aggregate Validation

## Goal

Make the new adversarial UI capture-surface and visual-control validators part of the repo's documented fast validation path, not only manually invoked one-off checks.

## Scope

- Add a fast aggregate public-plugin validation entrypoint.
- Include the new adversarial UI validators in that entrypoint.
- Exclude slow/live model lanes from the aggregate script.
- Document the aggregate command in repo guidance and package metadata.

## Non-goals

- Do not add hosted CI configuration unless the repo already has CI to extend.
- Do not run slow Pi live dispatch smoke in the aggregate validator.
- Do not add external dependencies.

## Acceptance

- [x] Aggregate validation script exists and runs existing fast validators plus UI capture/visual checks.
- [x] `npm run validate` invokes the aggregate validator.
- [x] `AGENTS.md`, `CLAUDE.md`, and `docs/HANDOFF.md` mention the aggregate validator.
- [x] Aggregate validator passes locally.
- [x] Changes are committed and pushed.

## Validation Log

2026-06-13:

- Started from the gap that `tests/validate-adversarial-ui-visual-controls.sh` could be forgotten if it was only manually invoked.
- Added `tests/validate-public-plugin.sh` as the fast aggregate validation entrypoint, excluding slow/live model lanes.
- Added `npm run validate` as a package-level wrapper around `tests/validate-public-plugin.sh`.
- Updated `AGENTS.md`, `CLAUDE.md`, and `docs/HANDOFF.md` so the aggregate validator is visible in normal repo guidance.
- `tests/validate-public-plugin.sh` — passed.
- `npm run validate` — passed.
