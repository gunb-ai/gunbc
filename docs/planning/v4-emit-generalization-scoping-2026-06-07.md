# v4 Emit-Generalization Scoping (DESIGN-ONLY, NO-GO)

> **Status:** DESIGN-ONLY — **NO implementation GO** from this doc. Operator asked to
> "get work moving" by scoping the mid-arc lever; dispatch lanes below are sequenced and
> gated, not authorized to start without a named consumer + manager sign-off per lane.
>
> **Session:** `loyal-owl-321` (Emit-generalization SCOPING phase)
> **Work item:** `node://adhoc-a5179ee3-934`
> **Thesis anchor:** THESIS.md omni-emission (Shape A vs Shape B), coercion = emission,
> derived homomorphism (model local, derive global).
> **Ground truth:** [`docs/v4-compiler-migration.md`](../v4-compiler-migration.md) Part 0
> (emit does not execute today); marks in `.dag` files remain authoritative over this doc.

---

## TL;DR

**Emit generalization beyond `emit(add)` is not one project — it is a tiered expansion ladder
already encoded in the claim corpus.** The keystone is a single executed witness:
`run(emit(add)) == rust_mvp1_source_text`. Every higher tier adds one structural dimension
(type expressions, non-grammar nodes, reference layers, value expressions, multi-target parity,
source ingest, emit→ingest round-trip) and is **pulled only by the next real consumer** (E-10).

**Recommendation:** treat `05_emit.dag` as frozen orchestration (`translate ∘ serialize_target`
only — per RR-C C.4). All generalization work lands in `06_translate.dag`, `extdeps/languages/*`
TargetModel population, and claim upgrades — never new compiler emit paths. The mid-arc lever
is **making the keystone execute**, then climbing the ladder one tier at a time; parallel
substrate dissolution (`find_witness`, `fold_node` memoization, coproduct morphisms) is justified
only when a green-running consumer on that tier demands it.

**NO-GO until:** S1/S2 from v4-compiler-migration Part 5 close — an executed `emit(add)` Bool
witness runs green (not CompilesClaim-only, not v3 text-grep).

---

## 1. What "beyond emit(add)" means

Today the only end-to-end emit slice with a named golden string is MVP-1 **add**:

```dag
fn add(x: i32, y: i32) -> i32 { x + y }   // Rust golden: rust_mvp1_source_text
```

The **surface API is already general** — `emit(tree: InferredTree, target: TargetModel)` in
`src/v4/compiler/05_emit.dag` accepts any inferred tree and any target model. What is *not*
general is everything downstream:

| Layer | File | Current reach |
|-------|------|---------------|
| Orchestration | `05_emit.dag` | `bind_outcome(translate, serialize_target)` — **done, do not expand** |
| Homomorphism | `06_translate.dag` | `translate` + `coerce_grounded_node` + projection + grammar-inverse serialize — **bulk, partially scaffolded** |
| Target facts | `extdeps/languages/*` | Per-target `TargetModel` bundles, grammar rows, binding spellings — **MVP-1 add populated for rust/python/go/cpp/ts** |
| Consumers | `test/claim/manual/*` | Executable spec tiers T0–T8 below — **mostly CompilesClaim / Bool witnesses that ERROR or PERF** |

"Generalizing emit" therefore means: **grow `translate` + TargetModel coverage until each tier's
claims execute green**, not grow `05_emit`.

---

## 2. Expansion ladder (claim-corpus authority)

The ladder is ordered by **consumer strength** and **structural dependency**. Higher tiers
assume lower tiers execute.

### T0 — Keystone: executed `emit(add)` (Rust)

| Consumer | Claim anchor | Golden |
|----------|--------------|--------|
| `anchor_mvp1_emit_accepts` | `mvp1_rust_add_translate.dag` | `rust_mvp1_source_text` |
| R2 keystone execution cert | `v4_roster_pilot.dag`, `mvp1_rust_add_translate.dag` | `run_test_claim` over emitted source |

**Blocker (measured):** not translate logic per se — v2 interpreter runtime blowup constructing
`rust.dag` fixture values (S0 in migration doc). First lever: interpreter `v2_rt` share/memo,
not `06_translate` memoization alone.

**Gate:** Bool witness via `gunbc run --claim-run`; must terminate under corpus perf cap (60s/6GiB).

