# Authority relation topology — AUTH-0 design note

Status: **DRAFT for operator review (AUTH-0, session cool-ram-632, 2026-08-04).** Design-note first — **no code lands without operator sign-off.** This doc does not fix the four closed specimens (CIT-0, CIT-1, QM, SCAFFOLD); each has its own green PR awaiting merge. It names the **relation kinds** those specimens instantiate so future work can ask “which relation is this?” instead of re-inventing a parallel authority.

Frames: DESIGN §2 (horizontal derive-global / deep decompose), §3 (single authority, external upstream decomposition, cite-the-symbol), §5 (construction before validation; decidable vs wall-after-grounding vs ratchet). Worked precedent: `extdeps.pin` `pin_dimension_note`, `pin_subject_must_not_be_self_identifying_note`.

---

## 0. Why this exists

Over one day, four independent PRs each invented a parallel authority. No author was careless — each had a defensible local reason. That is the signal: if four careful workers independently produce the same *class*, diligence cannot be the defence. The product is a **topology of authority relations** — what kinds exist, what makes each decidable, and what evidence discharges it — not a slogan like “do not duplicate.”

The four specimens are **four different failures**, not one:

| label | failure class | one-line harm |
|---|---|---|
| **CIT-0** | same fact at two writable positions | resource says A, pin says B; both writable, nothing proves agreement |
| **CIT-1** | authored where observed was required | fixture digest indistinguishable from cited evidence; no bytes→digest edge |
| **QM** | stored where derived was available | `observed_vs_prediction` stored beside the counts that determine it; inconsistent triple accepted |
| **SCAFFOLD** | inferred where identity was required | unique substring match between scaffold path and roadmap carrier became a durable binding |

A topology that collapses these into “don’t duplicate” is useless. One that names each relation and states its discharge evidence is the product.

---

## 1. What “authority relation” means

An **authority relation** is not “who is allowed to do what” (that is `std.access`). Here it means: **a fact’s home in the model, and the evidence that establishes membership in that home.**

Every relation answers three questions:

1. **What is the atomic fact?** (the proposition, digest, identity, ordering, binding)
2. **Where may it be written?** (one constructor, one field, or derived-only)
3. **What discharges it?** (observation receipt, identity join, projection bridge, algebraic derivation — never “we checked later” when construction was available)

**Not an authority relation** (adjacent, must not absorb these):

- **Policy / permission** (`std.access`, `std.effect_grant`) — deontic, not epistemic
- **Disposition / scaffold registry** (`std.disposition`) — lifecycle metadata beside a fact, not the fact
- **Containment / naming** (`QualifiedName`, `SymbolIndex`) — structural position, feeds identity joins but is not itself a duplicate-authority class

---

## 2. Four relation kinds (the topology)

These are **orthogonal axes**. Conflating two kinds is the recurring §3 state-space failure.

```text
                    discharge evidence
                           │
     Coauthority ──────────┼────────── Observation
   (one fact, two          │          (bytes/receipt
    writable homes)        │           required)
                           │
     Derivation ───────────┼────────── Identity
   (fn of inputs,         │          (join key,
    not stored)            │           not heuristic)
```

### 2.1 Coauthority — same fact, two writable positions

**Specimen: CIT-0.** `CitedResource.authority` and `Pin.subject.authority` (via `CitedRepresentation`) could disagree while both remained writable — resource-says-A while pin-says-B.

**Harm:** silent fork at the meaning layer; grep passes both symbols; a reconcile compares against the wrong end.

**Discharge evidence:**

- **Structural:** subject does not self-identify → `Pin<Subject>` carries `expected_identity` as *new* information (`pin_subject_must_not_be_self_identifying_note` case 1: `CliTool`, `ActionRef`).
- **Structural:** subject fully determines cited digest at its grain → `SubjectProjectedPin<Subject>` stores subject + selection only; identity and version are **projected at the bridge** (`subject_projected_pin_as_pin`), never stored twice (case 2: `SccacheBinaryArtifact`; the `OciDescriptor` / `RepresentationPin` failure class).
- **Reconcile:** `admit_pin_integrity` / `admit_subject_projected_pin_integrity` refuse cross-family or absent identity before comparison.

**Worked fix (CIT-0 branch, not on main):** dissolve `RepresentationPin` fork; consume `extdeps.pin.Pin<CitedRepresentation>` where `CitedRepresentation` carries authority + `RepresentationByteDomain`; `citation_pin_subject_note` routes through `pin_subject_must_not_be_self_identifying_note`.

