# Fact provenance topology — AUTH-0 design note

Status: **DRAFT for operator review (AUTH-0, session cool-ram-632, 2026-08-04).** Design-note first — **no code lands without operator sign-off.** Does not fix the four closed specimens; each has its own green PR awaiting merge.

**Homonym rule (operator, 2026-08-04):** In this repository *authority* names two unrelated concepts. This note uses explicit qualifiers only:

| term | meaning | lives in |
|---|---|---|
| **fact-home** | the one place a modeled fact may be written (DESIGN §3 single authority) | AUTH-0 subject |
| **permission-to-act** | who may touch what (`std.access`, reach grants) | separate open thread — **not this note** |

Bare *authority* does not appear below.

Frames: DESIGN §2 (derive-global / decompose), §3 (fact-home, external upstream decomposition, cite-the-symbol), §5 (construction before validation; wall-now / wall-after-grounding / ratchet).

---

## 0. Why this exists

Over one day, four independent PRs each invented a **second fact-home** for the same proposition. No author was careless — each had a defensible local reason. That is the signal: diligence cannot be the defence; the relation **kind** must be typed.

| label | relation kind | specimen symbol (grep-verifiable) |
|---|---|---|
| **CIT-0** | one fact at two writable positions | `CitedResource.authority` vs `Pin.subject` / `expected_identity` |
| **CIT-1** | authored where observed was required | same 64-hex digest as `example.test` fixture and cited evidence, no bytes→digest edge |
| **QM** | stored where derived was available | `observed_vs_prediction: Ordering` beside the three counts that determine it |
| **SCAFFOLD** | inferred where identity was required | unique substring match between scaffold `rel_path` and roadmap carrier path |

A topology that collapses these into “do not duplicate” is useless. One that names each relation, states what evidence discharges it, and says which §5 rung applies is the product.

### 0.1 Fifth specimen — homonym at the mandate layer

This mandate was titled *Authority relation topology*. That primed a design note about **permission-to-act** (`std.access`: who is acting, what may this actor touch, may this request proceed) — a competent map of the wrong subject.

The corrected subject is **fact provenance** (DESIGN §3: the one place a fact lives). Producing a whole note against the homonym is itself a §3 violation: one word, two concepts, a second representation at the documentation layer. If a fifth relation kind is needed in the topology, **lexical collision** is it: discharge = explicit qualifier or distinct coinage at every boundary where both senses appear (`fact-home` vs `permission-to-act`).

---

## 1. What fact provenance means

For a given modeled fact, fact provenance answers:

1. **What is the atomic proposition?** (digest, ordering, binding, identity)
2. **Where is its sole fact-home?** (one writable field, derived-only fn, or observation receipt arm)
3. **What evidence discharges membership?** (structural projection, bytes→digest pipeline, total derivation, identity join)

Every specimen is a **wrong answer** to one of those three — not a missing permission check.

**Adjacent, not fact-provenance:**

- **Permission-to-act** — deontic; belongs under the authorization-kernel thread
- **Disposition / scaffold registry** — lifecycle metadata beside a fact, not the fact
- **Containment / naming** (`QualifiedName`, `SymbolIndex`) — feeds identity joins; not duplicate fact-home by itself

---

## 2. Four relation kinds

### 2.1 Duplicate fact-home — one fact, two writable positions

**Question:** Is this one fact duplicated, or two facts whose relationship was never modeled?

**Specimen CIT-0.** `CitedResource.authority: ExternalAuthority` and pin-side identity (`Pin<CitedRepresentation>` with `expected_identity`, or authority duplicated on `Pin.subject` via `RepresentationPin`) were independently writable. `resource-says-A` while `pin-says-B` was a legal substrate state.

**Harm:** silent fork at the meaning layer; reconcile compares the wrong end; grep passes both symbols.

**Discharge evidence:**

- **Relation typing** (`extdeps.pin` `pin_subject_must_not_be_self_identifying_note`): does the subject **already determine its own digest** at the declared grain?
  - **No** (`CliTool`, `ActionRef`, cited web representation): `Pin<Subject>` — `expected_identity` is *new* information; fact-home is the pin carrier.
  - **Yes** (`SccacheBinaryArtifact` at release×arch grain; wrongly modeled `Pin<OciDescriptor>`): `SubjectProjectedPin<Subject>` — subject + selection only; identity and version **projected at the bridge** (`subject_projected_pin_as_pin`), never stored twice.