### T1 — Multi-target add parity (Shape A, same tree)

| Target | Claim file | Status |
|--------|------------|--------|
| Python | `mvp1_python_add_translate.dag` | CompilesClaim; host pin in `emit_host.dag` |
| Go | `mvp1_go_add_translate.dag` | CompilesClaim; T-22 eval row |
| C++ | `mvp1_cpp_add_translate.dag` | CompilesClaim |
| TypeScript | `mvp1_typescript_add_translate.dag` | CompilesClaim |
| TS typed fn (PR3) | `mvp1_typescript_pr3_typed_fn_translate.dag` | CompilesClaim |

**Scope:** same `InferredTree` (add fn), different `TargetModel` bundles. Exercises
cross-target homomorphism without new source forms.

**Depends on:** T0 green on Rust (proves translate∘serialize spine); then per-target serialize
arms + `emit_host` execution rows (T-22).

**Cross-ref:** RR-C C.6–C.10 (RCA mgr per-target population), RR-G `PerTargetGroundingReceipt`.

### T2 — Grammar-matched type declarations (wave2a)

| Consumer | Claim | Golden |
|----------|-------|--------|
| TS type alias | `mvp1_typescript_record_task_translate.dag` | `ts_task_type_alias_source_text` |

**Scope:** `type Task = { ... }` via grammar-inverse round-trip on derive path — still fixture
`InferredTree`, not source ingest.

**Depends on:** T1 TS TargetModel wave2a grammar rows populated.

### T3 — SG-2 type-expression projection (grammar-matched types)

| Consumer | Claim file | Exercises |
|----------|------------|-----------|
| Structural receipts | `sg2_type_expression_projection.dag` | `Instantiation` → `Rc<FooBar<X,Y>>`, Arrow, Conj, Sum |
| Falsification twins | same | malformed atom/arrow/instantiation → fail-closed |

**Scope:** `project_type_expression_node` + `target_type_expr_*_emitted` wire family in
`06_translate`. Dissolves `ProjectionAbsent` shim per mark
`feature:sg-2-mvp1-projection-absent-shim`.

**Depends on:** T0 (serialize path live); `target_model_edge_type_expression_projection` on
every active `TargetModel`.

### T4 — SG-2 mode-2: non-grammar-matched type nodes

| Consumer | Claim file | Golden |
|----------|-------|--------|
| Gate (post-A) | `sg2_mode2_non_grammar_emit.dag` | `"Rc<FooBar<X, Y>>"` |
| Perturbed twin | same | `"FooBar<X, Y>"` (no outer Rc) |

