# v4 rustc error catalog → class-fix table — 2026-05-29

**Work item:** `node://adhoc-ef5c50f4-c22` (v4-close diagnosis lane)  
**Session:** `sunny-cat-359` @ `f2c7f92556a6cf302f12cada2e0aa59753e7add5`  
**Authority:** point-in-time probe snapshot; rustc population from live `scripts/v4-m1-rust-emit-probe.sh` + `scripts/v4-testclaim-corpus-eval.sh` (T-38-PR1 scaffold). Dissolves when M1 emitter lane closes the cited classes and T-38-PR2 replaces scaffold receipts with per-row verdicts.

## Repro (interim shell artifacts)

```bash
# build v2-compiler (bypass ctrl-build shims if needed)
PATH=/opt/cargo/bin:$PATH CARGO_BUILD_JOBS=4 cargo build -p v2-compiler --release

export V2_COMPILER=target/release/gunbc
export V4_M1_RUST_EMIT_OUT=/tmp/v4-rust-emit-probe
export V4_M1_CARGO_CHECK_JOBS=4
bash scripts/v4-m1-rust-emit-probe.sh          # → ${OUT}.m1-probe-summary.txt, .rustc.log, .compile.log

export V4_TESTCLAIM_CORPUS_OUT=/tmp/v4-testclaim-corpus-eval
bash scripts/v4-testclaim-corpus-eval.sh       # T-38-PR1 JSON receipt (requires 0 v2 diagnostics)
```

**Captured on this session:** `/tmp/v4-rust-emit-sunny-cat-359.{m1-probe-summary.txt,compile.log,rustc.log}`

---

## §1 Runner receipts (T-38-PR1 + M1)

| Probe | Result | Notes |
| --- | --- | --- |
| `v4-testclaim-corpus-eval.sh` (T-38-PR1) | **FAIL** (24 v2 diagnostics) | Does not reach structural witness / `blocked_m1_subset` JSON — requires `compiled: N files emitted, 0 diagnostics` |
| `v4-m1-rust-emit-probe.sh` (M1 informational) | **PARTIAL** | v2 compile exit 0 with **24** diagnostics; **295** `.rs` emitted; `cargo check` exit 101 |

### v2 compile diagnostics (blocks T-38 0-diagnostic scaffold)

All **24** are `complexity: same-argument recursion in ci_*_projection_node` under:

- `src/v4/workflow/ci.dag` (16 sites)
- `src/v4/test/claim/workflow/pipeline_rejections.dag` (8 sites)

**Class-fix:** T-22 complexity / memoization lane (`ci_int_offset_authority_projection_node` family) — not M1 rustc; must clear before T-38-PR1 CI receipt is honest.

---

## §2 rustc population (full `src/v4` tree, v2 emitter)

