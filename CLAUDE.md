# gunbc — Agent Instructions

Read these docs before working:
- `INVARIANTS.md` — governing rules. STOP before violating any invariant.
- `ROADMAP.md` — architectural thesis, current state, active work, design directions.
- `MODELING.md` — DSL modeling philosophy. **Especially M9: DFS the concept DAG.**
  Before defining any new type, DFS from `dsl/std/` to find the existing concept
  it should attach to. See the concept DAG layers in MODELING.md.
- `src/v2/DESIGN.md` — compiler design principles.

## Key Commands

```bash
cargo test --workspace --exclude v2-compiler-tests  # hand-written tests
cargo test -p v2-compiler-tests                     # v2 compiler tests
cargo clippy --all-targets -- -D warnings           # lint
cargo fmt --all --check                             # format check (also runs via pre-push hook)
cargo test -p v2-compiler-tests v2_strict_compile_diagnostic_count -- --ignored  # stage0 diagnostic ratchet (0 diagnostics)
```

## One-time setup

```bash
scripts/install-hooks.sh  # enables .githooks/pre-push (runs cargo fmt --all --check before every push)
```

## Cost of Change

When the language grows by one type, one expression, or one transport,
how many files need editing? The answer should be 1.

