#!/usr/bin/env bash
# scripts/v4-mvp1-e2e-gate.sh
#
# MVP-1 end-to-end receipt: add.dag → v2-compiler (--target rust) → emitted Rust
# → cargo build/run → assert add(2, 3) == 5.
#
# 🟡 scaffold — feature:mvp1-ci-e2e-receipt — INVARIANTS §P5 (Progress Is Dissolution)
# Roadmap: ROADMAP.md § Nine lanes / T-PB-B (`pb_rust_tests_outside_residual_zero`);
#   TASKS.md T-10 / Wave-3-B; design-v4-compiler-homomorphism.md §MVP (ground→project interim:
#   v2-bootstrap compile path until full src/v4 closure cargo-builds).
# Dissolve-on-arrival (delete this script, the ci.yml step, and scripts-owned mvp1_gate.rs)
#   when ANY of:
#   (a) `.dag` TestClaim + generated harness owns the same receipt without scripts append —
#       authority: `src/v4/test/claim/manual/mvp1_rust_add_translate.dag` (+ sibling MVP add
#       claims) and T-22 eval `TestClaimRun` when `ground→project` emits cargo-clean Rust;
#   (b) `src/v4/workflow/ci.dag` (or `dsl/gunbc/ci.dag`) models this gate as `CiGate` data
#       (T-24 workflow-as-data; no shell harness);
#   (c) full `src/v4` dep pool emits cargo-build-clean Rust for the add-shaped program and
#       v4 `project(inferred, rust_projection_plan)` is the sole compile→run authority.
# Exit condition: removal when (a) is green on main CI for 14 consecutive days, or (b) lands
#   with parity harness deleting (a)'s shell bridge.
#
# Fail-closed. Does NOT use --target dag (known broken on full graphs).
#
# Usage (repo root):
#   bash scripts/v4-mvp1-e2e-gate.sh
#
# Env:
#   V2_COMPILER   — path to v2-compiler binary (default: target/release/v2-compiler)
#   MVP1_OUT_DIR  — compile output directory (default: $RUNNER_TEMP/v4-mvp1-out-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT,
#                   falling back to /tmp/v4-mvp1-out-$$ outside GitHub Actions)

set -euo pipefail

root="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$root"

bin="${V2_COMPILER:-target/release/v2-compiler}"
if [[ ! -x "$bin" ]]; then
  echo "=== MVP-1: build v2-compiler (release) ==="
  cargo build -p v2-compiler --release
fi
if [[ ! -x "$bin" ]]; then
  echo "error: v2-compiler not found at $bin after build" >&2
  exit 1
fi

entry_root="fixtures/v4-mvp1/add"
if [[ ! -f "${entry_root}/add.dag" ]]; then
  echo "error: missing MVP-1 entry fixture ${entry_root}/add.dag" >&2
  exit 1
fi

run_suffix="${GITHUB_RUN_ID:-local}-${GITHUB_RUN_ATTEMPT:-$$}"
tmp_root="${RUNNER_TEMP:-/tmp}"
out="${MVP1_OUT_DIR:-${tmp_root}/v4-mvp1-out-${run_suffix}}"
log="${MVP1_LOG:-${tmp_root}/v4-mvp1-${run_suffix}.log}"
rm -rf "$out"
mkdir -p "$out"

echo "=== MVP-1: compile ${entry_root}/add.dag (--target rust) ==="
set +e
# Dep pool: dsl/std only (not full src/v4 — transitive v4.std emission does not yet cargo-build).
"$bin" compile \
  --source-root "$entry_root" \
  --source-root dsl/std \
  --output-dir "$out" \
  --target rust 2>&1 | tee "$log"
status=${PIPESTATUS[0]}
set -e

if [[ "$status" -ne 0 ]]; then
  echo "error: MVP-1 compile exited $status (log: $log)" >&2
  exit "$status"
fi

if ! grep -E '^compiled: [0-9]+ files emitted, 0 diagnostics$' "$log" >/dev/null; then
  echo "error: MVP-1 compile did not emit a clean compiled receipt" >&2
  exit 1
fi

mod_rs="${out}/src/v4_test_mvp1_add.rs"
if [[ ! -s "$mod_rs" ]]; then
  echo "error: expected emitted module at $mod_rs" >&2
  exit 1
fi

if ! grep -q 'fn add(' "$mod_rs"; then
  echo "error: emitted Rust missing fn add (see $mod_rs)" >&2
  exit 1
fi

# Signature pin (looser i32-in-file grep is insufficient); cargo-run assert is authoritative.
if ! grep -Eq 'fn add\([^)]*: i(32|64|isize)' "$mod_rs"; then
  echo "error: emitted add missing i32/i64/isize parameter types on fn add (see $mod_rs)" >&2
  exit 1
fi

cargo_toml="${out}/Cargo.toml"
if [[ ! -f "$cargo_toml" ]]; then
  echo "error: compile produced no Cargo.toml at $cargo_toml" >&2
  exit 1
fi

# [package] section only — avoid picking a [[bin]]/[[lib]] name= if emitter reorder tables.
# Intentionally brittle to emitter Cargo.toml layout: a shape change should fail this gate loudly.
crate_name="$(
  sed -n '/^\[package\]/,/^\[/p' "$cargo_toml" \
    | grep -E '^name = ' \
    | head -1 \
    | sed 's/^name = "\(.*\)"/\1/'
)"
if [[ -z "$crate_name" ]]; then
  echo "error: could not parse [package].name from $cargo_toml" >&2
  exit 1
fi

# Orchestration harness (scripts-owned): invoke emitted add from a bin target.
# Assumes v2 emit is lib-only today; fail-closed if compiler already emitted [[bin]].
if grep -qE '^\[\[bin\]\]' "$cargo_toml"; then
  echo "error: emitted Cargo.toml already has [[bin]]; MVP-1 gate expects lib-only emission" >&2
  exit 1
fi
mkdir -p "${out}/src/bin"
cat > "${out}/src/bin/mvp1_gate.rs" <<EOF
// MVP-1 CI harness — scripts-owned interim (INVARIANTS §P5).
// Dissolution: delete when (a) TestClaim/generated harness or (b) workflow/ci.dag owns
// compile→rust→run assert; see scripts/v4-mvp1-e2e-gate.sh header.
use ${crate_name}::v4_test_mvp1_add::add;

fn main() {
    let sum = add(2, 3);
    assert_eq!(sum, 5, "add(2, 3) must equal 5");
    println!("mvp1-ok: add(2, 3) = {}", sum);
}
EOF

printf '\n[[bin]]\nname = "mvp1_gate"\npath = "src/bin/mvp1_gate.rs"\n' >> "$cargo_toml"

echo "=== MVP-1: cargo build mvp1_gate ==="
(
  cd "$out"
  cargo build --bin mvp1_gate 2>&1
)

echo "=== MVP-1: cargo run mvp1_gate (assert add(2,3)==5) ==="
run_out="$(
  cd "$out"
  cargo run --quiet --bin mvp1_gate 2>&1
)"
echo "$run_out"

if ! grep -q 'mvp1-ok: add(2, 3) = 5' <<<"$run_out"; then
  echo "error: MVP-1 run did not print expected receipt (got above)" >&2
  exit 1
fi

echo "MVP-1 end-to-end gate OK (add.dag → rust → cargo run → add(2,3)==5)"
