# Plan — Fail-closed lock-down (make fail-open physically impossible to write)

**Status:** audit + lock-down checklist · **DESIGN.md + carriers are authority** (§6 no parallel ledger); each item dissolves into a wired CI gate when it lands. Linked from `ROADMAP.md §0`.

**Verified against the live tree 2026-06-21.** Line numbers are receipts; re-check before acting.

## 0. Thesis — three faces of one problem

Cache misses/flakes, un-wired lenses, and complexity violations are **the same failure**: a discipline is *modeled* but not **enforced fail-closed, by execution, in CI**. DESIGN §6 names it — *"the tier where the machinery exists but nothing gates on it"* (coverage by illusion). The lock-down goal: for every discipline, a fail-open state is **physically impossible to write** because a discovered CI witness goes RED on it. Lock the machine down *before* expanding (the operator's bet: build the efficient correct machine, dogfood daily, sell the pieces — infra/website/billing/backend — once each is locked).

The test for "locked": (a) wired into the floor (discovered + run), (b) fail-closed (RED on violation, never a warning), (c) green-by-execution with a **discriminating** input that goes RED when wrong.

**The governing principle is CONSTRUCTION, not validation** — now an axiom in [DESIGN §5](../../DESIGN.md) (lenses are the residue mechanism, §6): make the bad state *unwritable* (single authority / realization derived from model), reserve lenses for the genuinely-unstructurable. #5423's spec-only key lens shipping a false-green is the proof. So §4 below is ordered **construction-first**; the lens items are the residue that survives it. (Construction does *not* evict the executable hygiene backstop — see §4 meta.)

## 1. What IS enforced fail-closed today (the good baseline)

| Gate | Scope | Evidence |
| --- | --- | --- |
| DagCompileCleanGate | whole tree (fail-fast root) | `ci_spec.dag:41`; `ci_floor_plan.dag:82,106` |
| RustMonolithGate | `.rs` + manifest | `ci_spec.dag:37`; `rust_gates_ci.dag` |
| GeneratedArtifactDriftGate | `ci.yml` / `ROADMAP.md` / `.gitignore` byte-drift + per-artifact perturb, over the committed (= generated AND not-ignored) registry | `ci_spec.dag` `GeneratedArtifactDriftGate`; `generated_artifact_gate.dag`; registry+commit-derivation `gunbc/generated_artifact.dag` |

These are the **structural** gates. The gap is everything *analytical* and the *bootstrap purity* gate.

## 2. Coverage by illusion — authored but inert / vacuous (fail-open by inertia)

| Lens / gate | State | Why it enforces nothing |
| --- | --- | --- |
| **complexity**, **cost** | inert | single authority for the §1 time axis; **no `test fn`/gate runs them on the tree** — a mis-costed loop ships silently (`lens/complexity.dag`, `lens/cost.dag`) |
| **host_language_transport_script** | LIVE (#7184) | §5 literal-blob ban — ACTIVATED, no longer inert: a per-PR ReadsLiveTree consumer over the `shell.Exec.Run` anchor sites (`wall_residue_live_test.dag`) reds a NEW raw literal at an enrolled Run position. Deliberately green on `ComputedApplication` (the counted `retained_*` bridge calls), so it backstops literal blobs only. CONSTRUCTION closes computed joins at the `ShellOnHost.script` edge, where the `RetainedShellScript` record makes a bare `String` unwritable; direct `shell.Exec.Run` still accepts the transparent `TransportScript` brand, so computed/cast inputs there remain open pending meta-exec confinement. Corrected 2026-07-27: this row said the record closed the `shell.Exec.Run` computed-join class, but that wall is scoped to `ShellOnHost`. |
| **leaf_model_verification** | inert | 845-line R1–R3 realization matrix; test data never discovered |
| **extdeps_shape_transport_policy** | inert | the §3 shape/transport/policy enforcer — authored, not wired |
| affected_set, fact_cardinality, mock_totality, ownership, subsumption, … | inert | authored lenses, no discovered gate |
| **discrimination, synthesis, idempotency, parallelism** | advisory / roster | run, but over a **curated roster**, not the change-set — a violation outside the roster merges (`lens_*_family_eval_test.dag`) |
| **EmitHostGate** | wired but thin | exactly **4 MVP fixtures** (rust/python/go/ts), not the emit tree (`emit_host_gate.dag:27`) |
| **regen_stage0 --verify** (Stage0LockstepGate) | **not wired** | exists (`verify_stage0_matches`), absent from the floor → seed hand-drift uncaught; gate sits in **closed #5325** |

## 3. Where the pipeline actually fails OPEN (wrong answer passes silently)

| Site | Class | What it fabricates / masks |
| --- | --- | --- |
| `resolved_graph_cache.rs:146` | **cache flake** | content digest = hash of `from_utf8_lossy(bytes)` — **lossy decode, not raw bytes** → warm≠cold / wrong-hit physically possible. **VERIFIED.** |
| `resolved_graph_cache.rs:169-178` | cache key | subject/content digests read via `from_utf8(..).unwrap_or("")` — malformed → `""` → collision |
| `v1_interpreter.rs` `parse_table_memo` (~2810) | under-keyed | key `(grammar_digest, token_digest, position, production)` — cross-file position collision; no red witness |
| `v1_interpreter.rs` `pure_call_memo` (~2912) | key-by-address | keys on `Rc::as_ptr`, not content → stale hit on realloc; structurally-equal args miss |
| `v1_interpreter.rs:3108` | fail-open infer | `record_lit_type_name_at(..).unwrap_or_default()` → `""` type instead of error |
| `v1_interpreter.rs` shell out (3884/4289), service (4557/4709) | fail-open IO | `from_utf8_lossy` / `unwrap_or_default` fabricate plausible text from malformed output |

**No warm==cold purity oracle** exists for `resolved_graph_cache` payload, `parse_table_memo`, or `pure_call_memo` (only `resolve_typed_cache_equivalence_test` covers the typed-module cache). This is the "cache flake physically possible" surface, concretely.

Already fixed (precedent for the pattern): cross-representation `==` straddle (now raises `CrossRepresentationEquality`); strict record-field typecheck (#5293).

## 3a. How deep does the root go — the remaining audits

§1–§3 were the first two passes (lens wiring + fail-open code). They found *symptoms*. The lane needs deeper audits before we trust the fixes are complete, because the symptoms likely share **one root**.

**Suspected root — the model↔realization fork (DESIGN open thread, §1/§2/§7).** Every primitive is *modeled* as a coproduct and *realized* as a native `Value`, reconciled by **per-site bridges** — so coverage is accidental and non-compositional. That is the same shape as: the `==` straddle (`Value::eq` `_ => false`), the lossy cache digest (native bytes vs modeled content), and the under-keyed memos (native Rc address vs modeled content identity). If true, the fixes in §4 are *per-site patches* unless the root is dissolved (ground each primitive into its realization — numeric tower first, `Int = GroupCompletion<Nat>` bottoming in Peano `Nat`), which makes whole classes of guard *dead code*.

Audits to run (each: how many sites, how deep, is there one root):

1. **model↔realization fork audit** — ✅ DONE → [model-realization-fork.md](model-realization-fork.md). ROOT CONFIRMED (one seam, ~13 per-site bridges). Two sub-roots: numeric tower (grounds cleanly → guard dead code) + `Value::Null` overload (None/Absent/miss/Violates — needs *splitting*, the deeper root; ~131 legitimate `present==None→false` sites mean it can't be guarded).
2. **coercion/equality fail-closure audit** — the `==` fix landed for the numeric tower; remaining: `Bool True|False` over `Value::Bool`, `Optional/Witness` over the overloaded `Value::Null` sentinel (resists a blanket guard — `present == None` at ~131 sites is a *legitimate* `false`; needs grounding).
3. **inference fail-open audit** — what remains after strict record-field (#5293): return-type inference defaults, other `unwrap_or_default` type fabrications.
4. **cache-purity audit** — enumerate every cache (resolved_graph, parse_table_memo, pure_call_memo, typed_module, sccache); each must have a warm==cold oracle; only typed_module has one today.
5. **CI-coverage-completeness audit** — tests/gates that *exist but don't run*. Confirmed instance: the rust gate runs a "known-green subset" (`interp_recorded_fixture wet_hermetic resolve_expr_types_retraversal`, `ci_spec.dag:160`) of **60** `src/v1/tests` files — the rest rot silently (how the Behavior-arm test went stale). Also: the `discrimination` lens (the §5 discriminating-witness enforcer) is itself roster-only, not whole-corpus — the enforcer is vacuous.

The lane is done when these audits return *no new fail-open class* and §4 is green.

**Deepest recursion (DESIGN open thread #1) — lock down the reasoning, not just the code.** The same fail-closed discipline applies to the *argument*: model A1–A3 + the §1–§7 chain in `.dag` and have a lens enforce the syllogism — every claim a consequence-chain back to an axiom, **no orphan, no cycle** (the §4 acyclicity test turned on this document; the §7 recursion). Currently not modeled at all (grep empty). This is the apex lock-down: a design claim with no axiom-chain is an orphan = should be RED.

## 4. Lock-down checklist (blocks expansion)

Two tiers: **construction** (the class becomes unwritable — this is the real work) then **lens** (the residue that genuinely can't be made impossible by construction). The cache items are the operator's top priority.

**Tier 1 — construction (make the class unwritable):**

- [ ] **Dissolve the model↔realization fork — THE root** ([model-realization-fork.md](model-realization-fork.md)): realization derived from model. (1) numeric tower `Int=GroupCompletion<Nat>` → the `==` straddle guard becomes dead code; (2) split `Value::Null` (None/Absent/miss/Violates → own carriers).
- [ ] **Cache key derived FROM declared `inputs_considered`** (single authority) → cannot declare an input you don't key, nor key one you don't declare; divergence unwritable. Subsumes: content-key on **raw bytes** (`resolved_graph_cache.rs:146`), content keys for `parse_table_memo`/`pure_call_memo` (kill position/address keys). *(worked first instance: child adhoc-cc232dbc-1be)*
- [ ] **Self-host purity by construction** — emitter emits the whole seed so `patch_*`/`HAND_MAINTAINED_STAGE0_FILES` are unwritable; `regen_stage0 --verify` (Stage0LockstepGate, #5325) then a residue check.
- [ ] **Widen/retire the rust gate** — run the v1 test set or explicitly retire it (no test exists-but-doesn't-run).

**Tier 2 — lens (only the genuinely-unstructurable residue; each must justify why not construction):**

- [ ] **Complexity / cost / necessity** — legitimate lens (can't structurally forbid an *unnecessary* loop); fix cost-lens zero-absorption (`symbolic_max` floor), then gate the change-set (not a roster) → §6.
- [ ] **Cache-redundancy completeness** (§7 P3) — shared-across-boundary → ERROR; reach, not measurement; the residue that survives the content-key construction.
- [ ] **warm==cold purity-oracle witness per cache** — residue check behind the content-key construction (guaranteed once the key is fn-of-inputs, but cheap to witness).
- [ ] **Promote-or-delete every inert lens** (extdeps_shape_transport_policy, leaf_model_verification, …; host_language_transport_script is DONE — promoted live by #7184); de-vacuum thin gates (EmitHostGate beyond 4 fixtures; advisory rosters → change-set); the `discrimination` enforcer is itself roster-only — whole-corpus it.

**Meta — two layers, the executable one is load-bearing:**

- **inert-lens hygiene (executable backstop):** every `lens/*.dag` has a discovered fail-closed witness, or is **removed** — an inert lens is a lie. This *runs* over the corpus and is the thing that actually closes the "authored ≠ enforced" loophole.
- **construction-justification rule (layered ON TOP, authoring-time judgment):** before adding any lens, justify why the class can't be made impossible by construction; convert what can (single authority / realization-from-model); lens-justify only the unstructurable residue. This is a better *principle* but does **not** supersede the executable backstop — a judgment applied at authoring time executes nothing, so it cannot replace a check that runs over the corpus.

## Dissolution trigger (DESIGN §6)

Delete this doc when the §4 checklist is green: every discipline has a discovered, fail-closed, discriminating CI witness, and the meta-invariant gate exists so a new inert lens cannot be added. At that point the floor *is* the lock-down and this audit is redundant.
