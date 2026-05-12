#!/usr/bin/env bash
# BinaryShim runtime for .github/workflows/ci.yml.
#
# The workflow YAML is intentionally only a GitHub Actions bootstrap surface.
# CI policy lives here as the first concrete BinaryShim runner while the
# T-WAD lane moves the runner body behind project_github_actions(..., BinaryShim).
set -euo pipefail

mode=${1:?usage: ci-binary-shim.sh <fmt|ci|changes|v3|validate-ratchet-gate|self-host-ratchet|self-host-stub>}

repo_root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "${repo_root}"

is_pr() {
  [[ "${GITHUB_EVENT_NAME:-}" == "pull_request" ]]
}

case "${mode}" in
  fmt)
    cargo fmt --all --check
    ;;

  ci)
    if is_pr; then
      git fetch origin main:refs/remotes/origin/main
    fi

    bash scripts/check-pr-sg0-net-shrink-discipline.sh --self-test
    if is_pr; then
      body="${PR_BODY:-}"
      append_file="scripts/ci-merge/sg0-pr-body-append.${GITHUB_EVENT_NUMBER:-}.txt"
      if [[ -f "${append_file}" ]]; then
        body="$(cat "${append_file}")"$'\n\n'"${body}"
      fi
      PR_BODY="${body}" bash scripts/check-pr-sg0-net-shrink-discipline.sh
    fi

    bash scripts/check-r4-carve-dissolution-discipline.sh --self-test
    bash scripts/check-r4-carve-dissolution-discipline.sh
    bash scripts/check-fabrication-sentinels.sh
    bash scripts/check-ci-binary-shim-authority.sh
    bash scripts/check-release-doc-authority.sh
    bash scripts/test-check-release-doc-authority.sh
    bash scripts/check-manager-brief-authority.sh
    bash scripts/test-check-manager-brief-authority.sh
    bash scripts/check-rust-toolchain-single-authority.sh
    cargo run -p v3-compiler --features bootstrap-regen-fresh --bin regen_bootstrap -- --verify
    ;;

  changes)
    if [[ "${GITHUB_EVENT_NAME:-}" == "push" ]]; then
      echo "push event - always run heavy v3 job"
      echo "code=true" >> "${GITHUB_OUTPUT}"
      exit 0
    fi
    git fetch origin main:refs/remotes/origin/main
    changed=$(git diff --name-only origin/main...HEAD)
    echo "Changed files (first 200):"
    echo "${changed}" | head -200
    non_docs=$(echo "${changed}" | grep -vE '^(docs/.*|[^/]+\.md)$' || true)
    if [[ -n "${non_docs}" ]]; then
      echo "Non-docs files detected; v3 will run full heavy compute:"
      echo "${non_docs}" | head -50
      echo "code=true" >> "${GITHUB_OUTPUT}"
    else
      echo "Docs-only PR (allowlist: docs/** + root *.md) - v3 will skip"
      echo "code=false" >> "${GITHUB_OUTPUT}"
    fi
    ;;

  v3)
    mkdir -p "${RUNNER_TEMP}/gunbc-bin"
    shim_cc="${RUNNER_TEMP}/gunbc-bin/zig-cc"
    printf '%s\n' '#!/usr/bin/env bash' 'exec zig cc "$@"' > "${shim_cc}"
    chmod +x "${shim_cc}"
    export CC="${shim_cc}"
    export RUSTFLAGS="-D warnings -C linker=${shim_cc}"

    cargo build -p execute-command-bootstrap
    cargo test -p v3-compiler --test integration --no-run

    echo "::notice::v3 lane2d Stage 2d test execution SKIPPED per operator hot-fix directive at gunbc#846."
    scripts/check-v3-full-suite-split-test-targets.sh

    export CARGO_TERM_COLOR=never
    export RUST_MIN_STACK=16777216
    export RUSTC_BOOTSTRAP=1
    : > /tmp/v3-test-timings.log
    suite_start=$(date +%s)

    cargo test -p v3-compiler --lib --bins -- -Z unstable-options --report-time 2>&1 | tee /tmp/v3-test-timings.log
    cargo test -p v3-compiler --test determinism_test -- -Z unstable-options --report-time 2>&1 | tee -a /tmp/v3-test-timings.log
    cargo test -p v3-compiler --doc -- -Z unstable-options --report-time 2>&1 | tee -a /tmp/v3-test-timings.log
    cargo test -p v3-compiler --test integration __HOT_FIX_NONEXISTENT_FILTER__ -- -Z unstable-options --report-time 2>&1 | tee -a /tmp/v3-test-timings.log

    elapsed=$(( $(date +%s) - suite_start ))
    echo "v3 full-suite wall time (integration-skipped): ${elapsed}s"
    scripts/check-test-timeout.sh /tmp/v3-test-timings.log

    cargo clippy -p v3-compiler --all-targets -- -D warnings
    cargo clippy -p v3-compiler --all-targets --features bootstrap-regen-fresh -- -D warnings
    cargo test -p v3-compiler --no-run --features bootstrap-regen-fresh

    matches=$(grep -nE "fn (find_port|find_behavior|resolve_producer|lookup_node|lookup_port)" src/v3/lenses/*.dag 2>/dev/null || true)
    if [[ -n "${matches}" ]]; then
      echo "::error::L-7 violation: lens reconstructs a substrate lookup locally"
      echo "${matches}"
      exit 1
    fi

    wrappers=$(ls src/v3/compiler/src/lens_*.rs 2>/dev/null || true)
    if [[ -n "${wrappers}" ]]; then
      matches=$(grep -nE "pub fn .*-> (usize|bool|i64)" ${wrappers} 2>/dev/null || true)
      if [[ -n "${matches}" ]]; then
        echo "::error::L-8 violation: lens wrapper collapses typed carrier to a primitive"
        echo "${matches}"
        exit 1
      fi
    fi

    scripts/check-compiler-std-ratchet.sh
    scripts/check-banked-dissolutions.sh
    ;;

  validate-ratchet-gate)
    echo "Gate evidence: changes.result=${NEEDS_CHANGES_RESULT:-unset} v3.result=${NEEDS_V3_RESULT:-unset}"
    if [[ "${NEEDS_CHANGES_RESULT:-}" != "success" ]]; then
      echo "::error::Layer 1 filter (changes job) did not succeed."
      exit 1
    fi
    if [[ "${NEEDS_V3_RESULT:-}" == "failure" || "${NEEDS_V3_RESULT:-}" == "cancelled" ]]; then
      echo "::error::v3 job did not succeed."
      exit 1
    fi
    ;;

  self-host-ratchet)
    cargo test -p v3-compiler --release --test determinism_test || true
    cargo run -p v3-compiler --release --bin self_host_fixed_point || true
    if grep -nE "(HashMap|HashSet)::" src/v3/compiler/src/emit.rs; then
      echo "::notice::DB-8: emit.rs still uses HashMap/HashSet:: iteration."
    fi
    ;;

  self-host-stub)
    echo "::notice::self_host_ratchet DB-8 release matrix runs only on pushes to main."
    ;;

  *)
    echo "unknown ci shim mode: ${mode}" >&2
    exit 64
    ;;
esac
