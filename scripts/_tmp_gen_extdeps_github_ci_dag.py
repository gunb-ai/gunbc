#!/usr/bin/env python3
"""One-shot generator for dsl/extdeps/github/ci.dag — delete after use."""

from __future__ import annotations

import textwrap

OUT = "dsl/extdeps/github/ci.dag"


def esc(s: str) -> str:
    return (
        s.replace("\\", "\\\\")
        .replace('"', '\\"')
        .replace("\n", "\\n")
        .replace("\r", "")
    )


def opt_str(s: str | None) -> str:
    if s is None:
        return "none"
    return f'"{esc(s)}"'


def opt_int(n: int | None) -> str:
    if n is None:
        return "none"
    return str(n)


def map_lit(m: dict[str, str]) -> str:
    if not m:
        return "{}"
    inner = ", ".join(f'{k}: "{esc(v)}"' for k, v in m.items())
    return f"{{ {inner} }}"


def uses(
    *,
    uses_ref: str,
    with_m: dict[str, str],
    env: dict[str, str] | None = None,
    if_cond: str | None = None,
    cont_err: bool = False,
) -> str:
    env = env or {}
    return textwrap.dedent(
        f"""\
        UsesStep {{
          name: none,
          uses: {uses_ref},
          with: {map_lit(with_m)},
          env: {map_lit(env)},
          if_condition: {opt_str(if_cond)},
          continue_on_error: {str(cont_err).lower()},
          timeout_minutes: none,
        }}"""
    )


def run(
    *,
    cmd: str,
    env: dict[str, str] | None = None,
    if_cond: str | None = None,
    cont_err: bool = False,
) -> str:
    env = env or {}
    return textwrap.dedent(
        f"""\
        RunStep {{
          name: none,
          run: "{esc(cmd)}",
          shell: Bash,
          env: {map_lit(env)},
          working_directory: none,
          if_condition: {opt_str(if_cond)},
          continue_on_error: {str(cont_err).lower()},
          timeout_minutes: none,
        }}"""
    )


def cons_list(items: list[str]) -> str:
    if not items:
        return "Empty"
    rest = cons_list(items[1:])
    return f"Cons {{\n  head: {items[0]},\n  tail: {rest}\n}}"


checkout_v4 = "checkout_action"
setup_rust = "setup_rust_action"
cache_act = "cache_action"
setup_zig = 'ActionRef { owner: "goto-bus-stop", repo: "setup-zig", ref: "v2" }'
setup_py = 'ActionRef { owner: "actions", repo: "setup-python", ref: "v6" }'
setup_go = 'ActionRef { owner: "actions", repo: "setup-go", ref: "v6" }'

# --- Steps: fmt job ---
fmt_steps = [
    uses(
        uses_ref=checkout_v4,
        with_m={"fetch_depth": "1"},
    ),
    uses(
        uses_ref=setup_rust,
        with_m={"components": "rustfmt", "cache": "false", "rustflags": ""},
    ),
    run(cmd="cargo fmt --all --check"),
]

# --- ci job ---
ci_steps = [
    uses(
        uses_ref=checkout_v4,
        with_m={"fetch_depth": "0"},
    ),
    run(
        cmd="git fetch origin main:refs/remotes/origin/main",
        if_cond="github.event_name == 'pull_request'",
    ),
    run(cmd="bash scripts/check-pr-sg0-net-shrink-discipline.sh --self-test"),
    run(
        cmd=(
            'body="${PR_BODY}"\n'
            'append_file="scripts/ci-merge/sg0-pr-body-append.${{ github.event.pull_request.number }}.txt"\n'
            "if [ -f \"${append_file}\" ]; then\n"
            "  body=\"$(cat \"${append_file}\")\"$'\\n\\n'\"${body}\"\n"
            "fi\n"
            'PR_BODY="${body}" bash scripts/check-pr-sg0-net-shrink-discipline.sh'
        ),
        env={"PR_BODY": "${{ github.event.pull_request.body }}"},
        if_cond="github.event_name == 'pull_request'",
    ),
    run(cmd="bash scripts/check-r4-carve-dissolution-discipline.sh --self-test"),
    run(cmd="bash scripts/check-r4-carve-dissolution-discipline.sh"),
    run(cmd="bash scripts/check-fabrication-sentinels.sh"),
    run(cmd="bash scripts/check-release-doc-authority.sh"),
    run(cmd="bash scripts/test-check-release-doc-authority.sh"),
    run(
        cmd="bash scripts/check-manager-brief-authority.sh",
        env={"GH_TOKEN": "${{ secrets.GITHUB_TOKEN }}"},
    ),
    run(cmd="bash scripts/test-check-manager-brief-authority.sh"),
    run(cmd="bash scripts/check-rust-toolchain-single-authority.sh"),
    uses(
        uses_ref=setup_rust,
        with_m={"components": "rustfmt", "cache": "false", "rustflags": ""},
    ),
    uses(
        uses_ref=cache_act,
        with_m={
            "path": (
                "~/.cargo/registry/index/\n"
                "~/.cargo/registry/cache/\n"
                "~/.cargo/git/db/\n"
                "target/"
            ),
            "key": "cargo-ci-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('src/v3/compiler/src/**', 'src/v3/compiler/tests/**') }}",
            "restore-keys": (
                "cargo-ci-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-\n"
                "cargo-ci-${{ runner.os }}-"
            ),
        },
    ),
    run(
        cmd="cargo run -p v3-compiler --features bootstrap-regen-fresh --bin regen_bootstrap -- --verify"
    ),
]

