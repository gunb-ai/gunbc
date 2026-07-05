# Emitter ownership de-fork: one authority for clone-vs-move, no silent fallback

Lane: `node://adhoc-0717d295-672` (session snappy-newt-504, parent cool-hawk-899 / PR #6248).
Displaced cost (§6): whole-tree `compile --target dag` measured at ~72 min, ~85–90% of it emit;
the emitter's default-clone paths keep `Rc` refcounts ≥ 2, so every `rc_map_insert` /
`rc_list_push` (`Rc::make_mut`) copy-on-writes the *whole* container per iteration — O(n²)
denominated in the corpus, surfacing as the CI timeout (§5's absorbing-fallback cost signature).

## The fork (§3)

The Rust emitter decides clone-vs-move in **three** places, each with its own predicate,
while the correct verdict already exists and is bypassed:

1. `movable` — from `ownership.dag build_movable_set`; governs whole-value var moves
   (`05_emit_rust.dag` `emit_var_ref`, `moves_by_value`). Before increment 1 it re-derived
   its own criterion (`binding_fan_out == 1`, `is_owned_local`), conflating Read/Projected
   *borrows* with *consumes* and excluding params entirely — so a param passed once by value
   still cloned at its use site.
2. `read_only_params` — from `build_read_only_params`; governs param borrows.
3. `owned_bindings` — governs **field** moves (`emit_typed_field_access` StoredField,
   `base_is_owned`), and is not built by `ownership.dag` at all: it is an ad-hoc emit-time
   set populated only with fold-accumulator names inside `emit_rust_fold_method_call`.
   So `acc.seen.clone()` — a field projection on a *parameter*, the dominant `dag_collect`
   cost — is governed by set 3, which can never contain it, and the correct verdict
   (`make_decision` → `SoleOwner`) is never consulted.

The single authority is `ownership.dag`'s `OwnershipProof` (`make_decision` over
`semantic_consumer_count`, i.e. Consumed edges only, branch-joined). Everything the emitter
does must be *derived* from it (§5 construction-over-validation: the emitter must not be
able to *state* a second ownership opinion).

## The silent fallback (§5)

`emit_typed_fold_lambda` emits `Rc::try_unwrap(acc).unwrap_or_else(|rc| (*rc).clone())`.
When the structural proof's residual assumption (the Rc is unshared at runtime) fails,
the arm silently widens to a whole-value clone: ⊤-as-answer standing in for
⊤-as-ignorance. Its frequency is zero-by-construction in every metric we watch, so the
deficit never ranks — the textbook absorbing fallback DESIGN.md §5 names. The rule:
**the arm must refuse or be counted, never silently widen.**

## Increment 1 — de-fork set 1 (whole-value moves) [in this PR]

`build_movable_set` now filters on `make_decision(usage) == SoleOwner` and admits params
via a `param_names` argument (call site `build_ownership_results` passes
`entry.param_names`). Discriminating witness:
`dag/test/claim/ownership_movable_test.dag` (projected-then-consume param movable;
double-consume not; zero-consume not; whole-read-then-consume not; threaded-then-consume
not).

Soundness (the wall that makes per-name movability sound at all): `movable` gates **every**
whole-value var-ref emission (`emit_rust_expr_var` → `emit_var_ref` is reached for call
args, fold inits, record fields — not just tail returns), so a per-name move license is
sound only if the binding has exactly **one** whole-value use site. `Consumed` is recorded
only at tail position (last in evaluation order); `Read`/`Threaded` edges are whole-value
positions emitted *before* it, so any such edge plus a licensed move = use-after-move.
Hence the rule: SoleOwner **and** zero Read/Threaded edges (`whole_value_borrow_count == 0`).
`Projected` edges stay compatible because field-access bases emit the bare ident
(`emit_typed_expr_base`), never a whole-value consume. The original increment-1 draft
admitted Read-then-Consume; that was a latent use-after-move and is fixed here — the
per-site verdict below recovers those sites soundly.

## Increment 2 — de-fork set 3 (field moves) on a per-site verdict [design]

Per-function name-sets cannot express "move `acc.seen` here because `acc` dies on this
path". The verdict moves from per-name to per-site (Perceus-style last-use; `.dag` is pure
and tree-shaped, so last-use is mechanical — no borrow checker needed).

Status: proof-side machinery AND whole-value emitter consumption are implemented;
field-move emitter consumption (entry take-owned generalization) remains. Wired:
`OwnershipBuildResult.move_sites_index` → `EmitGraphInfo.move_sites_index` / per-fn
`move_sites` (TCO functions get an empty map — loop-rewritten bodies reuse bindings across
iterations, so per-site last-use does not transfer; fail closed) → `emit_var_ref` moves
when `movable` (per-name) OR `move_licensed_at_site` (per-site, span-keyed,
`read_only_params` excluded, span-0 refused). Validation-time obligations: (a) a counted
licensed-but-cloned check in `count_ownership_violations` so a span-keying drift is
observable, not a silent clone (parent guard); (b) `count_ownership_violations` /
`ownership_movable_test` call `build_movable_set` with the new `param_names` arg once the
regenerated seed lands (the seed-side Rust test still compiles against the old signature
until then); (c) a by-execution RED/GREEN test for `take_owned_counted` counts.

