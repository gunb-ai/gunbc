# Fact provenance topology — AUTH-0 design note

Status: **DRAFT — operator sign-off on direction (loyal-ant-382, 2026-08-04).** Local to session `cool-ram-632`; no PR open. Design-note only — **no hub, no construction, no frontier** without further sign-off.

**Homonym rule (operator, 2026-08-04):** *Authority* names two unrelated concepts. This note uses explicit qualifiers only:

| term | meaning | lives in |
|---|---|---|
| **fact-home** | the one place a modeled fact may be written (DESIGN §3 single authority) | AUTH-0 subject |
| **permission-to-act** | who may touch what (`std.access`, reach grants) | separate open thread — **not this note** |

Bare *authority* does not appear below.

Frames: DESIGN §2, §3 (fact-home, external upstream decomposition, cite-the-symbol), §5 (construction before validation), §6 (bare-minimum cost — cited only where copy semantics are now verified).

**Sequence (operator, 2026-08-04):**

1. Refuse new instances in the four specimen lanes — **done**
2. **Exact live censuses** — this note (§5)
3. Decide whether shared construction replaces each class — **deferred** (not in scope)
4. No-growth frontier — **deferred**

Step-3 decisions (Readiness axis, digest partition vs `ContentHash` wall, hub vs notes) are **not** routed to the operator here — they are what the census is for. Standing answers that hold regardless: **second-consumer discipline** → notes and lenses until two unrelated consumers exist (four specimens are not hub consumers); **pinning** → `pin_dimension_note` — `Pin<CitedRepresentation>` is an instantiation like `Pin<CliTool>`, not a sibling `extdeps.citation.pin` type.

---

## 0. Why this exists

Over one day, four independent PRs each invented a **second fact-home** for the same proposition. No author was careless — each had a defensible local reason. That is the signal: diligence cannot be the defence; the relation **kind** must be typed.

A topology that collapses these into “do not duplicate” is useless. One that names each relation, states what evidence discharges it, and says which §5 rung applies is the product.

### 0.1 Fifth specimen — homonym at the mandate layer

The work item was titled *Authority relation topology*, priming a note about **permission-to-act** (`std.access`). The corrected subject is **fact provenance** (DESIGN §3: the one place a fact lives). Producing a whole note against the homonym is itself a §3 violation at the documentation layer. Discharge: explicit qualifier (`fact-home` vs `permission-to-act`) or distinct coinage at every boundary. PR #7789 closed; access-kernel map **not** extracted (would carry the mis-priming).

---

## 1. Specimen record (closed lanes — do not touch)

Four relation kinds, four specimens. Each PR is green in its own lane.

| label | relation kind | specimen (grep-verifiable) | discharge (landed on specimen branch) |
|---|---|---|---|
| **CIT-0** | duplicate fact-home | `CitedResource.authority` vs `Pin.expected_identity` / `RepresentationPin` fork — `resource-says-A` while `pin-says-B` writable | `std.citation` `citation_cit0_note`: authority sole on `CitedResource`; `Pin<CitedRepresentation>` per `pin_subject_must_not_be_self_identifying_note` case (1) |
| **CIT-1** | observation-required | real PDF digest hand-typed; same 64-hex as `example.test` fixture and cited evidence; no bytes→digest edge | fixture locator separated from cited URI; `Pin<CitedRepresentation>` + observation coproduct (`origin/session/cit-1`) |
| **QM** | derivation-only | `observed_vs_prediction: Ordering` on `PathAdditivityRefutationReceipt` beside counts; `observed: 5`, `prediction: 20`, `observed_vs_prediction: Greater` writable and passed | field deleted; `path_additivity_witness_observed_vs_prediction()` derived (`c200375c0c6`) |
| **SCAFFOLD** | identity join | unique substring match between scaffold `rel_path` and roadmap path → durable binding | `RoadmapNodeIdentity` join; 0 → `NoHonestObligationOwner`; plural → `AmbiguousObligationOwners`; `scaffold_disposition_sites_live()` derived population (`ec85b225782`) |

---

## 2. Relation kinds (topology)

### 2.1 Duplicate fact-home

**Question:** one fact duplicated, or two facts whose relationship was never modeled?

**Discharge:** `pin_subject_must_not_be_self_identifying_note` — does the subject already determine its own digest at this grain?

| answer | carrier | fact-home for identity |
|---|---|---|
| no | `Pin<Subject>` | `expected_identity` on pin |
| yes | `SubjectProjectedPin<Subject>` | projected at bridge only (`subject_projected_pin_as_pin`) |

`Pin<OciDescriptor>` copying `descriptor.digest` into `expected_identity` was “yes” modeled as “no”.

### 2.2 Observation-required

**Question:** what makes a value admissible as evidence rather than assertion?

**Discharge:** observation receipt (`CitationRetrievalObservation`); bytes→digest pipeline; fixture vs cited namespace separation; §5 oracle rule (2026-08-01).

### 2.3 Derivation-only

**Question:** when may a computable fact be stored?

**Discharge:** delete stored summary; total `fn` over inputs; algebraic refusal of inconsistent triples.