**§5 ladder:** wall now where “subject self-identifies?” is structurally decidable; wall after grounding where projection bridge is not yet inhabited; ratchet never (membership is decidable once grain is named).

**Review tell:** two fields on one carrier (or carrier + subject) that could be edited independently to disagree on digest, authority, version, or revision.

### 2.2 Observation — authored where observed was required

**Specimen: CIT-1.** A real document digest reused as an `example.test` fixture value — the same 64-hex string writable as both fabricated example and cited evidence, with no observation edge from bytes to digest.

**Harm:** specification-without-execution (§5): a test or declaration greens while the digest is a second definition, not a receipt of bytes.

**Discharge evidence:**

- **Observation receipt:** dated downstream fact with `request`, `observed_at`, typed outcome — never a timeless digest literal on an upstream module (`std.citation` `CitationRetrievalObservation`; layer gate refuses upstream-authored observations in product layer).
- **Bytes→digest pipeline:** `sha256sum_verify_via_shell` / `sha256_digest_content_hash` bridge — digest is output of verification, not an independent authorable string (`ubuntu_install_media_fetch` `install_media_fetch_hash_convergence_note` distinguishes cryptographic authority digest from `std.content_hash` structural fingerprint).
- **Oracle rule (§5, 2026-08-01):** merge-blocking tests may not compare live population to a tree-copied literal; fixture must independently author input *and* expected population.

**Worked fix (CIT-1 branch):** RFC 8118 safe profile, navigable projection, versioned PDF consumer with `Pin<CitedRepresentation>` and `pin_currency_gap`; fixture locator `example.test/fixture.pdf` separated from cited authority rows.

**§5 ladder:** wall now for “digest literal without observation arm”; wall after grounding where `ContentHash` family typing is not yet enforced at every weak site; external reality always boundary-observed.

**Review tell:** same hex string appears in a `data` row and in an evidence/claim path with no `RecordedFact` / retrieval / verify arm between them.

**Misclassification risk (flagged):** weak `NonEmptyStr` digest parameters (census §4.1) might be **weak typing** rather than observation-class — discharge may be `Sha256Digest`/`ContentHash` construction wall without needing a full observation receipt. Needs per-site grain review, not a blanket class.

### 2.3 Derivation — stored where derived was available

**Specimen: QM.** `observed_vs_prediction` stored on a record that also carries the three counts determining it. `observed: 5`, `prediction: 20`, `ordering: Greater` was writable **and passed** — the stored ordering disagreed with its inputs.

**Harm:** the redundant field becomes an independent authority; editors satisfy the check by editing the derived field while inputs stay wrong (the §5 “edit the declaration while the realization lies” tell).

**Discharge evidence:**

- **Single total function** over inputs (`path_additivity_witness_observed_vs_prediction` derived from `path_additivity_compare_observed_to_prediction`; stored field deleted on QM branch).
- **Algebraic refusal:** inconsistent triple is unwritable when the ordering is a fn of `(observed, prediction)` not a field.
- **Precedent:** `pin_value_eq` vs `key_of` — subject is identity (key), content fields include everything that changes install behaviour; comparing subject in `value_eq` would make every drift a no-op.

**§5 ladder:** wall now when the derivation is a closed total fn over modeled fields; wall after grounding when inputs themselves need grounding (e.g. counts from observation receipts); ratchet when the “derivation” is undecidable (optimality).

**Review tell:** a field name like `*_vs_*`, `*_ordering`, `*_comparison`, `is_*` sitting beside the operands it claims to summarize.

### 2.4 Identity — inferred where identity was required

**Specimen: SCAFFOLD.** A unique substring match between a scaffold disposition site’s path and a roadmap carrier path became a durable obligation binding. Uniqueness is not identity.

**Harm:** heuristic promoted to authority; the binding survives path renames that break the accidental substring; ambiguous and zero matches were either silent or fabricated.

**Discharge evidence:**

- **Identity join** on a declared key (`RoadmapNodeIdentity`, `record_identity`, `roadmap_identity_registry`) — not path substring.
- **Count discipline:** exactly one candidate → binding; zero → `NoHonestObligationOwner`; plural → `AmbiguousObligationOwners` (`gunbc.scaffold_retirement` pilot on branch `ec85b225782`).
- **Derived population:** `scaffold_disposition_sites_live()` structural walk — not a hand roster (`scaffold_retirement_pilot_derivation_note`).