Field-move step (not yet wired, design addendum): emitting `base.field` as a move is only
valid on an owned (non-`Rc`) base, so licensed field moves require the generalized entry
take-owned: params whose every use is a `Projected` edge (zero whole-value edges) on a
shared-type param get `let x = v1_rt::take_owned_counted(x, site)` at fn entry; licensed
projection sites then move, unlicensed ones clone the field (cheap `Rc` bump for container
fields). This is what deletes `owned_bindings` as an authority — the fold accumulator
becomes the special case of the same rule. Held for compile feedback (post-fixpoint):
blind-writing the `emit_fn_def` entry rewrite without `cargo build` of a regenerated seed
risks template-level use-after-move that only rustc can adjudicate.

Model (in `ownership.dag`, the one authority):
(`build_move_site_licenses` — a backward liveness walk in reverse evaluation order,
O(AST)) with unit witnesses in `ownership_movable_test.dag`; emitter consumption is the
remaining step. Guards baked in: a zero/absent span never licenses (synthetic nodes fail
closed); lambda/foreach bodies license nothing and poison their captures live; branch arms
compute licenses per-path from the shared continuation liveness and merge by union
(licenses) / OR-union (liveness). The emitter must look up a verdict by the same key the
walk recorded: `name@span_start` of the var node for whole moves, of the field-access node
for field moves.

Model (in `ownership.dag`, the one authority):

- `EdgeClassification` already carries `span_start` — the site key exists.
- New carrier: `UseSiteVerdict = MoveWhole | MoveField { field: String } | Borrow
  | CloneShared { decision: OwnershipDecision }`, produced per (binding, span_start).
- Last-use computation: `walk_expr` already visits in evaluation order. The lossy join is
  `merge_branch_usages` / `max_usage_by_fan_out`, which keeps one branch's edge list;
  it must instead keep per-branch edge lists so "no later use on any path from this site"
  is decidable. A site is `MoveWhole`/`MoveField` iff it is the final use of the binding
  on every path through it.
- Generalize `FoldAccUnwrapProof` (the existing structured field-move proof: constructs-acc,
  whole-acc-single-use, safe-field-moves) from fold-accumulators to *any* owned-entry
  binding, params included: if a binding's uses are exactly distinct field projections
  (each field ≤ 1 move per path, no whole-value use after any field move), the binding is
  take-owned at entry and each projection emits a move.

Emitter changes (all *readers* of the proof):

- `EmitGraphInfo` carries the per-fn `site_verdicts` index instead of the three sets.
- `emit_var_ref`, `emit_typed_field_access`, `emit_cloned_arg` key on
  (binding, use-site span) and emit exactly what the verdict says.
- `owned_bindings` is deleted as an authority: the fold accumulator becomes an ordinary
  owned-entry binding whose entry take-owned and field moves fall out of the same proof
  (`analyze_single_fold` stops being consulted at emit time — the proof rows in
  `OwnershipProof.fold_acc_unwrap` already exist and become the only surface).

## §5 conversion of the fallback [lands with increment 2, first in sequence]

Replace the emitted `Rc::try_unwrap(x).unwrap_or_else(|rc| (*rc).clone())` with a runtime
helper `v1_rt::take_owned_counted(x, site: &'static str)`:

- try_unwrap; on failure increment a named per-site counter (same instrumentation family as
  `record_source_chars_slice_walked`) and clone **once** — the degradation is typed,
  located, countable, and observable in the stats dump; never silent.
- Sites the proof classifies `SharedError` (genuine sharing) stay clones *and are counted*:
  that counter ranking is exactly the worklist for persistent-structure/HAMT candidates.
- RED control: a fixture with a deliberately shared init Rc must count 1; the reconciled
  fixture must count 0.

The helper lands in both runtimes: `v1_rt.rs` (seed) and the emitted-runtime template
(`v1_compiler_runtime_rust.rs` source in `.dag`).

## Guards / acceptance

- `bootstrap_fixed_point` byte-identical emit fixpoint (blocked-on: v1 self-resolution
  repair, sibling clever-swift-531 — until it greens, a v1 `.dag` change can only be
  validated by hand-syncing the seed; this PR stays **draft** and does not hand-sync).
- `dag_collect` emit receipt: minutes → seconds on the whole tree.
- `pipeline.rs` ownership ratchets move the right way: `movable_but_cloned` (45) drops,
  `try_unwrap_fallbacks` string census goes to 0 *silent* occurrences (the counted helper
  replaces the string), stage0 clone census drops (currently ~1138 over budget, FLAG'd).
- Non-collision: stern-dove-88 (PR #6242) fixes the immediate `dag_collect` cost at the
  `.dag` source level; this lane is the systemic emitter root (~167 sites). W1's source
  change stays valid — moves are strictly cheaper after both.