- **CIT-0 fix pattern** (`std.citation` on `origin/session/cit-0`, not on main): `citation_cit0_note` — `CitedResource.authority` is the **sole** source identity; never duplicated on `Pin.subject` or `ObservedCitedRepresentation`. `CitedRepresentation` names only `RepresentationByteDomain` (which bytes are hashed); `expected Sha256Digest` lives on `Pin`.

**§5 ladder:** wall-now when “subject self-identifies?” is structurally decidable; wall-after-grounding for carriers not yet split (`OciDescriptor` dissolve-on); not a ratchet once grain is named.

**Review tell:** two fields (or carrier + subject) independently editable for digest, `ExternalAuthority`, version, or revision.

---

### 2.2 Observation-required — authored where observed was required

**Question:** What makes a value admissible as evidence rather than assertion?

**Specimen CIT-1.** A real PDF digest was typed by hand with no retrieval or extraction edge from bytes to value. The same 64-hex string was simultaneously an `example.test` fixture locator value and cited evidence — nothing distinguished fabricated example from cited evidence.

**Harm:** specification-without-execution (§5); tests green while the digest is a second definition, not a receipt.

**Discharge evidence:**

- **Observation receipt** with dated downstream fact (`CitationRetrievalObservation`; `RepresentationRetrieved` requires `HttpStatus` + `ObservedCitedRepresentation` — partial success cannot inhabit the representation arm).
- **Bytes→digest pipeline** — digest is output of verify/extract, not an independent authorable string (`sha256_digest_content_hash`, wet adapter minting `observed_identity`).
- **Namespace separation** — fixture locator (`example.test/fixture.pdf`) vs cited URI (`extdeps.standards.rfc_8118` anchor) are different fact-homes for different propositions.
- **§5 oracle rule (2026-08-01):** merge-blocking population compare cannot use tree-copied literals as oracle.

**§5 ladder:** wall-now for digest literal without observation arm; wall-after-grounding where `ContentHash` family typing is incomplete; external bytes always boundary-observed.

**Review tell:** same hex in a `data` row and an evidence path with no `RecordedFact` / retrieval / verify arm between them.

**Misclassification flag:** weak `expected_hash: NonEmptyStr` sites (census §4.1) may be **typing** failure (need `Sha256Digest`) rather than observation-class — per-site read required.

---

### 2.3 Derivation-only — stored where derived was available

**Question:** When may a computable fact be stored at all?

**Specimen QM.** `observed_vs_prediction: Ordering` was stored on `PathAdditivityRefutationReceipt` (and related witness records) alongside `observed`, `prediction`, and the comparison inputs. The constructor accepted the triple independently; `observed: 5`, `prediction: 20`, `observed_vs_prediction: Greater` was **writable and passed**.

**Harm:** redundant field becomes an independent fact-home; editors satisfy checks by editing the derived field while inputs stay wrong (§5 “edit the declaration while the realization lies”).

**Discharge evidence:**

- **Delete stored field**; expose total fn e.g. `path_additivity_witness_observed_vs_prediction()` derived from `path_additivity_compare_observed_to_prediction` (QM branch `c200375c0c6`).
- **Algebraic refusal** — inconsistent triple unwritable when ordering is `fn(observed, prediction)`, not a field.

**§5 ladder:** wall-now for closed total fn over modeled fields; wall-after-grounding when operands are observation receipts; ratchet only if the summary is genuinely undecidable (not the case for numeric ordering).

**Review tell:** field names `*_vs_*`, `*_ordering`, `*_comparison` beside operands they summarize.

**Misclassification flag:** if the stored value were an **operator policy choice** (preference, not a function of counts), storing it would be honest — QM is derivation-class because the field *claimed* to summarize numeric comparison.

---

### 2.4 Identity join — inferred where identity was required

**Question:** What distinguishes a proposal from a binding?

**Specimen SCAFFOLD.** A unique substring match between a scaffold disposition site’s `rel_path` and a roadmap carrier path became a durable obligation binding. Uniqueness is not identity.

**Harm:** heuristic promoted to fact-home; binding survives renames that break the accidental substring; zero/plural matches silent or fabricated.

**Discharge evidence:**

- **Identity join** on declared key (`RoadmapNodeIdentity`, `record_identity`, `roadmap_identity_registry`) — not path substring.
- **Count discipline** (`gunbc.scaffold_retirement` `ec85b225782`): exactly one → binding; zero → `NoHonestObligationOwner`; plural → `AmbiguousObligationOwners`.
- **Derived population** — `scaffold_disposition_sites_live()` structural walk, not hand roster (`scaffold_retirement_pilot_derivation_note`).