**§5 ladder:** wall now for join keys that are closed and registered; wall after grounding when the registry is incomplete; ratchet forever for “semantic similarity” of paths (undeclared intent).

**Review tell:** `string_contains`, substring match, or “unique match” without `DeclarationRef` / `QualifiedName` identity key.

**Misclassification risk (flagged):** `scaffold_digest_for_module_path` in `emitter_producer_provenance` uses path-derived digests as **scaffold placeholders** explicitly marked interim — may be honest scaffold + dissolve trigger, not identity-class defect. Distinct from SCAFFOLD specimen because it does not claim to be an obligation owner.

---

## 3. Precedent — `extdeps.pin` as dimension, not property

`pin_dimension_note` (operator, 2026-07-29): pinning is a **dimension** (`Pin<Subject>`), not a property of `CliTool`. The topology generalizes this move:

| pin lesson | topology generalization |
|---|---|
| One axis, N instantiations (`Pin<CliTool>`, `Pin<SccacheBinaryArtifact>`, …) | One relation **kind**, N subject grains — do not mint `ToolPin`, `CitationPin`, `ScaffoldPin` as siblings |
| `pin_subject_must_not_be_self_identifying_note` chooses carrier by structural question | **Coauthority** discharge: “does the subject determine this fact?” decides writable shape |
| `SubjectProjectedPin` + bridge projection | Coauthority wall: duplication unwritable, not forbidden-after-the-fact |
| `pin_observed_grain_prerequisite_note` | Observation grain must match before reconcile — aggregate ≠ per-subject |
| Two `extdeps_external_authority_anchor` spellings (structural, not alias) | DESIGN §3: two citations when mandatory_tag reads structurally — not coauthority failure |

**External upstream decomposition (DESIGN §3):** a generic hub may hold agnostic shape; it may **not** enumerate concrete products, dispatch among authorities, or store consumer coverage. `std.citation` consumes `ExternalAuthority` without extending it; site realizations (`extdeps.standards.rfc_8118`, `extdeps.mediawiki`) stay per-upstream modules. A hub with one consumer is speculative abstraction — second-consumer discipline applies before elevating a relation to `std/`.

---

## 4. Census methodology (operator ruling, 2026-08-04)

**Do not begin with a hand-authored catalogue.** A hand roster counts instances someone remembered.

**Order:**

1. Refuse new instances in active lanes — **done** for the four specimens (closed PRs, not touched by AUTH-0).
2. **Exact live censuses** derived from the corpus (structural walks, grep with measured populations — inputs to classification, not conclusions).
3. Decide whether one shared construction replaces each class (per §5 decidability).
4. **No-growth frontier** — derived population enrollment, monotone debt only with closed universe (§5 oracle rule).

AUTH-0 stops at steps 2–3 as **candidates**. Step 4 is AUTH-1+ after sign-off.

---

## 5. Candidate census populations (measured 2026-08-04 on `main`, not classified)

The operator surfaced three candidates while reviewing other work. **None are proven to be a single class.** Counts below are live measurements for the design note only.

### 5.1 Weak digest fields

**Pattern:** digest/hash carried as `NonEmptyStr` or `expected_hash: NonEmptyStr` where `std.content_hash` (`ContentHash`, `Sha256Digest`) exists.

**Measured:** **12** parameter/field sites across **7** modules (e.g. `ubuntu_install_media_fetch.observe_install_media_fetch`, `sha256sum_verify_via_shell`, `freestanding_payload.dag`, boot placeholders).

**Open question:** split into (a) observation-class — needs bytes→digest receipt; (b) serialization boundary — hex is wire form of `Sha256Digest` (per `file_mode_octal_note` precedent: “serialization, not second representation”); (c) interim scaffold placeholders. **Cannot conclude one class without per-site structural read.**

### 5.2 `*Readiness` type surface

**Operator candidate:** seventeen coproducts whose names end in `Readiness`, structurally the N-types-one-axis shape `Pin` was corrected for.

**Measured:** **10** distinct top-level type names ending in `Readiness` on `main` (e.g. `ToolReadiness`, `ProviderReadiness`, `WitnessBinReadiness`, `CompilePoolReadiness`, `LandingEvidenceReadiness`, …), plus aliases/folds/receipts (`ClaimRequirementReadiness`, `LandingClaimRequirementReadiness`). Many are **records or claim-evidence specializations**, not closed coproducts.