changes_detect = r"""set -euo pipefail
if [ "${{ github.event_name }}" = "push" ]; then
  echo "push event — always run heavy v3 job"
  echo "code=true" >> "$GITHUB_OUTPUT"
  exit 0
fi
git fetch origin main:refs/remotes/origin/main
changed=$(git diff --name-only origin/main...HEAD)
echo "Changed files (first 200):"
echo "$changed" | head -200
non_docs=$(echo "$changed" | grep -vE '^(docs/.*|[^/]+\.md)$' || true)
if [ -n "$non_docs" ]; then
  echo "Non-docs files detected; v3 will run full heavy compute:"
  echo "$non_docs" | head -50
  echo "code=true" >> "$GITHUB_OUTPUT"
else
  echo "Docs-only PR (allowlist: docs/** + root *.md) — v3 will skip"
  echo "(Layer 1 CI mitigation; dissolution trigger = affected-set lens CI integration)"
  echo "code=false" >> "$GITHUB_OUTPUT"
fi"""

changes_steps = [
    uses(
        uses_ref=checkout_v4,
        with_m={"fetch_depth": "0"},
    ),
    run(cmd=changes_detect),
]

zig_linker = r"""mkdir -p "${RUNNER_TEMP}/gunbc-bin"
cat > "${RUNNER_TEMP}/gunbc-bin/zig-cc" <<'EOF'
#!/usr/bin/env bash
exec zig cc "$@"
EOF
chmod +x "${RUNNER_TEMP}/gunbc-bin/zig-cc"
echo "CC=${RUNNER_TEMP}/gunbc-bin/zig-cc" >> "${GITHUB_ENV}"
echo "RUSTFLAGS=-D warnings -C linker=${RUNNER_TEMP}/gunbc-bin/zig-cc" >> "${GITHUB_ENV}"""

prebuild_int = r"""set -euo pipefail
heartbeat() {
  while sleep 60; do
    echo "v3 prebuild heartbeat: $(date -u +%FT%TZ)"
  done
}
heartbeat &
heartbeat_pid=$!
trap 'kill "$heartbeat_pid" 2>/dev/null || true' EXIT
cargo test -p v3-compiler --test integration --no-run"""

hotfix_skip_2d = r"""echo "::notice::v3 lane2d Stage 2d test execution SKIPPED per operator hot-fix directive at gunbc#846 (cold-v3 → ~10min target)."
echo "::notice::Restore criteria: per-test wall ≤ 2s under OnceLock/cached_compile amortization (rebuild session per #2722 §5)."
exit 0"""

v3_part1 = r"""set -euo pipefail
echo "V3_SUITE_START_TS=$(date +%s)" >> "${GITHUB_ENV}"
: > /tmp/v3-test-timings.log
RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler --lib --bins -- -Z unstable-options --report-time 2>&1 | tee /tmp/v3-test-timings.log
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "::error::cargo test (lib+bins) failed (exit=$status)"
  exit "$status"
fi"""

v3_part2 = r"""set -euo pipefail
RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler --test determinism_test -- -Z unstable-options --report-time 2>&1 | tee -a /tmp/v3-test-timings.log
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "::error::cargo test (determinism_test) failed (exit=$status)"
  exit "$status"
fi"""

v3_part3 = r"""set -euo pipefail
RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler --doc -- -Z unstable-options --report-time 2>&1 | tee -a /tmp/v3-test-timings.log
status=${PIPESTATUS[0]}
if [ "$status" -ne 0 ]; then
  echo "::error::cargo test (--doc) failed (exit=$status)"
  exit "$status"
fi"""

