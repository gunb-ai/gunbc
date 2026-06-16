# gunbc — Agent Instructions

Read these docs before working:
- `DESIGN.md` — the governing design: worldview, the principles (the review spine), how to model
  (**especially M9: DFS the concept DAG** before defining any type), enforcement, the Rust-seed coding
  style, verification discipline, and the hard-won lessons. Currently a working harvest TODO being
  rebuilt from first principles (the prior doc corpus was bankrupted 2026-06-16; it's in git history).
  STOP before violating a principle.
- `ROADMAP.md` — current state, active work, design directions.

## Key Commands

```bash
cargo test --workspace                     # all workspace crates (hand-written + integration)
cargo test -p v3-compiler                  # v3 compiler — NOTE: most tests here are dormant in CI; the one exception is get_off_v3_compile_to_dag_caller_count_is_at_or_below_ceiling (wired into ci_floor_parity via PR #4659). New tests added to this crate are NOT automatically CI-gated
cargo clippy --all-targets -- -D warnings  # lint
cargo fmt --all --check                    # format check (also runs via pre-push hook)
```

## One-time setup

```bash
.githooks/install-hooks.sh  # enables .githooks/pre-push
```

The pre-push hook runs `cargo fmt --all --check` on push. If drift is detected **on the branch being pushed (HEAD)**: the hook runs `cargo fmt --all`, stages tracked files, lands a `chore: apply cargo fmt` commit — and then **aborts the push**. Re-run `git push` to ship the new commit. (Git builds the push pack before the hook runs, so a commit created inside the hook can't be added to the in-flight push — a second push is required to ship it.) Requires a clean working tree (no uncommitted changes) at push time. Delete-only pushes and cross-branch pushes skip the auto-commit path.

## Cost of Change

When the language grows by one type, one expression, or one transport,
how many files need editing? The answer should be 1.

**Ledger standing principle (operator 2026-05-19):** Do not create or maintain documentation that acts as a parallel ledger for facts whose source of truth is already in inline comments or model marks. The mark is authoritative; debate belongs in PR review. Flag new comment-duplicating ledger docs if you see them proposed.