**Scope:** serialize when `translation_rules` empty — lex-only atom spelling (GAP-2), GAP-1
boundary atom agreement. **Blocked on grammar-first short-circuit PR (#4462) per design-closure
doc** — `06_translate` mode-2 region edits are off-limits until A merges.

**Authority:** `docs/planning/v4-sg2-mode2-non-grammar-emit-design-closure-2026-06-06.md`.

### T5 — SG-RC reference layering (ownership at use sites)

| Consumer | Claim file | Exercises |
|----------|------------|-----------|
| F1–F6 receipts | `sg_rc_layering.dag` | `ReferenceLayerRc` / `ReferenceLayerOwned` at param/return/field |

**Scope:** `translate_apply_use_site_ownership_to_*` + `target_reference_layer_*` in
translate. Type and value emit must agree at each use site.

**Depends on:** T3–T4 (type-expression wire + serialize); interpreter `lookup` gap (claim map).

### T6 — Value-expression emit (full L1 behaviors in output)

**Not yet claim-anchored as executed witnesses.** MVP-1 add is a degenerate case: one
`Transform` (binary `+`) in a fn body. Generalization here means emitting:

- `Branch` / `match` with exhaustiveness
- `Bind` / `let` chains
- `Loop` / bounded descent (TCO paths)
- Service / transport shells (REST, shell, file)

**Scope boundary:** value-expression serialize is largely **unbuilt in v4** — v2 `05_emit_*.dag`
is the reference behavior to derive, not port wholesale. New claims required before GO.

**Depends on:** T0–T5; LS-4 / ownership dimension for Rust Rc wrapping (SELF_HOSTING §3).

### T7 — Source ingest (fixture → real `compile`)

| Consumer | Claim anchor | Gap |
|----------|--------------|-----|
| Compile anchor | `infer_emit_compile_anchor.dag` | `compile_ingest_staging` = legacy; claims use fixture trees |
| Full pipeline | `00_compile.dag` | tokenize→parse→normalize→resolve→infer→emit not end-to-end |

**Scope:** replace fixture `InferredTree` injection with `compile(source, target)` over real
`.dag` source. Emission generalization is necessary but not sufficient — ingest must land.

**Mid-arc coupling:** omni-ingestion symmetry (ROADMAP §coercion bidirectionality).

### T8 — Emit → ingest round-trip

| Consumer | Location | Status |
|----------|----------|--------|
| Normalized equality | `test/claim/round_trip/*` | W1b shape contract; executable compare staged |

**Scope:** emit then ingest back to canonical IR; compare normalized trees. Proves homomorphism
is faithful, not just printable.

**Depends on:** T7 + T0–T6 coverage of compared forms.

---

## 3. Mid-arc lever — what this unlocks

Emit generalization is the **sequencing hub** behind parallel arc lanes. Each lane's readiness
is tier-dependent:

| Downstream lane | Minimum emit tier | What emit unlocks |
|-----------------|-------------------|-------------------|
| **Multi-target platform** (RR-C/D, RCA mgrs) | T1 | Same IR → N `TargetModel`s; `post_emit_verifier` per target (`multi_target_emit_verification_gate.dag`) |
| **ci.dag conformance** (`workflow/ci.dag`) | T0 + T1 (probe) | `m1_rust_emit_probe_execution` graduates from v2 emit to v4 `emit` receipt; shadow selection receipts need executed claim verdicts |
| **Coercion engine** (translate sprawl + fixture grounding dissolution) | T0–T3 | `find_witness` engine exists today (`coercion_fold` / `solve_constraints`); dissolution target is projection sprawl + hand fixture grounding — negative claims become executable as tiers land |
| **Omni-ingestion** | T7–T8 | Ingest is coercion reversed; without emit that executes, round-trip claims are unfalsifiable |
| **Lenses** (cost/CX/parallelism on compiler) | T0 + executing spine | `claim_pipeline/translate.dag` (G3.4) needs translate to run, not just compile; lens claims over CI workflow pull emit verdicts |
| **Self-host fixed point** | T6–T7 | `compiler.dag` emits stage0 Rust; requires value-expression + full compile, not add-only |
| **Shape B omni-emission** (OpenAPI/SQL/React) | **Out of scope** | User `.dag` programs per RR-D GUARDED — not compiler `emit()` |

**Critical path:** T0 (keystone execute) → T1 (multi-target add) → T3/T4 (type expr) → T6
(value expr) → T7 (ingest) → self-host. T5 (RC layering) parallels T3–T4. T8 follows T7.

---

## 4. Architecture constraints (non-negotiable)

From THESIS, INVARIANTS, RR-C, RR-D:

1. **`05_emit.dag` stays thin** — `translate ∘ serialize_target` only. No new substantive paths
   (C.4). Escalate before expanding.
2. **No Shape B in compiler** — formats/frameworks are extdeps fact bundles; render is user
   `.dag` (D.1–D.2 GUARDED).
3. **No per-language branches in translate** — target vocabulary lives in `TargetModel` /
   `extdeps/languages/*`; translate reads data (derived homomorphism).
4. **Coercion = emission** — `find_witness` replaces inline coercion arms; do not build a
   parallel coercion engine (single-emitter-design.md, ROADMAP).
5. **E-10 consumers** — each tier needs an executed witness before substrate elaboration;
   archive unconsumed code (`docs/v4-compiler-migration.md` Part 5).
6. **Load-bearing files** — `05_emit.dag`, `06_translate.dag`, `00_compile.dag`, substrate
   types in `std/` require L2.5 model PR if the brief pre-dates the relevant model work.

---

## 5. GO / NO-GO matrix

| Lane | GO when | NO-GO now because |
|------|---------|-------------------|
| T0 keystone execute | Interpreter perf fix + emit logic bugs from running witness | S0 done; S1/S2 open — **no emit executes** |
| T1 multi-target add | T0 green + per-target serialize verified | Depends on T0 |
| T2 wave2a type alias | T1 TS row + grammar rows | Fixture-only; no new consumer beyond existing claims |
| T3 SG-2 projection | T0 green + `ProjectionAbsent` shim dissolved | Translate edits need executing spine |
| T4 SG-2 mode-2 | #4462 merged + T3 receipts | Explicitly blocked (design-closure doc) |
| T5 SG-RC layering | T3–T4 + `lookup` interpreter | ERROR bucket in claim map |
| T6 value expressions | New claims authored + T0–T5 | **No executable consumer** — design map only |
| T7 ingest | Ingest staging replaced in `00_compile` | Separate arc; don't conflate with emit tiers |
| T8 round-trip | T7 + emit coverage | Staged claims only |
| Substrate: `fold_node` memoization | T0 shows fold-family blowup after runtime fix | S0 refuted fold-as-root-cause for keystone |
| Substrate: translate sprawl + fixture grounding dissolution | Coercion claims execute on translate path | Engine exists (`coercion_fold`/`solve_constraints` → `find_witness`); deferral is projection sprawl + hand `mvp1_rust_canonical_grounding_for` — not "engine unbuilt" |
| `06_translate` bulk port from v2 | Never as a lump | Violates execution-first rebuild (Part 5) |

**Operator NO-GO (this scoping pass):** no implementation PRs against `06_translate` except
hotfix to make T0 green once interpreter fix lands. Scoping does not authorize tier ≥1 work.

---

## 6. Recommended dispatch sequence (for manager planning)

```
Phase 0 (keystone — BLOCKING EVERYTHING)
  └─ Fix v2 interpreter runtime (v2_rt share/memo) → executed emit(add) Rust witness green

Phase 1 (parallel after Phase 0)
  ├─ T1a: Python/Go add execute + emit_host rows (T-22)
  ├─ T1b: C++/TS add execute
  └─ G3.4: claim_pipeline/translate.dag spine claim (executes once T0 green)

Phase 2 (type surface)
  ├─ T3: SG-2 projection — dissolve ProjectionAbsent shim
  ├─ T4: SG-2 mode-2 (after #4462) — per design-closure checklist
  └─ T5: SG-RC layering (parallel if lookup lands)

Phase 3 (value surface + compile)
  ├─ T6: Author value-expression claims (match, let, service) — **new scoping pass**
  ├─ T7: Ingest spine consumer
  └─ T8: Round-trip claims

Phase 4 (downstream convergence)
  ├─ ci.dag: v4 emit probe replaces v2 shim
  ├─ Coercion: translate sprawl + fixture grounding dissolution behind executing negative claims
  └─ Self-host: compiler.dag slice emit
```

Each phase = one or more **worker briefs** with a named Bool/`TestClaimRun` consumer, not
"port N functions."

---

## 7. Claim corpus as executable spec

**Do not delete claims that only CompilesClaim today** — upgrade them to executed witnesses as
tiers land (`docs/v4-compiler-migration.md` Part 4).

| Category | Files | Tier |
|----------|-------|------|
| MVP-1 add translate | `mvp1_*_add_translate.dag` (5 targets) | T0–T1 |
| TS record / typed fn | `mvp1_typescript_record_task_translate.dag`, `pr3_*` | T2 |
| SG-2 projection | `sg2_type_expression_projection.dag` | T3 |
| SG-2 mode-2 | `sg2_mode2_non_grammar_emit.dag` | T4 |
| SG-RC layering | `sg_rc_layering.dag` | T5 |
| Collection projection | `sg_collection_projection.dag` | T3 adjunct |
| Language model anchors | `*_language_model_anchor.dag` | T1 metadata |
| Emit verification gate | `multi_target_emit_verification_gate.dag` | T1 (T-38 bridge) |
| Compile anchor | `infer_emit_compile_anchor.dag` | T0–T7 |

**Perf bucket (11 witnesses):** `claim_pipeline/translate.dag` and allies — keystone perf track,
not parallel emit-generalization work (`docs/planning/v4-claim-corpus-execution-map-2026-06-04.md`).

---

## 8. Non-goals (this arc)

- Compiler-side Shape B emit (OpenAPI, SQL, React, YAML, Markdown) — RR-D GUARDED
- New `05_emit_*` per-language modules or v2-style template tables in compiler
- Bulk `06_translate` port from v2 (~4300 lines) without an executing consumer per slice
- Cementing Rust stage0 to satisfy census ratchet
- Substrate extension (7th connective, 6th behavior) before four dissolution patterns fail
- Bidirectional `.dag` emission / diagnostic "show correct code" — architecturally enabled, not
  in scope

---

## 9. Escalation triggers

Escalate via `dashboard-ops escalate` if a worker brief would:

1. Edit `05_emit.dag` beyond orchestration glue
2. Add compiler imports from `extdeps/formats/*` or `frameworks/*`
3. Touch `06_translate.dag` mode-2 region before #4462 merges
4. Introduce a parallel coercion phase separate from translate
5. Declare a new substrate type without an L2.5 model PR and named consumer
6. Start T6 value-expression work without new claim authoring pass

---

## 10. Related docs (do not duplicate)

| Doc | Role |
|-----|------|
| [`docs/v4-compiler-migration.md`](../v4-compiler-migration.md) | Ground truth, S0–S4 execution-first plan |
| [`docs/planning/v4-cross-target-emission-rr-c-worksheet-2026-06-02.md`](v4-cross-target-emission-rr-c-worksheet-2026-06-02.md) | Shape A substrate contract C.1–C.10 |
| [`docs/planning/v4-cross-target-emission-rr-d-worksheet-2026-06-02.md`](v4-cross-target-emission-rr-d-worksheet-2026-06-02.md) | Shape B GUARDED boundary |
| [`docs/planning/v4-sg2-mode2-non-grammar-emit-design-closure-2026-06-06.md`](v4-sg2-mode2-non-grammar-emit-design-closure-2026-06-06.md) | T4 sequencing |
| [`docs/single-emitter-design.md`](../single-emitter-design.md) | v2 dissolution target (historical bulk reference) |
| [`docs/planning/v4-claim-corpus-execution-map-2026-06-04.md`](v4-claim-corpus-execution-map-2026-06-04.md) | Execution snapshot |
| [`THESIS.md`](../../THESIS.md) § omni-emission | Shape A/B thesis claims |

---

## 11. Scoping verdict

| Question | Answer |
|----------|--------|
| Is emit generalization one PR? | **No** — eight tiers, keystone-blocked |
| Where does work land? | `06_translate`, `extdeps/languages/*`, claims — **not** `05_emit` |
| What is the mid-arc lever? | **Executed T0** unlocks T1–T8 and downstream ci/coercion/ingest/lens lanes |
| GO for implementation? | **NO-GO** until T0 witness executes green |
| Next manager action? | Dispatch Phase 0 (interpreter perf) as keystone child; hold T1+ until T0 receipt |

---

## 12. Ingestion coupling — concern, probes, recommended reorder

### 12.1 The concern is valid and consciously scoped

This scoping pass **generalizes emit without connecting ingest**, by design:

- **T7** (arbitrary-source ingest) and **T8** (emit→ingest round-trip) are the top two ladder
  rungs.
- **§8** lists bidirectional `.dag` emission as an explicit **non-goal** for this arc.

That is a deliberate trade: nothing is falsifiable until emit executes (T0), and E-10 pulls
substrate only when a green consumer demands it. Flagging the disconnect at **NO-GO** — before
any rung is authorized — is the right moment.

The concern splits into two different problems with different fixes.

### 12.2 Mechanism — one engine or two?

**Constraint #4** forbids a parallel coercion engine (`coercion = emission`; `find_witness`
replaces inline arms). §5 now defers **translate sprawl + fixture grounding dissolution** (not
"engine unbuilt").