### 2.4 Identity join

**Question:** what distinguishes a proposal from a binding?

**Discharge:** join on declared key (`RoadmapNodeIdentity`); 0/plural → typed exclusion; derived population walk — never substring uniqueness.

---

## 3. Worked precedent — `pin_subject_must_not_be_self_identifying_note`

Relation-typing procedure AUTH-0 generalizes. Before adding a field: name the relation kind (§2), then apply discharge. “Do not duplicate” is not a procedure; “does the subject self-identify?” is.

---

## 4. Census methodology

**Derived population only** — no hand roster. A census closes step 2 when:

- population is mechanically reproducible (query + count documented), and
- each row carries enough to decide step 3 (kind, arms, interchangeability, claimed proposition, copy semantics — not counts alone).

Measured on `main`, 2026-08-04, unless noted.

---

## 5. Live censuses (step 2)

### 5.1 Weak digest parameters

**Derivation query:** `rg 'expected_hash: NonEmptyStr|expected_digest: NonEmptyStr|kernel_digest: NonEmptyStr|digest: NonEmptyStr|content_hash: NonEmptyStr' --glob '*.dag'`

**Population:** **10** writable sites across **5** modules (operator prior “12 sites / 7 modules” superseded by this query on current `main`).

| module | site | kind | what the digest claims to prove | step-3 partition |
|---|---|---|---|---|
| `extdeps.tools.sha256sum` | `sha256sum_verify_via_shell(..., expected_digest: NonEmptyStr)` | fn param | external SHA-256 match of file bytes (NIST FIPS 180-4 via `sha256_algorithm_authority`) | **observation boundary** — param is wire hex into verify fn; candidate `Sha256Digest` typing, not second fact-home |
| `extdeps.provisioning.ubuntu_install_media_fetch` | `observe_install_media_fetch`, `install_media_fetch_try_mirror`, `install_media_fetch_try_mirrors` params | fn param | Canonical published SHA-256 for install media (`install_media_fetch_hash_convergence_note` — cryptographic authority digest, **not** `std.content_hash` FNV structural) | **observation-required at verify arm** — digest originates from `UbuntuInstallMediaArtifactRow.content_sha256` cited row, verified via `sha256sum_verify_via_shell` |
| `extdeps.provisioning.ubuntu_seeded_install_media_remaster` | `install_media_remaster_sidecar_matches(..., expected_hash)` | fn param | sidecar matches remastered artifact | same family as ubuntu_fetch |
| `extdeps.provisioning.ubuntu_seeded_install_media_remaster` | `content_hash: NonEmptyStr` field | record field | stored sidecar digest | **duplicate fact-home candidate** if row also carries bytes path without derivation edge — needs row read |
| `extdeps.boot.linux_x86_boot` | `LinuxX86BootImage.kernel_digest: NonEmptyStr` | record field | kernel image identity for boot bundle | **scaffold / placeholder** until emit proof wires bytes→digest |
| `extdeps.boot.freestanding_payload` | `EmittedPayload` / `PrebuiltPayload` / `HollowPrebuiltInit` `.digest` | variant fields | payload ELF identity | **scaffold** — `HollowPrebuiltInit` explicitly refused at `accept_linux_x86_boot_image`; digest is authorable placeholder pending emit |

**Operator correction recorded:** weak-digest class is **unpartitioned** — step 3 must not assume one `ContentHash` wall covers cryptographic authority digests (`install_media_fetch_hash_convergence_note` explicitly separates them).

---

### 5.2 `*Readiness` type surface

**Derivation query:** `rg '^type [A-Za-z]*Readiness' --glob '*.dag'` then drop prefix-match false positives (`ReadinessLayer`, `ReadinessFoldState`, `ClaimReadinessPolicy`, `ToolReadinessRefusal`, …).

**Population:** **10** distinct top-level type names ending in `Readiness`.

**Operator correction recorded:** prior “~17 coproducts” was prefix-match artifact (`ReadinessLayer` matches `*Readiness`). The “N types on one axis” claim is **unverified** — needs kind + arms + interchangeability per name (below).

