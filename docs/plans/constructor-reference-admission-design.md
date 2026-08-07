# ConstructorReferenceAdmission — reference-edge admission for sealed constructors

**Status:** DRAFT for operator review (2026-08-04, royal-crab-235, msg_13fbbaaf / node `adhoc-f519571b-764`; updated msg_a87c2c75, operator ruling msg_1a57418c). **No code lands from this note** until the shape is agreed and the **which-compiler** horn is operator-picked (§4). Owner: royal-crab-235 (custody of tidy-fox-81 #7806 scaffold as of msg_1a57418c consolidation).

**Origin:** Executed falsification (royal-crab-235, msg_6701c04d): `sole_constructor` refuses cross-module **record literals** but does **not** refuse cross-module **calls** to an exported mint with caller-supplied fields; legitimate wet transport requires that mint. Process-output carrier reshuffle and opaque `WitnessBin.Run` output do not close the hole. **Derived ceiling:** `can-climb-after-a-capability` (not can-climb-now). **Preferred trigger:** this capability.

**Evidence PRs (draft, parked — operator ruling msg_1a57418c):** #7792, #7794 remain capability-audit/falsification records; **do not merge** with `FixtureBehaviorallyEquivalent` as terminal claim. May stay as falsification records or non-authorizing caller-supplied observation experiments. **Not** a docs-only merge under that claim. Dashboard disposition: *green, draft, do-not-merge falsification record; not pending merge work.*

## 0. Displaced cost

- **Forged execution evidence:** any module can call `mint_behavioral_comparison_observation` with fabricated stdout/status/rows and reach `FixtureBehaviorallyEquivalent` advance — the admission seal is a checksum of caller input, not proof of process execution.
- **Decorative carrier churn:** module-structure `sole_constructor` wrappers do not climb the ladder; each reshuffle re-documents the same leak (`fully_matching_derived_seal_forgery_advance_succeeds_red` stays green).
- **False merge signal:** PR titles/bodies that claim behavioral equivalence while the body disclaims it (msg_13fbbaaf) — two-authority disagreement where the list UI is wrong.

## 1. Measured gap (two construction forms, one specimen)

Specimen: `v2.compiler.self_host.behavioral_comparison_receipt` `BehavioralComparisonObservation` (`sole_constructor` on the type).

| Construction form | Verdict | Mechanism today |
|---|---|---|
| Cross-module record literal | **REFUSED** | `SoleConstructorViolation` in **v1 infer only** (`src/v1/04_infer.dag` `type_has_sole_constructor`; realized in `v1_compiler_infer.rs` record-literal arm). **`src/v2/compiler/04_infer.dag` has zero `sole_constructor` handling** — the type marker is modeled in v2 source, but the refusal executes only on the v1 seed path today. |
| Cross-module call to exported `mint_behavioral_comparison_observation` | **LEAKS** | No call-edge admission; exported fn is writable from any importer |
| Opaque host-effect return | **NEGATIVE** | `WitnessBin.Run` returns forgeable `String`/`Bool` fields |
| Process-output `sole_constructor` carrier (#7794) | **DECORATIVE** | Forge via `from_run_fields` / `hermetic_equivalent` |

**Closing both refused + leaking routes on the behavioral-comparison specimen** requires capabilities beyond `sole_constructor` alone — call-edge admission plus (for fixture-origin parameter types) the orthogonal carrier wise-ram-22 models. **Legitimate wet transport** (`tools.direct_rust_door_transport`) must remain admitted after the wall lands.

### 1.1 Joint with wise-ram-22 — orthogonal mechanisms (msg_e499e35c)

**wise-ram-22** (`FixtureOriginCanMintProductionQualification`, measured on direct-door): hand-authored `InferredTree` **accepted** by `mint_provenanced_rust_artifact`; `infer(assemble(direct_rust_door_ingest()))` **rejected** — production qualification reachable from fixture-origin input while the real pipeline cannot obtain it.

**Clean division (both notes required reading):**

| Mechanism | Question it answers | Authority |
|---|---|---|
| **ConstructorReferenceAdmission** (this note) | **WHO may construct** — which callers may invoke a sealed constructor | Call-edge + opt-in roster |
| **CompilerInferredArtifact + inference receipt** (wise-ram-22 B0 §1) | **WHAT may be constructed from** — parameter types that must carry compiler-derived provenance | Stage-origin carrier |

**Orthogonal and both required:** a production qualification reachable by a **permitted** caller from a **fixture-origin** value is refused by **neither alone**. Call-edge admission does **not** close wise-ram-22's class: the direct-door specimen is a permitted caller supplying a hand-authored `InferredTree` to a mint whose signature legitimately accepts one — the caller is not the problem; the discarded epistemic distinction in the **parameter type** is.

**Stronger adjacent case (wise-ram-22 census):** cargo may compile bytes and a behavioral corpus may **genuinely execute** them while the bytes came from a **substitute subject** — evidence is **real about the wrong subject** (nastier than checksum-derived seals without execution witness in §0).

**Rung correction (wise-ram-22, credited):** `src/v2/compiler/04_infer.dag` has **zero** `sole_constructor` handling — marker modeled in v2 source, refusal executes on **v1 seed path only**. Claiming "mechanically preventable on covered form" without per-path split is **rung inflation** (DESIGN §4b(1): minimum across in-scope paths).

**Origin coproduct warning (wise-ram-22 B0 correction — load-bearing for CargoGreen→BehavioralEquivalent):** do **not** put `ExecutionMeasured` as a sibling of `CompilerDerived` | `FixtureDerived` in a production-origin coproduct. A fixture-derived artifact can **also** be execution-measured — that is exactly the dangerous state. Split axes: `ProductionStageOrigin = CompilerDerived | FixtureDerived | UnknownOrigin` and `ProductionEvidenceGrounding = StructurallyGrounded | ExecutionMeasured{receipt} | Unmeasured`. Measurement **carries** an origin; it does not replace one.

```dag
import std.decl_ref { DeclarationRef, decl_ref, decl_field_ref }

// std layer — agnostic admission shape (authority TBD: std.construction or std.decl_ref extension)
type ConstructorReferenceAdmission {
  constructor: DeclarationRef          // callee that constructs the sealed type (mint fn)
  permitted_callers: List<DeclarationRef>  // nonempty by construction — see §3
}
```

**Citation rule (§3):** both `constructor` and each permitted caller are `DeclarationRef` rows — they ride `std.decl_ref` / cited-symbol resolution already in tree. Every ref must resolve to exactly one declaration or refuse at authoring; no second naming scheme.

**Constructor import rule (operator decision msg_a87c2c75):** import `DeclarationRef`, `decl_ref`, and `decl_field_ref` from `std.decl_ref` — **never re-mint** those constructors in another module. `decl_ref_constructor_authority_note` records that two lanes independently re-minted the pair on 2026-08-01 and broke every closure containing both modules (v1-seed fn names are not module-scoped). This note must not become the third duplicate.

**Refusal (when a call edge is checked):** typed, located diagnostic naming:

- the **constructor** `DeclarationRef` (resolved symbol),
- the **offending caller** `DeclarationRef` (the calling declaration — typically the enclosing `fn`),
- the **permitted set** (resolved symbols or explicit "none listed" — never silent).

Suggested variant name: `ConstructorCallAdmissionRefused` (exact spelling TBD with existing diagnostic taxonomy).

**Not a generic violation string** — coordinates must be structurally present for counting and ladder receipts.

## 3. Nonempty permitted callers

`NonEmptyList<DeclarationRef>` does not exist in `std` today. Precedent: `gunbc.doc_graph_roots` `HandAuthoredDocBind` — `primary_work` required scalar + `additional_works` list makes zero-anchor unwritable (construction, not validation).

**Proposed:** `permitted_callers` as `List<DeclarationRef>` with **authoring-time nonempty** enforced on the admission row carrier (wall on the metadata row), OR split `first_permitted_caller: DeclarationRef` + `further_permitted_callers: List<DeclarationRef>`. Pick one at implementation; design requirement is **empty permitted set is unwritable**.

## 4. Where the refusal fires — WHICH COMPILER (operator hold; session does not pick)

**Correction (measured, msg_a87c2c75):** the fork is **not** infer versus a separate pass. Record-literal `sole_constructor` refusal today lives in **v1 infer** (`src/v1/04_infer.dag` + `v1_compiler_infer.rs`). **`src/v2/compiler/04_infer.dag` has zero `sole_constructor` occurrences** — the capability does not exist in v2 infer at all; it lives only in the v1 seed stack. `ConstructorReferenceAdmission` must land in the same compiler path that will actually execute for corpus `.dag` today.

### 4.1 Decided now — ONE authority, no separate pass

Construction admission is **one authority**. Record-literal refusal and call-edge refusal are two **forms of the same question** — who may construct this sealed type from where. Splitting them into two passes means two mechanisms that must agree about what construction means and will eventually disagree.

**This note explicitly rejects a post-infer dedicated pass** as the tempting shape that avoids touching a load-bearing file. Avoiding a load-bearing file is not a design argument; it is the unmarked-workaround class (DESIGN §5). Whatever compiler hosts the wall, record-literal and call-edge checks live in the **same infer construction-admission authority**, beside the existing record-literal `sole_constructor` arm in v1.

### 4.2 Operator hold — three horns (program sequencing, not design)

**Do not pick in this session.** Three horns and their costs are named here; the operator picks. still-bat-561 is **recommending the third row** to the operator but has not picked it — this note presents all three.

| Horn | What it costs | Execution today |
|---|---|---|
| **Land in v1 seed infer** | Grows the thing the v1-deletion program is shrinking: new capability surface + Rust realization lines in hand-maintained `v1_compiler_infer.rs` (and `.dag` infer model). Same class as operator pushback today on adding hundreds of lines to seed files under correct modeling. **But:** wall **executes immediately** on the live corpus path — same as existing `sole_constructor` record-literal refusal. |
| **Land in v2 infer only** | Aligns with self-host direction; does not expand v1 seed. **But:** v2 infer has **no `sole_constructor` today** — the wall does not execute until v2 infer implements construction admission (record literals + call edges) and that stage is what runs for acceptance. A wall that does not execute is specification-without-execution — the failure this lane proved matters. |
| **Land in v1 seed infer inside an already-counted frontier row** (operator-ruling shape, gunbc#7804, same day — **not proposed fresh**) | Same seed growth as row 1 (infer `.dag` arms + `v1_compiler_infer.rs` realization for record-literal + call-edge admission). **No new frontier row** — growth enlarges a module **already** enrolled as seed-retained. **Residual latency:** that module is blocked on `^parse_grammar_choice_overlap_residue` (execution-measured; sibling lanes gunbc#7767 and gunbc#7762); trigger `^migrate_when_closure_self_emits_cargo_green` is real but **not imminent** — the wall sits in the seed for as long as that blocker stands. | **Executes immediately** on the live corpus path (same as row 1). Forge RED flips **by execution**, not in principle. |

**Third row — what it buys:** the wall runs on today's acceptance path; `fully_matching_derived_seal_forgery_advance_succeeds_red` can flip when the mechanism lands, not when v2 self-host completes in the abstract.

**Third row — what it costs:** the enumerated v1 infer additions below. Seed growth is **not** a new obligation invented to justify the landing — it enlarges one module already counted by the self-host frontier.

**Third row — existing row (verified, cited by symbol):** `compiler_frontier_row_04_infer` in `src/v2/compiler/self_host/frontier.dag`, constructed by `execution_measured_seed_retained_row` with `module_path: "src/v2/compiler/04_infer.dag"`, `closure_reads: 44`, `measured_blocker: RealizationGap`, `located_stage: ProbeStageAssemble`, `located_reason: ^parse_grammar_choice_overlap_residue`, `migration_trigger: ^migrate_when_closure_self_emits_cargo_green`. Both trigger and reason symbols resolve. This is **not** "add a new declared row and take on a new obligation" — `04_infer` is already seed-retained with an already-named migration trigger and an execution-measured reason for why it has not migrated. Constructor-reference admission would land inside that module's v1 seed realization; the frontier row's contents grow, the row count does not.

**Third row — dissolution trigger (named exactly):** `^migrate_when_closure_self_emits_cargo_green` on the row above. When `src/v2/compiler/04_infer.dag` closure self-emits cargo-green **and** v2 infer is what runs for production typecheck, the v1 seed construction-admission block deletes. Disposition follows the gunbc#7804 pattern: **not** "do not add Rust to the seed," but **this Rust block carries explicit disposition** — seed realization now, then emitted or native `.dag` realization, then **this block deletes** on the named trigger.

**Third row — latency (honest, cuts both ways):** the trigger is real but the module is blocked on `^parse_grammar_choice_overlap_residue` — the **same** residue sibling lanes gunbc#7767 and gunbc#7762 are working on now. An operator choosing this horn should know they are choosing it **with that latency**, not against a vague someday. Read two ways: (1) **weakens** the third horn if the trigger is read as near — dissolution is not imminent while the blocker stands; (2) **strengthens** it if the trigger is read as far — a wall that does not execute until v2 infer self-emits is a wall that does not execute for a long time, which is precisely the second horn's cost, now **quantified** by an execution-measured blocker rather than asserted.

**Operator ruling source (gunbc#7804, 2026-08-04):** seed growth is not forbidden; it is required to be **counted** — explicit disposition + dissolution trigger, never silent seed expansion. This third row applies that ruling consistently: the counting mechanism already exists and is enrolled on `compiler_frontier_row_04_infer`; it is the operator's own rule carried across PRs, not a middle path invented to avoid the fork.

**What v2 would need for the wall to actually execute there (named, not assumed):**

1. `sole_constructor` record-literal refusal ported into `src/v2/compiler/04_infer.dag` + its realization (capability absent today).
2. `ConstructorReferenceAdmission` call-edge refusal in the **same** v2 infer authority (not a second pass).
3. v2 infer as the stage that runs on the acceptance path for the specimen modules (self-host frontier row — today v1 seed still realizes infer for production typecheck).
4. `ConstructorCallAdmissionRefused` diagnostic in the v2 diagnostic taxonomy (or shared `CompilerDiagnostic` bridge until v2 fully subsumes).

**What v1 landing would add (named, not estimated to a false precision):**

- New infer arms in `src/v1/04_infer.dag` for call-edge admission (alongside existing `type_has_sole_constructor` / record-literal arm at ~4099+).
- Rust realization in `v1_compiler_infer.rs` parallel to the existing `SoleConstructorViolation` record-literal block (~8620+).
- New diagnostic variant with constructor, offender caller, permitted-set coordinates.
- Enrollment witnesses proving execute on the **v1** path (already how `sole_constructor` is proven today).

**Session disposition:** carrier shape, refusal coordinates, composition, first_slice bar, alias stance, and no-separate-pass rule are all decidable without picking the horn. **Implementation waits on operator choice among the three rows in §4.2.**

### 4.3 Model-only scaffold — tidy-fox-81 prototype (#7806, non-conforming; custody royal-crab-235)

**Operator ruling (msg_1a57418c):** #7806 is a **model-only capability scaffold** — must **not** claim enforcement without a **production consumer**. Keep draft (or title explicitly as capability model). A model-only carrier with no compiler consumer is coverage-by-illusion. **Consolidation:** tidy-fox-81 closed; #7806 parked artifact custody transfers to royal-crab-235 (same lane, horn-blocked — custody not active implementation).

**Source:** tidy-fox-81 implemented the v1 horn without the design note (stopped; writeups only). **Not the specified design** — a floor for horn comparison. **Hold stands:** nothing lands until operator picks.

**Model half (committed `.dag` diff):** 110 changed lines across five files (`00_core`, `02_parse`, `04_infer`, `04_lookup`, `04_sigs`). Parse surface (`admit_callers` property), lookup/sig gate, diagnostic variant — **not** wired to `mint_behavioral_comparison_observation` on that branch.

**Rust realization half (execution-measured via `regen_stage0`, not inferred):** 424 changed lines across five generated files — `v1_compiler_infer.rs`, `v1_compiler_infer_lookup.rs`, `v1_compiler_infer_sigs.rs`, `v1_compiler_parse.rs`, `v1_std_core.rs`. `v1_compiler_infer.rs` specifically: one new function (~28 lines), two match arms (~12 each), import edit. **Roughly four-to-one model-to-realization amplification.** An operator comparing horns on the `.dag` number alone is comparing the wrong quantity.

**Still a floor:** conforming to this note costs **strictly more** on **both** halves. Three named deltas still outstanding (each adds model + realization work):

1. **DeclarationRef grain** — not `module_path` strings (§3 `std.decl_ref` import rule).
2. **One construction-admission authority** — not a separate lookup gate beside `sole_constructor` infer (§4.1).
3. **Resolvable caller identity** — not `caller_module` alone.

**Over-admission finding (module-grain, separate from typo-resolution):** the prototype's check in `func_sig_from_global_bare` compares admitted callers against the **caller's module path**, so admission is **module-grain**: permitting one module permits **every function in it**, now and every function added later — the wall **widens silently** as that module grows; nobody edits `admit_callers` when a new function is added. The annotation names modules; the actual admitted set is whatever those modules currently contain. The admitted set cannot be read off the annotation. **§4b ceiling consequence:** precision is bounded by module size and **degrades over time** — grain determines what the wall admits, not just whether a `DeclarationRef` resolves — stronger than the authoring-time-resolution argument alone. **Named trigger for coverage roster:** enrollment lens over annotated constructors (`admit_callers` / `ConstructorReferenceAdmission` rows) vs `sole_constructor` census.

**Opt-in semantics (correct, not fail-open):** absent `admit_callers` → call resolves — unannotated functions are not sealed (same class as `sole_constructor`). The measured forgery hole on `mint_behavioral_comparison_observation` stays open on #7806 because **unwired** (one property on one fn), not because the mechanism is unsound.

**§4b opt-in ceiling:** an opt-in wall tops at **mechanically preventable** — cannot be structurally impossible for carriers nobody annotated. Closes the two measured construction routes **only** for opted-in constructors; coverage is a roster question, not automatic language closure.

**Explicit-import bypass (review 48287, operator sequence step 3):** `lookup_func_sig` reaches `func_sig_from_global_bare` — and therefore `admit_callers` — only on the census-fallback path. Callers with an explicit `import` resolve via import-closure and **never consult admission**. This is **not** a nit to defer on #7806; it is step 3 of the real implementation (unified post-resolution check at call-edge construction). tidy-fox-81 correctly declined implementing it on a branch that must not land.

### 4.4 Implementation sequence when horn unblocks (operator ruling msg_1a57418c)

**Enforcement status today:** `NotEnforced` — do not flip until the sequence completes on a production consumer.

1. Reuse exact declaration/reference identities from the binding substrate (`std.decl_ref` — §3 import rule).
2. At **call-edge construction**, carry caller declaration, callee declaration, and referenced constructor.
3. When the callee is sealed, require the caller in the **exact** admission roster (unified authority — closes explicit-import bypass; §4.1 one mechanism, not lookup-only).
4. Refuse an unlisted edge with a typed diagnostic (`ConstructorCallAdmissionRefused` coordinates — §2).
5. Add a real cross-module exported-function forgery RED (`fully_matching_derived_seal_forgery_advance_succeeds_red` flips).
6. Add a positive admitted-host-transport control (wet path caller listed).
7. **Only then** change enforcement from `NotEnforced`.

## 5. Design questions (decided or answered in this note)

### 5.1 Alias and higher-order positions (operator-approved, msg_a87c2c75)

**Approved stance:** outside the modeled guarantee until witnessed; **first slice is direct resolved mint call only.** This is honest ceiling-declaring, not an absorbing fallback.

**What unwitnessed alias / HOF calls do TODAY (residual risk, stated plainly):** there is **no** call-edge admission check at all — any importer that resolves a call to `mint_behavioral_comparison_observation` (including through a re-export alias binding to the same declaration) **silently passes** and can forge observation fields. Higher-order and indirect calls where the callee is not syntactically the mint **also pass silently** — no admission machinery exists to refuse them. The first-slice wall covers **direct resolved calls to the listed constructor symbol** only; alias chains and HOF remain **residual forge surface** until falsified.

**Named trigger for alias climb:** first falsification row where a re-export alias binding resolves to the same `DeclarationRef` as the constructor but bypasses admission — if it silently passes, extend the infer call-edge check to follow alias resolution to the constructor `DeclarationRef` (same authority, not a new pass). HOF climb trigger: separate row when a witness shows construction via a function-typed parameter reaching the mint without a direct callee edge.

### 5.2 Composition with `sole_constructor`

**Not redundant — complementary:**

- `sole_constructor` → blocks cross-module **record literals** for the type.
- `ConstructorReferenceAdmission` → blocks cross-module **function calls** to listed constructors except permitted callers.

Together they close the two routes measured on `BehavioralComparisonObservation`. Admission does **not** subsume sole_constructor (literals could still forge if sole_ctor were removed). sole_constructor does **not** subsume admission (exported mint calls leak today).

### 5.3 Where the permitted list is authored

**Proposal:** admission row lives **on the constructor's declaring module** (beside the type or mint fn), cited by `DeclarationRef` to the constructor symbol. Callers list wet-transport (and test) callers explicitly at **declaration grain** (`DeclarationRef` per permitted caller fn), not module grain — see §4.3 over-admission finding.

**Two independent derivations agreeing on constructor-side placement (not one measurement confirming the other):**

1. **First principles (this note):** the constructor is the sealed symbol; the permitted set is metadata **about** who may invoke that constructor — it belongs beside the constructor declaration.
2. **Structural (tidy-fox-81 #7806, discovered by building):** `func_sig_from_global_bare` already holds the constructor's declaration node and both module paths at the check site — reading `admit_callers` there is one more field read. Caller-side placement would need the caller's own declaration node resolved at that same site — plumbing that does not exist yet. Constructor-side is cheaper for a **structural** reason, not a preference.

**Module move:** `DeclarationRef` uses `module_path` + `decl_name` — if a permitted caller moves modules, the ref must be updated (same as any `DeclarationRef` row); no path-inferred auto-permission. **Module-path admission is over-admission** (§4.3): it silently widens as the permitted module grows.

### 5.4 Compiler-module exemptions

`sole_constructor` carries compiler-module check exemptions (unverified completeness). **ConstructorReferenceAdmission must not inherit exemptions by default.** If an exemption is needed, it is a named row with dissolution trigger — never silent skip inside `v2.compiler` / `v1.compiler` trees.

### 5.5 Census disposition taxonomy (wise-ram-22 / keen-lark-681 exchange)

Classified **specimens** — denominator not closed; keen-lark-681 successor owes full population. Dispositions for operator packet:

| Tag | Meaning | Notes |
|---|---|---|
| **A (v1 path)** | Cross-module record literal refused on `sole_constructor` types | **v1 infer only**; narrower than it reads (file-identity scope, fail-open lookup miss on uncovered paths) |
| **A-prime (v2 path)** | Same construction on **v2 infer path** | **Uncovered** — not A; zero `sole_constructor` handling in `src/v2/compiler/04_infer.dag` today |
| **B** | Closable after `ConstructorReferenceAdmission` | Opted-in constructor mints; declaration-grain `DeclarationRef` permitted callers; roster enrollment |
| **C — fixture-origin parameter** | `mint_provenanced_rust_artifact` accepting bare `InferredTree` | **Not** this capability; wise-ram-22 B0 §1 (`CompilerInferredArtifact` + inference receipt) |
| **C — real execution, wrong subject** | Downstream compile/run succeeds on substitute subject bytes | Adjacent to checksum seals; evidence genuine about wrong subject |
| **C — other** | `WitnessBin.Run` forgeable strings; decorative process-output `sole_constructor`; checksum-derived seals without execution witness | See §1 table |
| **D** | Residual after B | Alias/HOF; module-grain over-admission if mis-implemented; **subject discontinuity** (qualification assembled from receipts carrying different subjects — B0 §2 chain property, not visible to either admission mechanism alone) |

**Do not overstate B:** ConstructorReferenceAdmission does not close wise-ram-22's FixtureOrigin class or the direct-door permitted-caller + hand-authored tree specimen.

**keen-lark-681 draft rows tagged (msg_2eaf0857, probe @ `27e516ce` — entry-closure scoped, NOT closed population):**

| # | Surface | §5.5 tag(s) | Notes |
|---|---|---|---|
| 1 | `candidate_generation.mint_provenanced_rust_artifact` | **C — fixture-origin parameter** | B may roster call edges on this mint but **does not close C** |
| 2 | `candidate_generation.mint_written_provenanced_rust_artifact` | **C — fixture-origin parameter** | Same as (1) via wrapper |
| 3 | `candidate_generation.generate_provenanced_rust_candidate_from_ingest` | **—** (not C at caller) | Real-pipeline caller; fixture risk is parameter type at (1). **A-prime: N/A** (call, not record literal) — agreed |
| 4 | `candidate_generation.advance_written_provenanced_rust_artifact_cargo_green` | **C — fixture-origin parameter** (derivative); **C — real execution, wrong subject** when cargo greens on substitute-subject bytes | Inherits upstream origin |
| 5 | `tools.direct_rust_door_transport.run_direct_rust_door_emit_write_compile_smoke` | **C — fixture-origin parameter**; **C — real execution, wrong subject** (when cargo greens) | Primary measured direct-door specimen |
| 6 | `production_qualification_origin_probe_structural_fixture.structural_red_control_mint_site_probe` | Same as (5) | Test scaffold; probe control until G2 marshal exposes callees |
| 7–10 | Synthetic classifier controls | **Probe enrollment** (not live census) | Oracle rows for classifier proof only |
| 11 | `behavioral_comparison_receipt` `BehavioralComparisonObservation` + `mint_behavioral_comparison_observation` | **A (v1)** record literal; **A-prime (v2)** uncovered; **B** exported mint call leak | §1 specimen; first_slice target |
| 12 | `WitnessBin.Run` opaque returns | **C — other** | Forgeable strings |
| 13 | Process-output `sole_constructor` (#7794) | **C — other** | Decorative reshuffle |

**A/A-prime denominator:** origin probe does **not** discover `sole_constructor` record-literal sites today — correct scope split. **Recommend parallel census** (or dedicated v2-path discovery witness) for A/A-prime population; do not fold into origin probe unless one closure scan is explicitly scoped for both axes.

**Subject:** `mint_behavioral_comparison_observation` → `BehavioralComparisonObservation`.

**Admit:** `tools.direct_rust_door_transport` wet path caller(s) only (resolve to `DeclarationRef` rows in the admission metadata).

**Discriminating control (already enrolled):** `fully_matching_derived_seal_forgery_advance_succeeds_red` — performs **real forge** via `hermetic_equivalent` + derived seal; **must flip from PASS to refuse** when admission wall lands. Never `optional_absent()`.

**Positive controls (already in tree):** record literal refusal; legitimate transport path when caller is listed; advance refuses without admission seal when observation is genuinely diverged.

**Proof bar:** green-by-execution on the slice + forge RED flips — not grep, not title claim.

## 7. Phases (no enforcement until note signed + operator picks compiler horn + §4.4 sequence complete)

1. **P0 — this note:** operator sign-off on shape; **operator pick among §4.2 three horns** (recommendation exists in packet — still-bat-561 + royal-crab-235; operator reads packet, not summary).
2. **P1 — model carrier** in `std` with nonempty permitted callers; import `decl_ref` from `std.decl_ref` (#7806 is a non-conforming floor for one horn — not production enforcement).
3. **P2 — compiler refusal** per §4.4 steps 1–4 in the **chosen** infer authority (record literal + call edge, one mechanism — no separate pass).
4. **P3 — witnesses** per §4.4 steps 5–6 on `BehavioralComparisonObservation` + forge RED flip.
5. **P4 — enforcement flip** per §4.4 step 7 (`NotEnforced` → enforced only after witnesses green).
6. **P5 — falsification matrix** on other construction forms and alias/HOF climb triggers.

## 8. Out of scope

- Merging #7792/#7794 as behavioral-equivalence claims.
- Third decorative seal carriers.
- Full function visibility / module privacy (broader than reference-edge admission).
- Retrofitting every `sole_constructor` type in one motion.

## 9. Handback

- Signed design note + **operator compiler-horn decision**.
- `ConstructorReferenceAdmission` carrier + resolved `DeclarationRef` rows (imported constructors).
- Infer refusal in chosen compiler (one authority: record literal + call edge).
- `fully_matching_derived_seal_forgery_advance_succeeds_red` flipped to refuse on forge path.
