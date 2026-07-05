# Session findings inventory — for complexity-lens audit (2026-07-05)

Every efficiency/complexity issue found this session, tagged with a **lens verdict**: would a
shape-reading complexity lens (reads the `Node` tree shape, not runtime contents) have caught it?
Purpose: audit the complexity lens's coverage vs its gaps with the complexity manager.

Systemic framing (see memory `pipeline-systemic-reproduce-not-reference`): all perf roots are ONE issue —
passes REPRODUCE values (copy, or recompute) instead of REFERENCE the content-addressed immutable
substrate. 4 surface roots: A=aliased-Rc clone, B=authored_name_at, C=import-closure, D=re-resolve.

## Lens verdict legend
- **CAUGHT** — the cost is visible in the Node-tree SHAPE (a nested fold / whole-collection scan). The
  lens's core competency, given collection-size modeling.
- **MISS-ownership** — shape is a LINEAR fold; the quadratic lives in runtime Rc aliasing (`make_mut`
  deep-clones only when strong-count>1). Invisible to a shape-only lens; needs the lens to COMPOSE with
  the ownership/linearity model.
- **MISS-constant** — constant-factor overhead (allocation, un-memoized re-derive). Below Big-O
  resolution; needs a per-operation cost model, not asymptotics.
- **MISS-necessity** — dead/unnecessary work. Needs liveness/used-vs-computed. The §5 undecidable residue
  ("you cannot structurally forbid an unnecessary loop").

## A — Algorithmic, shape-visible (the lens SHOULD catch these)
| # | Finding | Location | Cost | Status | Verdict |
|---|---------|----------|------|--------|---------|
| 1 | emit_imports per-module re-export closure, 4-6× recomputed | 05_emit_rust.dag:2923 | O(M²·I) | open (ROOT C) | **CAUGHT** (nested scan over corpus-sized surface) |
| 2 | build_type_env ancestry `map_merge` per module | 04_infer.dag:5719/5727 | O(M²) | open (ROOT C, reform) | **CAUGHT** |
| 3 | union_parent_type_env_caches ancestry merge | 04_infer.dag:5491 | O(M²) | open (ROOT C) | **CAUGHT** |
| 4 | merge_scope_from_imports re-folds parents' items | 04_infer.dag:302 | O(Σedges·items) | open | **CAUGHT** |
| 5 | P3/P5 infer whole-corpus scans (owner_of_exported_arm etc.) | 04_infer.dag:6033/6079/6613 | O(M²) | #6239 | **CAUGHT** |
| 6 | ownership merge_branch_usages whole-map re-fold at joins | ownership.dag:109 | O(body²) | open (follow-up) | **CAUGHT** (nested fold) |
| 7 | import_closure_from_facts non-worklist | (P4) | O(M²) | FIXED #6241 | **CAUGHT** |
| 8 | interpreter Env::extend chain | (P6) | O(d²) | FIXED #6240 | **CAUGHT** |
| 9 | parser token_stream skip\|>first tail-clone | 02_parse.dag (P2) | O(M²) | FIXED #6241/#6255 | **CAUGHT** |

## B — Ownership-hidden quadratics (the lens MISSES — biggest gap)
The highest-impact quadratics. Shape is a linear fold `fold(xs, acc => rc_push(acc.clone(), x))`; the
O(n²) is the runtime `Rc::make_mut` deep-clone when the accumulator Rc is aliased.
| # | Finding | Location | Cost | Status | Verdict |
|---|---------|----------|------|--------|---------|
| 10 | emit dag_collect fingerprint (make_mut clone) | dag_collect.dag | O(M²) | FIXED #6242 (seed-only) | **MISS-ownership** |
| 11 | parse intern/pre_intern_tokens (make_mut clone) | 00_core.dag:1274/1322 | O(k²) | open (~30-60% parse) | **MISS-ownership** + MISS-necessity |
| 12 | parse aliased accumulators (7 sites) | 02_parse.dag:1313/1561/3015/3652/3727 | O(m²) | open | **MISS-ownership** |
| 13 | generated `.clone()` clone-fallback (§5 absorbing) | 05_emit_rust ownership | O(n²) runtime | snappy-newt #6249 | **MISS-ownership** |

