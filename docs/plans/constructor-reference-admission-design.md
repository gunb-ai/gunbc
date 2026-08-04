# ConstructorReferenceAdmission — reference-edge admission for sealed constructors

**Status:** DRAFT for operator review (2026-08-04, royal-crab-235, msg_13fbbaaf / node `adhoc-f519571b-764`). **No code lands from this note** until the shape is agreed. Owner: royal-crab-235 (measured both construction routes on `BehavioralComparisonObservation`; first_slice and discriminating controls already in tree).

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
| Cross-module record literal | **REFUSED** | `SoleConstructorViolation` in infer (`v1_compiler_infer` record-literal arm; same-module span check) |
| Cross-module call to exported `mint_behavioral_comparison_observation` | **LEAKS** | No call-edge admission; exported fn is writable from any importer |
| Opaque host-effect return | **NEGATIVE** | `WitnessBin.Run` returns forgeable `String`/`Bool` fields |
| Process-output `sole_constructor` carrier (#7794) | **DECORATIVE** | Forge via `from_run_fields` / `hermetic_equivalent` |

**Closing both refused + leaking routes** requires a capability beyond `sole_constructor` alone. **Legitimate wet transport** (`tools.direct_rust_door_transport`) must remain admitted after the wall lands.

## 2. End shape (carrier)

```dag
// std layer — agnostic admission shape (authority TBD: std.construction or std.decl_ref extension)
type ConstructorReferenceAdmission {
  constructor: DeclarationRef          // callee that constructs the sealed type (mint fn or sole ctor fn)
  permitted_callers: List<DeclarationRef>  // nonempty by construction — see §3
}
```

**Citation rule (§3):** both `constructor` and each permitted caller are `DeclarationRef` rows — they ride `std.decl_ref` / cited-symbol resolution already in tree. Every ref must resolve to exactly one declaration or refuse at authoring; no second naming scheme.

**Refusal (when a call edge is checked):** typed, located diagnostic naming:

- the **constructor** `DeclarationRef` (resolved symbol),
- the **offending caller** `DeclarationRef` (the calling declaration — typically the enclosing `fn`),
- the **permitted set** (resolved symbols or explicit "none listed" — never silent).

Suggested variant name: `ConstructorCallAdmissionRefused` (exact spelling TBD with existing diagnostic taxonomy).

**Not a generic violation string** — coordinates must be structurally present for counting and ladder receipts.

## 3. Nonempty permitted callers

`NonEmptyList<DeclarationRef>` does not exist in `std` today. Precedent: `gunbc.doc_graph_roots` `HandAuthoredDocBind` — `primary_work` required scalar + `additional_works` list makes zero-anchor unwritable (construction, not validation).

**Proposed:** `permitted_callers` as `List<DeclarationRef>` with **authoring-time nonempty** enforced on the admission row carrier (wall on the metadata row), OR split `first_permitted_caller: DeclarationRef` + `further_permitted_callers: List<DeclarationRef>`. Pick one at implementation; design requirement is **empty permitted set is unwritable**.

## 4. Where the refusal fires — OPEN DECISION (load-bearing)

**Measured today:** `sole_constructor` refusal fires in **infer** on **record literal** construction when the type's declaring-module span ≠ literal span (`v1_compiler_infer`, `SoleConstructorViolation`).

**ConstructorReferenceAdmission** needs a **call-edge** check: importer module calls exported mint/creator with caller declaration not in permitted set.

**Candidate placement:**

| Stage | Pros | Cons |
|---|---|---|
| **Infer — direct call resolution** (parallel to record-literal sole_ctor arm) | Same stage as existing sole_ctor; callee + caller module known at call site | **Infer is load-bearing** (DESIGN §7); touches hot path |
| Post-infer dedicated pass | Isolates new logic | Second representation of "who may call whom"; risk of drift from infer call binding |
| Name resolution only | Aligns with "reference edge" | May be insufficient if call shape is not a resolved direct callee (§5) |

**STOP — operator decision required before implementation:** if the honest general case requires infer changes or name-resolution surgery, confirm that placement rather than routing around it. This note does **not** assume a workaround path.

## 5. Design questions (must be answered in this note before code)

### 5.1 Alias and higher-order positions

When the callee is not syntactically the constructor (alias re-export, higher-order parameter, indirect call):

- **Default honest stance:** outside the modeled guarantee until a witness proves otherwise — refuse or declare `AdmissionUnknown` with typed cause, never silently admit.
- **Direct call to resolved `mint_behavioral_comparison_observation`** is the first-slice scope; alias chains and HOF are explicit follow-on falsification rows.

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

## 7. Phases (no code until note signed)

1. **P0 — this note + operator sign-off** on placement (§4) and alias/HOF stance (§5.1).
2. **P1 — model carrier** in `std` (or extend `std.decl_ref`) with nonempty permitted callers by construction.
3. **P2 — compiler refusal** at agreed stage with `ConstructorCallAdmissionRefused` diagnostic.
4. **P3 — first slice wiring** on `BehavioralComparisonObservation` + witness flip for forge RED.
5. **P4 — falsification matrix** continues on other construction forms (`Refined<B>`, casts, variants) on **other carriers** — no generalization from one specimen.

## 8. Out of scope

- Merging #7792/#7794 as behavioral-equivalence claims.
- Third decorative seal carriers.
- Full function visibility / module privacy (broader than reference-edge admission).
- Retrofitting every `sole_constructor` type in one motion.

## 9. Handback

- Signed design note with placement decision.
- `ConstructorReferenceAdmission` carrier + resolved `DeclarationRef` rows for first slice.
- Infer (or agreed stage) refusal with typed diagnostic.
- `fully_matching_derived_seal_forgery_advance_succeeds_red` flipped to refuse on forge path.
