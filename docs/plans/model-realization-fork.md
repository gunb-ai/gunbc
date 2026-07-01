# Plan — model↔realization fork (the root of the fail-open problem)

**Status:** audit (done) + grounding order · **DESIGN.md + carriers are authority** (this *is* the DESIGN open thread, §1/§2/§7). Linked from `ROADMAP.md §0` and [fail-closed-lockdown.md](fail-closed-lockdown.md) §3a.

**Verified against the live tree 2026-06-21** (two evidence-backed audits). Line numbers are receipts.

## 0. Verdict — single seam confirmed, dissolution is two-phase

Every primitive is **modeled** as a `.dag` coproduct and **realized** as a native host `Value`, reconciled by **per-site bridges** (~12–13 distinct ones across ~1500 lines of `v1_interpreter.rs`) — so coverage is accidental and non-compositional. **The single-root hypothesis is CONFIRMED:** the fail-open sites the lock-down found (the `==` straddle, the lossy cache digest, the under-keyed memos) all trace to this one seam. **But grounding does not dissolve it uniformly** — there are two sub-roots: the numeric tower (grounds cleanly) and the `Value::Null` overload (needs splitting, not grounding-away).

## 1. The seam

**Native realization** — `Value` enum, 13 variants (`v1_interpreter.rs:448-483`): `Null, Bool, Int, Float, Str, List, Map, Set, Record, Variant, Closure, Fn, Unit`.

**Modeled coproducts** (each → a native variant):

| modeled | form | native | bridge site | coverage |
| --- | --- | --- | --- | --- |
| `Nat` | `Zero \| Succ` (`nat.dag:21`) | `Value::Int` | pattern match `:2468`; eq guard `:1910` | partial (pattern + eq only; arithmetic stays native i64) |
| `Bool` | `True \| False` (`logic.dag:19`) | `Value::Bool` | literal `:1764`; ops via `is_truthy` | accidental (no coproduct-arm bridge; safe only because no corpus matches nominal arms) |
| `Int` | `GroupCompletion<Nat>` (`integer.dag:41`) | `Value::Int` | native i64 ops | total at op level, but native, not the modeled form |
| `Optional` | `Absent \| Present` (`collection.dag`) | `Value::Null` | pattern match `:2528` | partial (pattern only) |
| `Witness` | `Violates \| Holds` (`witness.dag`) | `Value::Null` | pattern match `:2507` | partial (pattern only) |
| `FreeMonoid` | `Empty \| Cons` (`algebra.dag:76`) | `Value::List`/`Str` | `free_monoid_to_vec :6225` | total (one centralized flatten — the compositional exemplar) |

## 2. Reconciliation chokepoints (fail-open / closed / reconciled)

| site | file:line | verdict |
| --- | --- | --- |
| `Value::eq` catch-all | `:689` | **FAIL-OPEN** (`_ => false` on straddle) — but left infallible by design (single `CanonKey` authority) |
| `eval_binop` Eq/Ne guard | `:1910-1922` | **FAIL-CLOSED** (raises `CrossRepresentationEquality` on numeric straddle) — landed |
| `pattern_matches` | `:2276-2563` | RECONCILED (explicit per-variant arms) |
| `value_hash` / `CanonKey::new` | `:344` / `:309` | RECONCILED / FAIL-CLOSED (reflexivity check rejects non-reflexive) |
| `parse_table_memo` / `pure_call_memo` / `resolved_graph_cache` | (see lock-down §3) | reconciled-with-caveat → the under-keyed / lossy cache holes are this seam at the cache layer |

`FreeMonoid` is the proof that compositional reconciliation is *possible* (one flatten, all ops). Every other primitive is per-site.

## 3. Grounding order (does it dissolve the guards?)

1. **Numeric tower — GROUNDED (#5428, 2026-06-21).** Nat construction-side grounded: `Zero → Value::Int(0)`, `Succ{prev:Int(k)} → Value::Int(k+1)` — native form == modeled form. `cross_representation_numeric_straddle` is dead-in-corpus for numerics; `eval_binop`'s `CrossRepresentationEquality` guard is **kept as fail-closed backstop** (not removed — guard removal is bundled with the `Value::Null` split in §3.2, fenced out of this window). The discriminating witness `cross_representation_equality_test` confirms: former fork cases now reconcile to `Bool(true)`; genuine diffs (`1==2`, `Succ{Zero}==Zero → Int(1)==Int(0)`) stay `false`. **Done.**
2. **`Value::Null` overload — the deeper root; NO, needs SPLITTING not grounding-away.** `Value::Null` means *None* / *Absent* (Optional) / *miss* (map lookup) / *Violates* (Witness) **all at once**. So a blanket equality guard is wrong — `present == None → false` is *legitimate* at ~131 sites. The fix is semantic: **split the sentinel** — `raw_map_lookup` returns `Optional<V>` directly (not `Null` + later bridge), Witness miss returns `Violates`, etc. — so each meaning has its own carrier. This is HIGH effort / HIGH impact and is the real depth of the root.
3. **`FreeMonoid`** — mostly reconciled already; just ensure every op honours both forms.
4. **`Bool`** — currently safe (no nominal-arm match in the corpus); if one lands, apply the numeric pattern (~5 lines). Low priority.

**Net:** grounding the numeric tower kills the *visible* straddle guards; splitting `Value::Null` is what actually closes the fail-open *class*. Until #2 lands, the cache/eq guards are per-site patches, not a dissolved root.

## 4. Relationship to the lock-down

This is the root the lock-down lane (`fail-closed-lockdown.md` §3a) was hunting. The §4 lock-down fixes (content-key caches, purity oracles, realizer-key lens) are *per-site patches* that stay necessary until this seam is grounded; grounding (esp. #2) makes whole classes of them dead code. Sequence the lock-down fixes to ship now (stop the bleeding), and the grounding (numeric tower → Null split) to dissolve the root behind them.

## Dissolution trigger (DESIGN §6)

Delete this doc when the numeric tower is grounded (`Int`/`Nat` native = modeled, guard is dead code) **and** `Value::Null` is split into its grounded carriers (Optional/Witness/miss each own their realization). At that point the per-site bridges are gone and the seam no longer exists.