v3_part4 = r"""set -euo pipefail
echo "::notice::v3 integration test execution SKIPPED via zero-test-filter per operator hot-fix at gunbc#846."
echo "::notice::Guard pattern preserved; integration binary runs zero tests (nonexistent filter)."
if [ -z "${V3_SUITE_START_TS:-}" ]; then
  echo "::error::missing V3_SUITE_START_TS (full suite part 1 must run first)"
  exit 1
fi
RUSTC_BOOTSTRAP=1 cargo test -p v3-compiler --test integration __HOT_FIX_NONEXISTENT_FILTER__ -- -Z unstable-options --report-time 2>&1 | tee -a /tmp/v3-test-timings.log
status=${PIPESTATUS[0]}
elapsed=$(( $(date +%s) - V3_SUITE_START_TS ))
echo "v3 full-suite wall time (integration-skipped): ${elapsed}s"
if [ "$status" -ne 0 ]; then
  echo "::error::cargo test (integration, zero-filter) failed (exit=$status) — should never happen with nonexistent filter; investigate."
  exit "$status"
fi
echo "::notice::Restore criteria: rebuild session per #2722 §5 OnceLock/cached_compile amortization → per-test wall ≤ 2s ratchet → re-enable per cluster filter (replace __HOT_FIX_NONEXISTENT_FILTER__ with cluster-specific filter)."""

ratchet_2s = r"""if [ ! -s /tmp/v3-test-timings.log ]; then
  echo "::notice::no timing log captured or log is empty (full-suite steps did not reach cargo test, or tee wrote nothing) — skipping per-test ratchet"
  exit 0
fi
scripts/check-test-timeout.sh /tmp/v3-test-timings.log"""

l7_gate = r"""matches=$(grep -nE "fn (find_port|find_behavior|resolve_producer|lookup_node|lookup_port)" src/v3/lenses/*.dag 2>/dev/null || true)
if [ -n "$matches" ]; then
  echo "::error::L-7 violation: lens reconstructs a substrate lookup locally"
  echo "$matches"
  exit 1
fi"""

l8_gate = r"""wrappers=$(ls src/v3/compiler/src/lens_*.rs 2>/dev/null || true)
if [ -z "$wrappers" ]; then
  echo "L-8: no lens wrapper files to scan"
  exit 0
fi
matches=$(grep -nE "pub fn .*-> (usize|bool|i64)" $wrappers 2>/dev/null || true)
if [ -n "$matches" ]; then
  echo "::error::L-8 violation: lens wrapper collapses typed carrier to a primitive"
  echo "$matches"
  exit 1
fi"""

v3_steps = [
    uses(uses_ref=checkout_v4, with_m={"fetch_depth": "1"}),
    uses(
        uses_ref=setup_rust,
        with_m={"components": "rustfmt, clippy", "cache": "false", "rustflags": ""},
    ),
    uses(
        uses_ref=setup_zig,
        with_m={"version": "0.13.0", "cache": "false"},
    ),
    run(cmd=zig_linker),
    uses(uses_ref=setup_py, with_m={"python-version": "3.x"}),
    uses(uses_ref=setup_go, with_m={"go-version": "stable"}),
    uses(
        uses_ref=cache_act,
        with_m={
            "path": (
                "~/.cargo/registry/index/\n"
                "~/.cargo/registry/cache/\n"
                "~/.cargo/git/db/\n"
                "target/"
            ),
            "key": "cargo-v3-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('src/v3/compiler/src/**', 'src/v3/compiler/tests/**') }}",
            "restore-keys": (
                "cargo-v3-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-\n"
                "cargo-v3-${{ runner.os }}-"
            ),
        },
    ),
    run(cmd="cargo build -p execute-command-bootstrap"),
    run(cmd=prebuild_int),
    run(cmd=hotfix_skip_2d),
    run(cmd="scripts/check-v3-full-suite-split-test-targets.sh"),
    run(
        cmd=v3_part1,
        env={"CARGO_TERM_COLOR": "never", "RUST_MIN_STACK": "16777216"},
    ),
    run(
        cmd=v3_part2,
        env={"CARGO_TERM_COLOR": "never", "RUST_MIN_STACK": "16777216"},
    ),
    run(
        cmd=v3_part3,
        env={"CARGO_TERM_COLOR": "never", "RUST_MIN_STACK": "16777216"},
    ),
    run(
        cmd=v3_part4,
        env={"CARGO_TERM_COLOR": "never", "RUST_MIN_STACK": "16777216"},
    ),
    run(cmd=ratchet_2s, if_cond="always()"),
    run(cmd="cargo clippy -p v3-compiler --all-targets -- -D warnings"),
    run(
        cmd="cargo clippy -p v3-compiler --all-targets --features bootstrap-regen-fresh -- -D warnings"
    ),
    run(cmd="cargo test -p v3-compiler --no-run --features bootstrap-regen-fresh"),
    run(cmd=l7_gate),
    run(cmd=l8_gate),
    run(cmd="scripts/check-compiler-std-ratchet.sh"),
    run(cmd="scripts/check-banked-dissolutions.sh"),
]