**Deciding-fact taxonomy (requirements the wall hands the ownership lane — per sunny-wren-799).**
The fix is NOT a smarter lens (that violates the "lens reads SHAPE" structural law + §4 no-heuristic). It is to land
the missing fact as a SUBSTRATE FACT on the node, so the *same* pure-shape rule reads it. Which fact decides each:
- #10 dag_collect — **move/last-use**: is `seen` dead after the `rc_push` (→ move, O(1)) or aliased/live (→ copy, O(n))?
- #11 intern/pre_intern_tokens — **move/last-use** (the `table` is still live at the push → copy) **+ consumer-count**
  (the dead-work half: interned ids consumed at 3 sites only → the necessity axis, Section D/R3).
- #12 parse accumulators — **move/last-use**: is `acc.clone()` a copy because `acc` is still aliased downstream?
- #13 clone-fallback — **move/last-use** (per-site last-use verdict; the de-fork's `owned_bindings`/Perceus fact).
All four reduce to the SAME fact — **move/last-use** (aliasing-at-site is its dual) — which the ownership de-fork
already lands. #11 additionally needs **consumer-count** for its dead-work half. Interim honest verdict for any such
site until the fact exists in-tree: **counted `Unknown(OwnershipUnmodeled)`** — never silently `Linear`/`Constant` (§5
refusal, not absorption). A census that grades `fold(xs, rc_push(acc.clone(), x))` as Linear without an ownership
fact is WRONG, and that is the RED control.

