# gunbc — Claude Code Project Instructions

## Mindset

You are a long-term co-owner of this codebase, not a short-term task executor. Every
change you make will be maintained by future-you. Optimize for cost of change: when
the language grows by one type, one expression, or one transport, how many files
need editing? The answer should be 1.

## The Invariants — STOP Before Violating

If your planned approach would violate any invariant below, **STOP**. Do not proceed.
Instead, explain which invariant you're about to hit, why your current approach leads
there, and propose an alternative that respects the constraint. Ask the user to
confirm before continuing.

### Structural invariants

1. **Domain lives in the DSL, not in Rust.** If something can be expressed in `.dag`
   files, it must not be hardcoded in Rust. Rust is the engine; `.dag` is the domain.

2. **World I/O is structural.** A DAG node either does I/O or it doesn't — you can
   tell by looking at the graph. Only `gunbc-lib-transport` performs direct I/O.

3. **Extdeps implement specifications, not abstractions.** Every `dsl/extdeps/` module
   models a real external system from its actual API docs. Real names, real endpoints,
   real versions. If you can't link to a spec, you're inventing one.

4. **Each compiler phase is a pure function.** No phase mutates its input. No phase
   performs I/O (except filesystem reads during import resolution). The compiler never
   executes the DAGs it produces.

5. **Composition through layers, not abstraction.** Each layer only knows about layers
   below it. Adding a dependency means instantiating existing vocabulary, not inventing
   new abstractions.

6. **DAG nodes are facts, rendering is separate.** A DAG node asserts truths about
   computation (types, cardinality, data flow). These are target-agnostic. How to
   express those truths in a target language (Rust `Box<T>`, C `T*`, Verilog wire
   bundle) is a rendering decision that lives in the backend, never in the IR.
   The structural test: can you swap the backend without changing the IR?

7. **The interpreter maps IR to execution — nothing more.** No domain logic, no
   compiler logic. Every `extern func` backed by Rust must be justified.

8. **Every expression lowers to structural DAG nodes or compilation fails.** No opaque
   AST fragments. No interpreter-backed fallback nodes. `lower_expr` returns `Result`,
   not `Option`, and its match is exhaustive with no wildcard.

9. **Correctness by construction, not by validation.** If a property must hold, the
   API must make violations unrepresentable. Don't add validation passes — refactor
   the types so invalid states can't be constructed.

### Sustainability invariants

- **No duplicate representations.** Every fact encoded in exactly one place. If
  changing a fact requires editing two files, one is a derived copy that should be
  deleted or computed.

- **No case enumeration for open sets.** Prefer structural walks over match arms that
  enumerate known cases. Closed enums are fine; string-keyed open lists are not.

- **No fallbacks that fabricate.** Every code path succeeds fully or fails with a
  clear error. No `.ok()` swallowing errors, no `continue` silently dropping work,
  no fallback defaults producing valid-looking but wrong output.

- **No parallel implementations.** If the same computation exists in two forms, one
  must be deleted. They will diverge.

- **Explicit boundary contracts.** Make illegal states unrepresentable at pipeline
  stage boundaries. If you want a validation pass, refactor the upstream output type
  instead.

- **Single-authority metadata.** One producer per piece of metadata. No runtime
  callbacks, string conventions, or hardcoded lists.

## Architecture Quick Reference

```
.dag source -> parse -> resolve -> typecheck -> lower -> derive -> emit
                                                  |
                                          VerifiedDag<LoweredOp>
                                           /              \
                                  emit -> cargo        interpret
```

- v1 compiler: `src/v1/` (Rust, 00_foundation through 10_test)
- v2 self-hosted compiler: `src/v2/` (.dag source, bootstrapped by v1)
- DSL source: `dsl/` (std, extdeps, config, tools)
- Key design change in v2: types are structural values (`TypeExpr`), not string
  references (`TypeId`). No registry, no deferred resolution.

## Testing

- Behavioral assertions only — never assert internal implementation details
- Hermetic unit tests — no filesystem/network/environment side effects
- No tautological tests — tests must encode independent specifications
- Three tiers: DryRun (structure), Selective Real (computation), Full Real (integration)

```bash
cargo test --workspace --exclude gunbc-dag-tests   # hand-written tests
cargo test -p gunbc-dag-tests                       # auto-generated DAG tests
cargo clippy --all-targets -- -D warnings           # lint
```

## Review Queue Discipline

When draining review feedback on an automation queue branch:
- choose one primary invariant theme from `src/v1/README.md` and record it
- resolve at most one review-feedback item on that branch/PR
- stop after that item instead of stacking another review-feedback fix on the same branch
- keep each commit strictly scoped to the invariant fix — no unrelated helper cleanup,
  dead-code removal, or opportunistic refactoring unless it is directly required for
  the fix to compile and pass tests

## When in Doubt

Read `src/v1/SUSTAINABILITY.md` for the ledger of past violations and their fixes.
The postmortems (especially FC-7) show exactly what goes wrong when invariants are
bypassed. Learn from them rather than repeating them.
