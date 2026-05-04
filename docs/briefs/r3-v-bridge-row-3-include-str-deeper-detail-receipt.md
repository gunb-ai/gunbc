# R3 Verification - Bridge Row 3 `include_str!` Deeper-Detail Receipt

**Status:** AUDIT RECEIPT - docs-only. This receipt does not flip any
`BridgeLedgerRow.status` and does not close a Debt-Paydown row directly.

**Row:** `bridge_include_str_side_channels_retired`.

**Primary input:** `docs/briefs/bridge-retirement-audit-include-str-family.md`
(`crisp-newt-163` family B packet). This receipt consumes the family B
`include_str!` side-channel subset not owned by Row 2 canonical-lens semantics
and not reserved for Row 4 exact-string patching.

**Boundary inputs:** `docs/briefs/r3-v-bridge-row-1-sourcespan-deeper-detail-receipt.md`
and `docs/briefs/r3-v-bridge-row-2-canonical-lens-deeper-detail-receipt.md`
are boundary checks only. Row 1 owns file-stamped participation and
SourceSpan/file identity; Row 2 owns canonical lens byte/name identity. Row 3
owns remaining embedded source/text side channels and their single-authority
loader/generator replacements.

**Prior row audit:** `docs/briefs/r3-v-bridge-row-by-row-retirement-audit.md`
frames the load-bearing row-3 blocker as `pipeline_authority.rs`: stage order is
already read structurally from `PipelineStageBinding`, but compile-body drift
cannot retire while the compile orchestrator still lowers to
`ArrowBody::Unparsed`.

## Verification Consumption Rule

Verification should reject a PR that claims to retire an `include_str!` side
channel unless it demonstrates that **all embedded-source participation surfaces
for that entry are gone or converted to structural consumption**. Moving text
from one macro/file read to another is not retirement.

Specific failure modes that keep an entry open:

1. pipeline-authority compile-body comparison remains impossible because
   `compile` still lowers to `ArrowBody::Unparsed`;
2. a rejected `include_str!("../pipeline.dag")`, `std::fs::read_to_string`, or
   span-slicing drift check is reintroduced instead of a structural witness;
3. tests still diff checked-in generated Rust or source files as whole strings
   when a typed AST, parser/generator metadata, or structural report should be
   the authority;
4. generated/static tables still embed `(virtual_path, include_str!(abs_path))`
   as a second enumeration authority instead of consuming one declared corpus or
   staged loader authority;
5. source-text mining remains load-bearing through `.contains`, line scraping,
   brace scraping, markdown marker parsing, or raw Rust substring checks;
6. a PR deletes one `include_str!` macro while leaving a sibling `include!`,
   generated table, test fixture, or runtime file-read cousin as the same
   authority.

At review time, each retirement PR must cite the BR id and show the replacement
carrier is consumed by the relevant production or verification path. A helper
that simply centralizes byte loading is still a side channel unless the byte
identity is generated from or checked against typed substrate.

## Cross-Packet Boundary Discipline

| Family B entry / surface | Row 3 owns | Row 3 does not own |
| --- | --- | --- |
| BR-01 / BR-02 text mirrors and markdown gates | Embedded canonical text used as authority for diagnostics or L6 closure data. | Any later generated enum/table design beyond proving the text mirror is no longer load-bearing. |
| BR-03 / BR-21 extdep schema/primitives embeds | Duplicate compile-time extdep `.dag` text channels outside the shared staged loader. | Extdep semantic modeling itself. |
| BR-04 / BR-05 bootstrap and generated static tables | Hand-listed std bundle and generated `(virtual_path, include_str!(abs_path))` tables as second source authorities. | Row 1 path-key participation after virtual paths are materialized; Row 4 exact-string fixture splicing. |
| BR-06 R1 gates fixture splice | Only generic embedded-source side-channel mechanics if a PR changes loader/generator authority. | Row 2 canonical-lens byte identity slice; Row 4 sentinel/exact-string splice replacement. |
| BR-07 / BR-08 / Appendix B canonical lens surfaces | Generic `include_str!` mechanics only if a PR retires the loader class globally. | Canonical lens bytes, name dispatch, and ratchet counts; Row 2 owns those. |
| BR-09 SG-0 / infer-helper text mining | Embedded source-text mining through runner ratchets. | Row 4 exact-string patching if a scan becomes a patch/rewrite class rather than a side-channel ratchet. |
| BR-10 through BR-13 generated Rust embeds | Drift checks that consume checked-in generated Rust as text. | The substrate mirror/generator contract itself after a structural report replaces text inclusion. |
| BR-14 / BR-16 emitter and Shape-A spec embeds | Whole-file emitter/spec text assertions and spec `.dag` bytes outside normal loaders. | Row 1 emit `SourceFilteringBinding::excludes` file participation. |
| BR-15 / BR-22 bootstrap corpus and substrate embeds | Test modules that hand-maintain bootstrap/substrate include clusters instead of consuming a staged corpus authority. | Row 1 bootstrap virtual-path participation. |
| BR-18 raw compiler source embeds | Raw Rust source text inclusion for structural proofs. | Row 1 only if a production `SourceSpan.file` participation surface is changed; Row 4 only if the source text is patched/replaced. |
| BR-19 Bool patch | No Row 3 ownership. | Row 1 owns its file gate; Row 4 owns post-parse/exact-string patching. |
| BR-20 pipeline authority suspended side channel | Process row for the rejected include/read/span-slice channel and the missing structural compile-body witness. | Stage-order semantics already covered by `PipelineStageBinding`; Row 1 owns any file guard cleanup. |
| Appendix A hermetic fixtures | Not queued as Row 3 bridge debt unless the fixture byte channel becomes a duplicated authority. | Scenario input bytes that do not duplicate production authority. |

