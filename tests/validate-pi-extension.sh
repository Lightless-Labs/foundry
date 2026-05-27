#!/usr/bin/env bash
# validate-pi-extension.sh — structural checks for Foundry's Pi extension package.
set -euo pipefail

PASS_COUNT=0
FAIL_COUNT=0

pass() {
  echo "$1: PASS"
  PASS_COUNT=$((PASS_COUNT + 1))
}

fail() {
  echo "$1: FAIL — $2"
  FAIL_COUNT=$((FAIL_COUNT + 1))
}

require_file() {
  local name="$1" path="$2"
  if [ -f "$path" ]; then pass "$name"; else fail "$name" "missing file $path"; fi
}

require_literal() {
  local name="$1" path="$2" literal="$3"
  if grep -Fq -- "$literal" "$path"; then
    pass "$name"
  else
    fail "$name" "missing literal in $path: $literal"
  fi
}

require_no_literal() {
  local name="$1" path="$2" literal="$3"
  if grep -Fq -- "$literal" "$path"; then
    fail "$name" "forbidden literal in $path: $literal"
  else
    pass "$name"
  fi
}

require_json_expr() {
  local name="$1" expr="$2"
  if python3 - "$expr" <<'PY'
import json, sys
expr = sys.argv[1]
with open('package.json', encoding='utf-8') as f:
    data = json.load(f)
if not eval(expr, {}, {'data': data}):
    raise SystemExit(1)
PY
  then
    pass "$name"
  else
    fail "$name" "package.json expression failed: $expr"
  fi
}

require_file "package-json" "package.json"
require_file "pi-foundry-team-extension" "extensions/pi-foundry-team/index.ts"

require_json_expr "pi-package-keyword" "'pi-package' in data.get('keywords', [])"
require_json_expr "pi-extension-manifest" "'./extensions/pi-foundry-team' in data.get('pi', {}).get('extensions', [])"
require_json_expr "pi-skills-manifest" "'./skills' in data.get('pi', {}).get('skills', [])"
require_json_expr "pi-peer-coding-agent" "'@earendil-works/pi-coding-agent' in data.get('peerDependencies', {})"

EXT="extensions/pi-foundry-team/index.ts"
SKILL="plugins/foundry/skills/foundry-adversarial/SKILL.md"

require_literal "tool-registered" "$EXT" 'name: "foundry_team"'
require_literal "spawn-json-mode" "$EXT" '"--mode", "json"'
require_literal "spawn-no-session" "$EXT" '"--no-session"'
require_literal "spawn-no-extensions" "$EXT" '"--no-extensions"'
require_literal "spawn-no-context-default" "$EXT" '"--no-context-files"'
require_literal "envelope-schema-validation" "$EXT" 'foundry.prompt-envelope.v1'
require_literal "outcome-label-sample-guard" "$EXT" 'Withheld sample duplicates allowed PASS/FAIL outcome label'
require_literal "arbiter-scope-guard" "$EXT" 'Arbiter envelope redactions must include single_test_scope'
require_literal "arbiter-overbroad-guard" "$EXT" 'Arbiter visible_context is over-broad'
require_literal "exact-envelope-prompt" "$EXT" 'args.push(prompt)'
require_literal "provider-model-lane-id" "$EXT" 'return `${provider}/${model}`'
require_literal "agent-discovery" "$EXT" 'plugins", "foundry", "agents"'
require_literal "parallel-bounds" "$EXT" 'MAX_PARALLEL_DISPATCHES = 8'
require_literal "official-subagent-pattern-note" "$SKILL" 'examples/extensions/subagent/'
require_literal "pi-foundry-team-skill-guidance" "$SKILL" 'foundry_team'

for skill in research brainstorm nlspec adversarial forge; do
  ADAPTER="skills/foundry-$skill/SKILL.md"
  CANONICAL="../../plugins/foundry/skills/foundry-$skill/SKILL.md"
  require_file "pi-skill-adapter-foundry-$skill" "$ADAPTER"
  require_literal "pi-skill-adapter-name-foundry-$skill" "$ADAPTER" "name: foundry-$skill"
  require_no_literal "pi-skill-adapter-no-colon-name-foundry-$skill" "$ADAPTER" "name: foundry:$skill"
  require_literal "pi-skill-adapter-canonical-foundry-$skill" "$ADAPTER" "$CANONICAL"
done

require_literal "pi-adversarial-foundry-team-guidance" "skills/foundry-adversarial/SKILL.md" 'foundry_team'
require_literal "pi-forge-foundry-team-guidance" "skills/foundry-forge/SKILL.md" 'foundry_team'

TOTAL_COUNT=$((PASS_COUNT + FAIL_COUNT))
printf '\nTOTAL: %d passed, %d failed out of %d Pi extension checks\n' "$PASS_COUNT" "$FAIL_COUNT" "$TOTAL_COUNT"

if [ "$FAIL_COUNT" -ne 0 ]; then
  exit 1
fi
