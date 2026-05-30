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

## §3 rustc error code histogram (top 15)

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
| E0573 | 159 | 2.0% |
| E0369 | 110 | 1.4% |
| E0560 | 98 | 1.2% |
| other | 393 | 4.9% |

**E0423 message (uniform):** `expected function, tuple struct or tuple variant, found type alias 'Symbol'` (all 2978).

**E0107 clusters:** missing type args on `Outcome` (578), `Witness` (113), `TestClaimRun` (104), `TestClaimEvalSubject` (83), `FreeMonoid` (71), `Validation` (61), …

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

## §6 Recommended dispatch order (M1 close)

1. **SG-7** — unblock T-38 scaffold (0 v2 diagnostics)  
2. **SG-1** — collapses ~38% of rustc lines  
3. **SG-2** — restores #3654-era top codes in correct shape  
4. **SG-4** + **SG-8** — import graph hygiene  
5. **SG-5** + **SG-6** — substrate `.dag` (remove `compile_error!` stubs)  
6. **SG-3** — mop-up cascades  
7. **T-38-PR2** — cargo-clean subset + real `CorpusEvalReport` verdict rows  

---

## §7 T-38-PR1 interim JSON (not emitted — compile gate failed)

Expected schema when SG-7 clears: `scripts/v4-testclaim-corpus-eval.sh::host_scaffold_receipt_v1` with `execution_status: blocked_m1_subset` and empty `entries[]`. Current run stops at v2 compile (24 complexity errors).