This receipt deliberately does not expand Row 3 into Row 2's
canonical-lens-name surface or Row 4's exact-string patching umbrella.

## Per-Entry Receipts

Rows follow the family B packet's leaf-first order, omitting entries owned by
Row 2 or reserved entirely for Row 4.

| Order | Entry | Verification-side reviewer rule | Carrier consumer required | Cross-row impact |
| --- | --- | --- | --- | --- |
| 1 | BR-01 grounding emission diagnostic mirrors | Delete `MIRROR_*` whole-file text mirrors and line/brace scraping; diagnostics tests must consume generated or typed diagnostic facts. | `.dag` to Rust `EmissionDiagnostic` codegen or structural diagnostic table consumed by grounding tests. | No Row 1/2 overlap. |
| 2 | BR-02 R2 closure ledger markdown gate | Stop scraping `docs/r2-closure-ledger.md` markers for L6 closure data; markdown may remain documentation only. | Structural L6 closure carrier or generated table consumed via `Dag` / declaration refs. | No Row 1/2 overlap. |
| 4 | BR-10 SG1/SG2/SG2c1 authority vs generated Rust pairs | Replace `.dag` / `.txt` plus `*_generated.rs` text diffing with typed parser/tokenizer/generator goldens; no checked-in Rust source text remains the authority. | Parser/tokenizer AST witness, parse-table metadata, or generator API output consumed structurally. | May depend on BR-05 for unified file enumeration. |
| 5a | BR-11 M2 lens migration generated-module embeds | Tests must stop reading generated lens Rust modules as text; migration acceptance should consume structural reports. | Structural isomorphism report, `DagShapeReport`-style witness, or generator contract artifact. | Adjacent to substrate mirror program, not Row 2 canonical lens identity. |
| 5b | BR-12 SG7 variant payload generated freshness | Replace `variant_payload_generated.rs` text freshness check with the same structural generator/report surface as BR-11. | Variant payload structural report or generator metadata. | Can batch with BR-11. |
| 6 | BR-13 Lane 2 symbolic cost generated embed | Remove symbolic-cost generated Rust text inclusion; tests should consume a structural symbolic-cost artifact or DAG witness. | Symbolic-cost DAG witness, generator metadata, or structural report. | Can batch with BR-11/BR-12. |
| 7a | BR-14 boundary emitter source embeds | Boundary tests must stop searching whole emitter source/spec files by text; assertions should go through structured MIR/snapshot hooks. | Structured MIR, typed emitter snapshot, or shared test helper over typed AST/output facts. | Row 1 owns any emit file-participation filtering changes. |
| 7b | BR-16 Shape-A target filtering authority | Tests must stop embedding Shape-A spec `.dag` bytes as a parallel authority; filtering authority should be read through normal spec loading. | `LanguageSpec` / normal compiler spec loader witness consumed by the test. | Row 1 owns production `SourceFiltering` file participation. |
| 8a | BR-15 integration bootstrap multi-file include cluster | Replace hand-maintained `integration.rs` include cluster with staged corpus iteration or a single bootstrap corpus declaration. | `GENERATED_FILES`, staged static table iterator, or typed bootstrap corpus carrier consumed structurally. | Row 1 owns virtual-path participation after materialization. |
| 8b | BR-22 method registry / substrate embeds | Tests must read algebra/substrate authority through the shared staged bootstrap/substrate authority, not ad hoc `include_str!` copies. | Staged bootstrap loader, substrate authority carrier, or generated corpus table consumed structurally. | Adjacent to substrate-carrier work, not Row 2. |
| 9a | BR-09 SG-0 census + infer-helper text mining | Runner ratchets must stop mining source text through `include_str!` / `.contains`; counts should come from declared registry tables or structural corpus facts. | Structural census from `Dag`, generated-file registry, or declared table. | Row 4 owns any exact-string patching class discovered from these scans. |
| 9b | BR-18 R2 B5 loop construction closure source embeds | Replace raw `lower.rs` / `builder.rs` source inclusion with structural proof over `Dag`, lowering IR, or sanctioned metadata. | Lowering IR witness, DAG proof carrier, or typed metadata query. | Row 1 only if production file participation changes; avoid mixing with Row 2 canonical lens work. |
| 12 | BR-05 `build.rs` generated `include_str!` static tables | Generated `(virtual_path, include_str!(abs_path))` tables must stop being a second source authority; a single declared corpus/loader authority should drive bootstrap. | `bootstrap.dag` or equivalent declared corpus authority; generated typed loader without embedding every file as independent string authority. | Root for BR-04/BR-15/BR-22 and extdep entries; Row 1 may count virtual-path participation separately. |
| 13 | BR-04 `bootstrap_regen_fresh` hand-listed std bundle | Delete the hand-listed std `.dag` const bundle once BR-05 replacement channel exists; regen must iterate one staged authority. | Same staged table / declared corpus authority as BR-05, consumed by `compile_std_bootstrap_dag` / `load_fixtures`. | Coordinate with Row 1 bootstrap path-key retirement but do not count path-key deletion here. |
| 15a | BR-03 grounding pilot Go primitives authority | Grounding pilot must use the unified extdep staged loader rather than a second compile-time primitives `.dag` copy. | Typed extdep fixture load through the shared staged-bootstrap table. | Depends on BR-05 extdep enumeration. |
| 15b | BR-21 Anthropic extdep schema lockstep | Replace ad hoc Anthropic extdep `include_str!` schema copy with unified extdep/schema reflection. | Shared extdep loader or schema reflection carrier. | Can batch with BR-03. |
| 16 | BR-20 pipeline authority suspended compile-body cross-check | Retirement requires a structural compile-body/stage-order witness; do not reintroduce `include_str!("../pipeline.dag")`, `read_to_string`, or span slicing. | Lowered compile-body witness, or single authored carrier representing both compile orchestrator body and `PipelineStageBinding` order. | PB Manager / pipeline authority blocker. Row 1 owns file guards; Row 3 owns the side-channel anti-bridge and missing structural witness. |

