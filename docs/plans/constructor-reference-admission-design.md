# ConstructorReferenceAdmission — reference-edge admission for sealed constructors

**Status:** DRAFT for operator review (2026-08-04, royal-crab-235, msg_13fbbaaf / node `adhoc-f519571b-764`; updated msg_a87c2c75). **No code lands from this note** until the shape is agreed and the **which-compiler** horn is operator-picked (§4). Owner: royal-crab-235.

**Origin:** Executed falsification (royal-crab-235, msg_6701c04d): `sole_constructor` refuses cross-module **record literals** but does **not** refuse cross-module **calls** to an exported mint with caller-supplied fields; legitimate wet transport requires that mint. Process-output carrier reshuffle and opaque `WitnessBin.Run` output do not close the hole. **Derived ceiling:** `can-climb-after-one-capability`. **Preferred trigger:** this capability.

**Evidence PRs (draft, do not merge claiming `FixtureBehaviorallyEquivalent`):** #7792, #7794.

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

**Closing both refused + leaking routes** requires a capability beyond `sole_constructor` alone. **Legitimate wet transport** (`tools.direct_rust_door_transport`) must remain admitted after the wall lands.

## 2. End shape (carrier)

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

### 4.2 Operator hold — two horns (program sequencing, not design)

**Do not pick in this session.** Both horns and their costs are named here; the operator picks.

| Horn | What it costs | Execution today |
|---|---|---|
| **Land in v1 seed infer** | Grows the thing the v1-deletion program is shrinking: new capability surface + Rust realization lines in hand-maintained `v1_compiler_infer.rs` (and `.dag` infer model). Same class as operator pushback today on adding hundreds of lines to seed files under correct modeling. **But:** wall **executes immediately** on the live corpus path — same as existing `sole_constructor` record-literal refusal. |
| **Land in v2 infer only** | Aligns with self-host direction; does not expand v1 seed. **But:** v2 infer has **no `sole_constructor` today** — the wall does not execute until v2 infer implements construction admission (record literals + call edges) and that stage is what runs for acceptance. A wall that does not execute is specification-without-execution — the failure this lane proved matters. |

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

**Session disposition:** carrier shape, refusal coordinates, composition, first_slice bar, alias stance, and no-separate-pass rule are all decidable without picking the horn. **Implementation waits on operator horn choice.**

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

**Proposal:** admission row lives **on the constructor's declaring module** (beside the type or mint fn), cited by `DeclarationRef` to the constructor symbol. Callers list wet-transport (and test) callers explicitly.

**Module move:** `DeclarationRef` uses `module_path` + `decl_name` — if a permitted caller moves modules, the ref must be updated (same as any `DeclarationRef` row); no path-inferred auto-permission.

### 5.4 Compiler-module exemptions

`sole_constructor` carries compiler-module check exemptions (unverified completeness). **ConstructorReferenceAdmission must not inherit exemptions by default.** If an exemption is needed, it is a named row with dissolution trigger — never silent skip inside `v2.compiler` / `v1.compiler` trees.

## 6. First slice (already written — executable acceptance)

**Subject:** `mint_behavioral_comparison_observation` → `BehavioralComparisonObservation`.

**Admit:** `tools.direct_rust_door_transport` wet path caller(s) only (resolve to `DeclarationRef` rows in the admission metadata).

**Discriminating control (already enrolled):** `fully_matching_derived_seal_forgery_advance_succeeds_red` — performs **real forge** via `hermetic_equivalent` + derived seal; **must flip from PASS to refuse** when admission wall lands. Never `optional_absent()`.

**Positive controls (already in tree):** record literal refusal; legitimate transport path when caller is listed; advance refuses without admission seal when observation is genuinely diverged.

**Proof bar:** green-by-execution on the slice + forge RED flips — not grep, not title claim.

## 7. Phases (no code until note signed + operator picks compiler horn)

1. **P0 — this note:** operator sign-off on shape; **operator pick v1-seed vs v2 infer horn** (§4.2).
2. **P1 — model carrier** in `std` with nonempty permitted callers; import `decl_ref` from `std.decl_ref`.
3. **P2 — compiler refusal** in the **chosen** infer authority (record literal + call edge, one mechanism — no separate pass).
4. **P3 — first slice wiring** on `BehavioralComparisonObservation` + forge RED flip.
5. **P4 — falsification matrix** on other construction forms and alias/HOF climb triggers.

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
