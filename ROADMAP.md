# gunbc Roadmap

**Goal:** Self-hosted v2 compiler. The compiler is written in `.dag`, compiles
itself, and produces identical output when compiling itself again (fixed point).

**Thesis:** Explicit cause-and-effect relationships with basic primitives
(truth-valued structure, `Conj`/`Disj`, composition) are sufficient to express
any information concept. Named types are aliases for compositions; the compiler
should always be able to see through the name to the structure underneath.

---

## Planning Status

As of 2026-03-17, this file is the single live planning document for the repo.

- The former v2 performance audit is folded into **Track S** below.
- The active, still-relevant portions of `src/v1/SUSTAINABILITY.md` are folded
  into **Tracks S/B/C/D** and the consolidated backlog section below.
- `src/v1/SUSTAINABILITY.md` is now archival context, not a second roadmap.

---

## Immediate Priority (2026-03-17)

### P0: Generated test stack safety

The current branch fixes the host-side generated-crate blockers
(`v2_crate_cargo_check`, `v2_crate_cargo_build`) and restores the failing
`phase6_if_else_branch_is_inferred` path, but the ignored generated-runtime
lane still exposes one open issue: `v2_crate_cargo_test` can build and start
running generated tests, then abort with a stack overflow in the heavy
generated integration tests.

This must not be normalized as "just a slow ignored test." Generated tests
should not introduce stack-overflow behavior that the corresponding host
compiler/runtime path would not also hit.

**Short-term handling:**

- Keep the heavy generated-runtime lane opt-in/ignored while the overflow is
  being isolated.
- Treat any generated-test stack overflow as a real compiler/runtime defect,
  not a harmless test artifact.

**Acceptance:**

- [ ] generated tests either pass or fail semantically; they do not abort with
      stack overflow
- [ ] generated test execution is no more stack-fragile than the equivalent
      host-side compiler path
- [ ] heavy self/gist generated tests can remain opt-in for runtime cost, but
      not because they overflow the stack

---

## Completed Work

- **Stream 2 (sustainability cleanup):** stale docs cleaned up, terminal v1-only
  findings called out explicitly, stack growth and TCO status documented.
- **Stream 3 phases A-E:** PortContract dissolved. Shape dissolved via
  `Connective = Conj | Disj`. Dead code and old bridge helpers removed where the
  new structure made them unnecessary.
- **P0 (S85):** recursive types use SCC cycle detection on the type dependency
  graph.
- **P1 (S84):** v2 emitter TCO verified.
- **P1 (S83):** stack overflow mitigated with `stacker` at re-entrant call sites.
- **B2:** `04_typecheck.dag` renamed to `04_infer.dag`.
- **C1:** `LanguageSpec` interface defined in `dsl/std/languages.dag`.
- **C2:** Rust, Python, and Go language extdeps modeled in
  `dsl/extdeps/languages/`.
- **Typed emit hot paths:** the high-payoff emitter fixes landed. `type_cache`
  no longer dominates emit scope cloning, typed service-call collection/TCO
  analysis exists, and Rust/Python/Go emitters all have typed expression paths.
- **Infer and resolver asymptotics:** descendant-list churn and the worst
  resolver list-scan hotspots were materially improved.
- **Node convergence partial landing:** `Field.type_expr`, `Param.type_expr`,
  and `ResourceUse.resource` are `Node`; `TypedExpr.resolved_type`,
  `FuncSig.return_type`, and `TypedNode.return_type` are `Node`; all three
  emitters have node-based type readers.

---

## Invariant Audit Blockers (2026-03-17)

The convergence branch audit identified three structural themes requiring
design decisions before they can be resolved. Mechanical fixes (perf
anti-patterns, dead imports, stale comments) were applied directly.

### Blocker 1: Emitter walk triplication (Invariant: No parallel implementations)

Node-type emission, transport dispatch, and service def emission are each
implemented three times (Rust, Python, Go emitters). Adding a new container
type or transport binding requires editing all three backends.

**Cost of change:** 3 files per feature addition.

**Design options:**
- (A) Shared walk in `05_emit.dag` parameterized by language-specific
  rendering callbacks. Requires function-as-data, available in v2 self-hosted
  but not v1 bootstrap.
- (B) Intermediate `TypeShape` classification: shared `classify_node_type()`
  returns a structural description, each backend renders it. Adds a type but
  eliminates walk duplication.
- (C) Accept bounded duplication (3 backends, unlikely to grow).

