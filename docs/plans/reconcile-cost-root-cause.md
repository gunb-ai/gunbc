# Reconcile cost: root cause

Measured 2026-08-16 on `integration/floor-cut` (`8a6728eaa9`), where the symptom lives — the floor
heartbeat and `run_required_floor` are not on `main`. Instrumentation was a throwaway patch to the
emitted Rust; nothing here was landed.

## The question

`compile.reconcile` reported **8 minutes** inside `prepare_repository_once` on a required-floor run,
one core, RSS 3.5 → 6.9 GB, `majflt=0`.

## CORRECTION (2026-08-16, srv2, the real symptom population)

Everything below the next heading was measured on closures of 63/369/768 modules and **extrapolated**
to the floor. The extrapolation was wrong, and the headline localization with it. Measured on srv2
against the actual floor (`claim_executor --required-floor`, three source roots,
`modules_resolved=3734`):

```
[rx] typecheck                 533 664ms   90.3%   ← the answer
[rx] rewire_import_str          53 885ms    9.1%   ← predicted ~460s
[rx] rewire_type_env_parent      1 163ms
[rx] build_emit_graph_info       1 618ms
[rx3] parse_census_fill               0ms   ← confirmed prediction
[rx3] census_si_plus_reconcile 591 357ms
```

Exponents 768 → 3734 modules: `typecheck` **1.89** (superlinear), `rewire_import_str` **1.49**,
`keys_total` **1.14**. The two exponents *crossed* between the small grid and reality. Why rewire's
bent down: `keys_max` saturated at exactly **18 662** — identical to the 768-module closure — and
`keys_mean` moved only 1 643 → 2 052. Ancestry **plateaus**; the `modules × universe` quadratic does
not hold at scale, so the cost-law section below is true of the grid and false of the floor.

What survives unchanged, and is stronger: the no-op finding — 11 046 real changes out of 7 665 169
scans, **0.14%** — every one same-name/different-`resolved`, over a cohort that shrank to **three**
names (`Bytes` 3657 · `Optional` 3732 · `Secret` 3657; `Float` dropped out). The §3 dual-authority
root cause and the narrowed replacement stand; their prize is 54s, not 480s.

**This is control-defect variant 7 — oracle denominator narrower than the change — committed by me.**
A 9× grid was used to rank children of a 60× population.

**New frontier:** `typecheck_with_census_extra`, unpartitioned. Its children are
`seed_kernel_intern_table`, the `resolved_by_name` fold, `symbol_index_with_qualified_fill`,
`build_global_bare_variant_locals`, the `realize_module` fold, `expand_transitive_services`. My
earliest suspicion — `type_env_for_import`'s `std.types` special case, reached via `build_type_env` →
`interface_env_for_import` — lives inside that 90%.

## The partition (three closure sizes, one build, one host)

```
modules              63       369       768      exponents
typecheck          4 035ms  13 389ms  27 018ms   0.68 → 0.96   ~LINEAR, 63% of reconcile
rewire_import_str    210ms   1 354ms   5 108ms   1.09 → 1.83   SUPERLINEAR
rewire_type_env_parent, rewire_func_env_parent,
  has_v1_seed, build_emit_graph_info
                     114ms     305ms     796ms
```

The span was fully accounted only after a second timer pair: the 9–14s residual is
`parse_census_fill_sources`, proportional to **tree − closure** (13.1s at a 369-module closure,
9.5s at 768), because `gunbc compile` reads and tokenizes every indexed module *outside* the
closure to populate a name census. On the floor's whole-corpus prepare the closure ≈ the tree, so
this is ~0 there — but any path resolving many small entries pays a whole-tree parse per
invocation. **Separate lane; not the 8 minutes.**

## The superlinear child, and why it is superlinear

`rewire_type_env_import_str_binding_identity`. The stable cost law is flat per key —
7.74 µs/key (63 modules), 3.77 (369), 3.95 (768) — so nothing is getting slower; the *population*
grows:

```
by_name (global type-name index)     1 773    9 649   19 470    ~linear in modules
keys_total (inherited keys walked)  24 941  347 299 1 262 093   → N^1.85
keys_mean per module                   395      941     1 643
keys_max                             1 485    9 170    18 662   ← 96% of the whole universe
```

`inherited_keys` is `keys(ancestry_str_bindings) ++ keys(str_bindings)`, and
`ancestry_str_bindings` is the **flattened per-module copy of everything transitively visible**. So
the pass is `modules × corpus-name-universe`.

## What that work actually accomplishes

```
keys scanned              1 262 093
hit the type-name index   1 262 093   ← 100%; the guards filter nothing
admitted by the guards    1 215 908   ← 96%
ancestry identity changed     2 995   ← 0.25%
ancestry already canonical 1 212 913   ← 99.75% no-op
ancestry new keys                 0
str changed / str already      0 / 0   ← the str overlay is empty on every module
```

And every change is same-name, different `resolved` node (`alloc_only=0`, `diff_name=0`), over a
**four-name cohort** in this closure:

```
Float 767 · Optional 766 · Bytes 731 · Secret 731      total 2 995 over 768 modules
```

A second, 7-module compile in the same process yields a *different* set — `Bool`, `Bytes`, `Json`,
`Secret`, `Unit`, 5 each. **The set is closure-derived, not a constant.**

## Root cause

