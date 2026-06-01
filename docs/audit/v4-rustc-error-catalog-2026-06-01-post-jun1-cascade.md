# v4 rustc error catalog -> post-Jun1-cascade remeasure - 2026-06-01

**Manager session:** `sleek-heron-13` (M1 rustc probe lane).
**Authority:** dashboard node `adhoc-7b46e080-3cd` ("Fresh M1 rustc probe post-Jun1-cascade").
**Live ratchet meter:** this catalog replaces `docs/audit/v4-rustc-error-catalog-2026-05-31-post-p5.md` / PR #4122 as the current full-tree M1 rustc residual measurement. The #4122 catalog remains the 2026-05-31 post-#4115 baseline.
**Reference commit:** `origin/main` at **`483d82a78`** (`docs(planning): post-#4111 burn-down - HEAD refresh + landing log row (#4112)`), verified by `git rev-parse HEAD origin/main` immediately before the probe.
**Probe:** `scripts/v4-m1-rust-emit-probe.sh`, with `V2_COMPILER=target/release/gunbc`, `V4_M1_RUST_EMIT_OUT=/tmp/v4-rust-emit-jun1-sleek-heron-13`, `V4_M1_CARGO_CHECK_JOBS=4`.
**Raw summary:** `docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.m1-probe-summary.txt`.

---

## Section 1 Headline

| Metric | 2026-06-01 post-Jun1-cascade | 2026-05-31 post-#4115 (#4122) | Delta |
| ------ | ----------------------------:| -----------------------------:| -----:|
| v2 emit diagnostics | **0** | 0 | 0 |
| `.rs` files emitted | 351 | 345 | +6 |
| `.rs` files on disk | 350 | 344 | +6 |
| rustc `error[E####]` lines | **7,724** | 7,175 | **+549** |
| Distinct emitted `.rs` files with errors | 310 | 305 | +5 |
| Top code | `E0308` (2,953) | `E0308` (2,905) | unchanged leader |
| `E0423` (SG-1 closure check) | **0** | 0 | 0 |

**Plain-English summary.** The stale #4122 count was 7,175 errors at the post-#4115 measurement point. After the Jun 1 cascade landings now on `origin/main` (#4118 SG-1b implementation, #4124 SG-2 worksheet closure/P5 smoke, #4116/#4133/#4135 SG-RC-LAYERING implementation/follow-up, #4127 SG-8 worksheet, #4120 T-38-PR2, #4101 CI bankruptcy), the fresh full-tree M1 rustc residual is **7,724**. That is **+549 errors** over #4122, while v2 emit remains clean at **0 diagnostics**.

This is a measurement receipt, not a class closure claim. The dominant error remains `E0308`; the largest visible movement is the SG-8 code family (`E0425`/`E0432`/`E0433`) growing by **+420** combined, which is consistent with more module/re-export surface being present after the worksheet/substrate landings rather than with SG-1 or SG-7 reopening.

---

## Section 2 Code histogram

| Code | 2026-06-01 | #4122 | Delta | Notes |
| ---- | ---------:| -----:| -----:| ----- |
| `E0308` | 2,953 | 2,905 | +48 | mismatched types; still the dominant residual |
| `E0107` | 1,654 | 1,629 | +25 | generic arity |
| `E0282` | 1,007 | 957 | +50 | type annotations needed |
| `E0425` | 538 | 485 | +53 | unresolved value/type names |
| `E0432` | 415 | 238 | +177 | unresolved imports |
| `E0277` | 330 | 330 | 0 | trait bound |
| `E0433` | 271 | 81 | +190 | failed to resolve |
| `E0573` | 159 | 159 | 0 | expected type, found variant |
| `E0560` | 126 | 122 | +4 | missing struct field |
| `E0369` | 110 | 110 | 0 | binary op on `Rc<T>` |
| `E0121` | 44 | 44 | 0 | placeholder `_` in item signature |
| `E0391` | 29 | 29 | 0 | cyclic dependency |
| `E0599` | 28 | 28 | 0 | no method found |
| `E0392` | 12 | 12 | 0 | unused type parameter |
| `E0061` | 12 | 10 | +2 | wrong argument count |
| `E0614` | 8 | 8 | 0 | cannot dereference |
| `E0609` | 6 | 6 | 0 | unknown field |
| `E0559` | 5 | 5 | 0 | variant field mismatch |
| `E0428` | 4 | 4 | 0 | duplicate definition |
| `E0252` | 4 | 4 | 0 | duplicate import |
| `E0610` | 2 | 2 | 0 | field access on primitive |
| `E0283` | 2 | 2 | 0 | type annotations needed |
| `E0109` | 2 | 2 | 0 | type arguments not allowed |
| `E0505` | 1 | 1 | 0 | move while borrowed |
| `E0422` | 1 | 1 | 0 | cannot find struct/variant |
| `E0072` | 1 | 0 | +1 | recursive type without indirection |
| #4122 outside top-25 singleton | 0 | 1 | -1 | prior baseline code outside the raw summary's displayed top-25 |
| **TOTAL** | **7,724** | **7,175** | **+549** | |

