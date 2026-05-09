# STOP: `??` / `%` Syntax Authority Mismatch

## Decision

Do not remove `??` or `%` from `dsl/extdeps/languages/dag/syntax.dag::dag_operators` in a v3-only syntax cleanup slice.

That file is not just a v3 tokenizer projection. It is the shared `.dag` language syntax authority consumed by v2 and extdeps language modeling. Removing rows there makes the model less faithful to the language that still exists in this repository.

## Verified facts

- `src/v2/01_tokenize.dag` tokenizes `%` as `ShPercent` and `??` as `ShNullCoalesce`.
- `src/v2/languages.dag` imports `extdeps.languages.dag.syntax { dag_operators }` and uses it for `RenderTarget::Dag`.
- The stage0 projection still contains `dag_operators()` rows for `??` / `NullCoalesce` and `%` / `Mod`.
- Repository `.dag` source still uses null coalescing: `dsl/gunbc/tools/gist.dag` contains `last(parts) ?? ""`.
- v3 currently does not have the full support chain for these rows: `src/v3/compiler/operators.dag` has no `Mod` or `NullCoalesce` mapping, and `src/v3/std/tokenize.dag` has no `%` or `??` punctuation variants.

## Retirement blocker

The removal path is blocked by extdeps fidelity and P2 single-authority discipline: `dag_syntax_spec` must remain faithful to the broader `.dag` syntax while v2 and repo-authored `.dag` still depend on these operators.

The implementation path is also broader than the dispatched slice:

- `%` requires at least tokenizer punctuation, parser table row, `OperatorKind` / arithmetic operator authority, inference dispatch, and target realization policy.
- `??` requires tokenizer punctuation, parser table row, a surface/operator representation for null-coalescing, lowering/inference semantics, and target realization policy.

Until that full chain is intentionally dispatched, v3 must keep its SG-1a boundary explicit: these shared syntax rows are not v3 tokenizer/operator support.

## Follow-up

Dispatch a full syntax/operator support slice for one operator at a time, or first split the shared `.dag` language authority into a broader extdeps syntax spec plus an explicit v3-supported operator subset. The latter would retire the mismatch without making the shared syntax model lie.
