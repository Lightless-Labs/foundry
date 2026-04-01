#!/usr/bin/env bash
# validate-agents.sh — Red team validation script for Foundry review agents
# Checks all 23 agents against structural, attribution, coverage, and territory specs.
# Compatible with Bash 3 (macOS default).
set -euo pipefail

AGENTS_ROOT="$(cd "$(dirname "$0")/../plugins/foundry/agents" && pwd)"

PASS_COUNT=0
FAIL_COUNT=0
TOTAL_COUNT=0

pass() {
  local agent="$1" check="$2"
  echo "$agent $check: PASS"
  PASS_COUNT=$((PASS_COUNT + 1))
  TOTAL_COUNT=$((TOTAL_COUNT + 1))
}

fail() {
  local agent="$1" check="$2" reason="$3"
  echo "$agent $check: FAIL — $reason"
  FAIL_COUNT=$((FAIL_COUNT + 1))
  TOTAL_COUNT=$((TOTAL_COUNT + 1))
}

# ─── Resolve agent name to file path ────────────────────────────────────────
resolve_agent() {
  local name="$1"
  local found
  found=$(find "$AGENTS_ROOT" -name "${name}.md" -type f 2>/dev/null | head -1)
  echo "$found"
}

# ─── Enumerate all agent files ──────────────────────────────────────────────
ALL_AGENT_FILES=()
ALL_AGENT_NAMES=()
while IFS= read -r f; do
  ALL_AGENT_FILES+=("$f")
  ALL_AGENT_NAMES+=("$(basename "$f" .md)")
done < <(find "$AGENTS_ROOT" -name '*.md' -type f | sort)

EXPECTED_COUNT=23
ACTUAL_COUNT="${#ALL_AGENT_FILES[@]}"
if [ "$ACTUAL_COUNT" -ne "$EXPECTED_COUNT" ]; then
  echo "WARNING: Expected $EXPECTED_COUNT agents, found $ACTUAL_COUNT"
  echo "  Found: ${ALL_AGENT_NAMES[*]}"
  echo ""
fi

# ═══════════════════════════════════════════════════════════════════════════════
# 1–7: STRUCTURAL CHECKS (all agents)
# ═══════════════════════════════════════════════════════════════════════════════
for i in "${!ALL_AGENT_FILES[@]}"; do
  name="${ALL_AGENT_NAMES[$i]}"
  file="${ALL_AGENT_FILES[$i]}"

  # --- Check 1: YAML frontmatter with required fields ---
  if ! head -5 "$file" | grep -q '^---'; then
    fail "$name" "yaml-frontmatter" "No YAML frontmatter delimiter found"
  else
    missing_fields=""
    for field in name description model tools color; do
      if ! grep -q "^${field}:" "$file"; then
        missing_fields="$missing_fields $field"
      fi
    done
    if [ -n "$missing_fields" ]; then
      fail "$name" "yaml-frontmatter" "Missing required fields:$missing_fields"
    else
      pass "$name" "yaml-frontmatter"
    fi
  fi

  # --- Check 2: "What you're hunting for" section ---
  if grep -q "## What you're hunting for" "$file"; then
    pass "$name" "hunting-section"
  else
    fail "$name" "hunting-section" "Missing '## What you're hunting for' section"
  fi

  # --- Check 3: Confidence calibration with three tiers ---
  if grep -q "## Confidence calibration" "$file"; then
    has_high=false
    has_moderate=false
    has_low=false
    grep -qE '0\.80\+|0\.90\+' "$file" && has_high=true
    grep -qE '0\.60.*0\.79|0\.70.*0\.89' "$file" && has_moderate=true
    grep -qE 'below 0\.60|below 0\.70' "$file" && has_low=true
    if $has_high && $has_moderate && $has_low; then
      pass "$name" "confidence-calibration"
    else
      missing=""
      $has_high  || missing="$missing high"
      $has_moderate || missing="$missing moderate"
      $has_low   || missing="$missing low"
      fail "$name" "confidence-calibration" "Missing confidence tiers:$missing"
    fi
  else
    fail "$name" "confidence-calibration" "Missing '## Confidence calibration' section"
  fi

  # --- Check 4: "What you don't flag" section ---
  if grep -q "## What you don't flag" "$file"; then
    pass "$name" "dont-flag-section"
  else
    fail "$name" "dont-flag-section" "Missing '## What you don't flag' section"
  fi

  # --- Check 5: Output format with JSON schema fields ---
  if grep -q "## Output format" "$file"; then
    missing_json=""
    for jfield in reviewer findings residual_risks testing_gaps; do
      if ! grep -q "\"$jfield\"" "$file"; then
        missing_json="$missing_json $jfield"
      fi
    done
    if [ -n "$missing_json" ]; then
      fail "$name" "output-format" "Missing JSON fields:$missing_json"
    else
      pass "$name" "output-format"
    fi
  else
    fail "$name" "output-format" "Missing '## Output format' section"
  fi

  # --- Check 6: model field is "inherit" ---
  if grep -q '^model: inherit' "$file"; then
    pass "$name" "model-inherit"
  else
    actual=$(grep '^model:' "$file" | head -1 || true)
    fail "$name" "model-inherit" "model is not 'inherit', got: ${actual:-(not found)}"
  fi

  # --- Check 7: tools field includes Read, Grep, Glob, Bash ---
  tools_line=$(grep '^tools:' "$file" || true)
  if [ -z "$tools_line" ]; then
    fail "$name" "tools-field" "No tools field found"
  else
    missing_tools=""
    for tool in Read Grep Glob Bash; do
      if ! echo "$tools_line" | grep -q "$tool"; then
        missing_tools="$missing_tools $tool"
      fi
    done
    if [ -n "$missing_tools" ]; then
      fail "$name" "tools-field" "Missing tools:$missing_tools"
    else
      pass "$name" "tools-field"
    fi
  fi