gate_evidence = r"""echo "Gate evidence: changes.result=${{ needs.changes.result }} v3.result=${{ needs.v3.result }}"
if [ "${{ needs.changes.result }}" != "success" ]; then
  echo "::error::Layer 1 filter (changes job) did not succeed (result=${{ needs.changes.result }})."
  echo "::error::Cannot determine v3 skip legitimacy — skipped required-checks default to SUCCESS"
  echo "::error::in GitHub branch protection (https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/troubleshooting-required-status-checks#handling-skipped-but-required-checks),"
  echo "::error::so an explicit step failure here surfaces the missing-evidence as a diagnostic"
  echo "::error::per P3 fail-closed / C-8 discipline."
  exit 1
fi
if [ "${{ needs.v3.result }}" = "failure" ] || [ "${{ needs.v3.result }}" = "cancelled" ]; then
  echo "::error::v3 job did not succeed (result=${{ needs.v3.result }}); ratchet cannot claim green."
  exit 1
fi
echo "Gate evidence validated; ratchet may proceed." """

db8_stub = r"""echo "::notice::self_host_ratchet DB-8 release matrix runs only on pushes to main (Lane 1e staged)."
exit 0"""

db8_hashmap = r"""if grep -nE "(HashMap|HashSet)::" src/v3/compiler/src/emit.rs; then
  echo "::notice::DB-8: emit.rs still uses HashMap/HashSet:: iteration — target is BTree* / sorted keys (determinism_test + ROADMAP Lane 3 Stage 3c prep)"
else
  echo "emit.rs: no HashMap::/HashSet:: — consider graduating this check to required when Lane 1e clears debt"
fi"""

shr_steps = [
    run(cmd=gate_evidence),
    uses(
        uses_ref=checkout_v4,
        with_m={"fetch_depth": "1"},
        if_cond="github.event_name == 'push' && github.ref == 'refs/heads/main'",
    ),
    run(
        cmd=db8_stub,
        if_cond="github.event_name != 'push' || github.ref != 'refs/heads/main'",
    ),
    uses(
        uses_ref=setup_rust,
        with_m={"cache": "false", "rustflags": ""},
        if_cond="github.event_name == 'push' && github.ref == 'refs/heads/main'",
    ),
    uses(
        uses_ref=cache_act,
        with_m={
            "path": (
                "~/.cargo/registry/index/\n"
                "~/.cargo/registry/cache/\n"
                "~/.cargo/git/db/\n"
                "target/"
            ),
            "key": "cargo-self-host-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-${{ hashFiles('src/v3/compiler/**') }}",
            "restore-keys": (
                "cargo-self-host-${{ runner.os }}-${{ hashFiles('**/Cargo.lock') }}-\n"
                "cargo-self-host-${{ runner.os }}-"
            ),
        },
        if_cond="github.event_name == 'push' && github.ref == 'refs/heads/main'",
    ),
    run(
        cmd="cargo test -p v3-compiler --release --test determinism_test",
        if_cond="github.event_name == 'push' && github.ref == 'refs/heads/main'",
        cont_err=True,
    ),
    run(
        cmd="cargo run -p v3-compiler --release --bin self_host_fixed_point",
        if_cond="github.event_name == 'push' && github.ref == 'refs/heads/main'",
        cont_err=True,
    ),
    run(
        cmd=db8_hashmap,
        if_cond="github.event_name == 'push' && github.ref == 'refs/heads/main'",
    ),
]

hosted = "HostedRunner { label: UbuntuLatest }"
ubicloud = 'SelfHosted { labels: Cons { head: "ubicloud-standard-8", tail: Empty } }'

