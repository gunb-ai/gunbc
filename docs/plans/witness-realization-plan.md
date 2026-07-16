# Witness realization — the v2 execution path (plan)

**Anchor scenario (operator, 2026-07-14):** emit witnesses to native code (Rust now; C stays a row-policy option) and use realization/materialization to schedule the runs. Companion to [v2-memory-control-audit.md](v2-memory-control-audit.md) — that doc is the findings; this one is the dependency-ordered work that discharges them. Nothing here waits on self-hosting.

## Two crutches, separable

"v1 as a crutch" is two distinct dependencies:

1. **Execution crutch** — every v2 witness evaluates on the v1 tree-walker (`src/v1/stage0/src/bin/claim_executor.rs:26` imports `v1_interpreter`; audit F10). v2 has no execution path of its own. **This plan removes this one.**
2. **Compiler crutch** — the compiler binary is the v1 seed. Removing it is §1 self-hosting, a separate lane. Emit-on-demand deliberately RIDES the seed emitter (ROADMAP ④: *"emit a v2 closure to Rust via the seed"*) — the artifact is keyed by content hash, so WHICH emitter produced it is part of the **key**, never part of the architecture.

The target pipeline, per witness run:

```
closure → ContentHash → artifact-store lookup
  hit  → run native → record receipt (wall + peak RSS, subject-keyed)
  miss → resolve → emit (seed) → build (sccache-warm) → store → run → record
schedule = realize: min(independence_width, budget / measured_peak)   [spine FLAG D]
```

## Economics (receipts: the 2026-07-14 whole-corpus Pi run, `.forensics/claim_full.log`)

| Today (interpreted, v1) | Under this plan |
|---|---|
| resolve = 82.5 min, **87%** of the run; superlinear (fit exponent ~1.55 in closure size, 3.6× over linear) | paid once per content hash; a warm run pays source-hashing + exec |
| eval = 12.4 min tree-walked | native (S1 census receipt: compiled 1.1 ms linear vs interpreted DNF) |
| retention: VmSize 6.25 GB, ~8.4 MB/entry, never released; 13 GB swap to finish | per-run process frames (`ScopeExit` realized by process exit) + `SpacePacked` store budget |
| OOM = untyped exit 137; governor reactive (474 back-offs, "time death caused by a memory shape") | packing on measured peaks; unknown-peak subjects run in a width-1 maturation reserve |

## The steps

**P0 — the field wall (audit F1).** Field **presence** + field **type** checking in `04_infer`'s record-literal arm. Realization's scheduling facts (`CostAccount`, `CacheProvider`, receipts) are all record literals; while presence/types are unchecked, every downstream derive can read garbage at `0 diagnostics`. Staged like the wrapper-retained frontier: (i) ledger mode — typed, counted roster of corpus violations; (ii) drain; (iii) hard wall. ACCEPT: the audit's four probes go red; corpus green after drain. Every roster row found in (i) is a latent bug today.

