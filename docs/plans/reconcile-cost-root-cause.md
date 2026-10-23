# Reconcile cost: root cause

Measured 2026-08-16/17 on srv2 against the **real required-floor population**
(`claim_executor --required-floor`, three source roots, `modules_resolved=3734`).
Instrumentation was a throwaway patch to the emitted Rust; nothing here was landed.

Five clean full-floor runs. Every number below reproduced across at least two.

## The question

`compile.reconcile` reported **8–10 minutes** inside `prepare_repository_once`, one core,
RSS 3.5 → 6.9 GB, `majflt=0`.

## The answer

**One module holds 63% of all type inference in the corpus, and its cost is per-call, not
per-item and not repeated work.**

```
compile.reconcile                       ~9 min
  typecheck_module                       89%   (494s;  collect_parent_envs = 11ms total)
    infer_items                          68%   (390–404s)
      gunbc.host_effect_realize                246–253s  = 63% of ALL inference
  rewire_type_env_import_str_binding_identity   54s      = 9%
  build_type_env                                44s      = 8.9%
  build_module_context                          31s      = 6.2%
```

The distribution is not skewed, it is one outlier:

```
[hist] n=3734  p0=0  p25=1  p50=5  p75=17  p95=91  p99=473  p100=252848   (ms)
       top10_share=75.6%   top50_share=84.2%
```

All twenty slowest *items* in the corpus are in that one module — including trivial ones
(`realize_effective_principal_read_on_host`, 3.7s).

## Why it is slow — the discriminators

**Not work amplification.** `infer_expr` calls vs distinct expression nodes, by pointer:

```
gunbc.host_effect_realize   calls= 2292  distinct= 2292  amp=1.0   →  107.3  ms/call
gunbc.live_deploy.apply     calls=  421  distinct=  421  amp=1.0   →   11.6  ms/call
v1.compiler.emit_rust       calls=29552  distinct=29526  amp=1.0   →    0.137 ms/call
v1.compiler.parse           calls=18278  distinct=18268  amp=1.0   →    0.210 ms/call
```

`amp = 1.0` corpus-wide: **no expression is inferred twice anywhere.** The outlier makes 13×
*fewer* calls than `emit_rust` and takes 60× longer — **782× the per-call cost.**

**Not environment size.** The discriminating pair, near-identical envs:

```
gunbc.host_effect_realize  252848ms  96 items  anc=15056 feN=614 feSig=8648 feVisit=33783
gunbc.live_deploy.apply      4922ms  31 items  anc=15239 feN=620 feSig=8736 feVisit=36251
```

`live_deploy.apply` is **larger on every axis** and runs **51× faster** (16.6× per item).

**The remaining variable is type size.** `HostEffect` = **33 variants**, `ResolvedHostEffectCell`
= **32** — the corpus maximum. The 86-arm nested match in `resolve_host_effect_cell` is their
cross product, a *symptom* of the type sizes; it ranks only 4th within its own module
(20.2s) behind `host_effect_apply_gated` (44.6s), which has 34 arms.

**Type size contributes, and does NOT explain the outlier.** Two controlled fixtures, identity
function so expression count is fixed at one call, only the type varying:

```
ARITY   n variants      2    4    8   16   32   64  128
        us/call        50   56   99   93  160  296  701     grows, ~linear above n=32

BREADTH 32 variants x K Int fields   K=1    2    4    8   16
        us/call                     175  205  333  636 1177   linear in total fields

DEPTH   32 variants, payload nested D   D=0    2    4    8   16
        us/call                        195  168  215  165  215   FLAT — no effect at all
```

So the traversal is a flat walk over immediate fields, not a recursive descent. But the largest
type synthesizable here — 32 variants x 16 fields = 512 fields — costs **1.18 ms/call**, while
`host_effect_realize` costs **113.3 ms/call** and its variants carry ~1-3 fields (~100 fields,
predicting ~0.3 ms). **Arity is 670x short; breadth and depth together ~90x short.** Type size is
true and insufficient; it is not the root cause.

