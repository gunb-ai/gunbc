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

**The cost IS inside expression inference** (this was checked, because "ms per call" had been
computed as module wall / call count, which would attribute item-level work to expressions):

```
gunbc.host_effect_realize   wall_us=259697135  rootIncl=259685651  OUTSIDE=11484 (0.004%)
```

`sum(exclusive)` equals `sum(root inclusive)` exactly, which is the arithmetic identity for a call
tree and confirms the partition neither double-counts nor loses recursion time.

**Two factors, one still unidentified.** Per-call cost by environment:

```
fixture (anc=165)                       0.2 - 1.2 ms/call
gunbc.live_deploy.apply (anc=15239)    11.6 ms/call     ~50x above fixtures
gunbc.host_effect_realize (anc=15056) 113.3 ms/call     ~9x above that
```

The earlier "environment size refuted" claim was scoped to the gap BETWEEN two real modules with
near-identical environments, and stands at that scope. It does not license concluding environment
is irrelevant: the fixture-to-corpus step is ~50x and is the larger factor. What remains genuinely
open is the ~9x that separates this module from its control.

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
  preparing every indexed module. Those are different axes and this document previously conflated
  them.
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
  or an obvious slowdown. The guards are why none contaminated a finding.
- `perf` is present on srv2 but `perf_event_paranoid=4`; lowering it needs root on a host
  running 22 CI processes. Not done.
