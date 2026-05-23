# v2 DAG Artifact Hang — PR #3494 Zip-Fold Repro

Date: 2026-05-21
Work item: `node://adhoc-4d29126f-eb0`
PR under investigation: [#3494](https://github.com/gunb-ai/gunbc/pull/3494), head `7f14e8f302f4576bcfc7519c3e69c1697eb1ff22`

## Summary

The observed hang is reproducible, but the failing path is not the zip-fold predicate body in `find_witness`.

The differentiating evidence is target selection:

- `target/release/v2-compiler compile --source-root src/v4 --output-dir /tmp/v4-out-pr3494-rust` on PR #3494 finishes in 6s with `compiled: 123 files emitted, 0 diagnostics`.
- `target/release/v2-compiler compile --source-root src/v4 --output-dir /tmp/v4-out-pr3494 --target dag` on the same checkout prints `indexed 119 modules from 1 source roots` and `resolved 119 sources (transitive import closure)`, then stays CPU-bound until killed at 362s with RSS over 8.4GB and no output files.
- Current `main` at `ac0106450698e379ae7769ca51cfa400244ab58c` (PR #3473 merge commit) also hangs under the same `--target dag` command.

That isolates the failure to the v2 DAG artifact backend: the pipeline reaches post-infer emission for non-DAG targets, but the DAG target cannot serialize the typed graph within the sampled resource envelope.

## Reproduction

Checkout used:

```text
git fetch origin pull/3494/head:refs/heads/repro-pr-3494
git worktree add /tmp/gunbc-pr3494 repro-pr-3494
cd /tmp/gunbc-pr3494
CTRL_BUILD_BYPASS_SHIMS=1 cargo build -p v2-compiler --release
```

Successful control:

```text
target/release/v2-compiler compile --source-root src/v4 --output-dir /tmp/v4-out-pr3494-rust
```

Receipt:

```text
indexed 119 modules from 1 source roots
resolved 119 sources (transitive import closure)
compiled: 123 files emitted, 0 diagnostics
```

Failing command:

```text
target/release/v2-compiler compile --source-root src/v4 --output-dir /tmp/v4-out-pr3494 --target dag
```

Local sample from the failing run:

```text
elapsed=362 timeout_status=124 process_status=143
indexed 119 modules from 1 source roots
resolved 119 sources (transitive import closure)
```

RSS/CPU samples from the same run:

```text
09:12:13  180  98.5 4056396  7914996 R v2-compiler
09:12:33  200  98.7 7832440 12237364 R v2-compiler
09:14:14  300  99.1 6565932 12924748 R v2-compiler
09:15:09  355  99.2 8461740 16737636 R v2-compiler
```

No files were present under `/tmp/v4-out-pr3494` when the process was killed.

Tooling note: this container did not have `perf`, `gdb`, `pstack`, or `strace`, and `/proc/<pid>/stack` was permission-denied. The phase isolation therefore comes from target differential behavior plus the v2 pipeline source.

## Pipeline Pin

`src/v2/compile.dag` wires `compile_sources` as:

```text
tokenize -> parse -> resolve -> normalize -> infer -> complexity -> ownership -> artifact-plan -> emit
```

The `target` argument is only consumed by artifact planning / emission. The default Rust target finishing in 6s proves the common stages complete on PR #3494:

- normalize
- `reconcile` / inference
- complexity
- ownership
- artifact planning

The hang is therefore in the target-specific DAG artifact emission path, not in the new `find_witness` predicate body or `coercion_fold` call shape.

The target-specific path is `emit_dag_artifact` in `src/v2/compile.dag`.

## Root Cause

The v2 DAG backend emits a JSON snapshot of the post-infer `ResolvedGraph` by recursively serializing typed graph values by value.

The key functions are:

- `emit_dag_artifact(typed)` maps every `TypedModule` through `serialize_typed_module`.
- `serialize_typed_module` maps every item through `serialize_node`.
- `serialize_node` serializes all of these recursively:
  - `children` via `serialize_node(child)`
  - `params` via `serialize_param`, which serializes `type_expr` and defaults by value
  - `inferred` via `json_optional_inferred_node`
  - `uses`, `body`, `transport`, `properties`, and `type_annotation` by value
  - `expr_data`, whose expression cases also serialize `children` by value
- `serialize_inferred_node(Resolved { node })` serializes the resolved node by value again.

This is not a bounded graph serialization. It is a recursive tree expansion of a graph-shaped typed program. Any shared subgraph reachable through multiple fields is duplicated, and any inference backlink or repeated resolved type causes another full `Node` serialization. For v4-sized graphs, `--target dag` can become CPU-bound and memory-expanding before it writes the single `dag-artifact.json` file.

The zip-fold PR made this visible because the requested verification command used `--target dag`. The zip-fold body is not the root cause: the same checkout succeeds through Rust emission, and the #3473 merge baseline also fails under `--target dag`.

## Scope Checks

The DAG backend is not broken for every input. These smaller checks used the same local release binary:

| Input | Command shape | Result |
|---|---|---|
| one-file `module simple` with `fn main() -> Int { 0 }` | `--source-root /tmp/v2dag-fixtures/simple --target dag` | succeeds immediately; `dag-artifact.json` is 1,801 bytes |
| one-file copy of `src/v4/std/node.dag` | `--source-root /tmp/v2dag-fixtures/node --target dag` | succeeds immediately; `dag-artifact.json` is 1,439,625 bytes |
| copied `src/v4/std` subtree | `--source-root /tmp/v2dag-std-root --target dag` | succeeds in 2s; one file emitted, 0 diagnostics |
| full `src/v4` on PR #3494 | `--source-root src/v4 --target dag` | hangs post-resolve; killed at 362s; RSS >8.4GB |
| full `src/v4` on current `main` / #3473 merge | `--source-root src/v4 --target dag` | hangs post-resolve; killed at 182s; RSS >12GB in the final valid sample |

That scope argues for a graph-size / graph-sharing trigger rather than a parse error, a specific `fold_node` function body, or a universal inability to serialize any v4 `Node`.

The full v4 closure is the first checked input large enough to combine compiler, lens, test-claim, extdeps, and typed inferred references. In that shape, recursive by-value serialization expands the graph along multiple authority edges rather than emitting each node once and referencing it thereafter.

## Comparison With sunny-deer-191 / cool-raven-123

sunny-deer-191 reported that the cool-raven/Locus investigation hit parser and pattern substrate gaps:

- sum-variant positional generic type applications like `NodeAt(LocusAnchor<T>)`
- nested `LocusAnchor { at: x }` destructuring on instantiated carriers

They did not isolate a full `src/v4` multi-concrete-instantiation OOM in that session. Their #3488 work is a parser/pattern capability fix, not a DAG artifact serializer fix.

The PR #3494 reproduction is separate:

- PR #3494 succeeds through default Rust emission, so parser, lowering, inference, and ordinary emission are not the blocker.
- The failing behavior is specific to `--target dag`.
- The failure is also present on the #3473 merge baseline.

Conclusion: these are separate v2 capability limits unless future profiling shows a second, independent zip-fold failure under a non-DAG target.

## Design Proposal

Do not implement a v2 substrate upgrade in this lane. The required design is a bounded DAG artifact model, not a zip-fold-specific infer/emitter workaround.

The missing capability is: v2's DAG target needs graph serialization by reference, with a single authority for node identity, rather than recursive `Node` expansion by value.

Proposed substrate shape:

- `DagArtifact` is a record with:
  - `nodes: Map<NodeId, NodeRecord>`
  - `modules: List<NodeId>`
  - `item_registry: Map<String, NodeId>` or a keyed reference table
  - `diagnostics: List<DiagnosticRecord>`
- `NodeRecord` carries local fields only:
  - stable id / content hash
  - local spans and authored name facts
  - child references as `List<NodeId>`
  - parameter/body/type/inferred references as `NodeId?`, not embedded `Node`
- `InferredRecord` carries:
  - `Resolved { node: NodeId }`
  - `TypeVariable { id }`
  - `CompilerError { message, span }`
- serialization is a memoized fold over the typed graph:
  - first visit emits a `NodeRecord`
  - later visits emit only a reference
  - cycles or backedges fail closed if they cannot be represented

This is the v2 mirror of the v4 direction: `Node` identity / canonical form / `content_hash` is the authority, and consumers cite nodes by reference instead of copying typed subgraphs into parallel payloads. It also matches P2 boundary discipline: every fact is serialized once, and references preserve sharing.

## Routing Recommendation

Recommendation: **B' — Separate case: v2 DAG artifact backend.**

Do not extend sunny-deer-191's parser/pattern lane for this finding. Their lane should keep closing generic type application and nested-pattern capability. This audit should route to a separate v2 DAG artifact backend design/implementation lane if the project still needs `--target dag` to be a hard viability gate before v4 owns a native graph artifact.

Current CI already separates those receipts: `.github/workflows/ci.yml` first attempts the full `--target dag` compile through its bootstrap command, then accepts the tracked `scripts/v4-bootstrap-resolve-posture-gate.sh` receipt on timeout/SIGTERM when the log proves resolve posture (`resolved N sources...`, zero `error:` lines). The v2-maintenance follow-up should therefore target the full DAG artifact backend itself, not the existing resolve-posture bridge. Outside that bridged CI path, `scripts/v2-run-preflight.sh` still invokes the same `--target dag` compile when run without `V2_PREFLIGHT_SKIP_COMPILE=1`, so standalone preflight use can still measure serializer blowup rather than zip-fold substrate soundness.

## PR #3494 Status Implication

vivid-deer-580's zip-fold predicate work should not be judged by the `--target dag` hang. The same PR compiles through the default Rust target with zero diagnostics, so the predicate body was expressible to v2 at the compile/emission level that actually completes.

Recommended next action for PR #3494: resume the worker or reviewer with the default Rust-target receipt as the compile viability signal, then review the actual `.dag` design on its merits. If PR #3494 has a required test or CI path that insists on `--target dag`, that requirement should be blocked on the separate DAG artifact backend issue rather than treated as a predicate-body failure.