**The cost IS inside expression inference** (checked because "ms per call" had been computed as
module wall / call count, which would attribute item-level work to expressions):

```
gunbc.host_effect_realize   wall_us=259697135  rootIncl=259685651  OUTSIDE=11484 (0.004%)
```

`sum(exclusive)` equals `sum(root inclusive)` exactly — the call-tree identity — so the partition
neither double-counts nor loses recursion time.

**Two factors, one still unidentified.** Per-call cost by environment:

```
fixture (anc=165)                       0.2 - 1.2 ms/call
gunbc.live_deploy.apply (anc=15239)    11.6 ms/call     ~50x above fixtures
gunbc.host_effect_realize (anc=15056) 113.3 ms/call     ~9x above that
```

The earlier "environment size refuted" claim was scoped to the gap BETWEEN two real modules with
near-identical environments, and stands there. It does not make environment irrelevant: the
fixture-to-corpus step is ~50x and the larger factor. Genuinely open is the ~9x separating this
module from its control.

**The dominant rule is function-call inference.** Exclusive time bucketed by expression kind
(exclusive, so a recursive parent does not absorb its children and read as dominant for sitting on
top of them):

```
gunbc.host_effect_realize :: Call    207725ms  n=402   516730 us/call   <- 80% of the module
gunbc.host_effect_realize :: Other    49593ms  n=466   106423 us/call
gunbc.host_effect_realize :: Match     1710ms  n=69     24790 us/call

matched on the SAME kind (Call):
gunbc.live_deploy.apply               3743ms  n=80     46792 us/call    11x cheaper
v1.compiler.emit_rust                 3872ms  n=4191     923 us/call   560x cheaper
v1.compiler.infer                     2943ms  n=2958     995 us/call
v1.compiler.parse                     1983ms  n=2334     850 us/call
```

A matched-rule control, not a whole-module one: the same rule costs 560x more here than in ordinary
compiler modules. Two consequences. `Match` is 1710ms, so the 86-arm nested match is
**definitively not the cause** — suspected twice, wrong twice. And a call site must both resolve a
callee and unify each argument against the signature's types — exactly where a large type surface
and a large visibility surface would MULTIPLY rather than add. That is why every one-dimensional
fixture above was true and insufficient: the identity fixture has a type but no call site; the
breadth fixture varies fields but never unifies them against a signature. Neither exercises the
product.

**The `Other` bucket resolved to `RecordLit`, and callee resolution is NOT the cost.** Full
`ExprData` enumeration plus direct timing of the two operations on the Call path:

```
gunbc.host_effect_realize :: Call       210873ms  n=402  524561 us/call
gunbc.host_effect_realize :: RecordLit   49576ms  n=205  241836 us/call
                                        -------- together 260.4s, essentially the whole module

body_shadow_aware_func_sig (callee resolution), corpus-wide top row:
v1.compiler.emit_rust :: sig_lookup    249ms  n=4191  59us/call  distinctNames=844  amp=5.0
   -> gunbc.host_effect_realize does not appear in the table at all
```

So **callee resolution is negligible**, refuting the func-env DAG walk as the mechanism — the
candidate suggested by that structure's sharing factor (614 distinct nodes, 33,783 naive traversals).
Its implied repair, indexing the environment, would have bought nothing.

A single record literal — `ProvisionBuildCacheOnHost { node: n, catalog_id: id }`, two fields —
costs **241 ms**. Both dominant kinds are places where a value is checked against a large
coproduct's variant shapes, and neither involves callee lookup.

**`global_variant_base` is refuted too (current-main measurement, 2026-08-31).** The measured
subject was an explicit real importer of `gunbc.host_effect_realize`, resolved as a 720-module
closure against a 3,716-module three-root name census. The instrument inferred the target three
times in one process: a cold control, a timed warm control, and a timed warm arm whose locals map
contained one non-colliding synthetic key for every real key. All three typed results were
structurally equal. Doubling the population from 6,614 to 13,228 entries changed target inference
from 7,725ms to 7,736ms: **+11ms, 0.14%**. The same-arm run before the padded arm prevents first-run
warming from being attributed to population size. A corpus-sized variant population is therefore
not the mechanism behind the historical 259s.