**Recommendation:** Option B aligns with Invariant 6 ("DAG nodes are facts,
rendering is separate").

**Blocked on:** Decision.

### Blocker 2: Fabrication-on-error pattern (Invariant: No fallbacks that fabricate)

The DSL has no `Result<T, E>` type. Every function returns a value. Parse and
inference error paths fabricate dummy values (`leaf_type_node(name: "")`,
`leaf_node(name: "Unit")`) alongside error diagnostics. ~50 sites in the
parser, ~15 in inference.

Parser dummies are tolerable (first-error-halt, callers check `err`).
Inference dummies are riskier (fabricated types propagate mid-pipeline).
Emitter `"TODO"` stubs were converted to loud failures.

**Design options:**
- (A) Add `Result<T, E>` to the DSL language. Structural fix but large scope.
- (B) Convention: `ok: Bool` field on all result types; pipeline halts on
  first `ok == false`. Weaker than (A) but cheaper.
- (C) Accept parser pattern; fix inference individually (replace wildcard
  at `infer_expr` line 1377 with exhaustive match; replace remaining
  `leaf_node("Unit")` fallbacks with diagnostics).

**Recommendation:** Option C now, Option A as a language feature later.

**Blocked on:** Decision for inference fixes. Parser pattern is accepted.

### Blocker 3: D1 completion — delete TypeExpr from 00_core.dag

`TypeExpr` type definition (8 variants), `type_expr_to_node` + 7 helpers,
and `Predicate` type remain in `00_core.dag`. The parser no longer calls
`type_expr_to_node` (rewritten to build Nodes directly), but
`field_to_node` and `variant_to_node` are still referenced somewhere in
the emit pipeline or v1 bootstrap codegen.

The `Predicate` → `FieldInit` conversion (`predicate_to_field_init`) is
lossy: `Range { min, max }` becomes string `"range"`, losing the actual
bounds. This is a fabrication fallback that should be fixed when Predicate
is dissolved.

**Blocked on:** Tracing the last callers of `field_to_node`/`variant_to_node`
in the v1 emit pipeline (`daglang-emit`), migrating them, then deleting the
entire TypeExpr block (~300 lines).

### Blocker 4: Boundary contracts — Node conflates resolved/unresolved

The unified `Node` type cannot structurally distinguish resolved from
unresolved type references. `validate_no_unresolved()` is a post-hoc
validation pass — its existence proves the boundary type is too permissive.

**Root cause:** After TypeExpr→Node unification, a leaf `Node { name: "Foo" }`
could be either a resolved reference or an unresolved one. The pipeline gates
correctly at runtime, but the type boundary doesn't enforce it.

**Design options:**
- (A) Introduce `ResolvedNode` wrapper type. Large scope, breaks most callers.
- (B) Add a `resolved: Bool` field to Node. Cheap but convention-based.
- (C) Accept runtime validation; document the invariant in code.

**Recommendation:** Option C for now. The validation pass catches violations
at typecheck time. The risk (emit receiving unresolved types) only manifests
if emit is called outside the normal pipeline.

**Blocked on:** Decision. Option C requires no code changes.

---

## Parallel Tracks

Work is organized into tracks that can proceed mostly independently.

```text
Track S: Stabilization + perf      Track A: Self-hosting           Track B: Node convergence
──────────────────────────────     ───────────────────────────     ───────────────────────────
S1: Finish Node migration          A1: Gist compilation           B1: TypeExpr -> Node
S2: Repair emit/generated crate    A2: Runtime bridge             B2: Rename typecheck -> infer
S3: Residual perf backlog          A3: Gist end-to-end            B3: Expr -> Node
S4: Restore v2 test baseline       A4: Full self-compile          B4: Transport dissolution
                                   A5: Bootstrap stage 0 -> 1
                                   A6: Fixed point
                                   A7: v1 retirement

Track C: Language emission         Track D: Complexity analysis    Track E: Artifact planning
──────────────────────────────     ──────────────────────────────  ─────────────────────────────
C1: LanguageSpec                   D1: Cost algebra               E0: Monolith wrapper
C2: Rust/Python/Go facts           D2: Typed summaries            E1: Artifact model
C3: Emitters consult extdeps       D3: DAG composition            E2: Target placement
C4: CLI target selection           D4: Proofs + reporting         E3: Boundary semantics
                                                                  E4: Planning/reporting

Track R: Representation sizing
──────────────────────────────
R1: Audit + catalog
R2: Box rare fields
R3: Clone reduction
R4: Interpreter value repr
R5: TCO clone leak
```

**Dependencies:**

- **Track A** is blocked on **Track S**. Self-hosting is not a performance-only
  problem anymore; it is first blocked by current v2 correctness regressions.
- **B1/B3** depend on **S1/S2** staying green while the representation changes.
- **Track C** is largely independent of **A/B/D**.
- **Track D** can begin once the `Node`/typed-expression boundaries stabilize
  enough to make summaries trustworthy.
- **Track E** should be designed early, before multi-target and deployment
  assumptions harden. Its implementation depends on **B4/C3** being far enough
  along that target facts and boundary structure are explicit.
- **A5** requires **A4**.
- **A6** requires **A5**.
- **Track R** blocks **Track A** at A1+. The generated crate and host
  interpreter both OOM/overflow on gist-scale inputs due to type
  representation bloat. R2 (boxing rare fields) is the minimum fix.
- **Track R** is independent of **B/C/D/E**.

---

## Track S: Stabilization And Residual Performance

This track replaces the old standalone perf audit. It is the concrete blocker
list for getting v2 back to a trustworthy self-hosting baseline.

### S1: Finish TypeExpr -> Node migration

**Current failure mode:** the core model has already moved to `Node` in several
places, but parser/infer/generated-crate code still mixes `TypeExpr` and `Node`
assumptions.

**Progress (2026-03-17):**

- Parser now builds Nodes directly (`type_expr_to_node` calls removed from
  `02_parse.dag`)
- All TypeExpr functions deleted from `04_infer.dag`
- `node_to_type_expr_full` deleted from `00_core.dag`
- Remaining: `TypeExpr` type definition + `type_expr_to_node` + `Predicate`
  in `00_core.dag` (see Blocker 3 above)

Representative failure modes in the current tree:

- generated v2 crate code still shows `Node` vs `TypeExpr` mismatches and boxed
  `Option<Node>` / `Option<Expr>` mismatches

**Acceptance:**

- [x] parser helpers stop constructing or destructuring `Node`-typed fields as
      `TypeExpr`
- [x] infer helpers no longer assume unboxed `Option<Node>` / `Option<Expr>`
- [ ] generated crate code no longer assumes unboxed `Option<Node>` /
      `Option<Expr>`
- [ ] `v2_crate_cargo_check` passes
- [ ] `cargo test -p v2-compiler-tests --quiet` returns to green

### S2: Repair emitter and generated-crate coherence

**Current failure mode:** typed hot paths are in place, but the emitted/generator
surface is still internally inconsistent.

Remaining coherence work:

- stale Rust emit call sites still pass removed parameters to
  `emit_simple_expr`
- plain-`Expr` transport/mock/config helpers still exist and still form a small
  bridge layer
- `typed_expr_to_expr` still exists and is still imported in `04_infer.dag`,
  even though the main hot-path callers are gone

**Acceptance:**

- [ ] stale Rust emit call sites are fixed
- [ ] remaining plain-`Expr` helpers are either deleted or explicitly justified
- [ ] `typed_expr_to_expr` is deleted once no imports/callers remain
- [ ] generated Rust/Python/Go crates compile from the v2 pipeline

### S3: Residual performance backlog

**What is already a real win:**

- emit scope clone blow-up: mostly fixed
- infer descendant-list / `type_entries` churn: mostly fixed
- resolver list-scan hotspots: substantially fixed
- map-based argument ordering and deduplication: fixed

**What still remains:**

- **Tokenizer bootstrap-path builders:** `scan_string_body()` and
  `process_escapes_loop()` still rely on immutable-list accumulation. This is
  only provably linear if the runtime append primitive is O(1).
- **Parser list builders:** many accumulators still use `concat([x], acc)` plus
  `reverse(acc)`. That is an improvement over the oldest forms, but it is not
  yet a proof of linear behavior under value semantics.
- **Resolver Kahn inner-loop scan:** topological sorting still carries a flat
  edge list and filters it each round. The worst old hotspots are improved, but
  the remaining adjacency traversal should still move to a precomputed
  neighbor map if module-graph scale matters.
- **v1 interpreted/bootstrap `list_push`:** still clones the entire list before
  appending, so repeated appends remain quadratic on the interpreted path.
- **Do not mistake native fusion for language-level proof:** emitted/native
  codegen can already fuse some `concat(acc, [x])` and `list_push` patterns into
  append-like operations, but that does not by itself prove the same bound for
  the bootstrap interpreter or for the source-level semantics.
- **Final emit bridges:** plain-`Expr` helpers and dead conversion helpers still
  exist.
- **Parse/infer boundary churn:** `type_expr_to_node` still appears at many
  parser boundaries; this goes away only when the parser is node-native end to
  end.

**Acceptance:**

- [ ] tokenizer builders have a proven linear builder path in bootstrap and
      native modes
- [ ] parser accumulators use an O(1) builder or a documented linear primitive
- [ ] Kahn/topological traversal no longer scans a flat edge list in the inner
      loop
- [ ] interpreted `list_push` semantics are fixed or explicitly removed from
      bootstrap-critical paths
- [ ] last bridge helpers are removed or documented as intentionally permanent

### S4: Restore v2 test and cargo baseline

**Current working-tree snapshot (2026-03-17):**

- `cargo test -p daglang-emit --quiet`: passes (361 tests)
- `cargo test -p v2-compiler-tests --quiet`: fails
  (85 passed, 4 failed, 9 ignored)
- `v2_crate_cargo_check`: passes
- `v2_crate_cargo_build`: passes
- `v2_crate_cargo_test`: still ignored and still open; the generated crate can
  reach its own heavy generated tests and then abort with a stack overflow

**Acceptance:**

- [ ] `cargo test -p daglang-emit --quiet` stays green
- [ ] `cargo test -p v2-compiler-tests --quiet` is green
- [ ] `v2_crate_cargo_check` passes
- [ ] `v2_crate_cargo_build` passes
- [ ] `v2_crate_cargo_test` no longer aborts with stack overflow in generated
      heavy tests
- [ ] multi-module synthetic and gist-adjacent tests are green

---

## Track R: Representation Sizing

Types must be proportional to their common case. If a rarely-populated
optional field dominates a type's size, every clone, every stack frame,
and every match arm pays for it — even when the field is `none`.

This track was added after the 2026-03-18 session discovered that `Node`
is 544 bytes (because it inlines `TransportBinding` at 184b and
`ServiceConfig` at 256b, both used by only 6% of nodes), making
`TypedExpr` 1,112 bytes, `infer_expr`'s closure frame 356KB, and
generated self-compile tests OOM after 13+ minutes.

The perf audit (Track S) caught algorithmic complexity (O(n²) builders,
linear scans) but missed representation sizing — a hidden constant-factor
multiplier on every operation.

### R1: Audit and catalog type sizes

Systematically measure the generated Rust type sizes for every `.dag`
type. Identify every type where rare optional fields account for >50%
of the total size.

**Method:**

- Add `static_assert!(std::mem::size_of::<Node>() <= 128)` style checks
  to the generated crate test harness
- Emit a size report as part of `v2_crate_cargo_check` that prints
  `size_of` for all generated types
- Flag any type exceeding a size budget (e.g., 256 bytes for types
  instantiated >100 times per module)

**Known violations (from 2026-03-18 investigation):**

| Type | Current size | Common-case size | Bloat source | Usage of bloat field |
|------|-------------|-----------------|--------------|---------------------|
| `Node` | 544b | ~80b | transport (184b) + config (256b) | 6% of nodes |
| `TypedExpr` | ~1,112b | ~200b | resolved_type: Node (544b) inline | 100%, but Node itself is bloated |
| `Param` | ~648b | ~100b | contains inline Node | 100% |
| `TypedNode` | ~600b | ~120b | same as Node + TypedExpr body | 100% |
| `ServiceConfig` | 256b | ~50b | 3 × `Expr?` (208b each) | ~30% populate all 4 fields |
| `TransportBinding` | 184b | 0-48b | RestBinding (328b) dominates enum | LocalBinding is 0b but pays 184b |

**Acceptance:**

- [ ] size report exists and runs as part of test baseline
- [ ] every type >256 bytes is either justified or has a boxing plan
- [ ] size assertions prevent silent regressions

### R2: Box rare fields on Node

The highest-impact single fix. Boxing `transport` and `config` on `Node`
shrinks it from 544 to ~120 bytes. This cascades:

- `TypedExpr`: ~1,112b → ~700b (resolved_type shrinks)
- `Param`: ~648b → ~200b
- `infer_expr` frame: 356KB → estimated <150KB
- Clone cost: every clone copies ~400 fewer bytes

**Scope:**

- Change `transport: TransportBinding?` to `transport: Box<TransportBinding>?`
  in generated Rust (v1 type_codegen.rs boxing algorithm)
- Change `config: ServiceConfig?` similarly
- Update all field access sites (`.transport` → `transport.as_ref()`)
- Verify generated crate compiles and tests pass
- Re-run gist compile tests to verify OOM is resolved

**Acceptance:**

- [ ] `Node` size ≤ 160 bytes
- [ ] `TypedExpr` size ≤ 800 bytes
- [ ] `v2_crate_cargo_test` passes without stack overflow or OOM
- [ ] `v2_compile_gist_rust` passes without host-side stack overflow
- [ ] size assertions from R1 enforce the new bounds

### R3: Clone reduction in v1 emitter

The v1 emitter has 650+ `.clone()` calls. Many defensively clone full
`Expr` (208b) or `Node` (544b) values where a borrow would suffice.

**Scope:**

- Audit clone sites in `daglang-emit/src/` for unnecessary copies
- Replace defensive clones with borrows where ownership isn't needed
- Focus on hot paths: `lower_rust.rs`, `fn_codegen.rs`, `render_rust.rs`

**Acceptance:**

- [ ] clone count reduced by ≥50% in emit pipeline
- [ ] no functional regressions
- [ ] measurable improvement in emit-phase memory and time

### R4: Interpreter value representation

The v1 interpreter's `list_push` clones the entire `Vec<Value>` on every
append, making repeated appends O(n²). This affects bootstrap
performance when the v2 compiler runs interpreted.

**Design options:**

- (A) Change `Value::List(Vec<Value>)` to `Value::List(Rc<Vec<Value>>)`
  with COW via `Rc::make_mut`
- (B) Use a persistent list structure (e.g., `im::Vector`)
- (C) Accept quadratic bootstrap; optimize only the emitted/native path

**Acceptance:**

- [ ] `list_push` is O(1) amortized in the interpreter, or
- [ ] quadratic behavior is explicitly accepted and documented for
      bootstrap-only paths

### R5: TCO clone leak in generated Rust code

The Rust emitter's TCO (tail call optimization) pattern emits
`let state = __tco_p_state.clone()` at the top of each loop iteration.
This bumps the `Rc` refcount on list fields, so when `list_push` calls
`Rc::try_unwrap`, refcount is >1 and it falls back to cloning the
entire `Vec`. Every token emission copies all prior tokens — O(n²)
even in the compiled native code.

This was discovered in the 2026-03-18 Track A investigation: the
compiled v2 crate's `tokenize_loop` grows at ~300MB/s and OOMs at
~10GB when processing 1,515 lines of gist sources.

**Root cause:** The emitter should `move` the TCO state instead of
cloning it, ensuring `Rc::try_unwrap` succeeds in-place.

**Scope:**

- Fix the TCO loop pattern in `05_emit_rust.dag` (and potentially
  `05_emit_python.dag`, `05_emit_go.dag`) to consume the state
  rather than cloning it
- Verify `Rc::try_unwrap` succeeds (refcount == 1) in the hot path
- Re-run gist compile tests

**Acceptance:**

- [ ] TCO loops do not clone `Rc`-wrapped state at iteration start
- [ ] `list_push` in TCO functions is O(1) amortized (no fallback clone)
- [ ] tokenizer processes 1,515 lines in <1s (currently OOMs)
- [ ] gist pipeline completes within reasonable time/memory bounds

---

## Track A: Pipeline Validation -> Bootstrap -> Self-hosting

**Blockers:**

- **Track S** must be complete enough for the v2 compiler to be a
  trustworthy target again.
- **Track R** (at least R2) must land before A1+. The generated crate
  and host interpreter both OOM/overflow on gist-scale inputs due to
  `Node` being 544 bytes with pervasive cloning.

### A1: Gist compilation

Feed `gist.dag` and its transitive dependencies through the v2 pipeline. Verify
that emitted code compiles in each target language.

**Acceptance:**

- [ ] `v2_compile_gist_rust`: v2 compiles gist -> Rust -> `cargo check`
- [ ] `v2_compile_gist_python`: v2 compiles gist -> Python -> `py_compile`

### A2: Runtime bridge

Generate entry point and runtime dependencies so the compiled gist executes.

**Acceptance:**

- [ ] generated `main.rs` + `Cargo.toml` with runtime deps
- [ ] `cargo run -- gist --dry-run` produces correct dry-run output
- [ ] Python equivalent produces the same dry-run output

### A3: Gist end-to-end execution

**Acceptance:**

- [ ] compiled Rust gist creates a real GitHub gist (manual gate)
- [ ] compiled Python gist creates a real GitHub gist (manual gate)

### A4: Full self-compile pipeline

Extend the current self-compile path from tokenize/parse/resolve to the full
tokenize/parse/resolve/infer/emit pipeline.

**Current caveat:** the generated `self_compile_all_modules` ratchet is still a
bootstrap smoke test. It currently uses lenient compilation and only requires
that at least one emitted file be non-empty, which is useful for progress
tracking but too weak to serve as a long-term semantic acceptance gate.

**Acceptance:**

- [ ] v2 crate processes its own `.dag` source through the full pipeline
- [ ] emitted Rust files compile with `cargo check`
- [ ] no OOM or stack overflow on any `.dag` file up to 4000 lines
- [ ] generated self/gist runtime tests do not introduce stack overflows that
      are absent from the corresponding host-side pipeline
- [ ] self-compile ratchet asserts semantic properties stronger than
      "non-empty file emitted"

### A5: Bootstrap stage 0 -> 1

```text
v1 compiles v2 .dag -> Rust -> rustc -> v2-stage0
v2-stage0 compiles v2 .dag -> Rust -> rustc -> v2-stage1
```

**Acceptance:**

- [ ] `v2-stage1` builds successfully
- [ ] `v2-stage1` passes the same test suite as `v2-stage0`

### A6: Fixed point

```text
v2-stage1 compiles v2 .dag -> Rust -> rustc -> v2-stage2
```

**Acceptance:**

- [ ] `stage1` output == `stage2` output

### A7: v1 retirement

Once the fixed point holds, v1 is bootstrap scaffolding rather than the active
compiler path.

**Acceptance:**

- [ ] v2 builds and tests without v1 in the dependency chain
- [ ] v1-only heuristics are dead code
- [ ] interpreter/evaluator is optional, not required for the compiler

---

## Track B: Node Convergence

Structural unification: one type (`Node`) flows through the pipeline.

### B1: TypeExpr -> Node

Dissolve the remaining `TypeExpr`-specific structure into `Node` patterns. This
track is no longer a speculative dual-write plan; the migration is already
partially landed.

**Already true:**

- `Field.type_expr` is `Node`
- `Param.type_expr` is `Node`
- `ResourceUse.resource` is `Node`
- `TypedExpr.resolved_type` is `Node`
- `FuncSig.return_type` is `Node`
- `TypedNode.return_type` is `Node`
- Rust/Python/Go emitters have node-based type readers

**Still remaining:**

- parser still produces and consumes `TypeExpr` internally in many places
- infer and emit still contain mixed `Node` / `TypeExpr` assumptions
- `type_expr_to_node` / `node_to_type_expr` bridging is not yet fully faithful;
  predicate conversion still drops information for some cases such as `Range`
- `TypeExpr` still exists as a real downstream representation instead of being
  strictly parse-local or fully deleted

**Acceptance:**

- [x] `Node`-typed fields on `Field` / `Param` / `ResourceUse`
- [x] `Node`-typed resolved types on `TypedExpr` / `FuncSig` / `TypedNode`
- [x] node-based emit type readers
- [ ] parser boundary becomes node-native without `type_expr_to_node` spray
- [ ] infer and emit are internally consistent on `Node` and boxed option types
- [ ] bridge conversions preserve predicate payloads rather than collapsing them
      to lossy placeholders
- [ ] `TypeExpr` is either deleted or reduced to a strictly parse-local form
- [ ] generated v2 crate and tests stay green

### B2: Rename typecheck -> infer

After convergence, the phase completes a node graph rather than checking a
separate parallel type model.

**Acceptance:**

- [x] `04_typecheck.dag` -> `04_infer.dag`

### B3: Expr -> Node

Dissolve `Expr` and the `Typed*` family into node patterns. After this, "typed"
just means "return_type is filled in."

**Current state:** typed emit hot paths landed, but the pipeline still carries
both `Expr` and `TypedExpr`.

**Acceptance:**

- [ ] `Expr` type deleted from `00_core.dag`
- [ ] `Typed*` family deleted
- [ ] `typed_expr_to_expr` deleted
- [ ] transport/config/mock literal handling is node-native or isolated behind a
      deliberate boundary type
- [ ] pipeline shape is `Node -> Node -> Node -> TextFile`

### B4: Transport dissolution

`transport: Node?` stays, but `TransportBinding` should eventually dissolve.
Transport behavior should come from structure rather than a fixed enum.

**Acceptance:**

- [ ] `TransportBinding` enum deleted
- [ ] emitters derive transport behavior from node structure
- [ ] `transport != none` is the only hardcoded transport knowledge

---

## Track C: Language Emission As Extdeps

Languages are external systems with specifications. They belong in extdeps and
should be modeled the same way other external systems are modeled.

### C1: Define LanguageSpec interface

**Acceptance:**

- [x] `LanguageSpec` defined in `dsl/std/languages.dag`
- [x] covers type mappings, naming, syntax patterns, runtime ops, error model,
      async model, import system
- [x] full compositions for Rust, Go, and Python

### C2: Rust, Python, and Go language extdeps

**Acceptance:**

- [x] language extdeps in `dsl/extdeps/languages/`
- [x] runtime ops captured per language
- [x] per-language `types.dag`, `runtime.dag`, `imports.dag`, `errors.dag`,
      `async.dag`

### C3: Emitters consult extdeps

**Current state:** type maps, container templates, and keywords are centralized
in `05_emit.dag`, but still inline rather than imported from language extdeps.

**Acceptance:**

- [x] type/keyword/container data centralized
- [ ] emitters import from language extdeps instead of inline data
- [ ] adding a new target means writing an extdep, not editing compiler logic
- [ ] emitted code remains identical for existing tests

### C4: CLI target selection

**Current state:** `RenderTarget = Rust | Python | Go` exists and pipeline
dispatches on it. No CLI flag yet.

**Acceptance:**

- [x] `RenderTarget` enum and pipeline dispatch
- [ ] `--target` CLI flag
- [ ] target selection loads the appropriate language extdep

---

## Track D: Runtime Complexity Analysis

The compiler should be able to prove upper bounds over DAG execution structure.
This is a static analysis problem over authoritative graph structure, not a
benchmarking feature.

### Core model

Use a small symbolic cost algebra rather than hard-coded big-O labels.

```text
ComplexitySummary {
  work: CostExpr
  span: CostExpr
  output_size: Map<Port, CostExpr>
  assumptions: List<Constraint>
  certainty: Proven | Conservative | Unknown
}
```

Two metrics are first-class:

- `work`: total operations performed
- `span`: critical path length under the DAG dependency structure

Support facts:

- symbolic size variables (`n`, `m`, `k`)
- explicit assumptions and loop bounds
- `Unknown` when the system cannot yet prove a tighter bound

### Reference design

Keep the program IR and the cost IR separate. The analyzer should walk the
authoritative program structure (`TypedExpr` today, `Node` after convergence)
and produce symbolic cost terms instead of trying to reuse program nodes as
proof objects.

```text
SizeExpr
  = Const(Int)
  | Var(String)
  | Len(String)
  | Add(SizeExpr, SizeExpr)
  | Max(SizeExpr, SizeExpr)

CostExpr
  = Const(Int)
  | Add(CostExpr, CostExpr)
  | Mul(CostExpr, CostExpr)
  | Max(CostExpr, CostExpr)
  | Sum { binder: String, upper: SizeExpr, body: CostExpr }
  | PrimCost { op: String, args: List<SizeExpr>, model: CostModelRef }
  | Unknown { reason: String }

Constraint
  = Eq(SizeExpr, SizeExpr)
  | Leq(SizeExpr, SizeExpr)
  | NonNegative(SizeExpr)

SemanticsCtx {
  backend: BootstrapInterp | LoweredRust | LoweredGo | LoweredPython
  exec_model: Sequential | Parallel
  list_model: PersistentList | RcVec
  map_model: TreeMap | HashMapExpected
  string_model: FlatString | RopeLike
}
```

Variable costs should be represented as symbolic primitive rules, not collapsed
into constants and not treated as "unknown" by default. The important question
is "cost as a function of what size under which runtime model?"

Reference examples:

```text
PrimCost("list_push", [n], BootstrapInterp/PersistentList) = n + 1
PrimCost("list_push", [n], LoweredRust/RcVec) = 1   // amortized
PrimCost("concat", [a, b], PersistentList) = a + b
PrimCost("reverse", [n], any) = n
PrimCost("map_insert", [n], TreeMap) = log(n)
PrimCost("map_insert", [n], HashMapExpected) = 1    // expected
```

This is the core reason the analysis should be parameterized by
`SemanticsCtx`: the same surface operation can have different faithful costs in
the bootstrap interpreter and in emitted/native code.

The analysis should also distinguish source-level forms from lowering-time
rewrites. For example, emitted/native code can fuse `concat(acc, [x])` into an
append-like block even when the bootstrap interpreter still pays value-copy
costs for the same surface pattern.

### Current motivating examples

Tokenizer string builders should be expressible exactly enough to preserve the
real lower-bound question.

```text
W_scan_string_body(L) = Sum(i = 0 .. L - 1,
  Const(c_scan) + PrimCost("list_push", [i], current_model)
)
```

Under O(1) append this simplifies to linear work. Under a persistent-list
model where append clones the prior list, it stays quadratic.

Parser accumulator builders should similarly remain symbolic rather than being
prematurely labeled "linear."

```text
W_parse_items(n) = Sum(i = 0 .. n - 1,
  item_cost(i) + PrimCost("concat", [Const(1), i], current_model)
) + PrimCost("reverse", [n], current_model)
```

Typed emit is the positive example: once the pipeline reads resolved type
structure directly, the formulas become simpler and more local. For an
expression tree `E`:

```text
W_emit(E) =
  Sum(v in nodes(E), tag_cost(v))
  + Sum(call in calls(E), order_args_cost(call))
```

The reporting rule should be: compute an exact symbolic formula first, then
derive asymptotic summaries from that formula. Do not discard representation or
backend assumptions early.

### D1: Cost algebra and proof vocabulary

Define `SizeExpr`, `CostExpr`, `Constraint`, `SemanticsCtx`, and
`ComplexitySummary` in the IR layer.

**Acceptance:**

- [ ] symbolic cost algebra exists in the IR layer
- [ ] primitive cost rules are parameterized by `SemanticsCtx`
- [ ] summaries can represent `Add`, `Mul`, `Max`, bounded sums, and `Unknown`
- [ ] proof vocabulary is small and backend-independent at the algebra level
- [ ] exact symbolic formulas can be rendered before asymptotic simplification

### D2: Typed summaries for v2 expressions and functions

Infer symbolic summaries from typed expressions/functions before lowering. Until
Track B3 lands, the analyzer walks `TypedExpr`; after B3, the same transfer
rules should attach directly to node-native bodies.

**Planned semantics:**

- straight-line code: additive work, additive span
- branch: conservative `max`
- recursion: require a decreasing measure or return `Unknown`
- primitive collection ops: summarize through `PrimCost(...)` with size
  variables, not hard-coded constants
- collection traversals: summarize in terms of input cardinality and callee
  summary

**Acceptance:**

- [ ] per-function complexity summaries exist for typed v2 items
- [ ] summaries carry backend/representation assumptions explicitly
- [ ] summaries mention symbolic input sizes rather than concrete values
- [ ] unsupported recursion fails closed with `Unknown`

### D3: DAG composition and pattern semantics

Compose summaries over the lowered DAG and over pattern nodes such as loop and
retry.

**Planned semantics:**

- DAG work = sum of node work
- DAG span = longest dependency path
- loop work = iteration count * body work
- retry work/span = bounded by retry policy

**Acceptance:**

- [ ] acyclic DAG complexity composition exists
- [ ] loop and retry patterns have explicit transfer rules
- [ ] output cardinality/size summaries flow through pattern boundaries

### D4: Proofs and reporting

Surface complexity as a proof/report, not just an internal calculation.

Integration points:

- workflow proof surfaces
- compiler diagnostics / reports
- optional policy checks ("prove this workflow is bounded under these inputs")

**Acceptance:**

- [ ] complexity proof/reporting entrypoint exists
- [ ] users can request work/span summaries for a workflow or compiled item
- [ ] reports can show both symbolic formulas and simplified asymptotic forms
- [ ] policy checks can reject unbounded or unknown-critical workflows when
      configured to do so

---

## Track E: Artifact Planning And Boundary Semantics

Language facts are only part of the story. Once one program can emit to
multiple targets, the compiler also needs to know how the program is partitioned
into artifacts and what semantics hold across target boundaries.

**Current state:** the pipeline still assumes a mostly monolithic artifact
model. Target selection is per compile, not per subgraph, and transport/boundary
behavior is not yet planned as a first-class compilation product.

**Guiding use case:** define an end-to-end web stack in one `.dag` graph:

- backend services and workers
- frontend application code
- HTML/CSS/UI generation
- middleware and shared API boundaries
- cloud/deployment infrastructure

The long-term goal is not just "many emitters." It is one graph that can be
partitioned into multiple artifacts, placed onto multiple targets, and emitted
with explicit contracts between those pieces.

### E0: Wrap the current monolith explicitly

Do this first. The compiler already behaves as though there is one artifact; the
next step is to make that shape explicit instead of implicit.

The purpose is not to solve multi-artifact planning immediately. It is to create
the wrapper that later composition can build on without changing the meaning of
today's pipeline.

Sketch:

```text
artifact default {
  kind: ServiceBinary
  target: Rust
  contents: [root graph]
}
```

This should act as a compatibility layer:

- current single-artifact compilation becomes an explicit default plan
- later multi-artifact partitioning becomes composition rather than redesign
- artifact-level metadata has a home before deployment concerns sprawl across
  unrelated node properties

**Acceptance:**

- [ ] current monolithic compilation is representable as an explicit artifact
      wrapper
- [ ] introducing the wrapper does not change generated output for the current
      monolithic path
- [ ] artifact metadata has a dedicated structural home before boundary planning
      expands

### E1: Artifact model

Make deployable/buildable units explicit in `.dag` instead of treating the
current single emitted crate/module set as the permanent model.

Examples:

- binaries
- libraries
- services
- frontends / bundles
- firmware or bare-metal outputs
- generated support artifacts (OpenAPI, schemas, stubs, manifests)

**Acceptance:**

- [ ] an explicit artifact model exists in the IR / source language
- [ ] the current monolithic output is represented as a default artifact plan,
      not a hardcoded special case
- [ ] the compiler can emit multiple artifact plans from one source graph

### E2: Target placement and graph partitioning

Allow target choice to be attached to nodes or subgraphs rather than only to the
entire compile invocation.

This is the step that makes "this subgraph emits to Rust, that one to
TypeScript, that one to MIPS" a principled compilation problem instead of an ad
hoc emitter switch.

**Acceptance:**

- [ ] target placement can be declared in `.dag`
- [ ] the compiler can partition a graph into same-target regions
- [ ] invalid placements fail closed with clear diagnostics
- [ ] target selection can still be overridden at the top level when desired

### E3: Boundary semantics and shared transports

Cross-target edges need explicit boundary semantics. A direct in-process call, a
shared HTTP server, an RPC boundary, and a file/protocol handoff are different
things and should not be inferred from syntax alone.

Examples:

- direct call
- HTTP/JSON
- shared REST server hosting multiple DAG boundaries
- message queue / event stream
- FFI / ABI boundary
- file or manifest protocol
- browser/server bridge

**Acceptance:**

- [ ] boundary kinds are first-class rather than hidden in emitter heuristics
- [ ] shared transport hosts (for example one HTTP server serving multiple DAG
      boundaries) can be represented explicitly
- [ ] adapters/stubs/serialization code are emitted from boundary facts
- [ ] cross-target edges without a valid boundary contract fail closed

### E4: Planning and reporting

Artifact and boundary planning should be inspectable, not implicit. The compiler
should be able to tell the user what artifacts it will build, which targets they
use, and how boundaries are realized.

**Acceptance:**

- [ ] the compiler can emit an artifact plan report
- [ ] reports show partitions, targets, boundaries, and generated support code
- [ ] artifact planning is available as a dry-run / proof surface before codegen
- [ ] deployment-facing facts (ports, package names, manifests, server sharing)
      are derived from explicit structure

---

## Consolidated Backlog From The Former Sustainability Ledger

These items remain real, but they are lower priority than Tracks S/A/B/C/D.

- **Anonymous record target resolution:** stop guessing nominal record targets
  in v1 codegen; ambiguous cases must fail closed.
- **Collection intrinsic semantics in shared IR:** collection operations should
  be explicit shared semantics, not duplicated across evaluator/codegen/emitter.
- **Generated self-hosting tests and stage contracts:** strengthen stage-level
  assertions and generated integration coverage.
- **TCO backend contract:** keep explicit tail-position analysis/rendering
  contracts; no silent partial fallback.
- **Embedded source metadata:** preserve source origin through lowering and
  emission so diagnostics stay anchored in user code.

---

## The Fully Converged Node

After Tracks B and C complete:

```dag
type Connective = Conj | Disj

type Node {
  name: String
  span: SourceSpan
  children: List<Node>
  connective: Connective?
  params: List<Node>
  return_type: Node?
  body: Node?
  transport: Node?
  properties: List<Node>
}
```

### Why each field is irreducible

| Field | Logical role | Why separate |
|-------|--------------|--------------|
| `children` + `connective` | Composition | The core primitive |
| `params` | Obligations | Consumed, not composed |
| `return_type` | Guarantee | Flows out, not in |
| `body` | Proof / computation | How, not what |
| `transport` | I/O grounding | Must remain structural |
| `properties` | Extensible metadata | Domain facts |

### The irreducible kernel

Only three things cannot themselves be ordinary nodes:

1. `Node` itself
2. `Connective = Conj | Disj`
3. kernel primitives such as `String`, `Int`, `Bool`, `List`, `Map`

Everything else is composition.

### Pipeline

```text
source -> parse -> resolve -> infer -> emit
           |        |         |        |
         Nodes    Nodes     Nodes    TextFiles
          raw     linked    typed
```

One type flows through the pipeline; each phase enriches it rather than
translating into a parallel representation.

---

## The End State

- self-hosted
- structurally unified
- compositional
- target-polymorphic
- artifact-aware
- bootstrap-free
- fixed-point reproducible
- complexity-analyzable in DAG-native terms (`work` and `span`)

---

## Non-goals

- deleting surface keywords from the language
- expanding strings to bits at compile time
- forcing all backends through one template-only renderer
- pretending exact runtime prediction is possible when only conservative static
  bounds are justified
