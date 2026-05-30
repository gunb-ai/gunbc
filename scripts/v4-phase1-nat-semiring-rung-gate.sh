#!/usr/bin/env bash
# scripts/v4-phase1-nat-semiring-rung-gate.sh
#
# Phase 1 rung gate: drives rungs 0–2 acceptance predicates over the ratified Phase 1
# fixture phase1/nat_semiring on a single module path, not the corpus-wide src/v4 sweep.
# Reports the §2.4 verdict matrix:
#
#   fixture=phase1/nat_semiring
#     rung0: PASS|FAIL (dag rust python go)
#     rung1: PASS|FAIL (rust)
#     rung2: PASS|FAIL (rust python go)
#   blocking_receipt: <predicate id> | none
#
# Authority: docs/planning/v4-ladder-rung-specs-2026-05-30.md §2.1–§2.5 (rung gate shape +
# CI matrix wiring). Modeled CiCommand arm Phase1NatSemiringRungGateCommand lands in
# src/v4/workflow/ci.dag in the same PR as this transport (PR #3955); the script's role
# is the §2.5 "interim host script" layer until TestClaimRun verdicts evaluate
# phase1_nat_semiring_rung_0_to_2_roster (v4.test.claim.nat_semiring.rung_0_to_2_three_targets).
# Pattern: scripts/v4-m1-rust-emit-probe.sh (single-module scope, not full src/v4).
#
# Env:
#   V2_COMPILER                — v2-compiler binary (default: target/release/gunbc)
#   V4_PHASE1_NAT_SEMIRING_OUT — emit output dir (default: /tmp/v4-phase1-nat-semiring)
#   V4_PHASE1_NAT_SEMIRING_STRICT — if 1, exit non-zero on any rung failure
#   V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS — timeout per toolchain check (CI: 300)
#   V4_PHASE1_NAT_SEMIRING_PYTHON — python3 binary (default: python3)
#   V4_PHASE1_NAT_SEMIRING_GO     — go binary (default: go)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

fixture_module_path="src/v4/test/claim/algebra_laws/nat_semiring.dag"
fixture_id="phase1/nat_semiring"

bin="${V2_COMPILER:-target/release/gunbc}"
if [[ -n "${GITHUB_ACTIONS:-}" && -z "${V4_PHASE1_NAT_SEMIRING_OUT:-}" ]]; then
  out="${RUNNER_TEMP:-/tmp}/v4-phase1-nat-semiring"
else
  out="${V4_PHASE1_NAT_SEMIRING_OUT:-/tmp/v4-phase1-nat-semiring}"
fi
if [[ -x /opt/cargo/bin/cargo ]]; then
  cargo_bin="/opt/cargo/bin/cargo"
else
  cargo_bin="${CARGO_BIN:-cargo}"
fi
python_bin="${V4_PHASE1_NAT_SEMIRING_PYTHON:-python3}"
go_bin="${V4_PHASE1_NAT_SEMIRING_GO:-go}"
timeout_secs="${V4_PHASE1_NAT_SEMIRING_TIMEOUT_SECS:-300}"
strict="${V4_PHASE1_NAT_SEMIRING_STRICT:-0}"
summary="${out}.rung-gate-summary.txt"

if [[ ! -f "$fixture_module_path" ]]; then
  echo "error: fixture module not found at $fixture_module_path" >&2
  exit 1
fi

if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin (build v2-compiler --release first)" >&2
  if [[ "$strict" == "1" ]]; then
    exit 1
  fi
  echo "::notice title=phase1/nat_semiring rung gate::skipped — v2-compiler missing"
  exit 0
fi

rm -rf "$out"
mkdir -p "$out/rust" "$out/python" "$out/go" "$out/logs"

# Fixture-scoped entry isolation: v2-compiler treats the FIRST --source-root as the entry pool
# (every .dag in it becomes an entry); subsequent --source-root values are dep pools resolved
# via imports. Mirror the Lens-CI step pattern: copy ONLY the fixture module into entry_root
# at its canonical module path, then layer the rest of src/v4 as deps with the fixture file
# removed from the dep pool to avoid module-path collisions. This scopes the compile to the
# fixture module's transitive closure — the §2.5 "fixture-scoped emit + toolchain" contract.
entry_root="$out/entry"
deps_root="$out/deps"
fixture_relpath="${fixture_module_path#src/v4/}"   # test/claim/algebra_laws/nat_semiring.dag
mkdir -p "$entry_root/$(dirname "$fixture_relpath")"
cp "$fixture_module_path" "$entry_root/$fixture_relpath"
cp -R src/v4/. "$deps_root/"
rm -f "$deps_root/$fixture_relpath"