The population and time are smaller than the 2026-08-16/17 receipt, so the 7.7s target time is
scoped to this 720-module closure and says nothing about whether the historical full-population
outlier remains. Current `--required-floor` no longer recreates the old subject: it indexed 4,436
modules but reconciled a 99-module gate closure, never reaching `gunbc.host_effect_realize`;
spelling the old command again would measure a different population. The explicit importer above
was used so the target's presence was observed rather than inferred from the CLI flag. This
measurement refutes candidate #3 because it varies that candidate within one fixed subject; it
does **not** establish a current full-population target cost or assign any change in cost to an
intervening revision.

Producer invocation (the subject, not a retained timing implementation): `gunbc compile` over
source roots `src/v1`, `dag`, and `src/v2`, entry
`dag/test/claim/typed_argv_exec_realization_witness_test.dag`, target `dag`. A throwaway timer at
`v1.compiler.infer::typecheck_module` around the `infer_items` call produced the three-arm line and
was removed after structural equality was checked. This is therefore a one-off receipt under the
same method as the 2026-08-16/17 study, not a permanently enrolled instrument; the named command
re-derives the subject and the named symbol locates the timing seam.

**The full-population outlier transformed (current-main measurement, 2026-08-31).** The current
equivalent of the old whole-floor subject is `gunbc compile` with `dag` as the first source root,
followed by `src/v1` and `src/v2`, no entry, target `dag`; the first root is semantic because
no-entry compilation selects the primary-root population. It resolved 3,317 modules. Two complete
runs put `gunbc.host_effect_realize` inference at 11.3s and 8.1s, not the historical 246–253s,
while total reconciliation remained six to seven minutes. A second throwaway timer at the same
`typecheck_module`/`infer_items` seam printed every module at or above 500ms. The largest was
`v2.std.compilers.target_model` at 13.5s; `v2.compiler.translate` was 8.6s and
`gunbc.host_effect_realize` 8.1s. All 20 printed modules together accounted for about 57s. There is
no longer one inference module holding most reconciliation cost on this population; most of the
remaining six-minute reconcile is outside these slow `infer_items` calls.

This is the transformed outcome, not a declaration that reconciliation is cheap. A current-main
phase partition on the same `dag`-primary population attributes the six-minute reconcile rather
than leaving a subtraction: typecheck 264.9s, comprising `infer_items` 133.7s,
`build_type_env` 61.0s, `build_module_context` 36.4s, and about 34s residual; outside typecheck,
the import-binding identity rewire is 62.6s, parent rewire 8.5s, function-environment rewire 5.5s,
and emit-info construction 12.8s. Producer: the no-entry three-root command named above, with a
throwaway timer in `v1.compiler.infer::reconcile_with_census_extra` and aggregate timers at
`typecheck_module`'s three named calls. The old partition cannot be carried forward: both the
population and the runtime map carrier changed after it was measured.

The new shape has no single inference outlier to optimize. Its preparation-shaped 97.4s
(`build_type_env` plus `build_module_context`) converges on the separately measured per-scope
closure-growth surface; coordinate with that authority rather than building a second cache beside
it. The remaining inference question is distribution, not recurrence of the old singleton:
133.7s aggregate against only about 57s in modules individually above 500ms.

**The two apparent per-module copied-accumulator repairs are obsolete on current main.** Reading
`merge_global_bare_variant_locals` and the immediately following
`merge_kernel_variant_locals_low_priority` against the old flat-map realization suggests that
`Rc::make_mut` copies the large base before the overlay. That premise stopped being true in
`v1.runtime_rust` when the runtime container carrier migrated to persistent `im::HashMap`: a
shared clone is O(1) structural sharing and an update copies one O(log n) node path. Rewriting
either merge to scan and rebuild the global population would optimize a dissolved mechanism and
can do strictly more work. No accumulator patch is landed; the carrier migration already removed
the proposed copy class at its root.