done

echo ""
echo "═══ ATTRIBUTION CHECKS ═══"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# 8: Attribution checks (adopted agents must contain attribution comment)
# ═══════════════════════════════════════════════════════════════════════════════
ADOPTED_AGENTS=(
  correctness-reviewer
  testing-reviewer
  reliability-reviewer
  learnings-researcher
  security-sentinel
  api-contract-reviewer
  architecture-strategist
  code-simplicity-reviewer
  maintainability-reviewer
  data-migrations-reviewer
  feasibility-reviewer
  adversarial-document-reviewer
)

for name in "${ADOPTED_AGENTS[@]}"; do
  file=$(resolve_agent "$name")
  if [ -z "$file" ]; then
    fail "$name" "attribution" "Agent file not found"
    continue
  fi
  if grep -q "Adopted from Compound Engineering" "$file"; then
    pass "$name" "attribution"
  else
    fail "$name" "attribution" "Missing 'Adopted from Compound Engineering' attribution comment"
  fi
done

echo ""
echo "═══ LANGUAGE-SPECIFIC COVERAGE CHECKS ═══"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# 9–14: Language-specific coverage checks
# ═══════════════════════════════════════════════════════════════════════════════

check_terms() {
  local name="$1" file="$2"
  shift 2
  local missing=""
  for term in "$@"; do
    # Handle OR terms (term1|term2)
    if echo "$term" | grep -q '|'; then
      # Check if any alternative matches
      local found_alt=false
      local saved_ifs="$IFS"
      IFS='|'
      for alt in $term; do
        IFS="$saved_ifs"
        if grep -qi "$alt" "$file"; then
          found_alt=true
          break
        fi
      done
      IFS="$saved_ifs"
      if ! $found_alt; then
        missing="$missing '$term'"
      fi
    else
      if ! grep -qi "$term" "$file"; then
        missing="$missing '$term'"
      fi
    fi
  done
  if [ -n "$missing" ]; then
    fail "$name" "coverage" "Missing terms:$missing"
  else
    pass "$name" "coverage"
  fi
}

# 9. rust-reviewer
check_terms "rust-reviewer" "$(resolve_agent rust-reviewer)" \
  "clone" "unsafe" "anyhow" "thiserror" "Mutex" ".await" "Send" "Sync" "Cargo.toml"

# 10. typescript-reviewer
check_terms "typescript-reviewer" "$(resolve_agent typescript-reviewer)" \
  "any" "unknown" "nullable" "narrowing" "type safety"

# 11. swift-reviewer
check_terms "swift-reviewer" "$(resolve_agent swift-reviewer)" \
  "@State" "@ObservedObject" "@StateObject" "Sendable" "@MainActor" "reference cycle" "@Observable"

# 12. bazel-reviewer
check_terms "bazel-reviewer" "$(resolve_agent bazel-reviewer)" \
  "visibility" "hermetic" "genrule" "cache" "rules_rust|proc_macro" "test size|timeout"

