# Compiler-`std/` Consolidation — the "no dual representations" end state

> Parent: [THESIS.md](../../THESIS.md)
> Related: [epistemic-stacking.md](epistemic-stacking.md), [structural-decompression.md](structural-decompression.md), [what-else-falls-out.md](what-else-falls-out.md)
> Operational: [Pure Bootstrap design](../design-pure-bootstrap.md), [lens capability register](../v3-lens-capability-register.md)

## Claim

**In the mature v3 state, the only things that exist are (1) `std/` concepts and (2) a minimal set of pure `.dag` compiler APIs.** A "token" in the compiler is not a special compiler-only token — it is the same `Token` a user's formatter or linter would use. A "file" is the same `File` any program reads. Compiler-specific taxonomies that duplicate concepts users would also need are **dual representations at the compiler/user boundary** — a P2 Boundary Discipline violation viewed at the architectural layer. This end state dissolves that boundary.

## Why this claim is load-bearing

The existing invariants already forbid dual representations *within* the compiler (no parallel authority for the same fact) and *within* user-land (single authority in `std/`). Silent exception: **between** the compiler and user-land. A compiler that carries its own `TokenKind` separate from any `std/Token` creates a layered-opacity that only exists because of historical bootstrapping convenience.

Three invariants this claim extends:

- **P1 Modeling Faithfulness**: compiler-specific types that duplicate user-facing concepts are *ungrounded internal taxonomies* — violating the requirement that every construct ground in an external or structural source. A compiler `TokenKind` has no external authority beyond "what the parser happened to need"; a shared `std/Token` grounds in lexical-analysis consensus (Aho/Sethi/Ullman, regex theory, etc.).
- **P2 Boundary Discipline**: the compiler/user layer boundary is currently imposed by bootstrap firewall, not by semantic need. Single-authority discipline says dissolve self-imposed boundaries that no longer serve correctness.
- **P5 Progress Is Dissolution**: every compiler-specific type that duplicates a user-facing concept is a bridge — a scaffold with an implicit dissolution trigger ("when bootstrap allows, move to `std/`"). Making the trigger explicit is the tracked-debt discipline.

The Pure Bootstrap thesis ("compiler is authored in `.dag`, Rust is generated, stage0 shrinks to a tiny shim") implies this consolidation: a Pure Bootstrap compiler that still carries compiler-specific ontology is not bootstrapped purely, it's bootstrapped self-referentially.

## The positive definition: what stays compiler-specific

A small set of concepts genuinely belong to the compiler because they describe the compiler's own architecture, not the programs it processes:

- **Pipeline orchestration** — `src/v3/compiler/pipeline.dag`. The stages the compiler runs in order, their dependencies, their snapshot boundaries. `PipelineStageBinding`, `PipelineSnapshotKind`, `CompilerHostRealization`. No user program has "tokenize → parse → lower → infer → emit" as its own ordered pipeline in this shape.
- **Code-generation registry** — `src/v3/compiler/regen.dag`. The map from `.dag` declarations to generated Rust output paths. `LensRegistryEntry`. Compiler-specific because only the compiler is generating code for itself.
- **Lens bodies** — the analyses the compiler ships with, under `src/v3/lenses/*.dag`. Users can write similar analyses, but the compiler's *bundled set* is its API surface.
- **Substrate reflection accessors** — `declaration_by_id`, `port`, `node`, `resolve_producer`, `lane2_workflow_at`. Reads over the Dag the compiler is currently operating on. Users can access *their own Dags* the same way, but the compiler's runtime accessor set is its API.
- **Bootstrap shim** — `src/v3/compiler/src/` at minimal steady state (target: ≤5 hand-maintained Rust files). The irreducible entry point.

Everything else moves to `std/`: Token, File, Path, SourceSpan, Diagnostic, Identifier, Behavior, TypeConnective, Declaration, DagPort, CardinalityBound, ArrowBody (when stable), and any future concept that can be used by both the compiler and user programs.

