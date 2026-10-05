# PLAN — nominal types as the first consumer of the coercion frontier; delete refinements

**Status: PLAN, revision 5 — design only. Do not re-enqueue.** Revision 4's O3, O4 and O5 are RULED (2026-10-02); O1, O2 and C5 are recut here on the operator's ruling; O6/C4 is a separate lane. Earlier history: Revision 3 (one declaration modifier
per point in the visibility plane, plus a `derives` list) was approved and then withdrawn by the
operator on 2026-10-02: too many keywords, and the nominal question is really a coercion question.
This revision recasts it as the **first consumer of the coercion frontier** that
[v2 compiler architecture](v2-compiler-architecture.md) §1.4 and its frontier line ("Coercion and
named conversions") name. §8 places revision 3 (option A) side by side with this model.

Governing sections: DESIGN §4 (operations from inhabitance; the coercion ruling, `bool_indicator`),
§3 (replacement migrations), §4b, §5; architecture plan §1 (completion law, residual addressees),
§1.4.

Every count is **provisional** — line greps at main `dd0614551b5`, not the census — and is not a size
(DESIGN §6). M0, the resolved census (`gunbc.instruments.type_declaration_use_census`, being built by
tidy-koi-264), is the only admissible size.

> **PARKED (operator ruling, 2026-10-03).** M0 landed as an instrument
> (`//gunbc/instruments:type-declaration-use-census`, gunbc#13093) with **no M2 sizing**. Its receipt,
> instrument-dispatch run [37119024146](https://github.com/gunb-ai/gunbc/actions/runs/37119024146), is
> no observation (exit 2): native resolve reaches only part of the corpus and decides no casts, so the
> migration cannot be sized. The program resumes when a re-run of the instrument shows enough native
> resolve and infer coverage. S1 and P1–P3 are moot until then. Re-run the instrument for any figure;
> none is carried here.
>
> **Open items from M0's census** (posted on gunbc#13024 by tidy-koi-264; to be settled when this
> resumes):
>
> 1. **Bodyless declarations that are not `nominal_opaque`** (`type MachineWidth<bits>` and similar)
>    have no row in the declaration table, and the census reports them as `BodylessUnclassified`. The
>    plan must say whether they are phantom/index types, abstractions with no view, or a further class.
>    Under rev 5's model they are the bodyless form, but whether every one of them needs modeled
>    operations (§3) is not decided.
> 2. **Existing one-field records.** The census classes them as `UnconstrainedNominal`. Under rev 5 a
>    one-field record *is* a nominal (records are nominal), so that reading agrees with §2. Whether they
>    should still be reported as a separate row, so that authored brands and pre-existing records stay
>    distinguishable in the migration, is open.
> 3. **Opaque's second disjunct** ("a nominal whose view no consumer outside its module reads") is not
>    computed. Only `nominal_opaque` decides `Opaque`. Computing it needs view-read evidence, a
>    representation read outside the declaring module. Rev 5's §3 relies on exactly that evidence
>    (`EffectivePosixPrincipalName` → record), so the generated cutover needs it.
> 4. **`OperationWorkaround` for encoding** cannot be classed while encoding's capability home (S2) is
>    unnamed. Until then, encode-shaped strips go to the exception list with the callee named in their
>    evidence.

## 1. What stays as ruled

- `type X = Y` is a transparent alias: X and Y are the same type.
- Refinements (`where <pred>`) are deleted. Every invariant becomes a checked construction one layer
  down.
- No typecase, and no conditional design for one. Matching is on constructors and closed sums.
- M0, the resolved semantic census, sizes the migration.
- The cutover is one atomic, generated PR that deletes `where`, `brand`, `nominal_opaque`, the v1
  spelling matches and hand tables, `gunbc.where_refinement_predicate_vocabulary` with
  `WherePredicateGrounding`, and #12506's parked row together (the full consumer list is revision 3 §6, commit 97dea7c55fd, and stands unchanged).
- Checked construction returns `Result` residue plus a typed cause (`std.error_primitives` `Result`).

## 2. The model: four forms, no new words

> alias = equality · record = visible constructed data · bodyless = abstraction · functions = explicit progress

| form | meaning | in | out | operations |
|---|---|---|---|---|
| `type X = Y` | alias; X and Y are the same type | — | — | Y's |
| `type X { value: Y }` | a new type with visible data | its constructor (confinable by `sole_constructor` + `admit_callers`) | field projection | product inhabitance (§4) |
| `type X` (bodyless) | an abstraction; the declaration is all a client sees | only by the module's modeled operations | only by the module's modeled operations | only modeled operations |
| `fn` | explicit progress between any two types | — | — | — |

The brand (`where brand("…")`), the refinement and `nominal_opaque` stop being declaration facts:

| old mode | becomes |
|---|---|
| brand | one-field record, public constructor |
| refinement | one-field record, `sole_constructor`, plus a public checked function `Y -> Result<X, R>` |
| `nominal_opaque` | bodyless `type X` whose operations its module models (§3), or a record when the census shows nothing needed hiding (as for `EffectivePosixPrincipalName`, §3) |

No crossing between a nominal and its representation is implicit (§5). A crossing is a residual
addressed to the call site (O4, ruled: the addressee home is `v2.std.residual`), and the developer
answers it with an ordinary function call (O5, ruled).

**Routes are proven, never inferred from a signature (O1 as ruled).** A signature proves neither
exactness nor injectivity: `id.value`, `concat("roadmap:", id.value)` and `sha256(id.value)` all have
type `RoadmapNodeId -> String`. So every function remains a valid **explicit** conversion when it is
called. A function becomes a **compiler-discoverable route** (one a residual may name as its answer,
and one operations may be derived along) only when the compiler derives a structural homomorphism
witness for its body:

- `X -> Y` whose body projects the sole field;
- `Y -> X` whose body applies the sole constructor;
- `Y -> Result<X, R>` whose success arms construct `X` from the input by the sole constructor.

Two proven routes for one crossing refuse as `AmbiguousTargetCandidate`. There is no keyword and no
route table: the witness is the authority.

## 3. Bodyless types, proved against every current `nominal_opaque` site

The bodyless form already exists and is already the abstraction/target-realized form: `std.types`
declares `type Unit`, `type Json` and `type Bytes` with no body, and `v2.std.type_binder` wraps a
bodyless declaration (`type_opaque_wrapper`). There are two `nominal_opaque` declarations today.

**`std.types` `Secret` (`nominal_opaque = String`).** Today it is opaque in name only. The corpus has
`as Secret` casts in (M0 `NominalIntroduction` rows targeting `Secret`; most in `test.claim.*` fixtures; production mints in
`extdeps.cloud.gcp.adc_document`, `extdeps.cloud.gcp.secret_manager`, `gunbc.spark.glm_canary_converge`),
and `as String` casts out, e.g. `extdeps.cloud.gcp.secret_manager`
`encode_sm_access_version_payload_wire` and `extdeps.cloud.gcp.secret_ref` (M0 counts
them). Both directions are unconfined, so DESIGN §4b's "cosmetic until construction enforces it" is
literally its state. As `type Secret` (bodyless) it needs:

- **trusted construction**: the mints are upstream reads (secret-manager payload decode, the ADC
  document, credential reads) plus test fixtures. Each becomes a call to a modeled operation of
  `Secret`, e.g. `secret_of_upstream_text(s: String) -> Secret`, confined by `admit_callers` to the
  named upstream decoders and the named fixture modules (precedent: `extdeps.apple.app_attest`
  `attestation_verification_from_implementation` admits named test fixtures);
- **trusted elimination**: the wire encoders and transports call a modeled `secret_wire_text(s: Secret)
  -> String`, likewise confined;
- **its target realization**: `gunbc.rust_source_type_bindings` already has a row binding `Secret` to
  `std::string::String`. It is `StillBareNameDebt`, waiting on "the same threading capability as Bytes".

**`extdeps.access.posix_effective_principal` `EffectivePosixPrincipalName` (`nominal_opaque = NonEmptyStr`).**
It has one mint (`effective_posix_principal_observation`) and no read outside its module. Nothing is
hidden from anyone, so it is a **record** with `sole_constructor`, not a bodyless type. M0 confirms
there is no outside read before the cutover generates it.

**The capability that does not exist yet: realized operations of an abstract type.** Today a
bodyless type's operations are host seams: `std.bytes` `utf8_encode_bytes` and `bytes_octets` are
self-calling bodies that the interpreter intercepts by **name**, and the Rust emitter realizes them from
a name-keyed bridge table (`extdeps.languages.rust.emit`). `std.bytes` marks that arrangement itself as
a `Scaffold` (`bytes_seam_host_realization_marker`). So `type Secret` has no trusted
construction or operation support that is not a name-keyed seam. Per the ruling, that capability is
what this program names and builds, rather than a word:

> **AbstractOperationRealization:** an operation of a bodyless type is a function whose body is a
> target-realization row keyed by the function's **declaration** (not its spelling), whose
> mint and elimination are confined by `admit_callers`, and whose realization per required target
> (interpreter and Rust today) is declared beside the type's own declaration-keyed binding row.

It dissolves the `Bytes` seam scaffold and the `Secret` `StillBareNameDebt` row with one mechanism.
It is also the "threading capability" that the row's restoration trigger already names. Its home is
the existing realization-binding authority (`gunbc.rust_source_type_bindings` and its interpreter
counterpart), extended by declaration key. It is not a new vocabulary.

## 4. Operations: equality and hash from product inhabitance

- A **record** has equality and hash when every field does. That is product inhabitance, so a
  one-field record over `String` is comparable and hashable because `String` is
  (`std.algebra` `algebra_profile_equality_extensional`; `std.content_hash` `HashFamily`).
  `RoadmapNodeId == RoadmapNodeId` works, and `RoadmapNodeId` where `String` is declared still refuses:
  the operation is derived, and the value does not cross.
- A **proven embedding** (a route with a derived witness, §2) may also carry equality and hash. A
  signature never does.
- **Ordering never by default** (O3, ruled). It requires an operation the module writes.
- A **bodyless** type gets only its modeled operations. `Secret` has no display, JSON or equality unless
  its module models them. Constant-time comparison is a modeled operation, realized per target.

No `derives` list exists in any form.

## 5. What `v2.std.coercion` and `v2.std.inhabitance` must gain (model-first)

What exists today: `CoercionResult { target, quality: Identity | Exact | Widened, witness }`;
`CoercionMismatchKind = NoTargetCandidate | AmbiguousTargetCandidate | StructuralMismatch |
WouldLoseInformation`; `find_witness` under `TargetSelectionPolicy`; `declared_type_inhabitance` over
`DeclaredTypePosition`. `v2.std.refinement_widening_predicate` is representation value-set widening
(`rust i32 → python int`). It is not the `where` machinery and is kept.

1. **C1: residual addressee** in `v2.std.residual` (O4, ruled). It is minted once and consumed by
   `v2.std.coercion` first, then by completion (architecture §1.5). Each `CoercionMismatchKind` maps
   totally to its addressee.
2. **C2: `ChangesInterpretation`**, a mismatch kind for an exact, lossless crossing that drops or adds
   meaning (`RoadmapNodeId → String`, `true as Int`). Its addressee is the call site.
3. **C3: proven route discovery.** It derives the §2 homomorphism witness from a function body
   (sole-field projection, sole-constructor application, success-arm construction), reusing
   `v2.std.witness` `HomomorphismWitness`. A residual names the proven routes as its answer. Two proven
   routes give `AmbiguousTargetCandidate`.
4. **C4: native `sole_constructor` minting and enforcement plus `admit_callers`.** This is **a separate
   lane**, dispatched by sharp-raven-357 on 2026-10-03, closing both native drops
   (`gunbc.rung_drop.admit_callers_discarded_on_the_native_route`, and `sole_constructor` neither minted
   nor enforced natively). It is referenced, not built here.
5. **C5: product inhabitance for equality and hash** in `v2.std.inhabitance`: a record inhabits
   equality or hash when its fields do. A proven-embedding arm is second. There is no signature arm.
6. **C6: AbstractOperationRealization** (§3), with `Secret` and `Bytes` as its first two consumers.
7. **C7: the first instrument** named by the architecture frontier: one core module emitted to the
   Rust and TypeScript targets, with the per-target mismatch list checked against M0's rows.

The migration is then generated from M0: brand → one-field record; refinement → `sole_constructor`
record plus checked function; `Secret` → bodyless with modeled operations; `EffectivePosixPrincipalName`
→ `sole_constructor` record; aliases untouched; introduction casts → constructor or modeled-mint calls;
workaround casts (strip to compare or hash) → the derived operation; true raw consumers → explicit
projection or modeled elimination.

## 6. Controls (expecting red first)

The six arms of `test.claim.brand_nominal_identity_witness_test` are kept, re-spelled over records:
COLLISION and MISMATCH refuse; CONSTRUCT, STRIP and DUAL accept; UNDECLARED_SITE refuses at parse.
These controls are added:

| control | expected after | today |
|---|---|---|
| `RoadmapNodeId` at a declared `String`, no call | residual `ChangesInterpretation`, call site, names the proven projection | no addressee: **red** |
| `concat("roadmap:", id.value)` as a function of type `RoadmapNodeId -> String` | valid when called; **not** named as a route | **red** until C3 exists |
| two proven `X -> Y` routes | `AmbiguousTargetCandidate` | **red** |
| `Port { value: 0 }` outside the module | refuse on the v2 route | **red** (C4 lane's control) |
| `port_of(n: 0)` / `port_of(n: 80)` | `Err` with residue and cause / advances, by execution | positive control |
| `RoadmapNodeId == RoadmapNodeId` | accept by product inhabitance | M0 measures |
| `s as Secret` for a String | refuse; only `secret_of_upstream_text` from an admitted caller | **red** (cast admitted today) |
| `secret_of_upstream_text` from an unadmitted module | refuse | **red** |
| `Secret` displayed, JSON-encoded or compared, nothing modeled | refuse | **red** |
| `Secret` on the Rust target through the declaration-keyed row | emits and runs | **red** (`StillBareNameDebt`) |

None of these retires (DESIGN §4b(4)).

## 7. Rungs (DESIGN §4b) and §4's coercion law

- **Identity:** structurally guaranteed. Records are nominal, and a bodyless declaration is its own type.
- **Invariants and confinement:** **no invariant is called structural until both native drops in the
  C4 lane are closed.** Until then the v2 route is mechanically preventable at best, stated per type.
- **Secret:** structural only once C4 and C6 both hold. Before that it is cosmetic, as it is today.
- **Coercion law:** unchanged. The only silent route is the alias's identity. Every nominal crossing is
  an explicit call, and a missing one is a residual with an addressee.

## 8. Revision 3 (option A) against this model

Populations are M0's rows, named by class; none is transcribed here (DESIGN §6).

| | option A (rev 3) | rev 5 |
|---|---|---|
| **new words** | `nominal`, `sealed`, `opaque`, `derives(…)` = 4 | 0. Bodyless, `sole_constructor` and `admit_callers` already exist |
| **TrueAlias** | untouched | untouched |
| **UnconstrainedNominal** (old brand) | `type T nominal = Y` | `type T { value: Y }` (constructor in, projection out) |
| **CheckedConstructionStage** (old refinement) | `sealed` + checked ctor | `sole_constructor` record + checked function |
| **Opaque** | `opaque` | bodyless `type X` + modeled operations (needs C6) |
| **introduction sites** (M0 `NominalIntroduction` rows) | rewritten to a constructor call | same rewrite, generated; identical burden |
| **projection sites** (M0 `NominalProjection` and `OperationWorkaround` rows) | rewritten to `.value` | rewritten to explicit projection; **workaround sites** (compare, hash) **vanish** under C5 product inhabitance instead of being rewritten, so this model's burden is lower by exactly M0's `OperationWorkaround` count |
| **capabilities** | `derives(…)`: author-listed, a second vocabulary to keep in step with `std.algebra` | derived by product inhabitance; nothing listed; a bodyless type has only modeled operations |
| **invariant rung** | structural once `sealed` is enforced | structural once the C4 lane closes both native drops: **the same precondition**, already modeled and with a seed implementation |
| **`RoadmapNodeId` as `String`** | refuses; author projects `.value`, with no stated reason | residual to the call site naming the proven projection: the acknowledgment the operator asked for |
| **needs that do not exist** | 3 modifiers + derives parser/infer, visibility semantics, capability derivation | C1 addressee, C2 kind, C3 proven routes, C5 product inhabitance, C6 AbstractOperationRealization (C4 is its own lane) |
| **reusable beyond nominals** | no | yes: C1–C3 and C6 are frontiers the architecture plan already owes (`bool_indicator`, target emission mismatches) |

The deciding row is the last one. Option A built a nominal-only mechanism next to an unfinished
coercion model. Revisions 4–5 finish the coercion model, and nominals fall out of it. That is §2's
test that net concepts must not grow by re-invention: `sealed` was a re-invention of
`sole_constructor`, which the revision 3 DFS missed.

## 9. Decisions

Ruled (2026-10-02/03): O3 (no ordering by default), O4 (`v2.std.residual`), O5 (answer by function
call); O1 recut to proven routes (§2); O2 recut to the bodyless form (§3); C5 recut to product
inhabitance (§4); C4 a separate lane. Left for the operator:

- **P1: AbstractOperationRealization's home.** Recommend: extend the existing declaration-keyed
  realization binding (`gunbc.rust_source_type_bindings` and its interpreter counterpart) from types to
  a bodyless type's operations. Do not open a new module.
- **P2: Secret's fixture mints.** Most `as Secret` sites are `test.claim.*` fixtures. Either each test
  module is listed in `secret_of_upstream_text`'s `admit_callers` (explicit and long), or a separate
  confined `secret_fixture_of` operation admits a fixture prefix. Recommend: **per-module
  `admit_callers`**, generated by the cutover from M0. A prefix is a policy hole any new test module
  falls into silently.
- **P3: sequencing.** C1 → C2 → C3 → C5 land model-first, beside the C4 lane. C6 lands after C4 (its
  confinement needs C4). The generated cutover lands after all of them. M0 runs independently.
  Recommend: as listed.