# 13. cucumber-reviewer
check_terms "cucumber-reviewer" "$(resolve_agent cucumber-reviewer)" \
  "imperative" "declarative" "Background" "Given" "When" "Then" \
  "scenario independence|scenario interdependence|independently executable" \
  "harness|cucumber-rs" "step definition"

# 14. uniffi-bridge-reviewer
check_terms "uniffi-bridge-reviewer" "$(resolve_agent uniffi-bridge-reviewer)" \
  "UDL" "Sendable" "callback" "lifecycle" "blocking" "main thread"

echo ""
echo "═══ ADVERSARIAL PROCESS AGENT COVERAGE CHECKS ═══"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# 15–19: Adversarial process agent coverage checks
# ═══════════════════════════════════════════════════════════════════════════════

# 15. spec-completeness-reviewer
check_terms "spec-completeness-reviewer" "$(resolve_agent spec-completeness-reviewer)" \
  "testable" "edge case" "scope" "ambiguous" "red|green|adversarial"

# 16. nlspec-fidelity-reviewer
check_terms "nlspec-fidelity-reviewer" "$(resolve_agent nlspec-fidelity-reviewer)" \
  "Definition of Done|DoD" "body" "coverage" "fidelity" "pseudocode" "1:1|mirror"

# 17. red-team-test-reviewer
check_terms "red-team-test-reviewer" "$(resolve_agent red-team-test-reviewer)" \
  "DoD|Definition of Done" "scenario" "trivially satisfiable" "assertion" "Gherkin"

# 18. green-team-reviewer
check_terms "green-team-reviewer" "$(resolve_agent green-team-reviewer)" \
  "information barrier|barrier" "How section" "hardcoded" "test code"

# 19. barrier-integrity-auditor
check_terms "barrier-integrity-auditor" "$(resolve_agent barrier-integrity-auditor)" \
  "prompt" "workspace" "PASS/FAIL" "assertion" "stack trace" "filter"

echo ""
echo "═══ TERRITORY BOUNDARY CHECKS ═══"
echo ""

# ═══════════════════════════════════════════════════════════════════════════════
# 20: Territory boundary — "What you don't flag" references at least one other agent
# ═══════════════════════════════════════════════════════════════════════════════

for i in "${!ALL_AGENT_FILES[@]}"; do
  name="${ALL_AGENT_NAMES[$i]}"
  file="${ALL_AGENT_FILES[$i]}"

  # Extract the "What you don't flag" section (from that heading to next ## or EOF)
  dont_flag_section=$(sed -n "/^## What you don.t flag/,/^## /p" "$file" 2>/dev/null || true)
  if [ -z "$dont_flag_section" ]; then
    # Try to end of file
    dont_flag_section=$(sed -n "/^## What you don.t flag/,\$p" "$file" 2>/dev/null || true)
  fi

  if [ -z "$dont_flag_section" ]; then
    fail "$name" "territory-boundary" "Could not extract 'What you don't flag' section"
    continue
  fi

  # Check if any other agent name is referenced in this section
  # We check both exact hyphenated names and space-separated variants
  found_reference=false
  for j in "${!ALL_AGENT_NAMES[@]}"; do
    other_name="${ALL_AGENT_NAMES[$j]}"
    [ "$other_name" = "$name" ] && continue

    # Exact match (e.g., "correctness-reviewer")
    if echo "$dont_flag_section" | grep -qi "$other_name"; then
      found_reference=true
      break
    fi

    # Loose match: convert hyphens to spaces (e.g., "correctness reviewer")
    loose_name=$(echo "$other_name" | tr '-' ' ')
    if echo "$dont_flag_section" | grep -qi "$loose_name"; then
      found_reference=true
      break
    fi
  done

  if $found_reference; then
    pass "$name" "territory-boundary"
  else
    fail "$name" "territory-boundary" "'What you don't flag' does not reference any other agent by name"
  fi
done

# ═══════════════════════════════════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════════════════════════════════
echo ""
echo "════════════════════════════════════════════════════════"
echo "TOTAL: $PASS_COUNT passed, $FAIL_COUNT failed out of $TOTAL_COUNT checks"
echo "════════════════════════════════════════════════════════"

# Exit with number of failures (capped at 125 for valid exit code)
if [ "$FAIL_COUNT" -gt 125 ]; then
  exit 125
else
  exit "$FAIL_COUNT"
fi
