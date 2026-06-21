# Plan — Fail-closed lock-down (make fail-open physically impossible to write)

**Status:** audit + lock-down checklist · **DESIGN.md + carriers are authority** (§6 no parallel ledger);
each item dissolves into a wired CI gate when it lands. Linked from `ROADMAP.md §0`.

**Verified against the live tree 2026-06-21.** Line numbers are receipts; re-check before acting.

## 0. Thesis — three faces of one problem

Cache misses/flakes, un-wired lenses, and complexity violations are **the same failure**: a discipline is
*modeled* but not **enforced fail-closed, by execution, in CI**. DESIGN §6 names it — *"the tier where the
machinery exists but nothing gates on it"* (coverage by illusion). The lock-down goal: for every
discipline, a fail-open state is **physically impossible to write** because a discovered CI witness goes
RED on it. Lock the machine down *before* expanding (the operator's bet: build the efficient correct
machine, dogfood daily, sell the pieces — infra/website/billing/backend — once each is locked).

The test for "locked": (a) wired into the floor (discovered + run), (b) fail-closed (RED on violation,
never a warning), (c) green-by-execution with a **discriminating** input that goes RED when wrong.

## 1. What IS enforced fail-closed today (the good baseline)

| Gate | Scope | Evidence |
|---|---|---|
| DslCompileCleanGate | whole tree (fail-fast root) | `ci_spec.dag:41`; `ci_floor_plan.dag:82,106` |
| RustMonolithGate | `.rs` + manifest | `ci_spec.dag:37`; `rust_gates_ci.dag` |
| LayeringImportsGate | whole tree | `ci_spec.dag:39`; `layering_imports_gate.dag:39` |
| ResolvedImportsGate | whole tree | `ci_spec.dag:40`; `resolved_imports_gate.dag:47` |
| CiYamlGate | `ci.yml` byte-drift + perturb | `ci_spec.dag:42`; `ci_yaml_gate.dag` |

These are the **structural** gates. The gap is everything *analytical* and the *bootstrap purity* gate.

## 2. Coverage by illusion — authored but inert / vacuous (fail-open by inertia)

| Lens / gate | State | Why it enforces nothing |
|---|---|---|
| **complexity**, **cost** | inert | single authority for the §1 time axis; **no `test fn`/gate runs them on the tree** — a mis-costed loop ships silently (`lens/complexity.dag`, `lens/cost.dag`) |
| **host_language_transport_script** | inert | §5 literal-blob ban; no gate asserts it RED — a `shell.Exec.Run` can regress to a bare string |
| **leaf_model_verification** | inert | 845-line R1–R3 realization matrix; test data never discovered |
| **extdeps_shape_transport_policy** | inert | the §3 shape/transport/policy enforcer — authored, not wired |
| affected_set, fact_cardinality, mock_totality, ownership, subsumption, … | inert | authored lenses, no discovered gate |
| **discrimination, synthesis, idempotency, parallelism** | advisory / roster | run, but over a **curated roster**, not the change-set — a violation outside the roster merges (`lens_*_family_eval_test.dag`) |
| **EmitHostGate** | wired but thin | exactly **4 MVP fixtures** (rust/python/go/ts), not the emit tree (`emit_host_gate.dag:27`) |
| **regen_stage0 --verify** (Stage0LockstepGate) | **not wired** | exists (`verify_stage0_matches`), absent from the floor → seed hand-drift uncaught; gate sits in **closed #5325** |

## 3. Where the pipeline actually fails OPEN (wrong answer passes silently)

| Site | Class | What it fabricates / masks |
|---|---|---|
| `resolved_graph_cache.rs:146` | **cache flake** | content digest = hash of `from_utf8_lossy(bytes)` — **lossy decode, not raw bytes** → warm≠cold / wrong-hit physically possible. **VERIFIED.** |
| `resolved_graph_cache.rs:169-178` | cache key | subject/content digests read via `from_utf8(..).unwrap_or("")` — malformed → `""` → collision |
| `v1_interpreter.rs` `parse_table_memo` (~2810) | under-keyed | key `(grammar_digest, token_digest, position, production)` — cross-file position collision; no red witness |
| `v1_interpreter.rs` `pure_call_memo` (~2912) | key-by-address | keys on `Rc::as_ptr`, not content → stale hit on realloc; structurally-equal args miss |
| `v1_interpreter.rs:3108` | fail-open infer | `record_lit_type_name_at(..).unwrap_or_default()` → `""` type instead of error |
| `v1_interpreter.rs` shell out (3884/4289), service (4557/4709) | fail-open IO | `from_utf8_lossy` / `unwrap_or_default` fabricate plausible text from malformed output |

