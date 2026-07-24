# Gate-1 E0308 repr-mismatch — root classification (2026-07-24)

Read-only diagnosis per sharp-bee-290's task. **No emitter code changed.** Supersedes
the ~4400-corpus-wide estimate quoted in the task brief with a fresh, execution-verified
measurement taken AFTER #7141 (TEXT+COLLECTION+OPTIONAL grounding) already landed on top
of the [2026-07-23 diagnosis](gate1_repr_mismatch_e0308_diagnosis_2026-07-23.md) that fixed
the TEXT carrier.

## Method

Reused classifier-v3 (`rule1-first-error-plus-residual-histogram-v3`, the methodology
stamped in `docs/probes/curated_cargo_frontier_probe_sweep.tsv`) — its underlying shell
driver (`scripts/curated_cargo_probe_one.sh`) was deleted 2026-07-23 as duplicate-of-`.dag`
carrier cleanup, so the exact invocation contract was recovered from git history
(`35c52b1750`) and ported verbatim: `gunbc compile --source-root dag --source-root src/v2
--entry <module> --target rust --dependency-pool-index primary-precedence`, then
`cssl_assemble --out-dir <out> --entry-dag <module> --root <repo>`, then `cargo build
--release --lib` against the same `[lib]`-layout Cargo.toml (im/serde/serde_json/stacker/
lazy_static/unicode-ident/unicode-properties + `v1-compiler` path dep). One deviation from
the original: the Cargo.toml was hand-reproduced rather than rendered via
`curated_probe_cargo_toml_from_cssl_authority` — faithful in content, not byte-provenance
from the `.dag` authority call.

No canonical "deep seven" list exists in the repo; I probed the 7 modules the open threads
name across `deep_module_lanes` and the prior diagnosis: `06_translate`, `04_infer`,
`05_eval`, `05_emit`, `emit_host`, `emit_module`, `materialization_carriers`.

## Aggregate E0308 counts (fresh measurement, post-#7141)

| module | E0308 | E0599 | E0277 | E0369 | E0107 |
|---|---|---|---|---|---|
| 06_translate | 587 | 107 | 104 | 40 | 0 |
| 04_infer | 551 | 102 | 103 | 40 | 0 |
| 05_eval | 651 | 105 | 124 | 58 | 0 |
| 05_emit | 587 | 107 | 104 | 40 | 0 |
| emit_host | 757 | 112 | 172 | 108 | 27 |
| emit_module | 595 | 107 | 104 | 40 | 0 |
| materialization_carriers | 87 | 82 | 100 | 60 | 15 |
| **total** | **3,815** | **722** | **811** | **386** | **42** |

Note: `05_emit`, `06_translate`, and `emit_module` are **byte-identical** first-error and
full residual histograms — they compile through the same shared closure and hit the same
wall at the same point. `materialization_carriers` is a structural outlier (see below) —
its baseline was already reduced from 652→87 by prior work per `v1_deletion_plan.dag`'s
`emit_representation_mismatch` lane note, confirmed still ~87 here.

The task brief's ~4400 estimate is now stale (pre-#7141); current total across these 7
modules is **3,815 E0308** (down from an implied ~5,300+ pre-#7141 baseline extrapolated
from the 2026-07-23 doc's pre-fix 06_translate count of 3,176 for a single module).

## Classification — 6 deep-family modules (06_translate, 04_infer, 05_eval, 05_emit,
emit_host, emit_module)

Bucket proportions are remarkably stable across all 6 modules (±5pp), which is itself
the strongest evidence these are shared, not per-module, roots:

| bucket | share (range across 6 modules) | task taxonomy class |
|---|---|---|
| DIAGNOSTICS | 26–30% | not in task's 4-class list — **named as a 5th root below** |
| WITNESS | 18–23% | (1) Value::Null-straddle |
| RC_WRAP | 10–13% | (1)/(2) boundary — Option-wrap decision |
| OWNERSHIP | 7–10% | (1)/(2) boundary — bare-vs-Rc wrap decision |
| OTHER (unclassified) | 20–29% | mixed — see residue notes |
| TEXT_RESIDUE | 3% | leftover from the #7141/07-23 TEXT fix, small tail |
| COPRODUCT_NATIVE_NUMERIC | 1–5% | (2) coproduct-native |
| COPRODUCT_NATIVE_BOOL | 2–2.5% | (2) coproduct-native |

