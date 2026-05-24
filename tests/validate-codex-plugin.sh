#!/usr/bin/env bash
# validate-codex-plugin.sh — structural checks for Foundry's Codex CLI plugin package.
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
  local name="$1" path="$2" expr="$3"
  if python3 - "$path" "$expr" <<'PY'
import json, sys
path, expr = sys.argv[1], sys.argv[2]
with open(path, encoding='utf-8') as f:
    data = json.load(f)
if not eval(expr, {}, {'data': data}):
    raise SystemExit(1)
PY
  then
    pass "$name"
  else
    fail "$name" "$path expression failed: $expr"
  fi
}

MANIFEST=".codex-plugin/plugin.json"
require_file "codex-manifest" "$MANIFEST"
require_file "codex-agent-card" "agents/openai.yaml"
require_file "codex-icon" "assets/foundry-codex.svg"
require_file "codex-command-adversarial" "commands/foundry-adversarial.md"
require_file "codex-command-forge" "commands/foundry-forge.md"

require_json_expr "codex-name" "$MANIFEST" "data.get('name') == 'foundry'"
require_json_expr "codex-skills-path" "$MANIFEST" "data.get('skills') == './skills/'"
require_json_expr "codex-interface-display" "$MANIFEST" "data.get('interface', {}).get('displayName') == 'Foundry'"
require_json_expr "codex-write-capability" "$MANIFEST" "'Write' in data.get('interface', {}).get('capabilities', [])"
require_json_expr "codex-icon-path" "$MANIFEST" "data.get('interface', {}).get('composerIcon') == './assets/foundry-codex.svg'"

for skill in research brainstorm nlspec adversarial forge; do
  ADAPTER="skills/foundry-$skill/SKILL.md"
  CANONICAL="../../plugins/foundry/skills/foundry-$skill/SKILL.md"
  require_file "codex-skill-adapter-foundry-$skill" "$ADAPTER"
  require_literal "codex-skill-name-foundry-$skill" "$ADAPTER" "name: foundry-$skill"
  require_no_literal "codex-skill-no-colon-name-foundry-$skill" "$ADAPTER" "name: foundry:$skill"
  require_literal "codex-skill-canonical-foundry-$skill" "$ADAPTER" "$CANONICAL"
  require_literal "codex-skill-packaging-glue-foundry-$skill" "$ADAPTER" "packaging glue only"
done

require_literal "codex-adversarial-barrier-adapter" "skills/foundry-adversarial/SKILL.md" "green sees only NLSpec How plus PASS/FAIL outcome labels"
require_literal "codex-forge-barrier-adapter" "skills/foundry-forge/SKILL.md" "green sees only NLSpec How plus PASS/FAIL outcome labels"
require_literal "codex-agent-card-blocker-note" "agents/openai.yaml" "does not document a Claude-style dispatchable subagent API"
require_literal "codex-agent-card-canonical-agents" "agents/openai.yaml" "plugins/foundry/agents/**/*.md"
require_literal "codex-command-adversarial-canonical" "commands/foundry-adversarial.md" "plugins/foundry/skills/foundry-adversarial/SKILL.md"
require_literal "codex-command-adversarial-envelope" "commands/foundry-adversarial.md" "PromptEnvelope boundary"
require_literal "codex-command-forge-canonical" "commands/foundry-forge.md" "plugins/foundry/skills/foundry-forge/SKILL.md"
require_literal "codex-docs-support" "docs/pi-codex-support.md" "Codex CLI plugin support"
require_literal "codex-docs-blocker" "docs/pi-codex-support.md" "dispatchable subagent"

TOTAL_COUNT=$((PASS_COUNT + FAIL_COUNT))
printf '\nTOTAL: %d passed, %d failed out of %d Codex plugin checks\n' "$PASS_COUNT" "$FAIL_COUNT" "$TOTAL_COUNT"

if [ "$FAIL_COUNT" -ne 0 ]; then
  exit 1
fi
