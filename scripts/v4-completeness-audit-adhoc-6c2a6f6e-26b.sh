#!/usr/bin/env bash
# Verify the node://adhoc-6c2a6f6e-26b v4 completeness audit against live files.
set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

audit="docs/planning/v4-completeness-audit-ctrl1425-deleted-docs-2026-06-02.md"

require_file() {
  local path="$1"
  if [[ ! -f "$path" ]]; then
    echo "error: required file missing: $path" >&2
    exit 1
  fi
}

require_absent() {
  local path="$1"
  if [[ -e "$path" ]]; then
    echo "error: historical deleted file was restored: $path" >&2
    exit 1
  fi
}

require_text() {
  local path="$1"
  local pattern="$2"
  if ! grep -F "$pattern" "$path" >/dev/null; then
    echo "error: expected text not found in $path: $pattern" >&2
    exit 1
  fi
}

require_rg() {
  local pattern="$1"
  local path="$2"
  if ! rg -n "$pattern" "$path" >/dev/null; then
    echo "error: expected pattern not found in $path: $pattern" >&2
    exit 1
  fi
}

require_file "$audit"

# #1425 is a key-pin receipt, not a semantic v4 close proof.
require_file "src/v3/compiler/src/self_host_receipt_p0.rs"
require_text "src/v3/compiler/src/self_host_receipt_p0.rs" "ALWAYS_EMITTED_TOP_LEVEL_KEYS"
require_text "$audit" "is not an implementation close receipt for v4"

# Do not restore the deleted historical audit authorities.
require_absent "docs/audit/v4-close-interrogation-validation-2026-05-30.md"
require_absent "docs/audit/v4-close-ledger-2026-05-30.md"
require_absent "docs/audit/r4-ctrl-phase15-subsystem-receipt-trail.md"

# Live omni-ingestion substrate and blockers.
require_file "src/v4/std/lexing.dag"
require_file "src/v4/std/grammar.dag"
require_file "src/v4/compiler/01_tokenize.dag"
require_file "src/v4/compiler/02_parse.dag"
require_rg "type LexPattern" "src/v4/std/lexing.dag"
require_rg "type GrammarExpr" "src/v4/std/grammar.dag"
require_rg "WellFormedFormalGrammar" "src/v4/std/grammar.dag"
require_rg "predicate-dissolution interim" "src/v4/compiler/01_tokenize.dag"
require_rg "predicate-dissolution interim" "src/v4/compiler/02_parse.dag"

# Live omni-emission substrate and blockers.
require_file "src/v4/std/target_model.dag"
require_file "src/v4/compiler/05_emit.dag"
require_file "src/v4/compiler/06_translate.dag"
require_rg "type TargetModel" "src/v4/std/target_model.dag"
require_rg "type TargetAtomRealization" "src/v4/std/target_model.dag"
require_rg "typed-concrete-syntax-token-variants" "src/v4/std/target_model.dag"
require_rg "source-atom-string-canonical-carrier" "src/v4/std/target_model.dag"

# R4-promoted substrate exists, while the audit keeps PROVEN reserved for executable receipts.
require_file "src/v4/std/verification.dag"
require_file "src/v4/extdeps/frameworks/react.dag"
require_file "src/v4/extdeps/coordination.dag"
require_text "$audit" "These are substrate-present, not PROVEN."
require_text "$audit" "TestClaimRun"

echo "v4 completeness audit receipt OK: $audit"