Representative pairs (06_translate, identical shape in 05_emit/emit_module):

```
46 expected Node found Rc<Node>                              OWNERSHIP
42 expected Option<String> found Diagnostics                  DIAGNOSTICS
36 expected Rc<Diagnostic> found String                       DIAGNOSTICS
31 expected Option<CostShape> found Rc<Option<CostShape>>      RC_WRAP
26 expected Option<Rc<Node>> found Diagnostics                 DIAGNOSTICS
18 expected Witness<Rc<Node>> found Witness<_>                 WITNESS
14 expected Witness<Rc<TerminationProof>> found Witness<_>     WITNESS
14 expected Witness<Rc<InferredFacts>> found Witness<_>        WITNESS
12 expected bool found Bool                                    COPRODUCT_NATIVE_BOOL
11 expected Option<_> found Diagnostics                        DIAGNOSTICS
3  expected i64 found GroupCompletion<Rc<Nat>>                 COPRODUCT_NATIVE_NUMERIC
```

### Root 1 — DIAGNOSTICS carrier fork (26–30% of E0308, largest single bucket)

`Diagnostics` (a plural collection/result-wrapper type) is emitted where call sites expect
`Option<String>`, `Option<Rc<Node>>`, `Rc<Diagnostic>` (singular), `Correction`, `()`, or
`Option<_>`. This shape — one wrong carrier declaration read at hundreds of call sites —
is structurally identical to the TEXT (`FreeMonoid<Char>` vs `String`) root the 07-23
diagnosis fixed: a single modeled type's emission decl doesn't match what its use-sites
actually need. Not named in the task's 4-class taxonomy; recommend naming it explicitly
as its own repr-fork class (a "Result/diagnostics accumulator" carrier, distinct from
Optional/Witness).

### Root 2 — WITNESS<T> parametrization gap (18–23%)

`Witness<Rc<X>>` (concrete) vs `Witness<_>` (unresolved generic) — this is exactly the
task's class (1), the DESIGN thread's Value::Null-straddle family extended to `Witness`.
The emitted witness-producing call sites aren't propagating the concrete type argument
through to the `Witness<_>` instantiation.

### Root 3 — RC_WRAP / OWNERSHIP (17–23% combined)