**Probe (T0 path, measured):** the MVP-1 coercion spine **already routes through
`find_witness`** — not around it:

```
coerce_grounded_node → coercion_fold_with_declared_priority → coercion_fold → find_witness
```

(`06_translate.dag` → `std/coercion.dag`.) The shared witness-search engine exists on the
coercion leg today.

What is still **inline / accreting**:

| Inline surface | Where | Risk if deferred |
|----------------|-------|------------------|
| Hand-enumerated fixture grounding | `mvp1_rust_canonical_grounding_for` if-chain in `mvp1_rust_add_translate.dag` | Emit proves a fixture printer, not infer-authored grounding |
| Projection sprawl | ~150 `project_*` / per-variant arms in `06_translate` | T2–T6 grow bespoke arms instead of derived morphisms |
| GO-matrix dissolution row | Scoping §5 (corrected) | Deferral is projection sprawl + hand fixture grounding — not "engine unbuilt" |

Ingest is coercion reversed and needs the same engine. Deferring dissolution until T7 means
either dissolving six tiers of accreted arms into `find_witness`/structural fold, or ingest
grows its own arms — the two-engine outcome constraint #4 forbids.

**Verdict:** mechanism risk is real but **mis-located** in the GO matrix. The fix is not
"build `find_witness`" (done) — it is **stop accreting inline translate/grounding arms** after
T0/T1 and route new coverage through the existing engine.