**Histogram reconciliation.** The 2026-06-01 top-25 lines in the raw probe summary sum to 7,723. The full rustc log has one additional `E0072` outside the displayed top-25, yielding the recorded total of 7,724. The #4122 comparison column has the same display-shape: its top-25 summary sums to 7,174, plus one code outside the displayed top-25, yielding the recorded baseline total of 7,175. The explicit `#4122 outside top-25 singleton` row above carries that prior-only count so the visible row deltas reconcile to the headline +549.

---

## Section 3 Delta Readout

| Family | Codes | Delta | Readout |
| ------ | ----- | -----:| ------- |
| SG-8 / module graph + re-exports | `E0425` + `E0432` + `E0433` | **+420** | Biggest movement; #4127 worksheet is now on main, and this probe provides the fresh residual population after that surface expansion. |
| SG-2 / generic arity + inference | `E0107` + `E0282` | **+75** | Generic/inference family grew modestly with six more emitted files. |
| Type mismatch / ownership/value boundary | `E0308` | **+48** | Still the largest code, but not the main source of the +549 delta. |
| SG-3 stable bands | `E0277` + `E0573` + `E0369` + `E0121` | 0 | These stayed pinned relative to #4122. |
| Long tail | all remaining codes | +6 | Mostly `E0560` (+4), `E0061` (+2), and new single `E0072`. |

SG-1 remains closed at this probe (`E0423 = 0`). SG-7 remains closed at this probe (`v2 emit diagnostics = 0`). The count increase is therefore residual-population growth after additional substrate/model surface landed, not a reopening of either closed class.

---

## Section 4 Representative Examples and Modeling Readout

These examples are from the emitted tree in `/tmp/v4-rust-emit-jun1-sleek-heron-13` and the rustc log for this probe. They are included to separate true missing-model opportunities from mechanical propagation gaps.

