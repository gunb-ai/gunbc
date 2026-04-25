# Pure Bootstrap to Zero — 34-file SG-0 census audit

**Status:** `LIVE` (deliverable). Authored as Pre-promotion Deliverable 1
for [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md)
per its §"Pre-promotion deliverables" section. Cited by the cascade
promotion PR; consumed as evidence that the 0-floor target is structurally
reachable rather than an aspiration.

**Authority:** Pure Bootstrap to Zero Program Manager (parallel to R2 per
[`docs/briefs/pure-bootstrap-zero-manager.md`](briefs/pure-bootstrap-zero-manager.md)).

**Source-of-truth for entries:** `src/v3/compiler/tests/integration/sg0_census_test.rs`
`EXPECTED_HAND_AUTHORED_NON_TEST` array. Read live at audit-authoring time;
this doc does **not** restate counts that will drift. The lane mapping is
the durable contribution.

## Count reconciliation

The Zero-Floor Manager brief's reference snapshot says "35 NON_TEST files
at brief authoring (post #763 `lens_depth.rs` retirement)." Live count on
main at audit-authoring time is **34**. Either an additional retirement
landed between brief authoring and audit, or the snapshot was off-by-one.
The brief's snapshot line should refresh to 34 (or whatever live is at
cascade-merge time) when the cascade PR lands. Not load-bearing; the
audit walks the live array.

## Frame

The design doc claims that the "irreducible tier" (`build.rs`,
`bootstrap.rs`, `lib.rs`, `dag.rs`, `dag/ports.rs`, `dag/effects.rs`)
isn't structurally irreducible — each file is generatable from a `.dag`
authority. This audit substantiates that claim file-by-file across the
whole 34-entry census, not just the irreducible tier.

For each file: **why it is currently hand-authored**, and **the
migration path that retires it**. Each row identifies a lane from the
Zero-Floor program's lane taxonomy
([brief §Slice](briefs/pure-bootstrap-zero-manager.md), [design doc
§Subsumed lanes / §New lanes](design-pure-bootstrap-zero.md)).

## Substrate generation is already proven and shipping (load-bearing reframe)