# Per-predicate verdict slots. PASS|FAIL; default FAIL until proven.
declare -A verdict=(
  [R0-dag-parse]=FAIL
  [R0-rust-parse]=FAIL
  [R0-python-parse]=FAIL
  [R0-go-parse]=FAIL
  [R1-rust-typecheck]=FAIL
  [R2-rust-compile]=FAIL
  [R2-python-compile]=FAIL
  [R2-go-compile]=FAIL
)
blocking_receipt="none"

note_blocking() {
  local pred="$1"
  if [[ "$blocking_receipt" == "none" ]]; then
    blocking_receipt="$pred"
  fi
}

run_step() {
  local log="$1"; shift
  if [[ "$timeout_secs" -gt 0 ]]; then
    timeout --preserve-status "$timeout_secs" "$@" 2>&1 | tee "$log"
    return "${PIPESTATUS[0]}"
  else
    "$@" 2>&1 | tee "$log"
    return "${PIPESTATUS[0]}"
  fi
}

# --- Rung 0 dag-parse: v4 ingest of the fixture module accepts. ---
parse_log="$out/logs/dag_parse.log"
set +e
run_step "$parse_log" "$bin" compile \
  --source-root "$entry_root" \
  --source-root "$deps_root" \
  --output-dir "$out/dag-parse" \
  --target dag
parse_status=$?
set -e
if [[ "$parse_status" -eq 0 ]]; then
  verdict[R0-dag-parse]=PASS
else
  note_blocking "phase1/nat_semiring/rung0/dag_parse_rejected"
fi

# --- Rust target: R0-rust-parse, R1-rust-typecheck, R2-rust-compile ---
# Per spec §2.1 R0-rust-parse: "Emitted Rust ... parses under rustc frontend (parse-only or
# full compile)." Per spec §2.3 R2-rust-compile: "Same as R1-rust-typecheck (rung 2 Rust ⊇
# rung 1)." So cargo check is the load-bearing receipt — it parses AND typechecks — and
# R0-rust-parse only flips on cargo-check exit-0 with artifact-presence proof, not on emit
# exit alone (INVARIANTS P3 fail-closed).
rust_emit_log="$out/logs/rust_emit.log"
set +e
run_step "$rust_emit_log" "$bin" compile \
  --source-root "$entry_root" \
  --source-root "$deps_root" \
  --output-dir "$out/rust" \
  --target rust
rust_emit_status=$?
set -e
rust_rs_count=0
if [[ -d "$out/rust" ]]; then
  rust_rs_count="$(find "$out/rust" -name '*.rs' 2>/dev/null | wc -l | tr -d ' ')"
fi
if [[ "$rust_emit_status" -ne 0 ]]; then
  note_blocking "phase1/nat_semiring/rung0/rust_emit_parse_rejected"
elif [[ ! -f "$out/rust/Cargo.toml" ]]; then
  echo "no Cargo.toml emitted under $out/rust — Rust rung predicates fail closed" >&2
  note_blocking "phase1/nat_semiring/rung0/rust_emit_parse_rejected"
elif [[ "${rust_rs_count:-0}" -lt 1 ]]; then
  echo "no .rs files emitted under $out/rust — Rust rung predicates fail closed" >&2
  note_blocking "phase1/nat_semiring/rung0/rust_emit_parse_rejected"
else
  rust_check_log="$out/logs/rust_check.log"
  set +e
  run_step "$rust_check_log" "$cargo_bin" check --jobs 4 --manifest-path "$out/rust/Cargo.toml"
  rust_check_status=$?
  set -e
  if [[ "$rust_check_status" -eq 0 ]]; then
    # cargo check completed: parse phase succeeded (= R0-rust-parse) and typecheck phase
    # succeeded (= R1-rust-typecheck). R2-rust-compile is defined as R1 per spec §2.3.
    verdict[R0-rust-parse]=PASS
    verdict[R1-rust-typecheck]=PASS
    verdict[R2-rust-compile]=PASS
  else
    # Treat any cargo-check failure as the R0 parse rejection unless a later predicate
    # (rung1/rung2) is the more specific blocker. Single-receipt model collapses parse
    # and typecheck reporting; structured rustc diagnostic categorization is future work.
    note_blocking "phase1/nat_semiring/rung1/rust_typecheck_failed"
  fi
fi

# --- Rung 0 python-parse + Rung 2 python-compile ---
py_emit_log="$out/logs/python_emit.log"
set +e
run_step "$py_emit_log" "$bin" compile \
  --source-root "$entry_root" \
  --source-root "$deps_root" \
  --output-dir "$out/python" \
  --target python