| Family | Representative emitted error | Modeling relationship | Likely collapse lever |
| ------ | ---------------------------- | --------------------- | -------------------- |
| SG-8 / module graph + re-exports | `src/v4_compiler_translate.rs:10` imports `CarrierKind` from `v4_compiler_target_carriers`, but rustc suggests `crate::v4_std_pipeline::CarrierKind`. | The authority moved toward `std/pipeline`; emitted imports still follow an old compiler-local home. This is not a new semantic concept; it is a module-authority/re-export projection gap. | Model/consume a single import-authority edge for promoted carriers so every consumer imports from the declared home after substrate promotion. |
| SG-8 / missing public surface | `src/v4_std_algebra.rs:10` and many peers import `NodeRef` from `v4_std_node`, but no such item exists there; rustc points at accidental peer re-exports such as `v4_extdeps_formats_spice::NodeRef`. | This smells like a real modeling question: is `NodeRef` a `std.node` concept, a lens-local helper, or an extdeps-local reference? The current emitted graph has no single authority. | Decide the concept home. If it is general node structure, model it in `std/node`; otherwise stop broad consumers from importing it as if it were general substrate. |
| SG-8 / generated claim surface | `EdgeLabel` is undeclared in many generated/manual claim files, e.g. `failed to resolve: use of undeclared type EdgeLabel` in generated algebra-law claims. | The type exists in the node model, but claim modules do not receive the dependency automatically. This is a dependency-edge emission gap, not a new algebra model. | Emit imports from actual type references in generated claim bodies, not only from top-level declarations or hand-listed module imports. |
| SG-2 / generic carrier arity | `Outcome` is used without its payload parameter 740 times; `TestClaimRun` is used without its two parameters 292 times. | The generic model exists (`Outcome<T>`, `TestClaimRun<S,A>`), but aliases/cached fixtures often erase the parameters when emitted. | Preserve instantiated type arguments through aliases, cached statics, and function signatures. This is a high-leverage generic-instantiation projection. |
| SG-2 / higher-kinded shape | `Homomorphism<C, Source, Target>` emits fields `source: C<Source>` and `target: C<Target>`, causing `E0109` because Rust type parameters are not type constructors. | This is a true realization-model gap: the substrate can describe a type constructor slot, but Rust needs a concrete encoding strategy for that higher-kinded position. | Add/consume a target realization row for type-constructor parameters, or lower this pattern through an explicit carrier family instead of raw `C<T>`. |
| SG-2 / inference holes | The `E0282` population is dominated by `type annotations needed` around generated constructors/caches. | Usually not a missing domain model by itself; it is downstream of erased generic parameters or ambiguous `Rc::new`/cache construction. | Fixing generic argument preservation should collapse a large slice before authoring any new semantic model. |
| E0308 / SG-1b follow-on | `expected String, found Symbol` appears 1,344 times. | This is exactly the function-signature realization problem: atom realization can construct `Symbol`, but the emitted function signature still says `String`. | Complete/consume target function-signature realization rows for atom-typed returns and parameters. |
| E0308 / ownership-layering | Examples include `expected Rc<Diagnostics>, found Diagnostics` (300), `expected Outcome<_>, found Rc<Outcome<_>>` (183), `expected Node, found Rc<Node>` (113), and `expected TestClaim, found Rc<TestClaim>` (69). | This is the SG-RC-LAYERING model surface: the substrate needs one authoritative ownership/use-site realization, not per-emitter guesses about raw vs `Rc` vs `Box`. | Drive every parameter, return, field, and constructor site from `TargetOwnershipUseSite` / ownership-realization rows. |
| E0308 / collection projection | `expected Vec<Rc<Edge>>, found FreeMonoid<_>` (47), plus PrimitiveFactBundle/FormalProduction/Node variants. | The substrate models `FreeMonoid<T>`; Rust consumer sites often want `Vec<Rc<T>>`. The missing fact is not "list exists" but which boundary projects monoid structure to target collection storage. | Add/consume collection-realization rows for consumer-boundary projection from `FreeMonoid<T>` to `Vec<Rc<T>>`. |
| SG-3 stable bands | `BoundedLattice data missing meet/join` appears as intentional `compile_error!`; Go language rows report `expected type, found variant String`; `E0121` reports `_` in item signatures. | These are mixed: some are deliberate fail-closed witnesses for incomplete algebra data, while others are fallout from name/variant/type realization. They stayed flat versus #4122, so they are not the current growth driver. | Do not open a new broad SG-3 worksheet from this probe. Treat as mop-up after SG-1b/SG-2/SG-8/ownership projection fixes, except intentional `compile_error!` rows that need their own algebra-data completion receipt. |
| Long tail | `E0391` recursive aliases such as `pub type CppMachineWidth8 = CppMachineWidth8`; one new `E0072` recursive type without indirection. | These are usually hollow/self-alias or recursive-carrier realization issues, which map directly to the modeling discipline's "no hollow alias" rule. | Replace self-alias emission with real fact-bundle/newtype/variant realization, or point the alias at a proven existing authority. |

**Main modeling opportunity readout.** The biggest fresh growth is not from a brand-new domain class. It is from three repeatable projection gaps: (1) module/import authority after concept promotion, (2) generic instantiation preservation, and (3) target ownership/collection realization at use sites. Those are proper modeling opportunities because they remove emitter discretion: once the substrate carries the import authority, generic arguments, and use-site realization, many current rustc errors should collapse mechanically rather than by per-file fixes.

---

## Section 5 Repro

```bash
PATH=/opt/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  CARGO_BUILD_JOBS=4 \
  /opt/cargo/bin/cargo build -p v2-compiler --release

PATH=/opt/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
  V2_COMPILER=target/release/gunbc \
  V4_M1_RUST_EMIT_OUT=/tmp/v4-rust-emit-jun1-sleek-heron-13 \
  V4_M1_CARGO_CHECK_JOBS=4 \
  bash scripts/v4-m1-rust-emit-probe.sh
```

`ctrl-build -- cargo build ...` was not used for the successful build in this session because the local shim path recursed through `/usr/local/bin/ctrl-build -- cargo ...`. Direct `/opt/cargo/bin/cargo` with `CARGO_BUILD_JOBS=4` avoided the shim recursion and kept the build capped.

---

## Section 6 Related Artifacts

- `docs/audit/v4-rustc-error-catalog-2026-06-01-post-jun1-cascade.m1-probe-summary.txt` - raw probe summary committed with this catalog.
- `docs/audit/v4-rustc-error-catalog-2026-05-31-post-p5.md` and `.m1-probe-summary.txt` - #4122 baseline at 7,175 errors.
- `scripts/v4-m1-rust-emit-probe.sh` - probe script used for both measurements.