## Substrate/PB Routing Notes

Row 3 routes primarily to PB Manager / compiler-internal bootstrap:

- `pipeline_authority.rs` needs a lowered compile-body witness or unified
  authored carrier; `PipelineStageBinding` alone is not sufficient because it
  covers stage order but not compile-body drift.
- BR-04/BR-05/BR-15/BR-22 need one staged corpus / bootstrap loader authority
  consumed structurally by bootstrap, regen, and tests.
- BR-03/BR-21 need extdep loading through that same staged authority or schema
  reflection path.
- BR-10 through BR-13 need generator metadata, structural reports, or typed
  artifacts so tests do not consume generated Rust as source text.
- BR-01/BR-02/BR-09/BR-14/BR-16/BR-18 each need a typed witness or structured
  test hook matching their domain rather than text scraping.

No STOP+PING is triggered by this receipt. The known open blocker is already
named: pipeline compile-body structure is missing while `compile` remains
`ArrowBody::Unparsed`. Any implementation PR that needs a new substrate shape
outside these routed carriers must pause and route that shape before claiming
Row 3 retirement.

## Per-PR Receipt

Debt found + routed: `include_str!` side-channel per-entry retirement roadmap
recorded inline; PB Manager bootstrap/pipeline carrier asks and any structural
test/generator witness asks routed through T-Bridge-Retirement.

This receipt does not close a Debt-Paydown row directly. Closure happens only
when PB/Substrate ships the actual retirement PRs and `bridge_ledger.dag` flips
the relevant row from `Open` to `Retired`.

## Test Plan

- `git diff --check`