> **Added 2026-04-25 post-merge.** The original audit (and the design
> doc's PROPOSAL framing) treated substrate generation as a **future
> pattern that needed proof**. Direct verification triggered by a
> codex BLOCKING finding on the withdrawn pilot brief #772 establishes
> that the pattern is **already proven and shipping**.

**Evidence (verified at `4d2423da8`):**

- **26 `*_generated.rs` files** under `src/v3/compiler/src/` covering
  substrate, parse, tokenize, infer, lower, diagnostics, lens,
  operators, types, serialize, variant_payload, bootstrap.
- `src/v3/compiler/src/dag.rs` is a **hybrid**: at `:497`, `:1678`,
  `:1699`, `:1710` it `include!()`s four substrate-shape generated
  files (`dag_scalar_generated`, `dag_branch_generated`,
  `dag_cluster_generated`, `dag_lookup_generated`).
- `build.rs` `REGEN_OUTPUTS` registers each generated file; emission
  runs at compile time without manual invocation.
- **Substrate.dag coverage survey** (38 declared `type` names vs
  `^pub enum X` / `^pub struct X` matches in `*_generated.rs`):
  - **11 already generated**: `BranchPattern`, `CardinalityBound`,
    `Cluster`, `IntraClusterCall`, `LiteralBits`, `LoopBound`,
    `MemberDescent`, `PayloadBinding`, `PortState`,
    `TemplateArgument`, `TypeShape`.
  - **27 not yet generated** by the survey heuristic (some may be
    covered indirectly): `ArithmeticOp`, `ArrowBody`, `AtomPayload`,
    `Behavior`, `BindNode`, `BranchNode`, `BranchPath`,
    `ComparisonOp`, `ConjField`, `Dag`, `DagPort`, `Declaration`,
    `ElementRef`, `FieldEntry`, `FieldValue`, `LogicalOp`, `LoopNode`,
    `OperatorKind`, `ParamRef`, `SubstrateAccessorBinding`,
    `SubstrateAccessorRealization`, `TransformNode`, `TransformRef`,
    `TransformTarget`, `TypeConnective`, `ValueBody`, `ValueNode`.
  - Concrete uncovered hand-authored example: `ArithmeticOp` /
    `ComparisonOp` / `LogicalOp` / `OperatorKind` declared in
    `substrate.dag` AND hand-authored in `dag.rs:694-725` AND not
    generated.

**Implication for the cascade evidence framing:**

- The 0-floor target is **further along than the design doc claimed**.
  The "irreducible tier" (`dag.rs`, `dag/ports.rs`, `dag/effects.rs`)
  isn't future-irreducible *and* isn't structurally hand-authored
  in full — `dag.rs` is hybrid today.
- **Pre-promotion Deliverable 4** reframes from "prove the pattern
  via a new pilot" to "**characterize the existing pattern** as the
  cascade's primary evidence; optional small (a)-style pilot on a
  surveyed-uncovered substrate type for incremental coverage." This
  is Director-signed-off as the remediation path post-escalation
  on [#766](https://github.com/gunb-ai/gunbc/pull/766).
- **PB-Substrate as a lane** narrows from "build the pattern" to
  "extend the existing pattern to the 27 uncovered substrate types
  + retire the residual orchestration-kernel hand-authoring in
  `dag.rs`."

**Honest accounting**: the original audit's PB-Substrate "why
hand-authored" rationale is **wrong for substrate-shape rows**.
Those files are hybrid (kernel hand-authored, mirrored types
generated). The lane assignments in the file-by-file table remain
correct as migration targets; the rationale column understates
existing progress. A future audit-discipline pass should rewrite
those rationale cells against the verified state above.

## Findings before the table

Two files in the live census have no explicit lane home in the brief's
lane taxonomy. Proposed lane assignments below; subject to Director sign-off
during cascade review.

- **`src/v3/compiler/src/diagnostics.rs`** (1110 LOC). File header marks
  "DEFERRED DISSOLUTION: Diagnostic enum is a scaffold" pointing at the
  v3-modeling-analysis §CompilerDiagnostic 5-field target. **Proposed
  lane: PB-Substrate** — Diagnostic + DiagnosticCategory are substrate
  types modeled in `src/v3/std/diagnostics.dag` and should generate
  alongside `dag.rs`.
- **`src/v3/compiler/src/pipeline_authority.rs`** (311 LOC). Reads
  `PipelineStageBinding` declarations from the bootstrapped Dag to drive
  pipeline ordering. **Proposed lane: PB-Bootstrap-Process** — the
  pipeline-ordering reader is the runtime-side consumer of the
  `bootstrap.dag` workflow declaration; it dissolves into the bootstrap-
  data substrate when PB-Bootstrap-Process lands.
- **Five regen-tool entries in PB-Tier1-Sweep** (`regen_parse.rs`,
  `regen_parse_tables.rs`, `regen_tokenize.rs`, `regen_parse_emit.rs`,
  `regen_parse_tables_emit.rs`). Brief lane taxonomy enumerates PB-1,
  PB-4 (lower), PB-5 (infer), PB-6 (emit) but **not** PB-Parse /
  PB-Tokenize as separate sub-lanes. Codex auto-review on c7c864d0
  flagged the original wildcard placeholders as scaffold-without-
  trigger (P5 violation). **Proposed homes (subject to Director
  sign-off):** the three `bin/regen_*` parse/tokenize binaries retire
  under **PB-1** (their backing authorities — parse tables, tokenizer
  output — are exactly what PB-1 generated constructors replace); the
  two `regen_*_emit.rs` files retire under **PB-Bootstrap-Process**
  (the emit-side of the regen cycle dissolves with bootstrap-as-data).
  If Director prefers PB-Parse / PB-Tokenize as explicitly named
  sub-lanes, the brief lane taxonomy needs an amendment in the
  cascade PR rather than the audit.

## Lane-distribution summary

| Lane | Files | % of 34 |
|---|---|---|
| PB-Substrate | 4 (incl. proposed `diagnostics.rs`) | 12% |
| PB-1 / PB-Bootstrap-Process | 2 (incl. proposed `pipeline_authority.rs`) | 6% |
| PB-4 / PB-5 / PB-6 (compiler-in-`.dag`) | 6 | 18% |
| PB-Lib + PB-Build | 2 | 6% |
| PB-Runtime | 4 | 12% |
| PB-Workflow | 2 | 6% |
| PB-Tier1-Sweep | 14 | 41% |

PB-Tier1-Sweep is the largest group but has no independent migration
design — entries retire mechanically as their backing PB-* migration
lands. The substantive migration design is concentrated in PB-Substrate /
PB-1+Bootstrap-Process / PB-4/5/6 / PB-Runtime (16 files, 47% of the
census).

## File-by-file table

### PB-Substrate (4 files)

Substrate types generate from `src/v3/std/substrate.dag`. Cementing
test: generated Rust matches the structural facts the substrate model
declares.

| File | Why currently hand-authored | Migration path |
|---|---|---|
| `src/v3/compiler/src/dag.rs` | Core substrate types (Dag, Node, Port, Conj, Disj, Cardinality, Bit) hand-authored as Rust. `substrate.dag` exists and is non-trivially populated (398 LOC, TERMINAL-marked structural types matching the runtime enum surface) but is not yet the generation source. | Generate from `src/v3/std/substrate.dag`. |
| `src/v3/compiler/src/dag/ports.rs` | Port submodule of dag.rs; hand-authored alongside the parent. | Same emission pass as `dag.rs`. |
| `src/v3/compiler/src/dag/effects.rs` | Effects submodule; hand-authored alongside dag.rs. | Same emission pass as `dag.rs`. |
| `src/v3/compiler/src/diagnostics.rs` | Diagnostic enum + fail-closed `mark_unresolved` API. Header explicitly names "DEFERRED DISSOLUTION" pointing at v3-modeling-analysis §CompilerDiagnostic 5-field target. `src/v3/std/diagnostics.dag` exists (39 LOC) but is currently a stub relative to the target shape. | Extend `diagnostics.dag` to the 5-field shape; generate alongside `dag.rs`. Fail-closed API contract preserved per `feedback_fail_closed_is_boundary`. |

### PB-1 / PB-Bootstrap-Process (2 files)

Bootstrap-as-data conceptual core. PB-1 sub-lanes a-e migrate runtime
authorities to generated constructors; PB-Bootstrap-Process replaces
the residual orchestration with a `bootstrap.dag`-driven trampoline.

| File | Why currently hand-authored | Migration path |
|---|---|---|
| `src/v3/compiler/src/bootstrap.rs` | Hand-Rust `Dag::new()` runs full compile pipeline on `include_str!`'d `.dag` source at every construction. Chicken-egg per design doc §"Bootstrap as data": `Dag::new()` needs the compiler pipeline. | PB-1 a-e migrates std/staged/specs/compiler authorities to generated constructors (already-scoped per [`pb-1-data-driven-bootstrap.md`](briefs/pb-1-data-driven-bootstrap.md), non-goals revised under 0-floor). PB-Bootstrap-Process then replaces residual orchestration with a `bootstrap.dag`-driven trampoline. |
| `src/v3/compiler/src/pipeline_authority.rs` | Reads `PipelineStageBinding` declarations from the bootstrapped Dag; Rust shim between bootstrap-data and pipeline execution. | Subsumed by `bootstrap.dag` workflow declaration: pipeline ordering reads structurally from the bootstrap data, not from a Rust accessor. |

### PB-4 / PB-5 / PB-6 — compiler pipeline in `.dag` (6 files)

| File | Why currently hand-authored | Migration path |
|---|---|---|
| `src/v3/compiler/src/lower.rs` | `lower.dag` authority + SG-3 series prerequisites not yet landed; hand-Rust lowering is the bridge. | **PB-4**: author `lower.dag`, build `regen_lower` binary, emit `lower_generated.rs`; retire `lower.rs`. |
| `src/v3/compiler/src/infer.rs` | `infer.dag` + SG-4 dispatch not yet landed. | **PB-5**: author `infer.dag` + dispatch; retire `infer.rs`. |
| `src/v3/compiler/src/emit.rs` | Emit dispatch hand-authored; spec-driven emission incomplete (Lane 1e dependency). | **PB-6**: emit reads structural facts from extdeps + spec authorities. |
| `src/v3/compiler/src/emit/python_target.rs` | Python target hand-authored alongside emit.rs. | **PB-6**: generate from `python.dag` spec authority. |
| `src/v3/compiler/src/emit/rust_target.rs` | Rust target hand-authored. | **PB-6**: generate from `rust.dag` spec authority. |
| `src/v3/compiler/src/emit_rust.rs` | Re-export shim retained for transition compatibility. | **PB-6**: shim collapses on `emit/rust_target.rs` retirement. |

### PB-Lib + PB-Build — Cargo-convention trampolines (2 files)

Per design doc §"Cargo conventions vs zero-source-tree" (Open call 1):
trampolines whose content is generated count as 0, because the file is
just a path-binding for Cargo, not hand-authored content.

| File | Why currently hand-authored | Migration path |
|---|---|---|
| `src/v3/compiler/build.rs` | Cargo build script enumerates STAGED_FILES / V3_SPECS / COMPILER_FILES / extdeps_generated / gunbc_generated; hand-Rust today. | Generated trampoline that `include!()`s OUT_DIR-emitted content from emit authority. |
| `src/v3/compiler/src/lib.rs` | Cargo-convention crate root; module declarations + crate exports hand-authored. | Generated trampoline. |

### PB-Runtime — test/lens runtime as data + tiny interpreter (4 files)

ExecuteCommand-based TestClaim runner support landed (#688/#741); full
migration of these four files is the PB-Runtime scope.

| File | Why currently hand-authored | Migration path |
|---|---|---|
| `src/v3/compiler/src/test_runner.rs` | Test runner hand-Rust. | Generate from `.dag` authority; runner becomes a generic interpreter over `TestClaim` data. |
| `src/v3/compiler/src/lens_apply.rs` | Lens application hand-Rust. | Generate from `.dag` authority; lens evaluator becomes data + tiny interpreter. |
| `src/v3/compiler/src/lens_testgen.rs` | Lens testgen hand-Rust. | Same shape as `lens_apply.rs` migration. |
| `src/v3/compiler/src/post_emit_verifier.rs` | Post-emit verifier hand-Rust. | Same shape; verifier becomes data + interpreter. |

### PB-Workflow — Lane 2 dissolution dependency (2 files)

| File | Why currently hand-authored | Migration path |
|---|---|---|
| `src/v3/compiler/src/workflow_idempotency.rs` | Lane 2 (workflow dissolution) hasn't landed. | Migrates as Lane 2 dissolution lands. |
| `src/v3/compiler/src/workflow_parallelism.rs` | Same; also gated on Stage 2e `.dag` surface. | Migrates after Stage 2e `.dag` surface lands. |

### PB-Tier1-Sweep — per-file fast-retire (14 files)

These retire as their backing PB-* migration lands. No standalone
migration design — each retires when its producer is generated.

| File | Backing dependency |
|---|---|
| `src/v3/compiler/src/bin/regen_bootstrap.rs` | PB-Bootstrap-Process |
| `src/v3/compiler/src/bin/regen_lens.rs` | PB-Runtime |
| `src/v3/compiler/src/bin/regen_parse.rs` | PB-1 (regen tool for parse-authority subsumed by PB-1 generated constructors) *(proposed — see Findings)* |
| `src/v3/compiler/src/bin/regen_parse_tables.rs` | PB-1 (parse-table authority subsumed by PB-1) *(proposed — see Findings)* |
| `src/v3/compiler/src/bin/regen_tokenize.rs` | PB-1 (tokenize-authority subsumed by PB-1 generated constructors) *(proposed — see Findings)* |
| `src/v3/compiler/src/bin/regen_v3.rs` | PB-Bootstrap-Process |
| `src/v3/compiler/src/bin/self_host_fixed_point.rs` | PB-Bootstrap-Process (DB-8 gate harness) |
| `src/v3/compiler/src/dag/builder.rs` | PB-Substrate (builder API regenerates with substrate types) |
| `src/v3/compiler/src/dimension.rs` | PB-Substrate / std modeling |
| `src/v3/compiler/src/lens_unused_parameters.rs` | Lens generalization (PB-Runtime adjacent) |
| `src/v3/compiler/src/regen_bootstrap_emit.rs` | PB-Bootstrap-Process |
| `src/v3/compiler/src/regen_parse_emit.rs` | PB-Bootstrap-Process (emit-side of regen cycle dissolves with bootstrap-as-data) *(proposed — see Findings)* |
| `src/v3/compiler/src/regen_parse_tables_emit.rs` | PB-Bootstrap-Process (same shape as `regen_parse_emit.rs`) *(proposed — see Findings)* |
| `src/v3/compiler/src/tokenize_char_class.rs` | R1 T-Sub `sub_charclass_in_std_unicode`. Retirement closes Class 5 Gap 3 → unblocks R2 T-Substrate 4th sub-lane (Director-coordinated per brief Hand-off points). |

## Manager note: first prototyped lane pick

Per Director Q2 ("manager-call after audit, lean PB-Substrate if
substrate.dag is evaluable today"): **substrate.dag is non-trivially
populated** (398 LOC, TERMINAL-marked types mirroring `dag.rs`'s runtime
enum surface). On the strength of that probe the Zero-Floor Manager
commits to **PB-Substrate as the first prototyped lane closure**
(Pre-promotion Deliverable 4) — `dag.rs` / `dag/ports.rs` / `dag/effects.rs`
generated from `substrate.dag` with cementing test, cited by the cascade
PR.

If the lane execution surfaces blockers (substrate-extension required,
emission gap, etc.), fall back to PB-1-a continuation per Director's
contingency note. STOP-AND-ESCALATE binds either way.

## Cross-refs

- Parent design: [`docs/design-pure-bootstrap-zero.md`](design-pure-bootstrap-zero.md) §"Pre-promotion deliverables"
- Manager brief: [`docs/briefs/pure-bootstrap-zero-manager.md`](briefs/pure-bootstrap-zero-manager.md) §"Sequence + dispatch"
- Subsumed brief: [`docs/briefs/pb-1-data-driven-bootstrap.md`](briefs/pb-1-data-driven-bootstrap.md) — non-goals invert under 0-floor (Pre-promotion Deliverable 2 amendment)
- Live count authority: `src/v3/compiler/tests/integration/sg0_census_test.rs` `EXPECTED_HAND_AUTHORED_NON_TEST`
