# gunbc — Agent Instructions

Read these docs before working:
- `INVARIANTS.md` — governing rules. STOP before violating any invariant.
- `ROADMAP.md` — architectural thesis, current state, active work, design directions.
- `MODELING.md` — DSL modeling philosophy. **Especially M9: DFS the concept DAG.**
  Before defining any new type, DFS from `dsl/std/` to find the existing concept
  it should attach to. See the concept DAG layers in MODELING.md.
- `CODING.md` — Rust implementation style (Google C++-style, pure functions, data + free functions).
- `TESTING.md` — test discipline (hermetic, behavior-driven, unit-first; mocks over full-pipeline compile).

## Key Commands

```bash
cargo test --workspace                     # all workspace crates (hand-written + integration)
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