### 12.3 Validation — what does a green emit tier prove?

A golden-string emit witness proves: *this tree prints to this text.* That is a code generator.
It does **not** prove the map is invertible or structure-preserving — two trees could print to
the same text, or the text might not parse back.

**Round-trip** (emit → ingest → normalized compare) is what proves *faithful*. That is T8 in
this doc — deferred to the top. The thesis claim is not "emit produces good code" but "one
faithful structure-preserving map, run both ways" (ROADMAP §coercion bidirectionality).

As scoped, T0–T6 climb six rungs proving a one-way printer; faithfulness proof waits for T8.

### 12.4 Probe — is `.dag` a cheap home round-trip target today?

**Probe (measured):** **partially, not yet cheap.**

| Asset | Status |
|-------|--------|
| `extdeps/languages/dag.dag` | Shape A language model — wave-1 lex + grammar for **parse**; no `TargetModel` / `dag_mvp1_*` bundle |
| `test/fixture/dag_round_trip_mvp1.dag` | Trivial identity fn — wave-1 surface smoke only |
| `test/claim/round_trip/dag_ingest_round_trip.dag` | **W1 readiness** (lex/grammar/C5 authorities); emit→ingest compare explicitly **W1b** — not executable fidelity yet |
| Parser for `.dag` | Exists (ingest side) |
| `.dag` emit for `fn add` | **Not populated** — unlike `rust_mvp1_target_model` + `rust_mvp1_source_text` |

