# Fact provenance topology — AUTH-0 design note

Status: **DRAFT — operator sign-off on direction (loyal-ant-382, 2026-08-04).** Local to session `cool-ram-632`; no PR open. Design-note only — **no hub, no construction, no frontier** without further sign-off.

**Homonym rule (operator, 2026-08-04):** *Authority* names two unrelated concepts. This note uses explicit qualifiers only:

| term | meaning | lives in |
|---|---|---|
| **fact-home** | the one place a modeled fact may be written (DESIGN §3 single authority) | AUTH-0 subject |
| **permission-to-act** | who may touch what (`std.access`, reach grants) | separate open thread — **not this note** |

Bare *authority* does not appear below.

Frames: DESIGN §2, §3 (fact-home, external upstream decomposition, cite-the-symbol — **grep the concept, not the name**), §5 (construction before validation).

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

### 0.1 Specimens at the mandate layer (not PR lanes)

| label | failure | specimen |
|---|---|---|
| **HOMONYM** | lexical collision | work item title primed permission-to-act note; PR #7789 closed |
| **CENSUS** | observation without population/method | `concat(acc, [` grep under-selected (117/204); PR #7779 body “No concat(acc, [x])” true under wrong pattern, false about `concat(wires, [wire])` in `rfc_8118.dag` — same accumulator-copy shape, different binder spelling |

Both are §3 “grep the name, not the concept” at the measurement layer — precise-looking numbers measuring an accidental lexical property.

---

## 1. Specimen record (closed PR lanes — do not touch)

Four relation kinds, four specimens. Operator framing (relation, not fix):

| label | relation kind | what went wrong | discharge (specimen branch) |
|---|---|---|---|
| **CIT-0** | duplicate fact-home, **unjoined** | source and representation identity deduplicated but left **unjoined** — `CitedResource.authority` vs `Pin.expected_identity` writable independently | `std.citation` `citation_cit0_note`; `Pin<CitedRepresentation>` per `pin_subject_must_not_be_self_identifying_note` |
| **CIT-1** | observation-required | authored digest **named as observation**, then made **its own expectation** — same 64-hex as `example.test` fixture; no bytes→digest edge | fixture vs cited URI separated; observation coproduct (`origin/session/cit-1`) |
| **QM** | derivation-only | held/undecided outcomes **convertible into proof-shaped receipts** — `observed_vs_prediction: Ordering` stored beside counts; inconsistent triple passed | field deleted; `path_additivity_witness_observed_vs_prediction()` derived (`c200375c0c6`) |
| **SCAFFOLD** | identity join | scaffold **arity and fixture names** standing in for **exact variant identity** — substring `rel_path` match → binding | `RoadmapNodeIdentity` join; derived `scaffold_disposition_sites_live()` (`ec85b225782`) |

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

**Derived population only** — no hand roster. **Every observation carries:**

| field | meaning |
|---|---|
| **population** | what was counted (reproducible query or walk) |
| **revision** | corrected counts when method was wrong (stated, not silent) |
| **method** | exact derivation — text pattern **or** structural `Node`-tree walk |
| **maximum conclusion** | what the census can establish — and what it **cannot** |

A census closes step 2 when each row carries enough to decide step 3. **Numbers alone do not close it.**

**Standing rule (operator, 2026-08-04):** grep the **concept**, not the name — binder spelling (`acc` vs `wires`), type-name suffix (`*Readiness` matching `ReadinessLayer`), and glob scope (`src/v1/**/*.dag` missing root files) are all the same defect class: accidental lexical property standing in for structure.

Measured on `main`, 2026-08-04, unless noted.

---

## 5. Live censuses (step 2)

### 5.1 Weak digest parameters