`Bool`, `Unit`, `Json`, `Bytes`, `Secret`, `Float`, `Optional` each have **two authorities**: a
synthetic kernel `TypeBinding` that inference requires (`build_type_env` deliberately ranks kernel
identity above imports for these), and an authored `std.types` declaration that assembly
canonicalises to. The flattened ancestry carries the kernel identity into every module, and this
pass exists to swap it afterwards.

So the pass is **not a repair** — it is an undeclared **phase translation** between two meanings of
one field (`TypeBinding.resolved` as inference-serving representation, then as canonical
declaration identity), discovered by scanning the whole universe. That is a §3 single-authority
violation whose reconciliation cost is quadratic in the corpus.

It also explains gunbc#8202: moving the canonicalisation earlier into `build_type_env` changed what
inference observed, because that experiment moved the *conversion* without moving the *meaning*.

## The narrowed replacement (designed, not landed)

The predicate factors into a closure-global candidate relation plus module-local vetoes; nothing
module-dependent can *add* a candidate.

```
C(name)          = name ∈ synthetic kernel bindings
                   AND type_name_index[name].exporter_count == 1
                   AND canonical_binding is Present
A(module, name)  = name ∈ ancestry_str_bindings          ← now explicit, was implicit in the scan
                   AND name ∉ local_names
                   AND direct-import occurrence count ≤ 1
```

Three constraints, each with a receipt:

- Derive the kernel set from `compiler_kernel_type_env(...).str_bindings`, **not** `kernel_type_set`.
  Precisely: `Float` *is* a row of that table; `Optional` is **not** — it is constructed separately
  by the synthetic kernel environment, and it is nevertheless the second-highest count (766/768).
  So `Optional` alone is the proof that deriving from the table is incomplete, and a hand-authored
  list would have dropped the largest name the table cannot supply.
- Keep the candidate relation **closure-global** (built from this reconcile's typed closure), never
  the repo census or `SymbolIndex.global_bare`, or a homonym outside the closure flips uniqueness.
- Do not retain `direct_import_exporter_counts` — it enumerates every exported name of every direct
  import, which would leave a universe walk inside a four-name algorithm.

Cost becomes `modules × |kernel ∩ closure-unique-authored|` — 3,072 probes instead of 1,262,093 on
this closure. The flattened representation stays wrong; it stops being walked.

Durable proof obligation: keep the old full-universe pass as a **cold differential oracle**
asserting `unexpected_non_kernel_transition_count == 0`, so a future phase translation reds as a
named model deficit rather than being silently skipped by the fast path.

## Separate defect found on the way: the artifact is not reproducible

Same binary, same arm, same entry, three fresh processes:

```
RUN A  arm=0  0c3366a1…   195 879 595 bytes
RUN B  arm=0  77d06e74…   195 879 595 bytes     ← A ≠ B
RUN C  arm=1  db984524…   195 879 595 bytes
```

Identical byte counts, different digests. `v1_rt` builds every map with `im::HashMap::new()`, whose
default `RandomState` is seeded per process.

Structural comparison of two baseline runs, resolved:

```
nodes                identical, 193 854 713 chars
files, version       identical
diagnostics          1557 / 1557    multiset-equal      → order only
item_registry_keys   19473 / 19473  multiset-equal      → order only
modules              768 / 768      deep-sorted equal, 0 entries differing
```

So the divergence is **ordering at every level; content is stable**. (A first comparison reported
`modules` unequal — that canonicalization sorted object keys but not nested lists, so a permutation
*inside* a module entry read as a difference. The instrument, not the artifact.)

Two bounds on the claim. **Deep-sorting erases intended order as well as incidental order**, so this
establishes content-identity up to ordering, not that every varying order is semantically inert —
and a globally-sorting comparison can *erase the first order-sensitive boundary it is being used to
find*. A sounder ladder for the next such question: canonicalize object-key order only → compare
cardinalities → compare records as keyed multisets where a stable key exists → locate the first
differing JSON path → only then classify (pure permutation / generated-name renumbering / real field
difference). And identical byte lengths are *consistent with* a permutation, not proof of one:
equal-length substitution preserves length too.

**Third, independent defect, recorded rather than parsed away:** `dag-artifact.json` contains raw
control characters, so `json.load` refuses it in strict mode (`Invalid control character at line 3
column 5631573`). `strict=False` was an investigative recovery route; it does not make the artifact
valid JSON. If this file claims to be standard JSON rather than throwaway telemetry, that is a
serialization defect of its own.

**Consequence: emitted-artifact byte digests are not a valid equivalence oracle on this path.** Any
drift gate or cache keyed on them rests on a false premise, and a semantic fingerprint is required
rather than optional.

## Method notes (kept because they cost time)

- A first scaling sweep fitted a *sublinear* exponent for a mechanism independently shown to be
  quadratic — a ~18.3s fixed term (`parse_census_fill`) swamped a 9× grid. **A growth curve is only
  informative across a range where the variable term dominates the fixed one.**
- A benchmark reporting 638× for the emitter's `Rc::make_mut` clone used
  `std::collections::HashMap`; `v1_rt.rs` uses `im::HashMap`, where a whole-container copy is
  O(log m) not O(m). Withdrawn; the real figure is ~9×, a constant factor.
- An A/B whose two arms are two *processes* is not one subject: `RandomState` differs. The
  measurement-layer form of the set-level control is **run the same arm twice**.
