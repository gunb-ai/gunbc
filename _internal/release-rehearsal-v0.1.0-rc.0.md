# v0.1.0-rc.0 Release Rehearsal Checklist

Purpose: walk the release path end-to-end on a release-candidate before
tagging `v0.1.0`. Each step has a concrete command + a pass condition. If
any step fails, stop and fix the source-of-truth (script, doc, or
substrate) — do **not** patch the artifact.

Pairs with `RELEASE_TODO.md` §0 (merge gate) and §2 (public snapshot).

Strip-from-public note: this file lives under `_internal/` and is removed
by `scripts/publish-snapshot.sh` (top-level `_internal` is on the strip
list).

---

## 0. Pre-flight (run from a clean working tree on internal `main`)

- [ ] `git status` shows no uncommitted changes
- [ ] `git pull --ff-only` — local `main` matches `origin/main`
- [ ] `cargo fmt --all --check` passes
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `cargo test --workspace` green
- [ ] Most recent CI run on `main` is green (`gh run list -L 1 --branch main`)

## 1. Clean-checkout build gate

Rehearses the user experience: fresh clone, build, hero demo.

- [ ] `scripts/_internal/check-clean-checkout-build.sh` exits 0
  - Default: clones `gunb-ai/gunbc` `main`
  - For the rc itself: `GUNBC_REF=v0.1.0-rc.0 scripts/_internal/check-clean-checkout-build.sh`
- [ ] Tail of output reads `clean-checkout build gate: PASS`
- [ ] Re-run with `KEEP_WORK_DIR=1` and spot-check the emitted Rust in
      `$WORK_DIR/weather-out/src/` matches the hero-demo expectation
      (records, enums, pattern match — no fabricated content)

## 2. Public snapshot dry-run

- [ ] `scripts/publish-snapshot.sh` (no `--publish`) succeeds
- [ ] `git -C /tmp/gunbc-pub-export log -1 --stat` shows:
  - No paths under `docs/briefs/`, `docs/history/`, `docs/debt/`,
    `docs/admin/`, `_internal/`, `wip/`, `scripts/session-dashboard/`,
    `scripts/_internal/`, `src/v3/`, `src/v4/TASKS.md`
  - No `T-##` task numbers, session IDs, or `operator-ratified` strings
    in `THESIS.md` / `INVARIANTS.md` / `ROADMAP.md` / `README.md`
    (spot-check: `grep -E 'T-[0-9]+|operator-ratified|[a-z]+-[a-z]+-[0-9]{2,4}' \
       THESIS.md INVARIANTS.md ROADMAP.md README.md`)
- [ ] Workspace metadata is publish-clean:
      `git -C /tmp/gunbc-pub-export grep -E 'RESTORED|T-WAD|T-Ground-Pilot' Cargo.toml`
      returns nothing
- [ ] Run `cargo fmt --all --check` and `cargo build --release -p v2-compiler --bin gunbc`
      inside `/tmp/gunbc-pub-export` — both green
- [ ] Run `scripts/_internal/check-clean-checkout-build.sh` against the
      dry-run worktree as a local repo:
      `GUNBC_REPO_URL=/tmp/gunbc-pub-export GUNBC_REF=snapshot-<sha> \
         scripts/_internal/check-clean-checkout-build.sh`

## 3. Tag the release candidate

- [ ] `git tag -a v0.1.0-rc.0 -m "release candidate 0 for v0.1.0"`
- [ ] `git push origin v0.1.0-rc.0`
- [ ] GH Release workflow (`release.yml`) starts on the tag push
      (RELEASE_TODO §5 Phase 1a)
- [ ] All six platform artifacts upload to the draft release:
      `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`,
      `x86_64-apple-darwin`, `aarch64-apple-darwin`,
      `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`
- [ ] Asset names match `release_published_artifact_names` from
      `src/v4/workflow/release.dag` (windows rows end `.exe`)

## 4. Per-platform smoke (download from the draft release)

For each platform you have access to:

- [ ] Download the `gunbc` binary for the platform
- [ ] `./gunbc --help` prints usage
- [ ] `./gunbc compile --source-root dsl/examples/weather --source-root dsl/std \
         --output-dir ./weather-out --target rust` (against a fresh clone)
- [ ] `cargo check --manifest-path ./weather-out/Cargo.toml` passes

Minimum: the host platform of the release manager must pass. Other rows
get attested by whoever ran them and pasted output into the rc tracking
issue.

## 5. Public-snapshot rehearsal push (against a *staging* remote)

Do **not** push to the real public remote during rc rehearsal.

- [ ] `git remote add public-staging git@github.com:<your-fork>/daglang-staging.git`
- [ ] `PUBLISH_CONFIRM=yes scripts/publish-snapshot.sh --publish --remote public-staging`
- [ ] Clone the staging public repo from scratch in `/tmp` and re-run
      `scripts/_internal/check-clean-checkout-build.sh` with
      `GUNBC_REPO_URL=<staging url>` — passes
- [ ] Staging repo has a single root commit with message `snapshot YYYY-MM-DD`
      and no internal SHA in the message body

## 6. Stop conditions (do NOT promote rc.0 → v0.1.0 if any of these hold)

- Any §1–§5 step failed and was patched in the artifact rather than at source
- Any internal session ID, `T-##` token, or `_internal/` path appears in
  the staging public snapshot
- The hero demo emits Rust that `cargo check` rejects on any tested platform
- `release.yml` produced fewer than six artifacts, or any artifact name
  diverges from `release.dag`

## 7. Go decision

- [ ] Release manager records pass/fail per row in the rc tracking issue
- [ ] If all green: open the cut-over PR per RELEASE_TODO §7 timeline,
      then tag `v0.1.0` from the same commit as `v0.1.0-rc.0`
- [ ] Delete `RELEASE_TODO.md` and this checklist after `v0.1.0` is live
      (RELEASE_TODO retirement clause)