> **P0 STATUS (2026-07-15 — LANDED, drained to 0, hard wall live; session lively-heron-614 → cool-newt-143):** wall LANDED in the v1 authority (`src/v1/04_infer.dag`: presence via trusted field authorities only; `kernel_value_declared_type_mismatch` tier wired into record fields AND `direct_call_arg_type_mismatch`; new `MissingField` diagnostic in `v1.std.core`). All four audit probes RED with located diagnostics; `diagnostics_witness record_field_walls` suite (5 REDs + 3 GREENs) green **and CI-wired** (`diagnostics_record_field_walls_hold` enrolled in `ci_layer_roots.bin_witness_wet_entries`, since `diagnostics_test.dag` is discovery-excluded). The wall is a **hard wall**: `MissingField` is interpreter-blocking, so batch-1 `dag_compile_clean_gate` (`--target dag`) reds on any residue — and that gate is now **green at 0** (`gunbc compile --source-root dag --source-root src/v2 --target dag` → `0 diagnostics`; whole-tree ledger **568 → 45 → 0**; the 909-file discovery-leaf sweep → 0 field-wall errors).
>
> **The three operator rulings — RESOLVED (Path A: drain to 0, keep the hard wall):**
> 1. **Empty-literal tag idiom** — took (a): `is_zero_field_variant_tag_reference` in `infer_record_lit` sanctions a zero-field literal *only* when it names a variant owner (`variant_owner_node`); partial literals and non-variant zero-field records stay red. Dissolve-on = a first-class variant-tag carrier (`zero_field_variant_tag_reference_frontier_note`).
> 2. **String into a `Secret` field** — took (a): the test constructs the value via the sanctioned explicit `"" as Secret` cast (`filesystem_write_closure_scale_witness.dag`), not a `String→Secret` rule.
> 3. **String vs `Coproduct(FreeMonoid)`** — grounded: `Int→Nat` and `String→FreeMonoid` are **grounded identities** in their own `grounded_primitive_coproduct_identities` list (`dag/std/coercion.dag`), consulted by `dag_can_cast` but **not** `is_dag_cast_domain_type`, so the tower unblocks without dragging `Nat`/`FreeMonoid` into `as`-cast validation (which would regress pre-existing `Nat as Int` / `Int as String` sites). Non-grounded straddles (`Bool→FreeMonoid`, `String→AuthScheme`, `String→Doc`) stay red by construction.
>
> **Discovery-corpus drain (beyond the compile closure — the wall fires on every compiled file, incl. discovery-only test/manual leaves that local whole-tree compile never reaches):** `occurrence_id: SyntheticOccurrence` on `Node` fixtures (`find_witness_project_to_core_controls`, `batch_runner`, `module_resolution_plan_witness_test`, `realization_runner_witness_test`); `field_label_separator` / `token_class_emit_transforms` on the Rust emit models (`rust.dag`, `rust_wire_serde.dag`); `InterpretationAlgebra.match: MatchInterpreter { choose_arm: runtime_reject_match_arm }` (`infer_ground_add`); `card_intake` disposition count moved into a `fn count_flagged` that `fold`s over results (the earlier `[match …]` for-body built `List<List<Bool>>` — `eval_for_each` is a map, not flat-map — and lists are truthy, so the count equalled total always; **fixed and proven by execution**, 2 of 5 → 2, old form → 5); and `credential_chain`'s `scheme` param — a stringly `"Bearer"` nickname the wall exposed as a latent `String→AuthScheme` mismatch, and non-CLI-expressible as an enum — dropped, `build_token` now takes the `Bearer` literal directly (§4 CLI boundary).

**P1 — space becomes observable (F2, F4).** `ObservePeakResidentAtSubject` as a second variant on `RealizationMeasureEffect`. The host physics **already exists unmodeled** — `claim_batch` reads `getrusage(RUSAGE_CHILDREN)` and prints `[measurement] children-max-rss` — so this is a mark-on-the-carrier step, not new physics. `PerformanceReceipt` gains the peak field. `space_measure_seq` = **max** / `space_measure_par` = **add** as peers of the time rules; the `fleet_intent.dag:168` sum fixed; the keystone witness flipped. ACCEPT: first `CostAccount.space` with `basis: Measured` produced by execution; RED: a seq-composition asserting sum goes red.

