---
name: foundry:nlspec
description: "Derive a Natural Language Specification (NLSpec) from a spec document, then review it for completeness and fidelity. Use when a spec exists and you need a buildable specification for adversarial implementation. Triggers on 'nlspec', 'derive nlspec', 'write the nlspec', 'make it buildable'."
argument-hint: "[path to spec document]"
---

# Foundry NLSpec

Derive a Natural Language Specification (NLSpec) from a spec document. The NLSpec follows the Why/What/How/Done structure and is directly consumable by coding agents. It is then reviewed against the source spec for completeness and fidelity.

## What is an NLSpec

An NLSpec is a human-readable specification intended to be directly usable by coding agents to implement and validate software behavior. A coding agent supplied with an NLSpec and no other context should be able to produce a conforming implementation.

The key structural property: the Definition of Done mirrors the body sections. This creates a closed audit loop where gaps in either direction are visible defects.

## Input

A path to a spec document (from `foundry:brainstorm`). If not provided, search `docs/specs/` for the most recent spec matching the topic.

## Workflow

### Phase 1: Analyze the Spec

Read the source spec thoroughly. Extract:
- All requirements (R1, R2, etc.)
- All behaviors and their triggers/inputs/outputs/errors
- All decisions and constraints
- All scope boundaries
- Success criteria

### Phase 2: Derive the NLSpec

Write the NLSpec to `docs/nlspecs/YYYY-MM-DD-<topic>.nlspec.md` following this structure:

```markdown
---
date: YYYY-MM-DD
topic: <topic>
source_spec: docs/specs/YYYY-MM-DD-<topic>-spec.md
status: draft
---

# <Topic> NLSpec

[Opening paragraph: what the thing is + who it is for. No jargon.]

## Table of Contents
[Linked TOC]

---

## 1. Why

### 1.1 Problem Statement
[Three-part: status quo, pain, solution — from the spec]

### 1.2 Design Principles
[Bold-lead paragraphs. Each must be checkable, not aspirational.]
**Principle name.** Explanation of constraint and consequence.

### 1.3 Layering and Scope
[What this spec covers / does NOT cover / boundary]

---

## 2. What

### 2.1 Data Model
[RECORD definitions for key types. Use NLSpec pseudocode conventions:]
- Keywords UPPER CASE (RECORD, ENUM, INTERFACE, FUNCTION, IF, RETURN)
- Variables/functions snake_case, types PascalCase, enum values UPPER_CASE
- Comments use -- (double dash)
- Assignment =, equality ==, type annotations : Type

### 2.2 Architecture
[Component boundaries, interfaces, data flow]

### 2.3 Vocabulary
[Define terms on first use. Never use synonyms for defined terms.]

---

## 3. How

### 3.x <Behavior Name>
[For each behavior from the spec:]

[Pseudocode with labeled steps:]
```
FUNCTION behavior_name(input: InputType) -> OutputType:
    -- Step 1: Validate input
    IF input.field == INVALID:
        RETURN Error("reason")

    -- Step 2: Process
    result = transform(input)

    -- Step 3: Produce output
    RETURN result
```

-- Behavior:
- On valid input, produces transformed output
- On invalid input, returns error with reason
- [Edge cases inline in pseudocode, not separate prose]

[Tables are normative — each row is a testable requirement]

---

## 4. Out of Scope

[For each excluded feature:]
- **Feature name.** What it is. Why excluded. How it could be added later (extension point).

---

## 5. Design Decision Rationale

[Bold-question format:]
**Why X instead of Y?** Answer explaining the trade-off. Names the rejected alternative.

---

## 6. Definition of Done

[Checkbox format. Subsections mirror body sections 1:1.]

### 6.1 Core (<mirrors section 2/3>)
- [ ] Requirement derived from spec R1
- [ ] Requirement derived from spec R2
- [ ] ...

### 6.2 <Behavior> (<mirrors section 3.x>)
- [ ] Trigger produces expected output
- [ ] Error case handled
- [ ] Edge case covered

### 6.x Integration Smoke Test
[Pseudocode integration test exercising the system end-to-end:]
```
FUNCTION integration_smoke_test():
    -- Setup
    system = create_system(config)

    -- Exercise core flow
    result = system.do_thing(valid_input)
    ASSERT result.status == SUCCESS

    -- Exercise error path
    error_result = system.do_thing(invalid_input)
    ASSERT error_result.is_error

    -- Verify end state
    ASSERT system.state == EXPECTED_STATE
```
```

### NLSpec Quality Rules

- **Requirement density:** Target ~1 testable requirement per 25 lines of spec
- **DoD mirrors body:** Every body section has a DoD subsection and vice versa. Both gaps are defects.
- **Tables are normative:** Every row in a table is a testable requirement with a Default column where applicable
- **No aspirational language:** "should consider" is not a spec. "MUST validate" is.
- **Terms defined on first use:** Never use synonyms for defined terms
- **Edge cases in pseudocode:** Not in separate prose sections
- **Out of scope includes extension points:** Turns exclusions into deferred features

### Phase 3: Review NLSpec Against Source Spec

This is the critical fidelity check. Spawn the nlspec-fidelity-reviewer:

```
Agent(
    subagent_type="foundry:review:nlspec-fidelity-reviewer",
    prompt="Review this NLSpec for completeness and fidelity against its source specification.

    Source spec: [path to spec]
    NLSpec: [path to nlspec]

    Read BOTH documents. Return findings as JSON matching the findings schema."
)
```

The nlspec-fidelity-reviewer checks coverage (every R in body AND DoD), fidelity (pseudocode matches spec behaviors), structure (DoD mirrors body 1:1), scope creep, and ambiguity. See the agent definition for the full hunting list.

For NLSpecs with 10+ DoD items or complex multi-component designs, also spawn in parallel:
- `foundry:document-review:adversarial-document-reviewer` — challenge premises and unstated assumptions in the NLSpec
- `foundry:review:spec-completeness-reviewer` — verify the source spec itself is still complete (the NLSpec derivation may have surfaced gaps)

Present findings to the user. Fix any FAIL items. Iterate until the reviewer passes.

Update the NLSpec frontmatter: `status: reviewed`

### Phase 4: Handoff

Present options:
1. **Run adversarial implementation** — `/foundry:adversarial` with this NLSpec
2. **Revise** — fix specific sections
3. **Done** — NLSpec stands alone