**Relation-level amplification is real but cheap where measured:** `sig_lookup` runs at amp 5.0-21.0
(calls per distinct callee name) across the corpus, so the same lookup IS recomputed — it just
costs too little to matter. Evidence about this operation only; the RecordLit and call-argument
paths have not had the same census.

**Scope of `amp = 1.0`:** no expression NODE is re-entered. It says nothing about repeated type
comparisons, generic substitutions, or name lookups across different expressions; a relation-level
amplification of 100x is fully compatible with it. That census has not been run.

## Hypotheses withdrawn, each by measurement

| Hypothesis | Killed by |
|---|---|
| `std.types` kernel/authored dual authority | `build_type_env` 8.9%; `collect_parent_envs` 11ms total |
| the rewire pass explains the 8 minutes | 54s = 9% on the real floor |
| the 86-arm nested match is the cause | ranks 4th inside its own module |
| ordinary uniform inference, phase misnamed | p50=5ms against p100=252,848ms |
| flattened ancestry size | control has a *larger* ancestry and is 51× faster |
| func-env DAG size | same control, larger on every axis |
| re-inference / missing memo | `amp = 1.0` corpus-wide |

The first was mine for four days. The fourth was an average asserted as a shape.

## Secondary, separately bounded

- **rewire narrowing — 54s.** 7,665,169 keys scanned to make 11,046 real changes (**0.14%**),
  every one same-name/different-`resolved`, over a three-name cohort (`Bytes` 3657,
  `Optional` 3732, `Secret` 3657). Design in git history at `8c204414c6`. `keys_max` saturates
  at 18,662 — ancestry *plateaus*, so this is not quadratic at floor scale.
- **population necessity — ≤70s, probably far less.** 3,787 indexed modules against 3,128 in the
  union of the roster's import closures; 659 outside. But an import closure is a *lower* bound on
  reachability (DESIGN Class B: bare references resolve by pool coincidence), so 659 is an upper
  bound on what could be dropped, and acting on it would be the empty-observation narrow.
  `SelectionOff` (operator, 2026-08-13) forbids skipping consumers; it does **not** mandate
  preparing every indexed module — different axes, previously conflated in this document.
- **cross-process duplication — unmeasured.** srv2 ran 7 concurrent `claim_executor` CI processes.
  One inference per module *within one fold* says nothing about once per commit / run / host.
- **artifact non-determinism.** Same binary, same arm, three processes: identical byte counts,
  different digests; deep-sorted structural comparison equal. `im::HashMap` seeds `RandomState`
  per process. **Emitted-artifact byte digests are not a valid equivalence oracle on this path.**
  Separately, `dag-artifact.json` contains raw control characters and is not strict JSON.

## Method notes (kept because they cost time)

- A 63/369/768-module grid was used to rank children of a 3,734-module population, and the
  exponents **crossed**: predicted rewire ~460s, actual 54s. **An oracle denominator narrower
  than the change.** No small-closure extrapolation.
- A growth curve fitted a *sublinear* exponent for a mechanism independently shown quadratic —
  an 18.3s fixed term swamped a 9× grid. **A growth curve is only informative where the variable
  term dominates the fixed one.**
- A 638× benchmark used `std::collections::HashMap` while the subject uses `im::HashMap`
  (O(log m) clone). Withdrawn; real figure ~9×. A second benchmark agreed at 816.9× —
  **agreement across instruments that share an assumption measures the assumption.**
- An A/B whose arms are two *processes* is not one subject. The measurement-layer form of the
  set-level control is **run the same arm twice**.
- Six instrumentation defects: silent `str.replace` miss; a shell `&&` chain that ignored the
  patcher's refusal and shipped an unmodified file; a run lost to SIGHUP; thread-local samples
  invisible to the printing thread; `Vec` aliased to `im::Vector` in a static; and an
  unmemoized walk over a shared DAG that **tripled the run it was measuring**. The first two
  produced wrong output; the last four failed loudly — compile error, explicit `ABSENT` guard,
  or an obvious slowdown — which is why none contaminated a finding.
- `perf` is present on srv2 but `perf_event_paranoid=4`; lowering it needs root on a host
  running 22 CI processes. Not done.