| Metric | Current (2026-05-29) | Prior baseline (`docs/v4-compilation-milestones.md` / PR #3654) |
| --- | ---: | ---: |
| `rustc` `error[E####]` lines | **7951** | ~4900 |
| Distinct emitted `.rs` files with errors | **262** / 294 | — |
| Top code | **E0423** (2978) | **E0282** (~2125), **E0107** (~792), **E0308** (~669) |

**v4-emit vs v2-emit diff (profile):** both paths use the **same v2 emitter** (`gunbc compile --target rust`). The diff is **corpus-induced**: `src/v4` modeling stresses constructs v2's dsl-era corpus did not (generics-heavy `Outcome`/`Refined`/`TestClaimRun`, cross-module `Symbol` atoms, lens/CI graphs). Error count rose ~62% vs #3654 snapshot; **dominant failure mode shifted** from annotation/arity (E0282/E0107) to **value-vs-type emission** (E0423: `Symbol` treated as callable).

A standalone `src/v2` / `dsl/` rust emit baseline is **not cleanly runnable** at HEAD (circular module graph / parse errors) — population comparison is **historical probe vs current full-tree v4 emit**, not a second live crate.

---

## §3 Per-row evidence catalog (routing table)

Full population: **7951** `error[E####]` lines, **262** / 294 emitted files with errors. Probe: `f2c7f925` @ 2026-05-29. Regenerate: `bash scripts/v4-m1-rust-emit-probe.sh` (§Repro).

Columns: **class** | **rust_error_code** | **count** | **v4_emit_excerpt** (primary site) | **v2 oracle** | **minimal repro** (`.dag` → emitted `.rs`).

| class | rust_error_code | count | v4_emit_excerpt | v2 oracle | minimal repro |
| --- | --- | ---: | --- | --- | --- |
| SG-1 | `E0423` expected fn/variant, found type alias `Symbol` | 2978 | `pub fn loop_bound_edge() -> String { Symbol("loop_bound_edge".to_string()) }` @ src/v4_std_node.rs:125 | `dsl/extdeps/languages/rust/types.dag:41` — `Symbol` → `String`, `literal_suffix: ".to_string()"` (value, not ctor) | `src/v4/std/node.dag` `data loop_bound_edge: Symbol = loop_bound_edge` |
| SG-3 | `E0308` mismatched types | 1191 | `pub fn feature(&self) -> Feature {` @ src/v4_extdeps_languages_fidelity.rs:27 | `dsl/extdeps/languages/rust/emit.dag` realization row vs emitted signature | Emit site under `src/v4/extdeps/**` or `src/v4/std/**` |
| SG-2 | `E0282` type annotations needed | 743 | `bounded_lattice: Rc::new(BoundedLattice {` @ src/v4_std_logic.rs:57 | Incomplete emitted closures / missing generic context (often post-SG-2) | `src/v4/std/logic.dag` `BoundedLattice` meet/join closures |
| SG-2 | `E0107` missing generics for enum `Outcome` | 578 | `pub type FileReadResult = Rc<Outcome>;` @ src/v4_std_diagnostic.rs:115 | `src/v4/std/diagnostic.dag` — `Outcome<T>` must emit type args | `fn outcome_accepted<T>(value: T) -> Outcome<T>` → emitted `Rc<Outcome>` |
| SG-8 | `E0433` failed to resolve: use of undeclared type `EdgeLabel` | 193 | `label: Rc::new(EdgeLabel::Named {` @ src/v4_std_effects.rs:351 | Emitted child type not in scope — module graph | `src/v4/std/effects.dag` `EdgeLabel::Named` |
| SG-8 | `E0432` unresolved import `crate::v4_std_node::NodeRef` | 162 | `pub use crate::v4_std_node::{..., NodeRef, ...};` @ src/v4_std_algebra.rs:10 | `src/v2/05_emit_rust.dag` crate `pub use` graph | `src/v4/std/algebra.dag` imports `NodeRef` |
| SG-3 | `E0573` expected type, found variant `String` | 159 | `pub element_type_ref: String,` @ src/v4_extdeps_languages_go.rs:152 | Dag type name where Rust expects a type (extdeps tables) | `src/v4/extdeps/languages/go.dag` language table rows |
| SG-2 | `E0107` missing generics for enum `Witness` | 113 | `pub type BootstrapRoundtripCheck = Rc<Witness>;` @ src/v4_std_witness.rs:16 | Generic carrier — emit type args | `src/v4/std/witness.dag` |
| SG-2 | `E0107` missing generics for struct `TestClaimRun` | 104 | `pub fn test_claim_run_claim<S, A>(run: Rc<TestClaimRun>)` @ src/v4_compiler_eval.rs:246 | Generic carrier — emit type args | `src/v4/std/verification.dag` `data TestClaimRun<C>` |
| SG-8 | `E0425` cannot find type `TargetModelBundle` in this scope | 95 | `-> Rc<TargetModelBundle>` @ src/v4_extdeps_languages_cpp.rs:1031 | `dsl/extdeps/languages/rust/emit.dag` | `src/v4/extdeps/languages/cpp.dag` |
| SG-2 | `E0107` missing generics for struct `TestClaimEvalSubject` | 83 | `subject: Rc<TestClaimEvalSubject>` @ src/v4_compiler_eval.rs:220 | Generic carrier — emit type args | `src/v4/std/verification.dag` |
| SG-2 | `E0107` missing generics for enum `FreeMonoid` | 71 | (import/ctor site) @ src/v4_std_algebra.rs:116 | Generic carrier — emit type args | `src/v4/std/algebra.dag` `FreeMonoid<T>` in eval fold |
| SG-8 | `E0432` unresolved import `crate::v4_std_text::GoScalarKind` | 70 | `pub use crate::v4_std_text::{GoScalarKind};` @ src/v4_compiler_target_carriers.rs:11 | Module graph | `src/v4/compiler/target_carriers.dag` |
| SG-8 | `E0432` unresolved import `crate::v4_std_collection::List` | 63 | `pub use crate::v4_std_collection::{List};` @ src/v4_compiler_self_host.rs:10 | Module graph | cross-module `List` import |
| SG-8 | `E0425` cannot find type `Char` in this scope | 63 | `pub type TargetSource = FreeMonoid<Char>;` @ src/v4_compiler_target_carriers.rs:16 | Map `Char` → `char` in rust realization | `src/v4/compiler/target_carriers.dag` |
| SG-2 | `E0107` missing generics for struct `Validation` | 61 | `-> Rc<Outcome>` in `refine` @ src/v4_std_refinement.rs:21 | Generic carrier — emit type args | `src/v4/std/refinement.dag` |
| SG-3 | `E0277` trait bound `OpenApiSpecificationExtensionKey: Eq` not satisfied | 45 | @ src/v4_extdeps_formats_openapi.rs:154 | Missing `Eq`/`Hash` on emitted carrier | `src/v4/extdeps/formats/openapi.dag` |
| SG-3 | `E0277` trait bound `OpenApiSpecificationExtensionKey: Hash` not satisfied | 45 | @ src/v4_extdeps_formats_openapi.rs:154 | Missing `Eq`/`Hash` on emitted carrier | `src/v4/extdeps/formats/openapi.dag` |
| SG-2 | `E0107` missing generics for struct `ClassifiedDependencyView` | 40 | `classify_dependency_view<C>` @ src/v4_std_dependency.rs:56 | Generic carrier — emit type args | `src/v4/std/dependency.dag` |
| SG-2 | `E0107` missing generics for enum `Verdict` | 40 | `verdict_combine<T>` @ src/v4_std_verdict.rs:20 | Generic carrier — emit type args | `src/v4/std/verdict.dag` |
| SG-3 | `E0308` arguments to this function are incorrect | 35 | `leap_second_insertion_date(...)` @ src/v4_std_datetime.rs:316 | Realization / arity | `src/v4/std/datetime.dag` |
| SG-8 | `E0425` cannot find type `K` / `V` in this scope | 68 | `v2_rt::rc_empty_map::<K, V>()` @ src/v4_compiler_parse.rs:662 | Emit must bind fn type params | `src/v4/compiler/parse.dag` |
| SG-8 | `E0433` failed to resolve: `PointwisePower` | 28 | `Rc::new(PointwisePower::Set {` @ src/v4_lens_subsumption.rs:72 | Module graph | `src/v4/lens/subsumption.dag` |
| SG-3 | `E0369` binary op `<=` on `Rc<Nat>` | 24 | `(value <= 9999)` @ src/v4_std_datetime.rs:368 | `Nat` carrier needs operator impl | `src/v4/std/datetime.dag` |
| SG-3 | `E0560` struct field missing (`TargetModelBundle`) | 94 | `bundle:` / `lex:` @ src/v4_extdeps_languages_cpp.rs:1033 | Struct literal vs `.dag` decl | `src/v4/extdeps/languages/cpp.dag` |
| SG-8 | `E0433` undeclared type `FreeMonoid` | 22 | `FreeMonoid::Empty` @ src/v4_std_diagnostic.rs:408 | Import hygiene (SG-4) | `src/v4/std/diagnostic.dag` |
| SG-3 | `E0121` placeholder `_` in item signature | 22 | `BTreeSet<_>` @ src/v4_std_logic.rs:58 | Emit must infer set element type | `src/v4/std/logic.dag` |
| SG-2 | `E0107` (remaining carriers, count &lt; 40 each) | 127 | e.g. `Rc<Generator>`, `Rc<Optional>`, `Rc<NodeFold>` | Per-row generic in substrate | See `src/v4/std/**`, `src/v4/lens/**` |
| SG-7 | `v2-diagnostic` complexity: same-argument recursion | 24 | `fn ci_int_offset_authority_projection_node(i: Int) -> Node {` @ src/v4/workflow/ci.dag:721 | v2 `complexity` pass — **not rustc** | `ci_int_offset_authority_projection_node` self-call |
| SG-5 | `compile_error!` Set not Ord-eligible for BTreeSet | 7 | `compile_error!(...DiffId...BTreeSet)` @ src/v4_lens_subsumption.rs:67 | `src/v2/05_emit_rust.dag:295` Ord whitelist | `src/v4/lens/subsumption.dag` `Set<DiffId>` |
| SG-6 | `compile_error!` BoundedLattice missing meet/join | 8 | `compile_error!(...meet...)` @ src/v4_lens_cost.rs:684 | Substrate lattice instance incomplete | `src/v4/lens/cost.dag` |

*Full E0107 sub-rows (Outcome 578, Witness 113, TestClaimRun 104, …) are in the table above; 30 additional E0107 variants with count &lt; 40 are rolled into the summary row — re-run the probe parser to expand if dispatch needs each carrier split.*

### §3.1 Code histogram (rollup)

| Code | Count | Share |
| --- | ---: | ---: |
| E0423 | 2978 | 37.5% |
| E0308 | 1226 | 15.4% |
| E0107 | 1219 | 15.3% |
| E0282 | 747 | 9.4% |
| E0432 | 359 | 4.5% |
| E0425 | 332 | 4.2% |
| E0277 | 330 | 4.2% |
| E0433 | 259 | 3.3% |
| other | 960 | 12.1% |

---

## §3.2 M1 probe vs T-38 — SG-7 is **not** a gate on the M1 iteration meter

| Path | Requires `0` v2 diagnostics? | Requires `compile` exit 0? | Runs `cargo check`? | Use for post-fix residual count? |
| --- | ---: | ---: | ---: | --- |
| **`scripts/v4-m1-rust-emit-probe.sh`** | **No** (24 diagnostics OK at HEAD) | Yes | Yes → **7951** rustc lines | **Yes — primary iteration meter** |
| **`scripts/v4-testclaim-corpus-eval.sh`** (T-38-PR1) | **Yes** | Yes | No (stops before runtime) | No — blocked until SG-7 clears |

**Answer for dispatch:** use **M1** to measure rustc residual after SG-1 / SG-2 / SG-5 / SG-6 land. **SG-7 does not need to be first** for that loop. SG-7 only blocks (a) T-38-PR1’s `0 diagnostics` receipt and JSON scaffold, and (b) any workflow that insists on a clean v2 compile log before emit. Regenerating **this catalog** is `bash scripts/v4-m1-rust-emit-probe.sh` + probe-log parse — independent of T-38.

**Recommended dispatch order (PM iteration):** **SG-1** (Pareto) → **SG-2** → SG-4/8 → SG-5/6 → SG-3 mop-up; **SG-7 in parallel** when T-38 scaffold / zero-diagnostic CI is needed, not as a prerequisite for M1 ratchet.

---

## §4 Top substrate-gap classes → class-fix table

Eight classes ordered by **actionable M1/v4-close leverage** (substrate vs emitter ownership per `INVARIANTS.md` / `docs/v4-compilation-milestones.md` M1 notes).

| # | Class | Pop. | Owner | Fix posture (do **not** cement hand-Rust in templates) |
| --- | --- | ---: | --- | --- |
| **SG-1** | **Symbol/Atom value emission** — `Symbol` identifiers emitted where Rust expects fn/variant | 2978 (E0423) | **M1 v2 `emit_rust`** — call/variant lowering for `Atom`/`Symbol` nodes | Teach emitter: `Symbol` data → const / enum variant / zero-arg fn per `LanguageSpec` realization row; add v4 TestClaim round-trip once T-38 eval lands |
| **SG-2** | **Generic arity on modeled carriers** — `Outcome<T>`, `Refined<B>`, `TestClaimRun<C>`, … | 1219 (E0107) + 747 (E0282) | **M1 v2 emit + std typing** | Emit type arguments from inferred `TypeNode`/`Refined` metadata; substrate already models carriers — emitter must project arity from infer facts, not default raw enums |
| **SG-3** | **Type mismatch / trait bound** — wrong `Rc` nesting, missing `Clone`, wrong arm types | 1226 (E0308) + 330 (E0277) | **M1 v2 emit** | Fix per-module after SG-1/2; many are cascade from wrong `Symbol` call shape |
| **SG-4** | **FreeMonoid import hygiene + parametric `Char`** | 56+ (E0252/E0425/E0433) | **M1 v2 module layout** | Dedupe `pub use` in crate root; map `Char` → `char` / single monoid import path; resolves `eval` fold accumulator types |
| **SG-5** | **Ord-eligible `Set` carrier** — `BTreeSet` on non-`Ord` type params | 12 `compile_error!` stubs | **Substrate** (`std` set model / T-25+) | Model `Set<T>` with `Ord` constraint or use `HashSet` realization in `rust.dag`; remove emit-time `compile_error!` placeholders |
| **SG-6** | **BoundedLattice partial instances** — missing `meet`/`join` | 8 `compile_error!` stubs | **Substrate** (`lens/cost.dag` + algebra) | Complete `BoundedLattice` instances in `.dag` or narrow generated impl arms before emit |
| **SG-7** | **v2 complexity gate (same-arg recursion)** | 24 v2 diagnostics | **T-22 / compiler complexity** | Memoize or rewrite `ci_*_projection_node` — **blocks T-38-PR1** zero-diagnostic receipt |
| **SG-8** | **Import / module graph (`CarrierKind`, unresolved `crate::`)** | 618 (E0432/E0433/E0425) | **M1 emit module graph** | Stabilize `v4_compiler_target_carriers` re-exports; often clears after SG-1/4 |

### T-38-PR2 wedge note (from `v4-testclaim-corpus-eval.sh` header)

Runtime `run_manual_testclaim_corpus_eval` remains **`blocked_m1_subset`** until a cargo-clean **subset** of emitted Rust exists. Full-tree **7951** errors dominate; historical note cited **FreeMonoid drop-check + Nat cata Fn-clone** in `v2_compiler_emit_rust` — low share in this snapshot (0 `drop check`, 4 `recursion` strings) but still worth subset-isolation in M1 lane.

---

## §5 Highest-error emitted modules (line refs)

| Emitted module | Error line refs |
| --- | ---: |
| `v4_std_diagnostic.rs` | 645 |
| `v4_compiler_eval.rs` | 385 |
| `v4_lens_testgen.rs` | 339 |
| `v4_extdeps_languages_rust.rs` | 327 |
| `v4_extdeps_languages_go.rs` | 319 |
| `v4_std_datetime.rs` | 296 |

Prioritize SG-1/2 fixes in **std/diagnostic** + **compiler/eval** + **extdeps/languages/** — matches error density.

---

## §6 Recommended dispatch order

**M1 rustc ratchet (use `v4-m1-rust-emit-probe.sh`):**

1. **SG-1** — ~38% of lines (E0423)  
2. **SG-2** — generic arity (E0107/E0282)  
3. **SG-4** + **SG-8** — import / module graph  
4. **SG-5** + **SG-6** — substrate `.dag` (remove `compile_error!` stubs)  
5. **SG-3** — mop-up cascades  

**T-38 track (parallel, not blocking M1 meter):** **SG-7** when zero-diagnostic compile + T-38-PR1 scaffold is required; then **T-38-PR2** cargo-clean subset + `CorpusEvalReport` verdict rows.

---

## §7 T-38-PR1 interim JSON (not emitted — compile gate failed)

Expected schema when SG-7 clears: `scripts/v4-testclaim-corpus-eval.sh::host_scaffold_receipt_v1` with `execution_status: blocked_m1_subset` and empty `entries[]`. Current run stops at v2 compile (24 complexity errors).