## C — Constant-factor (the lens MISSES — below asymptotic resolution)
| # | Finding | Location | Cost | Status | Verdict |
|---|---------|----------|------|--------|---------|
| 14 | authored_name_at un-memoized String rebuild (cross-cutting) | 00_core.dag:455 | const ×everywhere | open (ROOT B) | **MISS-constant** |
| 15 | Node 18-field per-node allocation (~5-6 allocs/node) | 00_core.dag:263 | const ×nodes | open (~15-25% parse) | **MISS-constant** |
| 16 | tokenize per-token allocation | 01_tokenize.dag | const ×tokens | open | **MISS-constant** |
| 17 | ownership doubled record_use (copy-paste 2×) | ownership.dag:251/262 | const 2× | open (follow-up) | **MISS-constant** |
| 18 | analyze_single_fold walks each fold body 2× | ownership.dag:505 | const 2× | open | **MISS-constant** |
| 19 | emit field-access type re-resolution per node | 05_emit_rust.dag:4724 | const ×nodes | post-cutover (ROOT D) | **MISS-constant** (correctness = #6243 predicate; #6266 retracted as over-broad) |

## D — Borderline / bounded-inner
| # | Finding | Location | Cost | Status | Verdict |
|---|---------|----------|------|--------|---------|
| 20 | infix_bp filter-scans operator table per token | 02_parse.dag:3316 | O(25)/token | open | **PARTIAL** (scan-in-loop, but inner bounded) |

## E — Modeling / §7 debt (not the complexity lens's domain — noted for completeness)
| # | Finding | Location | Type |
|---|---------|----------|------|
| 21 | TypeEnv String-keyed vs Int-keyed dual representation | 04_env.dag:34 | §2/§3 parallel-repr (marked dissolve) |
| 22 | dag_collect memo lives in seed only, not .dag model | dag_collect.dag | §7 model↔seed fork |
| 23 | TypeBinding fuses import-resolution + termination-provenance | 04_env.dag:18 | §2/§3 concern-fusion |

## Correctness / §5 / dark-CI (other lenses — listed so the audit is complete)
- field_access_field_is_boxed wrongly routes SHARED types down the deref path (1482 errors) — FIX = #6243 (drops the `is_recursive_type_by_name` disjunct). #6266 was a blanket `clone_value` template fix, RETRACTED (over-broad — broke Nat cycle-only boxing which genuinely needs the deref); #6266 is now byte-identical hygiene only. The single-predicate diagnosis (explains 1482/1667) stands. Ownership/type lens.
- match-bound &ref emitted bare (use-after-move-adjacent) — snappy-newt wall. Ownership lens.
- Silent clone-fallback = §5 absorbing-fallback. §5 lens.
- #6235 resolver break / #6241 get-unregistered / #6242 registry+comment — all invisible-while-dark. Dark-CI class → merge-norm recommendation (regen self-compile receipt in corpus-touching PR bodies).

## Audit summary — the three gap-classes
The lens catches its core competency (**Section A: nested-fold / whole-corpus-scan shapes**, ~9 findings).
It MISSES three classes that together account for a large share of the measured ~1s/module:

1. **MISS-ownership (Section B)** — the *highest-impact* quadratics (dag_collect was the emit dominator;
   intern is ~30-60% of parse) are INVISIBLE to a shape-only lens because the O(n²) is runtime Rc
   aliasing, not tree shape. **Implication: the complexity lens must COMPOSE with the ownership/linearity
   model** — the same move analysis the ownership de-fork is building. A shape-only lens will keep missing
   the biggest wins. This is the top recommendation for the complexity manager.
2. **MISS-constant (Section C)** — un-memoized re-derivation (authored_name_at) and per-node allocation are
   constant-factor, below Big-O. A pure asymptotic lens can't rank them, yet authored_name_at alone is
   20-30% of emit. Needs a per-operation cost model (or a "recompute-vs-memoize" redundancy lens).
3. **MISS-necessity (Section D/11)** — pre_intern_tokens computes k² work but uses 3 results. Dead-work
   needs liveness; §5 marks this the undecidable residue. Likely never a hard wall, always a ratchet.

**Headline:** a shape-reading complexity lens would have caught roughly half the findings by count, but the
two biggest single costs (intern O(k²), dag_collect O(M²)) are BOTH MISS-ownership — so the lens's most
important growth is composition with the ownership model, not more shape rules.

## Corrections — calm-dove-209 located research 2026-07-05 (SUPERSEDES the audit-synthesis rows above)
The static parse audit over-claimed; call sites were verified, and located reality supersedes:
- **#11 `pre_intern_tokens` is NOT dead and NOT O(k²).** Both call sites are read downstream
  (`compile.dag:894` corpus-wide; `02_parse.dag:836` per-file). It is **~2× LINEAR-redundant**: the per-file
  rerun re-interns tokens already in the shared corpus-wide table, all hitting cheap `Present` branches.
  Fix = drop the redundant per-file rerun once callers rely solely on the corpus-wide table (NOT a blind
  delete — load-bearing on both sides). Reclassify **MISS-constant (~2×)**, not MISS-ownership+necessity.
  The MISS-necessity/dead-work example is **retracted** — pre_intern is redundant, not dead.
- **#12 is 4 sites, not 7, and LOW severity.** All 4 in `01_tokenize.dag`, Rc-clone-whole-vec-per-call, but
  bounded by interpolation-NESTING-DEPTH, not corpus size → not a corpus-scaled quadratic. Low priority.
- **NEW higher-severity finding (ROOT D, reference-carrier): `intern_str` O(table-size) lookup scan** at
  `00_core.dag:1290` — a `skip |> first` scan over the intern table instead of an O(1) index lookup, called
  once per identifier reference from `04_env`/`04_infer` → genuine **O(n·k) corpus-scaled**. This is the
  SAME anti-pattern as the already-fixed P2 `token_stream` `skip |> first` (#6241/#6255), so the fix mirrors
  it: reference the existing index by key (O(1)), not re-scan. This is the real parse/core corpus-quadratic
  here and it DISPLACES #11 as the parse/core priority. REFERENCE-carrier item; lens verdict CAUGHT-adjacent.
- **Deliverable-3 receipt:** `analyze_ownership` `merge_branch_usages` O(body²)→O(touched) fix compiles
  clean (901 sources, 0 errors); scaling receipt pending.

**§5 lesson:** the static audit synthesis is a *hypothesis*; located-by-execution reality corrects it.
pre_intern's "O(k²)+dead" was the audit reading the shape; the call sites say ~2× linear + load-bearing.
This is exactly the "treat your own output as unverified until a consumer runs it" discipline, applied to
an audit.