py_emit_status=$?
set -e
# R0-python-parse and R2-python-compile both flip on the same py_compile receipt: py_compile
# IS the Python parse check (spec §2.1 "python3 -m py_compile or equivalent") and the spec
# §2.3 R2-python-compile pass condition is the same toolchain invocation. Both predicates
# require fail-closed artifact presence: zero .py files cannot pass either.
py_file_count=0
if [[ -d "$out/python" ]]; then
  py_file_count="$(find "$out/python" -name '*.py' 2>/dev/null | wc -l | tr -d ' ')"
fi
if [[ "$py_emit_status" -ne 0 ]]; then
  note_blocking "phase1/nat_semiring/rung0/python_emit_parse_rejected"
elif [[ "${py_file_count:-0}" -lt 1 ]]; then
  echo "no .py files emitted under $out/python — Python rung predicates fail closed" >&2
  note_blocking "phase1/nat_semiring/rung0/python_emit_parse_rejected"
else
  py_check_log="$out/logs/python_check.log"
  set +e
  find "$out/python" -name '*.py' -print0 2>/dev/null \
    | xargs -0 "$python_bin" -m py_compile 2>&1 | tee "$py_check_log"
  py_check_status=${PIPESTATUS[1]}
  set -e
  if [[ "$py_check_status" -eq 0 ]]; then
    verdict[R0-python-parse]=PASS
    verdict[R2-python-compile]=PASS
  else
    note_blocking "phase1/nat_semiring/rung2/python_compile_failed"
  fi
fi

# --- Rung 0 go-parse + Rung 2 go-compile ---
go_emit_log="$out/logs/go_emit.log"
set +e
run_step "$go_emit_log" "$bin" compile \
  --source-root "$entry_root" \
  --source-root "$deps_root" \
  --output-dir "$out/go" \
  --target go
go_emit_status=$?
set -e
# R0-go-parse and R2-go-compile both flip on the same go-build receipt: `go build` parses
# AND compiles, satisfying both spec §2.1 R0-go-parse ("parse phase") and spec §2.3
# R2-go-compile ("builds without compile errors") in one invocation. Artifact-presence
# guarded — empty .go tree cannot pass either predicate (INVARIANTS P3).
go_file_count=0
if [[ -d "$out/go" ]]; then
  go_file_count="$(find "$out/go" -name '*.go' 2>/dev/null | wc -l | tr -d ' ')"
fi
if [[ "$go_emit_status" -ne 0 ]]; then
  note_blocking "phase1/nat_semiring/rung0/go_emit_parse_rejected"
elif [[ "${go_file_count:-0}" -lt 1 ]]; then
  echo "no .go files emitted under $out/go — Go rung predicates fail closed" >&2
  note_blocking "phase1/nat_semiring/rung0/go_emit_parse_rejected"
else
  go_check_log="$out/logs/go_check.log"
  set +e
  ( cd "$out/go" && run_step "$go_check_log" "$go_bin" build ./... )
  go_check_status=$?
  set -e
  if [[ "$go_check_status" -eq 0 ]]; then
    verdict[R0-go-parse]=PASS
    verdict[R2-go-compile]=PASS
  else
    note_blocking "phase1/nat_semiring/rung2/go_compile_failed"
  fi
fi

# --- Rung roll-up (AND of constituent predicates). ---
rung0_pass="PASS"
for p in R0-dag-parse R0-rust-parse R0-python-parse R0-go-parse; do
  if [[ "${verdict[$p]}" != "PASS" ]]; then rung0_pass="FAIL"; fi
done
rung1_pass="${verdict[R1-rust-typecheck]}"
rung2_pass="PASS"
for p in R2-rust-compile R2-python-compile R2-go-compile; do
  if [[ "${verdict[$p]}" != "PASS" ]]; then rung2_pass="FAIL"; fi
done

{
  echo "fixture=${fixture_id}"
  echo "  rung0: ${rung0_pass}  (dag=${verdict[R0-dag-parse]} rust=${verdict[R0-rust-parse]} python=${verdict[R0-python-parse]} go=${verdict[R0-go-parse]})"
  echo "  rung1: ${rung1_pass}  (rust=${verdict[R1-rust-typecheck]})"
  echo "  rung2: ${rung2_pass}  (rust=${verdict[R2-rust-compile]} python=${verdict[R2-python-compile]} go=${verdict[R2-go-compile]})"
  echo "blocking_receipt: ${blocking_receipt}"
  echo ""
  echo "logs: ${out}/logs/"
} | tee "$summary"

if [[ -n "${GITHUB_ACTIONS:-}" ]]; then
  body="$(head -10 "$summary")"
  escaped="${body//$'\n'/%0A}"
  echo "::notice title=phase1/nat_semiring rung gate::${escaped}"
fi

if [[ "$strict" == "1" ]]; then
  if [[ "$rung0_pass" != "PASS" || "$rung1_pass" != "PASS" || "$rung2_pass" != "PASS" ]]; then
    exit 1
  fi
fi

exit 0