| type | module | kind | arms (summary) | interchangeability |
|---|---|---|---|---|
| `ToolReadiness` | `gunbc.tool_readiness` | coproduct (3) | `PinnedAdmitted`, `PinnedRefused`, `UnpinnedFrontier` | **none** — pin membership + `admit_pin_integrity` conjunction; domain = `CliTool` |
| `ProviderReadiness` | `extdeps.llm.cli_lifecycle` | coproduct (12) | auth/setup/quota/rate-limit/… verdicts over `ProviderInstance` | **none** — upstream CLI lifecycle facts per `cli_lifecycle_fact_separation_note` |
| `ToolExecutionReadiness` | `extdeps.llm.cli_lifecycle` | coproduct (5) | sandbox probe verdicts | **none** — orthogonal axis to `ProviderReadiness` (same module, explicit separation note) |
| `WitnessBinReadiness` | `tools.host_prelude` | coproduct (2) | `WitnessBinReady`, `WitnessBinRefused` | **none** — witness-binary artifact probe |
| `CompilePoolReadiness` | `gunbc.host_compile_pool` | coproduct (2) | `CompilePoolReady`, `CompilePoolNotReady` | **none** — cgroup slice vs derived pool size |
| `DashboardProviderReadiness` | `gunbc.roadmap_dashboard_instance_apply` | coproduct (5) | `ProviderReady`, path/runtime/auth/refused arms | **none** — dashboard SSH observation fold; distinct from `extdeps.llm` `ProviderReadiness` (homonym, different module) |
| `LandingEvidenceReadiness` | `gunbc.source_integration_landing_spine` | coproduct (2) | `LandingEvidenceReady`, `LandingEvidenceBlocking` | **none** — landing claim-evidence specialization |
| `MergeReadinessVerdict` | `gunbc.pr_digests` | coproduct (2) | `Ready`, `NotReady` | **none** — PR merge digest gate |
| `DigestReadinessProjection` | `gunbc.digest_render` | coproduct (2) | `DigestOpenForReview`, `DigestNotReady` | **none** — digest render projection |

**Interchangeability check:** no function accepts a generic `Readiness` parameter; no shared fold across these types. `ProviderReadiness` name is shared between `extdeps.llm.cli_lifecycle` (type) and `gunbc.provider_readiness_claim_evidence` (claim value parameter referencing the extdeps type) — **same type, not two axes**.

**Step-3 verdict (census output):** ten unrelated domain verdict types sharing a naming suffix — **no evidence of one `Readiness<Domain>` axis**. Generic machinery already exists at `std.claim_evidence` (`ClaimReadinessReceipt`, `ClaimRequirementReadiness`) for claim-indexed evidence; these 10 are domain instantiations, not duplicates of that axis.

---

### 5.3 `concat(acc, [x])` accumulator sites

**Derivation query:** `rg 'concat\(acc, \[' --glob '*.dag'`

**Population (operator reconciliation recorded):**

| scope | sites | files |
|---|---|---|
| `dag/` only | **92** | **44** |
| `src/v2/` only | **20** | **11** |
| `src/v1/` only | **5** | **4** |
| **whole repo** | **117** | **59** |

Operator reconciliation: prior `src/v1` count of **3** used `src/v1/**/*.dag`, missing `src/v1/04_resolve.dag` and `src/v1/coercion.dag` at the tree root — same scoping-artifact class as the Readiness prefix-match inflation. **92 + 20 + 5 = 117** reconciles exactly.

**Looser** `concat(acc,` (any rhs): **276** sites / **133** files on `main` (prior agent count 274 — **2-site delta is `src/v1` only**).

**Fold context (derived):** **104 / 117** strict-pattern sites occur within ~20 lines of a `fold(` or `f: (acc` binder.

**Cost-shape mechanism (corrected — not fact-provenance):** the population count is step-2; pricing belongs elsewhere.

| realization | `Vec` carrier | fold `concat(acc, [x])` cost shape |
|---|---|---|
| **Emitted Rust** (`v1_rt.rs`) | `im::Vector` aliased as `Vec` (`Cargo.toml`: RRB tree, O(log n) concatenation) | `list_concat` / `append` use structural sharing — **not** linear accumulator copy per step |
| **Interpreter** (`v1_interpreter.rs`) | `std::vec::Vec` — imports `im::HashMap` / `im::OrdSet` only, **not** `im::Vector` | `free_monoid_to_vec` materializes both operands; `BinOp` concat arm extends `a` with `b` — **linear copy of accumulator per fold step** → quadratic in fold length |

Prior claim that “concat copies” full-stop was wrong: it conflated the two realizations. The CI floor runs **interpreted**, so the bite is on the interpreter path — where corpus-denominated timeouts live — not because emitted Rust is broken.

**Instrument (already in tree, not yet run on this population):** `v1_interpreter.rs` `MutationCounters.list_concat_items_copied` (incremented at the `BinOp` concat arm, summed in the mutation report ~line 1130); `interp_stats_test.rs` asserts counter behavior. A corpus run yields a **number**, not an adjective — step 3 for this class.

**AUTH-0 pointer only:** DESIGN §6 cost-shape lane; enrolled debt via `v2.lens.complexity_accumulator_copy.roster_gate` (`accumulator_copy_roster_gate_std_test.dag`). No further concat analysis in this note.

---

## 6. What this note does not do

- Permission-to-act map — wrong mandate; not extracted from #7789
- Fix the four specimens — closed in their lanes
- Step 3 construction, `std/` hub, lens enrollment, no-growth frontier
- Route step-3 decisions to the operator
- **`concat(acc, [` cost-shape pricing** — population in §5.3 only; mechanism and `list_concat_items_copied` instrument cited out to DESIGN §6 / `complexity_accumulator_copy` lane

---

## 7. Dissolution

Dissolves when: (a) each relation kind has a construction wall or honest ratchet row; (b) censuses in §5 are enrolled as derived populations with RED controls or refuted by measured negative receipts. Until then, AUTH-0 is the reference for **which fact-provenance relation**, not **which module**.
