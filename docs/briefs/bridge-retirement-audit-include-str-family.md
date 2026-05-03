# Bridge Retirement Audit — `include_str!` / Source-Text Patching Family (v3)

**Status:** Audit packet for R3 B4 (`include_str!` / patching lane).  
**Anchor:** `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` row `B4 bridge-retirement queue`.  
**Scope:** Enumerate v3 sites that (a) embed canonical structural authority through
compile-time `include_str!`, (b) splice or patch textual artifacts (fixtures,
generated Rust, emitter sources) instead of reading typed substrate, or (c)
document rejected side channels. **Out of scope:** implementing retirements
(owned by Verification / fierce-ferret-556 #1276 per-row PRs).

**Method:** `rg 'include_str!\\(' src/v3` at repo HEAD (≈120 macro invocations
across 38 Rust files) plus manual review of `src/v3/compiler/build.rs` splicing
helpers and `bootstrap.rs` structural patch entry.

**Related briefs:** `docs/briefs/r3-v-bridge-retirement-ledger-zero-audit.md`
(ledger rows 3b–5, 4); `docs/briefs/r2-pb-canonical-lens-bridge-disposition.md`;
`docs/briefs/r2-evaluator-test-runner-authority-ratchet.md` (SG-0 / infer
helpers text mining).

---

## Family Definition (what counts as a “bridge entry”)

| Class | Included here | Typical retirement |
| --- | --- | --- |
| A | `include_str!` of **canonical** `.dag`, `.md`, `.rs`, or `.txt` that duplicates an authority also represented structurally elsewhere | Typed accessor, `DeclarationRef`, codegen from substrate, or single regen producer |
| B | `include_str!` of **checked-in generated** `*_generated.rs` for drift vs tokenizer/parser/emitter | Parser/AST witness or generator API that tests consume structurally |
| C | **Build-time text splice / patch** (not necessarily `include_str!`) into fixtures or generated modules | Structural `TestClaim` carriers, no sentinel replace |
| D | **Post-parse Dag mutation** (“patching” typed graph, not source bytes) | Express edge in source `.dag`; delete host patch |
| E | Hermetic **test fixture** `include_str!` whose bytes are *only* scenario input | Not a retirement target for this family; appendix only |

---

## STOP+PING — SourceSpan / File-Participation Boundary (royal-newt-846)

The following audited entries **overlap** the sibling slice (ledger row #1
`bridge_source_span_file_participation_retired`, `reflect_program_dag_nodes_in_file`
/ fold carriers, emit `source_filtering`, etc.). **Do not retire in isolation**
without coordinating `royal-newt-846`.

| Entry id | Why it touches the sibling slice |
| --- | --- |
| BR-18 | `r2_b5_loop_construction_closure_test` embeds `lower.rs` and `builder.rs` as **raw source text** for substring / structure proofs — parallel to “read compiler source via file identity” bridges. |
| BR-07 / BR-08 | Canonical lens bytes in `test_runner` and the user-authored lens gate test feed `reflect_program_dag_nodes_in_file` / fold paths that key on **logical file names** today; dissolution text in `test_runner.rs` names `DeclarationRef` / program substrate. |
| BR-19 | `patch_kernel_bool_boolean_algebra_inhabits` keys `Bool` via `Declaration.span.file == "dsl/std/types.dag"` and attaches `Diagnostic` with `SourceSpan`. |
| BR-16 | Shape-A spec `include_str!` cluster sits next to emit `source_filtering` / file-participation semantics — coordinate if retirement touches filtering authority. |
| BR-A (appendix) | Many integration tests call `compile_to_dag(source, virtual_path)` where `virtual_path` participates in diagnostics / extraction policy — same ledger row even when the bytes come from `include_str!` vs inline strings. |

---

## Retired / Ratchet-Only Slices (context, not open queue)

| Item | Notes |
| --- | --- |
| `patch_lower` + `_helpers` contiguous substring | Retired class; enforced by `src/v3/compiler/tests/integration/bridge_lower_helpers_patch_zero_residual_test.rs` (PR #1014). Not an `include_str!` bridge but same **exact-string patching** family historically. |

---

## Bridge Inventory (numbered entries)

### Wave guide — leaf-first retirement order

Retire **leaves** (no other open bridge *depends on* the same textual channel)
before **roots**. Numeric order below is the recommended PR sequence; “blocked
by” lists **family** prerequisites, not every test import.

---

#### BR-01 — Grounding emission diagnostic mirrors

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/grounding_tests/src/emission_diagnostic_lockstep.rs` — `MIRROR_*` consts (`include_str!` of three lane `diagnostic.rs` files). |
| **Consumers** | **1** module (internal tests only; no `pub` re-export). |
| **Retirement shape** | `.dag → Rust enum` codegen for `EmissionDiagnostic` from substrate `diagnostics.dag`, deleting line/brace scraper + `include_str!` mirrors (module doc already states this). |
| **Sibling blocker** | None within `include_str!` family. |
| **Order** | **1** (leaf; grounding-only). |

---

#### BR-02 — R2 closure ledger markdown gate

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/grounding_cross_target_meta/src/closure_ledger_gate.rs` — `R2_CLOSURE_LEDGER` = `include_str!("…/docs/r2-closure-ledger.md")`. |
| **Consumers** | **1** gate (`contains` + `parse_l6_missing_keys_between_markers` on markdown text). |
| **Retirement shape** | Structural L6 carrier in substrate or generated table consumed via `Dag` / declaration refs instead of markdown scraping. |
| **Sibling blocker** | None. |
| **Order** | **2** (leaf; parallel pattern to BR-01). |

---

#### BR-03 — Grounding pilot Go primitives authority

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/grounding_pilot/src/lib.rs` (~L697) — `include_str!("…/dsl/extdeps/languages/go/primitives.dag")`. |
| **Consumers** | **1** pilot code path (compile / analysis entry). |
| **Retirement shape** | Typed extdep fixture load through the same staged-bootstrap table as the compiler (`EXTDEPS_FILES` / virtual path keys) instead of a second compile-time copy. |
| **Sibling blocker** | **BR-05** meta (`build.rs` extdeps enumeration) until a single authority enumerates extdep paths. |
| **Order** | **15** (after staged-file enumeration is unified). |

---

#### BR-04 — `bootstrap_regen_fresh` hand-listed `dsl/std/*.dag` bundle

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/src/bootstrap_regen_fresh.rs` — 14× `include_str!("../../../../dsl/std/…")` consts (`LOGIC_DAG`, `BIT_DAG`, … `METHODS_DAG`). |
| **Consumers** | **1** regen entry (`compile_std_bootstrap_dag` / `load_fixtures` path; feature `bootstrap-regen-fresh` only). |
| **Retirement shape** | Single iterator over `OUT_DIR` / `STAGED_FILES` + `dsl_std` entries already emitted by `build.rs`; delete parallel hand list (module header names PB-Bootstrap-Process replacement). |
| **Sibling blocker** | **BR-05** (generated static tables are the replacement channel). |
| **Order** | **13** (after BR-05 design lands or in same PR as BR-05 chunk). |

---

#### BR-05 — `build.rs` generated `include_str!` static tables

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/build.rs` — `generate_static` emits `pub static …: &[(&str, &str)]` rows of `(virtual_path, include_str!(abs_path))` for std/spec/compiler/extdeps/gunbc/dsl_std trees. |
| **Consumers** | Included into `OUT_DIR/*.rs`, consumed by `bootstrap_regen_fresh` (`include!`) and bootstrap load paths — **wide** (entire v3 bootstrap surface). |
| **Retirement shape** | Declared `bootstrap.dag` (or equivalent) as sole enumeration authority; regen emits one typed loader without embedding every file as a string literal (ledger row `bridge_include_str_side_channels_retired` direction). |
| **Sibling blocker** | PB-1-e / T-Ground-LanguageSpec process decisions; touches same rows as BR-04. |
| **Order** | **12** (root for many A-class bridges; split PRs by subtree if needed). |

---

#### BR-06 — R1 gates fixture splice (`build.rs`)

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/build.rs` — `emit_r1_gates_fixture` reads `r1_gates.template.dag`, `named_function_count.dag`, `.v3` fixtures; **`replace`** on sentinels `R1_*_SPLICE_V1`; writes `tests/fixtures/r1_gates.dag`. |
| **Consumers** | **1** generated fixture file; many integration tests consume `r1_gates.dag` (not all through `include_str!` — often `compile_file` / path). |
| **Retirement shape** | Structural `TestClaim` / declaration ref so lens body is not duplicated inside escaped string literals; or single generator owned by substrate. |
| **Sibling blocker** | **BR-07**, **BR-08** (canonical lens bytes must stay consistent across build splice, `test_runner` const, and user gate test). Optional alignment: **BR-17** (`lens_apply` tests also `include_str!` the same lens). |
| **Order** | **11** (bundle with canonical-lens retirement wave). |

---

#### BR-07 — `test_runner` canonical lens public consts

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/src/test_runner.rs` — `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS`, `R1_CANONICAL_COMPLEXITY_LENS` (`include_str!` via `concat!(env!("CARGO_MANIFEST_DIR"), …)`). |
| **Consumers** | **Internal:** runner compile/eval paths (e.g. cost / named_function_count claims). **External:** `m1_5_user_authored_lens_gate_test.rs` (import + byte equality vs on-disk lens). **Ratchet:** `canonical_lens_bridge_ratchet_test.rs` pins naming pattern (synthetic `include_str!` in test module only). |
| **Retirement shape** | `DeclarationRef` (or equivalent) so runner resolves lens body from `program_dag` / `TestClaim.source` without a second byte channel (`test_runner.rs` dissolution comment). |
| **Sibling blocker** | Ledger **3b** `bridge_canonical_lens_name_patching_residual`; **BR-06**; **BR-18**-adjacent only if fold path still uses file keys — coordinate **royal-newt-846**. |
| **Order** | **10** (with BR-06 / BR-08). |

---

#### BR-08 — `m1_5_user_authored_lens_gate_test` on-disk vs runner bytes

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/tests/integration/m1_5_user_authored_lens_gate_test.rs` — `include_str!` for `r1_gates*.dag`, `named_function_count.dag`, `lens_composition_associative_witness.dag` **plus** import of `R1_CANONICAL_NAMED_FUNCTION_COUNT_LENS`. |
| **Consumers** | **1** integration test module (multiple tests). |
| **Retirement shape** | Same as BR-07: single structural authority for “which bytes are the canonical user-authored lens under test.” |
| **Sibling blocker** | BR-07, BR-06. |
| **Order** | **10** (same PR stack as BR-07). |

---

#### BR-09 — SG-0 census + infer-helpers text mining (`test_runner`)

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/src/test_runner.rs` — `SG0_CENSUS_SOURCE` (`include_str!` of `sg0_census_test.rs`), `INFER_HELPERS_SOURCE` (`include_str!` of `infer_helpers.dag`); substring / `contains` scans in runner. |
| **Consumers** | **1** (`test_runner.rs` internal ratchet helpers). |
| **Retirement shape** | Structural census from `Dag` / `GENERATED_FILES` / declared registry tables — see `docs/briefs/r2-evaluator-test-runner-authority-ratchet.md`. |
| **Sibling blocker** | SG-0 / SG-6 hand-authored census design; possibly **BR-05** if paths move to generated tables only. |
| **Order** | **9** (after or with SG authority test refactors). |

---

#### BR-10 — SG1 / SG2 / SG2c1 authority vs `*_generated.rs` pairs

| Field | Value |
| --- | --- |
| **Declaration** | `sg1_tokenize_authority_test.rs`, `sg2_parse_authority_test.rs`, `sg2c1_parse_tables_authority_test.rs` — each pairs `include_str!` of `.dag` / `.txt` inputs with `include_str!` of `tokenize_generated.rs`, `parse_generated.rs` + `parse_parser_body.txt`, or `parse_tables_generated.rs`. |
| **Consumers** | **1** test module each (**3** modules total). |
| **Retirement shape** | Parser/tokenizer exposes structural goldens or AST snapshots; tests stop diffing checked-in Rust as text (or consume generator metadata structurally). |
| **Sibling blocker** | Regen pipeline ownership; **BR-05** supplies file lists. |
| **Order** | **4** (parallel leaves after BR-01/02 if desired). |

---

#### BR-11 — M2 lens migration generated-module embeds

| Field | Value |
| --- | --- |
| **Declaration** | Six `m2_lens_*_migration_test.rs` modules — each `include_str!("../../src/lens_*_generated.rs")` (cost, provenance, structural_resolution, unused_parameters, variant_payload; plus related suites). |
| **Consumers** | **6** integration modules (one primary `include_str!` each). |
| **Retirement shape** | Structural isomorphism / `DagShapeReport`-style gate already referenced in ledger (`ValueBody` mirror drift row); tests read generator outputs as typed artifacts. |
| **Sibling blocker** | Substrate mirror program + generator contract. |
| **Order** | **5** (batched PR acceptable). |

---

#### BR-12 — SG7 variant payload generated freshness

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/tests/integration/sg7_prep_variant_payload_freshness_test.rs` — `include_str!("../../src/variant_payload_generated.rs")`. |
| **Consumers** | **1** test. |
| **Retirement shape** | Same class as BR-11 (structural gate on variant payload table). |
| **Sibling blocker** | BR-11 / generator. |
| **Order** | **5** (can merge with BR-11 wave). |

---

#### BR-13 — Lane 2 symbolic cost generated embed

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/tests/integration/lane2_stage_2d_symbolic_cost_test.rs` — `include_str!("../../src/lens_cost_symbolic_generated.rs")`. |
| **Consumers** | **1** test (large module). |
| **Retirement shape** | Structural symbolic-cost artifact or DAG witness instead of Rust text. |
| **Sibling blocker** | BR-11 family. |
| **Order** | **6** (with BR-11/12). |

---

#### BR-14 — Boundary emitter source embeds

| Field | Value |
| --- | --- |
| **Declaration** | `m1_3_emit_rust_test.rs` (`include_str!("../../src/emit_rust.rs")`), `m1_4_emit_python_test.rs` (`emit/python_target.rs` + `spec/python.dag`). |
| **Consumers** | **2** boundary test modules. |
| **Retirement shape** | Emitter assertions via structured MIR / snapshot types, or shared test helper with typed AST hook — not whole-file string search. |
| **Sibling blocker** | Emit architecture; optional overlap with SourceSpan if assertions key on `span` text. |
| **Order** | **7**. |

---

#### BR-15 — `integration.rs` bootstrap multi-file `include_str!` cluster

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/tests/integration.rs` — block of `include_str!` for `dsl/std/{logic,bit,algebra,types,integer,rational,magnitude,nat,float,string_type}.dag`, `v3/std/{list,verification,effects}.dag`, plus `parse_corpus_manifest.txt`. |
| **Consumers** | **1** parent module (`parse_file` / bootstrap smoke paths). |
| **Retirement shape** | Reuse `GENERATED_FILES` / staged static table iterator or single “bootstrap corpus” declaration instead of N hand-maintained macros. |
| **Sibling blocker** | **BR-05** / generated-module cluster (**BR-11**–**BR-13**). |
| **Order** | **8**. |

---

#### BR-16 — Shape-A target filtering authority (spec `.dag` text)

| Field | Value |
| --- | --- |
| **Declaration** | `shape_a_target_source_filtering_authority_test.rs` — `include_str!` of `computation_model.dag`, `rust.dag`, `go.dag`, `python.dag`. |
| **Consumers** | **1** test module. |
| **Retirement shape** | Read specs through compiler’s normal spec loader / `LanguageSpec` witness. |
| **Sibling blocker** | Emit `source_filtering` / **SourceSpan** participation — **coordinate royal-newt-846** if retirement touches file-filter semantics. |
| **Order** | **7** (can batch near BR-14) — **boundary flag**. |

---

#### BR-17 — `lens_apply.rs` unit tests (`#[cfg(test)]`)

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/src/lens_apply.rs` — three `include_str!("../../lenses/named_function_count.dag")` in unit tests. |
| **Consumers** | **3** unit tests in same module. |
| **Retirement shape** | Shared test fixture helper or `CARGO_MANIFEST_DIR` read in test only (still a bridge unless structural); ideal: compile lens once from substrate declaration. |
| **Sibling blocker** | BR-07 (canonical lens story). |
| **Order** | **3** (small leaf; optional early cleanup). |

---

#### BR-18 — R2 B5 loop construction closure (`lower.rs` / `builder.rs` as text)

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/tests/integration/r2_b5_loop_construction_closure_test.rs` — `include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lower.rs"))`, same for `dag/builder.rs`. |
| **Consumers** | **1** integration test. |
| **Retirement shape** | Structural proof over `Dag` / lowering IR, or sanctioned metadata query — not raw Rust source text embed. |
| **Sibling blocker** | **SourceSpan/file family — royal-newt-846** (see STOP+PING). |
| **Order** | **9** (isolate; do not mix with BR-07 without coordination). |

---

#### BR-19 — `patch_kernel_bool_boolean_algebra_inhabits`

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/src/bootstrap.rs` — mutates `Declaration.inhabits` for kernel `Bool` after parse. |
| **Consumers** | `bootstrap_regen_fresh` regen path; `bootstrap.rs` `#[cfg(test)]` module; **indirect** via every `Dag::new()` bootstrap. |
| **Retirement shape** | Express `type Bool inhabits BooleanAlgebra<Bool> = …` in `dsl/std/types.dag` (or algebra kernel) once v2 surface accepts `inhabits` in `dsl/`; delete patch (`bootstrap.rs` docstring lines 117–120). |
| **Sibling blocker** | v2 parse surface; uses **`span.file` string equality** — **coordinate royal-newt-846** (STOP+PING). |
| **Order** | **14** (substrate + v2 gate). |

---

#### BR-20 — `pipeline_authority` suspended compile-body cross-check

| Field | Value |
| --- | --- |
| **Declaration** | `src/v3/compiler/src/pipeline_authority.rs` — **no** live `include_str!`; doc cites rejected `include_str!("../pipeline.dag")` / `fs::read_to_string` approaches (`bridge_include_str_side_channels_retired`, PR #1171). |
| **Consumers** | **0** macro sites; disposition is **process + future structural witness**. |
| **Retirement shape** | Ordered stage list also embedded structurally in lowered `compile` arrow or dedicated carrier (see module comment). |
| **Sibling blocker** | Lowering / `ArrowBody::Unparsed` resolution. |
| **Order** | **16** (last — meta “anti-bridge” row). |

---

#### BR-21 — Anthropic extdep schema lockstep

| Field | Value |
| --- | --- |
| **Declaration** | `anthropic_schema_lockstep_test.rs` — `include_str!("…/dsl/extdeps/llm/anthropic.dag")`. |
| **Consumers** | **1** test. |
| **Retirement shape** | Same as BR-03: unified extdep loading / schema reflection. |
| **Sibling blocker** | BR-03, BR-05. |
| **Order** | **15** (with extdep table work). |

---

#### BR-22 — Method registry algebra + `m2_substrate_inhabitance` substrate embed

| Field | Value |
| --- | --- |
| **Declaration** | `method_registry_test.rs` (`dsl/std/algebra.dag`); `m2_substrate_inhabitance_test.rs` (`substrate.dag` via `include_str!` + `concat!`). |
| **Consumers** | **2** tests. |
| **Retirement shape** | Read through staged bootstrap / single `substrate` authority instead of second compile-time copy. |
| **Sibling blocker** | BR-05 / substrate carrier program (ledger substrate-carrier row). |
| **Order** | **8** (near BR-15). |

---

## Appendix A — Hermetic fixture `include_str!` (class E, not queued)

These embed **scenario** `.dag` / `.v3` / templates for `compile_to_dag` or
runner smoke only; they do not duplicate a separate production byte channel in
the sense of BR-07/BR-09. Verification may still migrate them to `compile_file`
paths for uniformity, but that is outside this family’s retirement thesis.

Representative files: `t_pb_b_1_dag_runner_test.rs` (9 invocations),
`m1_5_verification_test.rs` (5), `r3_verification_l4_l7_l5_skeleton_test.rs` (5),
`m1_5_user_authored_lens_gate_test.rs` (fixture halves only), `tc*.rs`,
`r1_release_acceptance_test.rs`, `r1c_*`, `r3_free_consequences_*`,
`l1_5_fixed_point_test.rs`, `lens_substrate_carrier_test.rs`,
`test_runner_test.rs` (fixture line), `grounding_lifetime/src/lib.rs` test
`include_str!` of `r1_mock_backed_invariant_gate.dag`, etc.

---

## Appendix B — `canonical_lens_bridge_ratchet_test.rs` synthetic `include_str!`

The ratchet module contains **intentional** miniature `include_str!("a.dag")` /
`"b.dag"` patterns to regex-test the real `test_runner.rs` surface. They are
**not** additional canonical authorities; do not count toward open-bridge
surface except as test harness for BR-07.

---

## Appendix C — Ledger crosswalk

| `bridge_ledger` row (see `r3-v-bridge-retirement-ledger-zero-audit.md`) | This audit |
| --- | --- |
| `bridge_include_str_side_channels_retired` (open) | BR-05, BR-20, `pipeline_authority` story |
| `bridge_canonical_lens_name_patching_residual` (open) | BR-07, BR-08, ratchet in appendix B |
| `bridge_exact_string_patching_residual_retired` (open umbrella) | BR-06, BR-19; retired lower-helper slice |
| `bridge_source_span_file_participation_retired` (open) | STOP+PING table |

---

## Receipt

Audit packet filed **2026-05-03** for ledger **B4** row — family **B**
(`include_str!` / patching). Per-row implementation PRs remain with Verification
(#1276).