`Option<T>` vs `Rc<Option<T>>`, and bare `Node`/`CostShape`/`DecimalDigitsStep` vs their
`Rc<...>`-wrapped form. This is squarely the domain of the **already-landed**
`wrap_decision_predicate` (#6776, `v1_deletion_plan.dag` brick state `Placed`). **Finding:**
the gate does not appear to cover the type-alias / struct-field / fn-signature emission
paths this probe exercises — 46–65 OWNERSHIP + 63–65 RC_WRAP occurrences per module survive
post-#6776. This is a contradiction between the milestone's "Placed" status and observed
behavior and should be escalated/verified rather than assumed fixed, not re-diagnosed here
(read-only bound).

### Root 4 — missing-trait residue riding E0599/E0277/E0369 (task's class 3)

Classified separately from the E0308 pairs (E0599/E0277/E0369 don't carry `expected/found`
pairs in the same shape). Resolves into 3 shared sub-roots, consistent across all 6 modules:

- **Missing `Clone` bound on emitted generic type parameters** (`T`, `A`, `B`, `U`, `R`) —
  `no method named 'clone' found for type parameter T` / `the trait bound T: Clone is not
  satisfied` — ~30–45 occurrences/module. The emitted generic fn signature omits a `where
  T: Clone` the body actually requires.
- **Missing arithmetic/comparison trait impls for coproduct-native types** —
  `GroupCompletion<Rc<Nat>>` (add/sub/mul/div/`>=`/`<`) and `Rc<im::Vector<T>>` (`==`) —
  ties directly to Root 2's coproduct-native class; #5428's numeric-tower grounding did
  NOT cover the emitted-Rust operator-trait surface, only interpreter-side `Value::eq`.
- **Missing `serde::Deserialize`/`Serialize`/`Debug` derives** on many named modeled
  structs (`ValueInterpreter`, `TransformInterpreter`, `EvaluationFrame`, `Namespace`,
  `PrimitiveFactBundle`, `CommutativeSemiring<Magnitude>`, ~15+ distinct types, ~3
  occurrences each per module) — a genuinely new root, not anticipated by the task's
  taxonomy: the emitter declares these structs without deriving the traits their generic
  usage (map values, cache keys, closures) requires.

### Residual OTHER (20–29%, not yet fully sub-classified)

Largest named pairs: `usize` vs `RangeFrom<{integer}>` (24×, an emitted range-slice vs
index-scalar confusion), `()` vs `Option<_>`/`Option<Rc<Node>>` (21+2×, a discarded-vs-
returned diagnostic value), `Rc<Vector<_>>` vs `String` (4×), plus scattered singletons.
Below the "explains most of the total" bar the task set — not chased further given the
read-only time-box.

## materialization_carriers — a structurally different, smaller wall

Only 87 E0308 (vs 551–757 elsewhere), first error is `E0107: missing generics for struct
Measure`, not the shared import-closure marker the other 6 hit. Dominant pattern:

```
45  CommutativeSemiring<Magnitude>: serde::Deserialize<'de>  not satisfied
29  T: Clone  not satisfied
14  expected i64 found Rc<CommutativeSemiring<Magnitude>>
9   CommutativeSemiring<Magnitude>: serde::Serialize  not satisfied
9   CommutativeSemiring<Magnitude> doesn't implement Debug
8   expected Rc<CommutativeSemiring<Magnitude>> found i64
5   missing generics for struct RealizedStep
4   missing generics for struct Measure
3   type alias takes 0 generic arguments but 3 were supplied
2   missing generics for type alias List
```

Two distinct, smaller sub-walls here: (a) `CommutativeSemiring<Magnitude>` is a
coproduct-native mismatch (task class 2) plus the same missing-derive root (Root 4) —
same mechanism as elsewhere, one specific algebra type; (b) **missing-generics on emitted
type references** (`Measure`, `RealizedStep`, `List`, `Outcome`) is a distinct emitter
defect not seen in the other 6 modules — the emitted declaration/reference drops required
type parameters entirely, rather than picking the wrong wrap. Not investigated further
(read-only, out of the 4-class taxonomy, small in absolute count).

## Recommendation: N sub-walls, not one construction predicate

Unlike the numeric-tower grounding (#5428) or the TEXT fix (07-23 diagnosis), no single
predicate covers this residue — the roots are representationally orthogonal (a Result-
carrier fork, a generic-witness-instantiation fork, an ownership-wrap fork, and a trait-
derive-completeness fork are different failure shapes with different fixes). Recommend
**4 targeted walls**, roughly in leverage order:

1. **DIAGNOSTICS carrier fix** (Root 1, ~28% of E0308) — single highest-leverage fix,
   same shape as the TEXT fix: correct one type declaration/signature for the diagnostics-
   accumulator concept.
2. **Trait-derive-completeness predicate** (Root 4, spread across E0599/E0277) — plausibly
   ONE construction predicate itself (derive whatever the generic usage requires: Clone,
   the arithmetic ops for coproduct-native algebra types, serde where the struct crosses a
   map-key/cache boundary) — same shape as the #6776 wrap-decision predicate, applied to a
   different axis (trait derivation instead of Rc-wrapping).
3. **WITNESS<T> concrete-parameter propagation** (Root 2, ~20%) — needs its own fix in the
   witness-producing emission path.
4. **RC_WRAP/OWNERSHIP gap-in-#6776** (Root 3, ~20%) — NOT a new predicate; this is either
   a coverage gap in the existing landed gate or evidence the "Placed" status is premature.
   Recommend the operator verify #6776's construction predicate against these exact
   emission paths (type-alias decls, struct fields) before authoring anything new here.

`materialization_carriers`'s missing-generics defect (E0107) is a 5th, much smaller,
separate residue — worth a follow-up diagnosis of its own, not urgent given its size.