Home round-trip (`.dag → IR → emit-.dag → parse → IR → compare`) is the **cheapest** fidelity
proof (no foreign parser), but it requires a **`dag_mvp1_target_model`** slice in `dag.dag`
(translation rules, declared inhabitants, serialize prefix — mirror `rust_mvp1_*` pattern) plus
a W1b executable `RoundTripClaim` on `add`. Budget: small scoping child after T0 green, not
T7 breadth.

### 12.5 Recommended reorder (future scoping — **not authorized by this doc**)

**Authority:** §2, §6, and §11 define the operational ladder for this arc. The items below are
design recommendations for a **follow-on scoping pass** after T0 executes — not gates in the
GO matrix and not additions to §6 dispatch.

Keep **T0 first** — nothing falsifies until emit executes. A future pass may pull, on `add`
only:

1. **Infer-authored grounding** — dissolve `mvp1_rust_canonical_grounding_for` in favor of
   infer-authored facts (`feature:W-T-10-mvp1-inferred-tree-grounding`).

2. **Minimal home `.dag` round-trip on `add`** — populate `dag_mvp1_target_model`, golden
   `dag_mvp1_source_text`, and a W1b claim (`emit → parse → normalized IR equality`). This
   remains a **non-goal for this arc** per §8 until a separate scoping pass authorizes it.

**T7** (arbitrary-source ingest) and **T8** (full round-trip coverage) stay on the §2 ladder
regardless.

### 12.6 Doc tension — co-equal vs top-of-ladder

| Doc | Bidirectional stance |
|-----|---------------------|
| **ROADMAP** §coercion bidirectionality | Co-equal: "one mechanism, run both ways"; ingest = emit⁻¹ |
| **This scoping doc** §2 / §6 / §11 | Authoritative ladder: T7 ingest, T8 round-trip at top |
| **§8 non-goals** | Bidirectional `.dag` emission / show-correct-code — not in this arc's scope |
| **§12.5** | Future recommendation only — does not amend §2 / §6 / §8 / §11 |

**Resolution (ratified):** ROADMAP states the **thesis mechanism** (co-equal). This scoping doc
states the **implementation ladder** for *this arc* (§2, §6, §11). §12 records ingest-coupling
probes and a **deferred** reorder recommendation (§12.5); it does not introduce T0.5 or override
§8. Bidirectional breadth lands at T7–T8; early faithfulness on `add` is input to a future
scoping pass, not a mandatory gate here.