## Gap analysis — current v3 state vs end state

### Compiler-specific types that SHOULD migrate to `std/` (dual-representation debt)

#### From `src/v3/compiler/tokenize.dag` → `std/tokenize.dag` (6 types)

- `TokenKind` — ~35-variant sum of keyword/literal/punctuation kinds
- `Token` — `{ kind: TokenKind, span: SourceSpan }` record
- `KeywordTokenKind` — partition of TokenKind variants that are keywords
- `PunctTokenKind` — partition for punctuation
- `LocalPunctSpec` — tokenizer-local punctuation not in the shared syntax spec
- `StringEscapeSpec` — string-escape grammar rows

**Why shared:** any program that processes `.dag` source text (formatters, linters, IDE tooling, metaprogramming) needs the same token taxonomy. The compiler having its own `TokenKind` is a historical accident of authoring order, not a semantic requirement.

**Migration note:** `dsl/extdeps/languages/dag/syntax.dag` already carries the shared syntax authority (keyword names, operator symbols) that `regen_tokenize` reads. The natural home for `Token` / `TokenKind` is a `std/tokenize.dag` that composes from that existing shared spec.

#### From `src/v3/compiler/runtime_mirrors.dag` → `std/syntax.dag` (14 types)

- `DagDifference`, `SurfaceModule`, `SurfaceParam`, `SurfaceField`, `SurfaceVariant`, `VariantPayload`, `SurfaceType`, `SurfaceRecordField`, `SurfaceMatchArm`, `SurfacePatternField`, `SurfacePattern`, `SurfaceLiteral`, `SurfaceExpr`, `SurfaceItem`

**Why shared:** the parse-surface AST is the representation any code-manipulation tool would consume. A user macro, a source transformer, an IDE-driven refactor — all would use the same `SurfaceExpr` / `SurfacePattern` shapes. The compiler carrying its own is pure dual-representation debt.

**Migration note:** `runtime_mirrors.dag` itself calls these out as 🟡 SCAFFOLD with a dissolution trigger ("`parse_parser_body.txt` header emit algorithm from `.dag` alone + SG-2b/SG-3f parse-rule cutover"). The migration target is `dsl/std/syntax.dag` (already exists as a partial syntax authority) or a new `dsl/std/parse_surface.dag`.

#### From `src/v3/compiler/parse_tables.dag` → unclear

- `BinaryOpLevel`, `BinaryOpRow`, `TopLevelItemKwRow`, `BracketRow` (4 types)

**Status: debatable.** These are parser-dispatch data tables. On one hand, expression precedence and keyword-dispatch are language properties that tools beyond the compiler might want to read (e.g., a syntax highlighter). On the other hand, the specific row *shapes* are parser-internal. A reasonable split: move the *data* (operator levels, keyword-to-item mappings) to `std/syntax.dag`, keep the dispatch-row *shapes* compiler-specific.

**Named decision trigger: SG-2c proper cutover.** The decision about which rows are "parser internal dispatch" vs "language-level syntax facts" is genuinely under-determined until the parser itself moves to `.dag` authority — only then can we see which rows the `.dag` parser actually dispatches on (stays compiler-API) versus which are pure language data downstream tools should consume (moves to `std/syntax.dag`). Whoever picks up SG-2c proper's parser cutover is the owner of this decision. **Precedent rule:** if any individual row needs to move before SG-2c proper lands, the mover-lane sets the classification for that row and the others follow the same rule when touched.

#### v3-specific `std/` tree → unified `dsl/std/`

Every file under `src/v3/std/*.dag` is a migration candidate once the bootstrap firewall dissolves:

- `substrate.dag`, `substrate_minimal.dag` — the substrate primitives. **Already the largest single consolidation target.**
- `algebra.dag`, `effects.dag`, `workflows.dag`, `verification.dag`, `resources.dag`, `dimensions.dag`, `diagnostics.dag`, `computation_model.dag`, `emit_model.dag`, `clean_emission.dag`, `list.dag`