jobs = [
    textwrap.dedent(
        f"""\
        Job {{
          id: "fmt",
          name: none,
          runner: {hosted},
          steps: {cons_list(fmt_steps)},
          needs: Empty,
          env: {{}},
          if_condition: "github.event.pull_request.draft != true",
          strategy: none,
          timeout_minutes: {5},
          continue_on_error: false,
          concurrency: none,
        }}"""
    ),
    textwrap.dedent(
        f"""\
        Job {{
          id: "ci",
          name: none,
          runner: {hosted},
          steps: {cons_list(ci_steps)},
          needs: Empty,
          env: {{}},
          if_condition: "github.event.pull_request.draft != true",
          strategy: none,
          timeout_minutes: {25},
          continue_on_error: false,
          concurrency: none,
        }}"""
    ),
    textwrap.dedent(
        f"""\
        Job {{
          id: "changes",
          name: none,
          runner: {hosted},
          steps: {cons_list(changes_steps)},
          needs: Empty,
          env: {{}},
          if_condition: "github.event.pull_request.draft != true",
          strategy: none,
          timeout_minutes: {3},
          continue_on_error: false,
          concurrency: none,
        }}"""
    ),
    textwrap.dedent(
        f"""\
        Job {{
          id: "v3",
          name: none,
          runner: {ubicloud},
          steps: {cons_list(v3_steps)},
          needs: Cons {{ head: "changes", tail: Empty }},
          env: {{}},
          if_condition: "github.event.pull_request.draft != true && (needs.changes.outputs.code == 'true' || github.event_name == 'push')",
          strategy: none,
          timeout_minutes: {120},
          continue_on_error: false,
          concurrency: none,
        }}"""
    ),
    textwrap.dedent(
        f"""\
        Job {{
          id: "self_host_ratchet",
          name: none,
          runner: {ubicloud},
          steps: {cons_list(shr_steps)},
          needs: Cons {{ head: "v3", tail: Cons {{ head: "changes", tail: Empty }} }},
          env: {{}},
          if_condition: "${{{{ always() && github.event.pull_request.draft != true }}}}",
          strategy: none,
          timeout_minutes: {60},
          continue_on_error: true,
          concurrency: none,
        }}"""
    ),
]


def main() -> None:
    header = '''// extdeps/github/ci.dag — Workflow-as-data projection of `.github/workflows/ci.yml`.
//
// T-WAD FULL R3 / WI-2 first draft (gunbc PR #2744 / #2745). Carriers:
// `extdeps.github.actions`. Authoritative transport remains the YAML file.
//
// TODO(WI-2 — held / follow-on slices):
//   - Workflow-level `concurrency:` has no `Workflow` field yet — emitter/Slice 4.
//   - `PullRequest` trigger: YAML lists `ready_for_review`; add `PullRequestActivity`
//     variant in actions.dag or dissolve via structural cluster (Slice 7).
//   - `changes.outputs.code` + step `id: filter` — job outputs / step id not on
//     `Job`/`Step` carriers yet (Slice 7).
//   - Layer 2 path-conditional `if:` cluster for regex-heavy steps (Slice 7).
//   - `scripts/slow-test-exemptions.txt` ratchet coupling (Slice 6).
//   - Affected-set / provable minimal selection replaces Layer 1 grep filter (Slice 7).
//   - `GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}` — `WorkflowSecret` / env Map sum-type (Slice 1 held).
//   - `runs-on: ${{ vars.CI_RUNNER || 'ubuntu-latest' }}` approximated as `HostedRunner UbuntuLatest`.
//
// Consumers: T-WAD R3 emitter + timing/dimension reports; keep in lockstep with ci.yml.

module extdeps.github.ci

import std.list { List, Empty, Cons }
import extdeps.github.actions {
  Workflow, Job, Step, RunStep, UsesStep,
  HostedRunner, SelfHosted, UbuntuLatest,
  Push, PullRequest, Opened, Synchronize, Reopened,
  checkout_action, setup_rust_action, cache_action,
  WorkflowPermissions, PermRead, PermNone,
  Bash, ActionRef,
}

'''

    on_triggers = """Cons {
  head: Push { branches: Cons { head: \"main\", tail: Empty }, paths: Empty },
  tail: Cons {
    head: PullRequest {
      branches: Cons { head: \"main\", tail: Empty },
      types: Cons {
        head: Opened,
        tail: Cons { head: Synchronize, tail: Cons { head: Reopened, tail: Empty } }
      }
    },
    tail: Empty
  }
}"""

    body = f"""data gunbc_ci_yml_workflow: Workflow = Workflow {{
  name: \"ci\",
  on: {on_triggers},
  jobs: {cons_list(jobs)},
  env: {{ CARGO_TERM_COLOR: \"always\", RUSTFLAGS: \"-D warnings\" }},
  permissions: WorkflowPermissions {{
    contents: PermRead,
    pull_requests: PermRead,
    issues: PermNone,
    actions: PermNone,
  }},
}}
"""

    Path = __import__("pathlib").Path
    path = Path(__file__).resolve().parent.parent / OUT
    path.write_text(header + body, encoding="utf-8")
    print(f"wrote {path}")


if __name__ == "__main__":
    main()