**Open question:** whether these share one axis (readiness = evidence threshold over a claim) and should project through `std.claim_evidence.ClaimReadinessReceipt` rather than mint local readiness sums. **Not confirmed** — `ToolReadiness` and `ProviderReadiness` may be legitimately distinct upstream shapes per external upstream decomposition. Needs structural coproduct census, not name suffix alone.

### 5.3 `concat(acc, …)` accumulator growth

**Operator candidate:** `concat(acc, [x])` at ~92 sites; quadratic under bare-minimum-cost rule (§6) — **operator has not verified** that `concat` on lists copies.

**Measured on `main`:** **274** occurrences of `concat(acc,` across **119** `.dag` files; **117** of the form `concat(acc, [`; **17** `append(acc,`. Total differs from operator’s ~92 — likely different pattern scope or tree movement since the estimate.

**Open question:** cost-shape defect vs idiomatic linear fold depends on whether list `concat` copies (semantics + realization), not occurrence count alone. `src/v2/test/claim/complexity/accumulator_copy_roster_gate_std_test.dag` exists as partial machinery. **Candidate only** until execution-measured cost shape.

---

## 6. Relation → construction map (AUTH-1+ sketch, not signed)

| kind | preferred construction | residue ratchet |
|---|---|---|
| Coauthority | single writable home + projection bridge (`SubjectProjectedPin`, optional field not variant) | cross-family compare until `ContentHash` family wall complete |
| Observation | typed observation coproduct + layer gate; digest only on receipt arm | host-fetched census until witness realization |
| Derivation | delete stored field; expose `fn *_derived(...)`; refuse inconsistent inputs | undecidable summaries stay explicit `Unknown` |
| Identity | join on `DeclarationRef` / `RoadmapNodeIdentity`; 0/plural → exclusion | path similarity suggestions never auto-bind |

**Explicit non-goals for AUTH-0:** no new `std/` module, no lens enrollment, no edits to the four specimen PRs.

---

## 7. Misclassification risks (reasoning, not conclusions)

1. **CIT-0 vs Observation:** `CitedResource.authority` vs `Pin.subject.authority` is coauthority (same symbolic fact, two homes), not missing observation — the fix is carrier choice, not a retrieval receipt.

2. **CIT-1 vs Coauthority:** same hex string in fixture and evidence is primarily **observation** (no bytes edge). It only becomes coauthority if both are long-lived writable fields on the same carrier — CIT-1’s fix also separates locator namespaces (`example.test` vs cited URI).

3. **QM vs Derivation:** if the stored ordering were a **policy choice** (operator preference) rather than a function of counts, it would not be derivation-class — it would be honest data. QM specimen is derivation because the field *claimed* to summarize numeric comparison.

4. **SCAFFOLD vs Coauthority:** substring binding is **identity** failure (wrong join key), not duplicate writable fields. The scaffold_retirement pilot fixes join, not merge of duplicate columns.

5. **Weak digests vs CIT-1:** upgrading `NonEmptyStr` → `Sha256Digest` may be sufficient without full observation coproduct where the value is **only** passed into a verify fn that already owns the bytes edge (`ubuntu_install_media_fetch` may be partially fixed by typing alone — **needs site read**, not assumed).

---

## 8. Operator questions (block AUTH-1 implementation)

1. **Readiness axis:** Is there one `Readiness<Domain>` parameter, or do `ToolReadiness` / `ProviderReadiness` / `DashboardProviderReadiness` remain independent upstream shapes per §3 decomposition?

2. **Digest census split:** Should the weak-digest census partition into {serialization, observation-required, scaffold-placeholder} before any construction, or land one `ContentHash` construction wall first?

3. **Concat census:** Is the operator’s ~92 a strict `concat(acc, [single])` fold-body pattern? Should AUTH-1 measure interpreter list-copy cost before any rewrite?

4. **Hub location:** Does a future `std.authority_relation` hub violate second-consumer discipline, or should relation kinds live only as notes + lenses until two unrelated consumers exist?

5. **Citation + pin unification:** CIT-0’s `Pin<CitedRepresentation>` — is `std.citation` the long-term home for all cited-representation pinning, or an `extdeps.citation.pin` instantiation beside `extdeps.tools.pin`?

---

## 9. Dissolution

This document dissolves when: (a) each relation kind has a named construction wall or honest ratchet row in the guarantee recovery ledger; (b) the three candidate censuses in §5 are either promoted to enrolled derived populations with RED controls or refuted with measured negative receipts; (c) no new specimen PR invents a fifth kind without updating this topology. Until then, AUTH-0 is the authority for **which relation**, not **which module**.