**No warm==cold purity oracle** exists for `resolved_graph_cache` payload, `parse_table_memo`, or
`pure_call_memo` (only `resolve_typed_cache_equivalence_test` covers the typed-module cache). This is the
"cache flake physically possible" surface, concretely.

Already fixed (precedent for the pattern): cross-representation `==` straddle (now raises
`CrossRepresentationEquality`); strict record-field typecheck (#5293).

## 3a. How deep does the root go — the remaining audits

§1–§3 were the first two passes (lens wiring + fail-open code). They found *symptoms*. The lane needs
deeper audits before we trust the fixes are complete, because the symptoms likely share **one root**.

**Suspected root — the model↔realization fork (DESIGN open thread, §1/§2/§7).** Every primitive is
*modeled* as a coproduct and *realized* as a native `Value`, reconciled by **per-site bridges** — so
coverage is accidental and non-compositional. That is the same shape as: the `==` straddle (`Value::eq`
`_ => false`), the lossy cache digest (native bytes vs modeled content), and the under-keyed memos
(native Rc address vs modeled content identity). If true, the fixes in §4 are *per-site patches* unless
the root is dissolved (ground each primitive into its realization — numeric tower first,
`Int = GroupCompletion<Nat>` bottoming in Peano `Nat`), which makes whole classes of guard *dead code*.

Audits to run (each: how many sites, how deep, is there one root):

1. **model↔realization fork audit** — enumerate every primitive modeled-as-coproduct + realized-as-native
   and its reconciliation site; classify fail-open vs fail-closed; confirm/refute the single root.
2. **coercion/equality fail-closure audit** — the `==` fix landed for the numeric tower; remaining:
   `Bool True|False` over `Value::Bool`, `Optional/Witness` over the overloaded `Value::Null` sentinel
   (resists a blanket guard — `present == None` at ~131 sites is a *legitimate* `false`; needs grounding).
3. **inference fail-open audit** — what remains after strict record-field (#5293): return-type inference
   defaults, other `unwrap_or_default` type fabrications.
4. **cache-purity audit** — enumerate every cache (resolved_graph, parse_table_memo, pure_call_memo,
   typed_module, sccache); each must have a warm==cold oracle; only typed_module has one today.

The lane is done when these audits return *no new fail-open class* and §4 is green.

## 4. Lock-down checklist (the meta-gate — blocks expansion)

Ordered so each makes a class of fail-open *unwritable*. The cache items are the operator's top priority.

- [ ] **Cache flakes impossible**
  - [ ] content-key on **raw bytes**, not `from_utf8_lossy` (`resolved_graph_cache.rs:146`); fail-closed on malformed
  - [ ] content-hash keys for `parse_table_memo` + `pure_call_memo` (kill position/address keys)
  - [ ] **warm==cold purity-oracle witness per cache** (discovered, RED on divergence) — the structural ban
  - [ ] realizer-key lens (P1.1): executed digest ⊇ declared `inputs_considered`, else RED (would have caught the lossy collision structurally)
- [ ] **Self-host purity enforced** — wire `regen_stage0 --verify` (Stage0LockstepGate, #5325) into the floor; dissolve `patch_*`/`HAND_MAINTAINED_STAGE0_FILES`
- [ ] **Complexity/cost enforced** — fix cost-lens zero-absorption (`symbolic_max` floor), then wire a complexity-budget gate over the change-set (not a roster) → §6 whole-codebase
- [ ] **Cache redundancy fail-closed** — land the §7 P3 redundancy-only completeness cut (shared-across-boundary → ERROR; needs reach, not measurement) **before expansion**
- [ ] **Promote inert lenses** — for each authored-but-inert lens (host_language_transport_script, extdeps_shape_transport_policy, leaf_model_verification, …): wire a discovered gate with a discriminating RED, or delete it (an inert lens is a lie)
- [ ] **De-vacuum the thin gates** — EmitHostGate beyond 4 fixtures; advisory rosters → change-set enumerated
- [ ] **Meta-invariant** — a lens/gate hygiene check: every `lens/*.dag` has a discovered fail-closed witness, or is removed (closes the "authored ≠ enforced" loophole permanently)

## 5. Dissolution trigger (DESIGN §6)

Delete this doc when the §4 checklist is green: every discipline has a discovered, fail-closed,
discriminating CI witness, and the meta-invariant gate exists so a new inert lens cannot be added. At that
point the floor *is* the lock-down and this audit is redundant.