**Migration gate:** file-preference scaffold dissolution trigger — tracked in ROADMAP as the v2 retirement. Once `dsl/std/` can load v3 grammar (or v2 retires and `dsl/std/` is replaced wholesale), both std trees merge.

### Compiler-specific that stays (positive definition satisfied)

- `src/v3/compiler/pipeline.dag` — 3 types (`PipelineStageBinding`, `PipelineSnapshotKind`, `CompilerHostRealization`)
- `src/v3/compiler/regen.dag` — 1 type (`LensRegistryEntry`)
- Lens bodies at `src/v3/lenses/*.dag` — bodies only (they import types from `std/` post-consolidation)
- Reflected accessors declared in `substrate.dag` (these move to `std/` but their *existence as compiler API* stays)

### Rust hand-written files (bootstrap shim target)

The Pure Bootstrap design doc targets ≤5 hand-maintained Rust files. Current state has ~5-10 substantial hand-Rust files (parser body algorithm, lower algorithm body, infer algorithm body, emit backbone, bootstrap.rs). Each has a named capability-gap reason; each dissolves when its gap closes.

### Upcoming-lane alignment check

Against the 2026-04-22 wave SG manager dispatched:

| Lane | Alignment with end state |
|---|---|
| Lane 1e Phase 3.1 (render-helper consolidation) | Neutral — reduces duplicate render helpers, doesn't introduce new compiler types |
| SG-3g-c (next lower helper slice) | Aligned — lower_helpers imports types from substrate (already `std/`-shaped) |
| SG-2c-5 (next parser table extraction) | Neutral — extends `parse_tables.dag` which may itself be partly `std/`-bound per above |
| SG-4b-4 (next infer helper tranche) | Aligned — infer_helpers imports types from substrate |
| Algebra Quartet | Directly aligned — dissolves hand-rolled algebra ops into inhabitance pattern (fewer compiler-local types) |
| Pipeline Authority structural read | Directly aligned — `pipeline.dag` becomes authoritative, `pipeline_authority.rs` consumes the typed `PipelineStageBinding` carrier |

No lane in the wave introduces new dual-representation debt. Two (Algebra Quartet + Pipeline Authority) actively dissolve it. The wave is wave-neutral-to-positive on this end goal.

Against the deferred-debt section:

- `algebra.dag` signature/comment reconciliation — aligned
- `container_to_algebra` dissolution — aligned (eliminates a string-keyed dual representation)
- `pipeline_authority` (in wave) — aligned
- Emitter render-helper consolidation — neutral
- Stale cross-refs — cosmetic
- SFP subdoc drift — cosmetic
- CI ratchet audit — not architectural
- Stale-receipt sweep — cosmetic
- Complexity v2/v3 comparison (narrowed to structural_depth rename+cement) — aligned (renames compiler-specific `cost` registry entry to match actual content)

## Ratchet definition

The consolidation can be measured:

**Primary ratchet:** count of `type` declarations in `src/v3/compiler/*.dag` outside of `pipeline.dag`, `regen.dag`, and `lenses/*.dag`.

Baseline (2026-04-22, measured by `grep -cE "^type [A-Z]" src/v3/compiler/*.dag`):

| File | Type decls | Category |
|---|---|---|
| `tokenize.dag` | 6 | **migrates to `std/`** |
| `runtime_mirrors.dag` | 14 | **migrates to `std/`** |
| `parse_tables.dag` | 4 | **migrates** (per-type decision deferred; some may stay compiler-API as dispatch-row shapes) |
| `operators.dag` | 0 | — |
| `pipeline.dag` | 3 | positive-def: stays |
| `regen.dag` | 1 | positive-def: stays |

**Primary ratchet count today: 24 (tokenize + runtime_mirrors + parse_tables).**

End state: 0. Each migration lane reduces the count; `pipeline.dag` and `regen.dag` type counts track positive-definition growth separately and are not bounded downward by this ratchet.