**§5 ladder:** wall-now for closed registered join keys; wall-after-grounding for incomplete registry; ratchet forever for undeclared path similarity.

**Review tell:** `string_contains`, substring match, “unique match” without `DeclarationRef` / `RoadmapNodeIdentity`.

**Misclassification flag:** `scaffold_digest_for_module_path` in emitter provenance may be honest interim scaffold (dissolve trigger), not identity-class — it does not claim obligation ownership.

---

## 3. Worked precedent — `pin_subject_must_not_be_self_identifying_note`

This is the corpus’s closest **relation-typing** procedure — the pattern AUTH-0 generalizes.

It does not ask “is this a pin?” It asks: **does the subject already determine its own digest at this grain?**

| answer | carrier | fact-home for identity |
|---|---|---|
| no | `Pin<Subject>` | `expected_identity` on pin (new information) |
| yes | `SubjectProjectedPin<Subject>` | projected at bridge only — duplication unwritable |

`Pin<OciDescriptor>` copying `descriptor.digest` into `expected_identity` was case “yes” wrongly modeled as case “no” — the CIT-0 class at pin grain.

Generalization: before adding a field, name the relation kind (§2.1–§2.4) and apply the matching discharge. “Do not duplicate” is not a procedure; “does the subject self-identify?” **is**.

---

## 4. Census methodology (operator ruling, 2026-08-04)

1. Refuse new instances in active lanes — **done** for four specimens.
2. **Exact live censuses** from corpus (structural walks / measured grep — inputs, not conclusions).
3. Decide whether one shared construction replaces each class (per §5 decidability).
4. No-growth frontier — derived enrollment only.

AUTH-0 stops at 2–3 as candidates.

---

## 5. Candidate census populations (measured on `main`, 2026-08-04)

Operator surfaced three candidates while reviewing other work. **Not proven single classes.**

### 5.1 Weak digest parameters

**Pattern:** `expected_hash: NonEmptyStr` (and similar) where `std.content_hash` exists.

**Measured:** **12** sites across **7** modules (e.g. `ubuntu_install_media_fetch.observe_install_media_fetch`, `ubuntu_seeded_install_media_remaster.install_media_remaster_sidecar_matches`).

**Open:** partition into observation-required vs serialization-boundary vs scaffold-placeholder before construction.

### 5.2 `*Readiness` type surface

**Operator candidate:** ~17 coproducts, N-types-one-axis shape (like pin dimension correction).

**Measured:** **10** distinct top-level type names ending in `Readiness` (`ToolReadiness`, `ProviderReadiness`, `WitnessBinReadiness`, …); many are records/claim specializations, not coproducts.

**Open:** structural coproduct census — name suffix insufficient; may be legitimately distinct upstream shapes per §3 decomposition.

### 5.3 `concat(acc, …)` accumulator sites

**Operator candidate:** ~92 sites; quadratic cost if list `concat` copies — **copy behavior not verified**.

**Measured:** **274** `concat(acc,` across **119** files; **117** `concat(acc, [`.

**Open:** narrow pattern to match operator’s ~92; measure cost shape before rewrite (`accumulator_copy_roster_gate` machinery exists).

---

## 6. Relation kind → construction sketch (AUTH-1+, unsigned)

| kind | construction direction | residue |
|---|---|---|
| Duplicate fact-home | single writable home + projection bridge | cross-family compare until `ContentHash` wall complete |
| Observation-required | typed observation coproduct + layer gate | host census until witness realization |
| Derivation-only | delete field; `fn *_derived(...)` | honest `Unknown` for undecidable summaries |
| Identity join | join on `RoadmapNodeIdentity`; 0/plural → exclusion | path similarity never auto-binds |
| Lexical collision | qualify or rename at every boundary | until cited-symbol lens retires positional homonyms in prose |

**Non-goals:** no new `std/` module, no lens enrollment, no edits to four specimen PRs.

---

## 7. What this note does not do

- **Permission-to-act map** (`std.access`, `AuthScope`, publication ontology) — competent subject, wrong mandate; if preserved, belongs under the existing authorization-kernel open thread as a **separate** proposal, not AUTH-0.
- **Fix the four specimens** — closed in their lanes.
- **Hand-authored instance roster** — censuses are derived only.

---

## 8. Dissolution

Dissolves when each relation kind has a named construction wall or honest ratchet in the guarantee recovery ledger, and §5 candidate censuses are promoted to enrolled derived populations with RED controls or refuted with measured negative receipts.

Until then, AUTH-0 is the reference for **which fact-provenance relation**, not **which module**.
