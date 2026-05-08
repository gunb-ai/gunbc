# Canvas — Q-ValueBody-Drift-Resolution (substrate↔Rust mirror parity question)

**Authority**: Q-ValueBody-Isomorphism RATIFIED (Director at gunb-ai/gunbc#828 #issuecomment-4403972737, 2026-05-08; Brian sanctioned IN-R3). D1 (mirror-parity-vs-codegen-generation choice) routed to Substrate Mgr per V1 worker brief §Dependencies.

**Status**: **canvas — DRAFT 2026-05-08**; PROPOSAL maturation pending Director ratification on D1 path-call.

**Sub-issue**: cross-Mgr — Verification Mgr V1 worker brief at `docs/briefs/r3-v-valuebody-substrate-mirror-isomorphism-v1-worker.md`; readiness input via PR #2218 merged 2026-05-08 (variant-inventory at `docs/r3/r3-v-valuebody-variant-inventory.md`).

## Drift fact (per PR #2218 inventory at HEAD)

**Rust live carrier** (`src/v3/compiler/src/dag.rs:436-504`): **5 variants**
- `Unparsed(SourceSpan)` — substrate parity ✓
- `Structural { fields: Vec<(String, FieldValue)> }` — substrate parity ✓
- `Scalar(LiteralBits)` — Rust-only
- `List(Vec<FieldValue>)` — Rust-only
- `Map(FieldMap)` — payload-parity-with-semantic-gap (FieldMap dup-key invariant Rust-side; substrate `List<FieldEntry>` lacks the invariant)

**Substrate** (`src/v3/std/substrate.dag:179-182`): **3 constructors**
- `ValueBodyUnparsed(SourceSpan)` ✓
- `ValueBodyStructural { fields: List<FieldEntry> }` ✓
- `ValueBodyMap(List<FieldEntry>)` ✓ (payload-parity; semantic-gap on dup-key)

**Asymmetry**: Rust-only Scalar + List; substrate-Map weaker invariant than Rust FieldMap.

## D1 — two path options

### Path (a) — Mirror-parity (substrate adds constructors)

Add `ValueBodyScalar(LiteralBits)` + `ValueBodyList(List<FieldValue>)` to substrate; optionally add substrate-level dup-key invariant on Map.

**Pro**: maintains explicit substrate authoring; no codegen tooling change.
**Con**: parallel-authority — two hand-edited authorities for the same algebraic surface; same drift class persists; future divergences trivially recur. Per `feedback_parallel_representation_debt`: this is the wrong tier of debt to absorb.

### Path (b) — Codegen-generation (Rust derives from substrate canonical authority)

Substrate is canonical; Rust enum codegens from substrate definition via existing precedent.

**Substrate-grep at HEAD finds existing codegen tooling**:
- `src/v3/compiler/src/regen_bootstrap_emit.rs`: `render_bootstrap_generated_rs` + `render_bootstrap_std_generated_rs` exposed at `lib.rs:3277` — substrate→Rust emit pipeline at HEAD
- Generated files at HEAD: `bootstrap_generated.rs`, `dag_lookup_generated.rs`, `dag_scalar_generated.rs`, `diagnostics_generated.rs`, `parse_surface_generated.rs` — multiple precedents
- `regen_bootstrap` binary: `cargo run -p v3-compiler --features bootstrap-regen-fresh --bin regen_bootstrap` runs the substrate→Rust regen pipeline (used in PR #2194 1a.0 regen receipt)

**Codegen precedent IS established at HEAD**. Extending the existing pipeline to cover the Rust-only `Scalar` + `List` variants (substrate adds them, codegen emits Rust enum from substrate) is **pattern-3 (strict-mirror codegen extension)**, NOT pattern-5 (genuinely novel substrate-fact-introduction). Per canvas-finding-taxonomy at `feedback_canvas_finding_taxonomy.md`: pattern-3 ratifies at slice-tier directly, no canvas-tier P1 procedure required.

**Pro**: dissolves dual-taxonomy class entirely; substrate is canonical; future drift impossible by construction. Per `feedback_isomorphism_or_generation_for_mirrors` (Verification Mgr's referenced discipline): codegen-generation is the right shape for substrate↔Rust mirror surfaces. Per `feedback_parallel_representation_debt`: cost-of-change=1 (one authority, codegen-derived consumer).
**Con**: codegen extension is bounded engineering effort (not free); Map dup-key invariant gap still needs decision (substrate-level invariant or Rust-side post-codegen wrapping).

## Mgr lean

**Path (b) codegen-generation RECOMMENDED**:

1. **Codegen precedent at HEAD verified** (5 `_generated.rs` files; `regen_bootstrap_emit` pipeline; binary in flight via PR #2194). Pattern-3 strict-mirror codegen extension; no canvas-tier P1 procedure needed.
2. **Single-authority discipline preserved** per `feedback_parallel_representation_debt`; substrate is canonical for the algebraic surface; Rust-side dissolves to codegen-derived consumer.
3. **Future-drift-impossible by construction**: future variant additions land substrate-side once; Rust regen propagates automatically. Dissolves the discipline-failure class that surfaced this canvas.
4. **Map dup-key semantic-gap** is a separate sub-question: either (i) substrate-level invariant on `List<FieldEntry>` Map (constructor-level uniqueness — would require new substrate primitive), or (ii) Rust-side post-codegen wrapping in FieldMap (preserves existing Rust invariant; substrate-side remains agnostic). Mgr lean (ii) — preserves Rust-side invariant; substrate-side stays minimal.

## Director ratification ask

1. **D1**: ratify (b) codegen-generation? (Mgr lean: yes — codegen precedent at HEAD; pattern-3 strict-mirror extension; dissolves drift class)
2. **D1-followup**: Map dup-key gap — (i) substrate-level invariant or (ii) Rust-side post-codegen wrapping? (Mgr lean: (ii) Rust-side wrapping; substrate stays minimal)

## Two-axis verification (per `feedback_canvas_two_axis_verification`)

**Axis 1 — substrate-precedent**: codegen tooling at HEAD verified at `regen_bootstrap_emit.rs` + 5 `_generated.rs` files. (b) extends established precedent.

**Axis 2 — consumer-side requirement**: V1 worker brief at `docs/briefs/r3-v-valuebody-substrate-mirror-isomorphism-v1-worker.md` names "mirror parity vs codegen-generation" as the explicit D1 choice. Consumer-side framing matches the path-pair this canvas presents. PR #2218 inventory confirms the drift fact + path-pair structurally.

Both axes grep-verified before authoring.

## Carrier slice (post-D1 ratification)

If (b) ratifies, downstream carrier slice:

1. Extend `src/v3/compiler/src/regen_bootstrap_emit.rs` (or analog) to emit Rust `ValueBody` enum from substrate `ValueBody` type — pattern-3 codegen extension
2. Add `ValueBodyScalar(LiteralBits)` + `ValueBodyList(List<FieldValue>)` constructors to `src/v3/std/substrate.dag` (substrate-canonical authoring)
3. Run `regen_bootstrap` to propagate; commit regenerated files
4. Remove hand-authored Rust `ValueBody` enum from `src/v3/compiler/src/dag.rs:436-504`
5. (Per D1-followup ratification) handle Map dup-key gap: either substrate-invariant or Rust-post-codegen-wrapping
6. zesty-moth-793 V1 worker (#2142) sequences Slice 2 (mirror-parity verification cementing test) post-carrier-slice land

Worker pin for carrier slice: fresh-pool pick at dispatch time per same-window-dispatch discipline (post-ctrl#217 fix RESOLVED 2026-05-08).

## Cross-Mgr coordination

- **Verification Mgr (#2075 / wise-bear-525)**: V1 worker brief consumer; standing wait through D1 ratification + carrier slice land per their c#4404094879 sequencing posture. Surface receipt at carrier-slice PR-open.
- **PR #2218 readiness input** (variant inventory) merged 2026-05-08; concrete artifact pointer for canvas authoring.

## Framework discipline anchors

- **`feedback_canvas_finding_taxonomy`**: pattern-3 (strict-mirror codegen extension) per existing precedent; ratifies slice-tier-direct under canvas authority.
- **`feedback_canvas_two_axis_verification`**: both axes grep-verified at HEAD before authoring (substrate-precedent + consumer-side).
- **`feedback_parallel_representation_debt`**: (b) dissolves dual-taxonomy class; structurally cleaner per single-authority discipline.
- **`feedback_isomorphism_or_generation_for_mirrors`** (Verification Mgr's referenced discipline): codegen-generation matches the established shape.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-08 per Q-ValueBody-Isomorphism RATIFIED at gunb-ai/gunbc#828 #issuecomment-4403972737; D1 routed via V1 worker brief §Dependencies; readiness input from PR #2218 (variant-inventory) merged 2026-05-08.