**Secondary ratchet — v3-std tree consolidation.** Count of `type` declarations in `src/v3/std/*.dag`. Baseline: 142 types across 13 files (substrate.dag 38, emit_model.dag 28, effects.dag 26, clean_emission.dag 12, computation_model.dag 10, verification.dag 8, substrate_minimal.dag 6, dimensions.dag 4, algebra.dag 3, diagnostics.dag 3, list.dag 2, resources.dag 2, workflows.dag 0). These collapse to `dsl/std/*.dag` wholesale when the file-preference scaffold dissolves (ROADMAP: v2 retirement gate). This ratchet is gated by that dissolution, not per-lane moves.

**Tertiary ratchet — hand-Rust surface.** This doc does not duplicate the Pure Bootstrap ratchet; it anchors to it. The authoritative measure is **SG-0's `EXPECTED_HAND_AUTHORED ∖ GENERATED_FILES`** (live in `src/v3/compiler/tests/integration/sg0_census_test.rs`), also surfaced in [docs/design-pure-bootstrap.md](../design-pure-bootstrap.md)'s PB-0 ratchet. Current per PB design: **78 hand-maintained `.rs` files in `src/v3/compiler/`**. Target: **≤5** (the irreducible shim). Concrete hand-Rust that still needs to dissolve: parse algorithm, lower algorithm body, infer algorithm body, emit backbone, bootstrap shim, and ~20 lens-adjacent Rust files (some of which are Band-C-STUB backs per the lens capability register — dissolving when substrate carriers and emit `match` capabilities land).

## How this claim composes with existing tracked work

- **File-preference scaffold dissolution trigger** (ROADMAP post-merge-debt) — the mechanism that unlocks most of the `src/v3/std/*` → `dsl/std/*` consolidation. When v2 retires or `dsl/std/` can load v3 grammar, the bulk of compiler-side type declarations move wholesale.
- **Pure Bootstrap design** — the algorithmic-emission capability program that unlocks moving compiler *logic* (not just types) out of hand-Rust. When v3 can emit `match`-on-user-sums and recursive list-folds, more algorithms move to `.dag`.
- **Lens capability register** — tracks whether each lens is behaviorally complete; a PROXY/STUB lens can't yet substitute for its v2 counterpart. Independent of this claim but complements it: both are "honesty about the current gap" tools.
- **`project_node_to_std` memory** — specific prior instance of this pattern (move `Node` to `std/`).

## Discipline

1. **Every new type declared in `src/v3/compiler/*.dag` or `src/v3/std/*.dag` requires a home-check:** does it belong under the positive definition (pipeline/regen/lens-body/accessor)? If yes, declare here. If no, declare in `dsl/std/` (or `src/v3/std/` with a migration trigger).
2. **Every brief that introduces new compiler-side types must cite this doc** and classify each new type as (a) pure compiler API or (b) scheduled migration to `std/` with a named trigger.
3. **The ratchet-only-down rule applies:** the count of non-positive-definition types in compiler-side files can only decrease except during a tracked upstream substrate-extension lane.
4. **Downgrades are first-class:** a type that was declared in `std/` and then duplicated compiler-side for any reason needs an explicit justification and a dissolution trigger, not silent drift.

## Related docs

- [THESIS.md](../../THESIS.md) — parent thesis; this doc is a claim elaboration
- [docs/design-pure-bootstrap.md](../design-pure-bootstrap.md) — the algorithmic-emission side of the bootstrap program
- [docs/v3-lens-capability-register.md](../v3-lens-capability-register.md) — analysis-side honesty register (parallel discipline, different axis)
- [INVARIANTS.md](../../INVARIANTS.md) §P1, §P2, §P5 — the invariants this claim extends
- [epistemic-stacking.md](epistemic-stacking.md) — every concept grounds in primitives via an ontological DAG (the in-repo long-form of the grounding claim this doc extends to the compiler/user boundary)