**P2 — the artifact store: the corpus's first REAL `CacheProvider`.** `ArtifactKey = hash(closure source set, emitter id, target, toolchain)`. `ArtifactTier`/`ContentKeyed`, `SpacePacked{budget}` — and the store's budget enforcer is the first **reader** of `eviction` in the tree (F5). `ExistenceKeyed` unwritable by shape: the key *is* the hash; presence/mtime never consulted (#6352 wall, by construction this time). ACCEPT: put/get/evict receipts. REDs: source mutation → new key → miss (stale never served); budget breach → drop + **counted** recompute.

> **P2 STATUS (2026-07-16, session lively-heron-615): both halves LANDED, green by execution.** Pure spec: `dag/std/artifact_store.dag` — `ArtifactKey`→`artifact_key_hash` via `std.content_hash`; `store_over_provider` **reads** `CacheProvider.eviction` and refuses non-`SpacePacked` providers (typed `StoreRefusedEviction` — F5's first behavioral consumer); `store_get` bumps recency; `store_put` packs to budget by least-recent eviction, every eviction **counted** in `StorePutReceipt`. Host transport: `dag/extdeps/realization/artifact_store_fs.dag` — path encodes identity (`store_root/<hash>.artifact`), roundtrip + mutated-input-miss proven wet against the real `Filesystem` service (artifact observed on disk at its hash path). Witnesses: 4 pure (`artifact_store_witness_test.dag`) + 2 wet (`artifact_store_fs_witness_test.dag`), all green. **Named gaps (next rungs, not silent widens):** (a) the cited `Filesystem` service has no `Delete`/`List`, so `SpacePacked` enforcement on the *persistent* tier is unrealizable until those ops land — the in-process fold is the only packed tier today; (b) the store's enforceable budget is a typed parameter beside the provider row because `EvictionClass.budget` carries cited upstream policy *prose* in extdeps rows — two concepts in one field; the variant split is the dissolve-on (recorded in the module note).

**P3 — emit-on-demand v0 (ROADMAP §1's own accept criteria).** Wrap the landed `emit_host` transport (15 `emit_host_*_equals_eval` witnesses with REDs) with the P2 store. First customer per ROADMAP: the parse census / one witness family. ACCEPT: (a) interpreted==native agreement witness; (b) second-run zero-compile store-hit receipt; (c) the interpreter's surviving roles named + bounded. Needs FLAG A.

**P4 — `realize` v0: the memory-packed scheduler.** Inputs: the runnable set; per-subject `CostAccount` (Measured from P1 receipts of prior runs; typed `CostUnknown` for first-timers); a **typed** host-budget read (unreadable → refuse — retiring `conservative_fallback_width`, the audit's live §5 finding). Law: `min(independence_width, budget/peak)` (spine FLAG D). Unknown-peak subjects run in a width-1 maturation reserve, then their receipt converts them — this is the `memory_governor`'s admission logic made predictive and modeled, and its dissolve-on (*"graph-derived per-node demand replaces the reactive estimator"*) finally fires. **ACCEPT on this Pi:** the whole hermetic corpus completes on 905 MB with flat VmSwap — the machine where memory-blind packing dies is the standing RED control. Exit-137 becomes a refusal class, not a death.

**P5 — demands derived, not hand-typed.** `dependency_view` over the roster: N entries sharing a closure = plurality ≥2 across isolation → memo obligation at the LCA → discharged by the P2 provider. `materialize`'s `Share` verdict drives actual reuse for the first time; `materialization_carriers.dag:41`'s dissolve-on fires. ACCEPT: zero duplicate builds measured across a run.

**P6 — cutover + falsifier; F10 cleared.** `SubstrateInputsOnly` witnesses route native. A nightly sampled interpreted==native falsifier (the affected-set-falsifier pattern) keeps the agreement honest. The interpreter is retained for compile-time eval + bootstrap receipts only, each a counted frontier row with its dissolution trigger. The v1-deletion ordering constraint dissolves correctly: eviction and scheduling authority now live in P2/P4 carriers, not in v1 Rust.

**Horizon — verdict-level Share.** `TestClaimCacheKey` (`src/v2/compiler/05_eval.dag:925`) gets its store: a pure witness whose closure hash is unchanged makes the *run itself* a `Share`. Affected-set selection then becomes a derived special case of materialization instead of a parallel mechanism.

## FLAGs (operator sign-off)

- **FLAG A — hermetic admissibility of builds.** `emit_host_run_transport` refuses under hermetic mode today. Ruling needed: (1) a build from substrate-only inputs with a **pinned toolchain in the key** is a derived realization, admissible hermetically (recommended); or (2) builds happen only in a wet/record lane and hermetic runs consume the prebuilt store.
- **FLAG B — artifact granularity.** v0 = one binary per entry closure (simple; sccache dedups at the object level underneath). Module-grain shared artifacts come later, DERIVED by ladder plurality at module grain — not hand-chosen now.
- **FLAG C — C as a target.** C = authoring `extdeps/languages/c` rows (the §4 one-grammar-both-directions architecture keeps this open at constant price) **plus** a realization story for the `v1_rt`/persistent-carrier runtime — that second half is the real cost, and today it has zero displaced-cost justification. Rust is proven (census receipt). Defer until a pain names it (e.g. hosts where the Rust toolchain is prohibitive).

## Non-goals

GC / arena / bytecode VM / JIT (the design's standing rejection — correct, keep it). New interpreted-workload features. v1 emitter investment beyond what P3 consumes.