| | |
|---|---|
| **method** | `rg 'expected_hash: NonEmptyStr|expected_digest: NonEmptyStr|kernel_digest: NonEmptyStr|digest: NonEmptyStr|content_hash: NonEmptyStr' --glob '*.dag'` |
| **population** | **10** writable sites / **5** modules |
| **revision** | operator prior “12 / 7” superseded by this query on current `main` |
| **maximum conclusion** | per-site partition candidate below — **not** a single class |

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

| | |
|---|---|
| **method** | `rg '^type [A-Za-z]*Readiness' --glob '*.dag'`; drop prefix false positives (`ReadinessLayer`, `ClaimReadinessPolicy`, …) |
| **population** | **10** distinct top-level type names ending in `Readiness` |
| **revision** | operator “~17” was prefix-match artifact (`ReadinessLayer` ∈ `*Readiness`) |
| **maximum conclusion** | per-name kind + arms + interchangeability — **no shared axis found**; does **not** establish one `Readiness<Domain>` parameter |

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

### 5.3 Fold accumulator `concat(<binder>, [x])` sites

**Concept:** a fold whose accumulator is rebuilt by appending a single element via `concat`.

#### Text-pattern floor (current method — under-selects)

| query | sites | files | maximum conclusion |
|---|---|---|---|
| `concat(acc, [` — **binder name `acc` only** | **117** | **59** | **under-selected** — measures who named the variable, not the shape (57% of floor below) |
| `concat(<ident>, [` — any binder, single-element list rhs | **204** | **88** | **text-pattern floor** — still not structural; misses shapes not spelled as `concat(binder, [elem])` in source text |

**Revision history:**

- v1: `concat(acc, [` only — **wrong pattern** (operator + agent, 2026-08-04)
- v2: `concat(<ident>, [` — operator **204** sites; measured **205** on current `main` (1-site drift)

**Binder distribution (v2 query, top):** `acc` 117 · `result` 8 · `base` 6 · `witness_layer_roots` 5 · `visited` 5 · `inner` 5 · `causes` 5 · `rows` 4 · `existing` 4 · `arg_diags` 4 · `import_parents` 3 · tail of 1–2 each (`wires`, `xs`, …).

**CENSUS specimen surface:** PR #7779 claims “No concat(acc, [x])” — **true** under the v1 pattern, **false** about the code: `rfc_8118.dag` carries `concat(wires, [wire])` (accumulator-copy shape, different binder). The v1 census handed an author a way to be **honestly wrong**.

#### Structural population (typed gap — not yet derived)

**Target:** `Node`-tree property — fold body where callee is `concat`, accumulator binder is the fold accumulator, rhs is a single-element list literal. Decidable from the tree (precedent: `scaffold_disposition_sites_live()` structural walk over witness roots).

**Gap:** no enrolled builtin or lens walks fold bodies for this pattern corpus-wide today. **Do not** widen regex and call it exact.

**Cost-shape pointer (out of AUTH-0 scope):** interpreter vs emitted-Rust realization differ (`im::Vector` O(log n) concat in `v1_rt.rs` vs `std::vec::Vec` materialize+extend in `v1_interpreter.rs`); instrument = `list_concat_items_copied` counter (`v1_interpreter.rs`, asserted in `interp_stats_test.rs`). DESIGN §6 / `complexity_accumulator_copy` lane — run counter on structural population when available.

**Not a fact-provenance class** — population + method discipline only in this note.

---

## 6. What this note does not do

- Permission-to-act map — wrong mandate; not extracted from #7789
- Fix the four specimens — closed in their lanes
- Step 3 construction, `std/` hub, lens enrollment, no-growth frontier
- Route step-3 decisions to the operator
- **Cost-shape pricing for fold-`concat`** — text-pattern floor in §5.3 only; structural population + `list_concat_items_copied` run cited out to DESIGN §6

---

## 7. Dissolution

Dissolves when: (a) each relation kind has a construction wall or honest ratchet row; (b) censuses in §5 are enrolled as derived populations with RED controls or refuted by measured negative receipts. Until then, AUTH-0 is the reference for **which fact-provenance relation**, not **which module**.
