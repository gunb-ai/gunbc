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

#### From `src/v3/compiler/tokenize.dag` → `std/tokenize.dag` (6 types, landed)

- `TokenKind` — ~35-variant sum of keyword/literal/punctuation kinds
- `Token` — `{ kind: TokenKind, span: SourceSpan }` record
- `KeywordTokenKind` — partition of TokenKind variants that are keywords
- `PunctTokenKind` — partition for punctuation
- `LocalPunctSpec` — tokenizer-local punctuation not in the shared syntax spec
- `StringEscapeSpec` — string-escape grammar rows

**Why shared:** any program that processes `.dag` source text (formatters, linters, IDE tooling, metaprogramming) needs the same token taxonomy. The compiler having its own `TokenKind` is a historical accident of authoring order, not a semantic requirement.

**Migration note:** `dsl/extdeps/languages/dag/syntax.dag` already carries the shared syntax authority (keyword names, operator symbols) that `regen_tokenize` reads. This migration landed by moving the six declarations into `src/v3/std/tokenize.dag`; `src/v3/compiler/tokenize.dag` now imports them and retains only scanner-local rows/data.

#### From `src/v3/compiler/runtime_mirrors.dag` → `src/v3/std/parse_surface.dag` (14 types, landed)

- `DagDifference`, `SurfaceModule`, `SurfaceParam`, `SurfaceField`, `SurfaceVariant`, `VariantPayload`, `SurfaceType`, `SurfaceRecordField`, `SurfaceMatchArm`, `SurfacePatternField`, `SurfacePattern`, `SurfaceLiteral`, `SurfaceExpr`, `SurfaceItem`

**Why shared:** the parse-surface AST is the representation any code-manipulation tool would consume. A user macro, a source transformer, an IDE-driven refactor — all would use the same `SurfaceExpr` / `SurfacePattern` shapes. The compiler carrying its own was dual-representation debt.

**Migration note:** Landed as `module v3.std.parse_surface` in `src/v3/std/parse_surface.dag`; the old `runtime_mirrors.dag` module is deleted. The 🟡 SCAFFOLD dissolution trigger for parse-rule authority is unchanged ("`parse_parser_body.txt` header emit algorithm from `.dag` alone + SG-2b/SG-3f parse-rule cutover"). Long-term, `dsl/std/` may subsume this file when the v2/v3 `std/` trees merge; the module path stays `v3.std.parse_surface` until then.

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
- **Lens-local return-type carriers in `src/v3/lenses/*.dag`** that represent the lens's published API shape (e.g., `Origin` for provenance, `UnusedParameter` for unused_parameters, `VariantPayloadShapeLookup` for variant_payload — 3-variant with the `NotPayloadProduct` semantic distinction, `TemplateArgumentBinding = Conflict | NoOp | Append` in infer_helpers). These declare what the lens returns and stay compiler-API *when genuinely lens-specific* — including 3-way or more carriers where the variants encode distinct semantic states, not just "missing vs found." **Strict 2-variant `Missing | Found(T)` Lookup pattern** — `complexity.dag`, `cost.dag`, and `infer_helpers.dag` all consume `v3.std.lookup::Lookup<…>` (imports); `infer_helpers`'s `template_argument_value` returns `Lookup<DeclarationId>` via `std.substrate::{miss_declaration_id_lookup, hit_declaration_id_lookup}` (substrate shims — same role as `miss_symbolic_cost_lookup` in `std.algebra` for `SymbolicCost`). Rust emit lowers these to `Lookup::Miss` / `Lookup::Hit`. No lens-local `Missing|Found` sum duplicates remain for this pattern. **Closure (compiler–std ratchet #642 surfaces only — counted `src/v3/compiler/*.dag` + `src/v3/lenses/*.dag` per `scripts/check-compiler-std-ratchet.sh`, with `parse_tables.dag` exempt):** migration-gate **closed**; P2 single-authority for strict Missing|Found Lookup-pattern carriers on that surface. (Same closure sentence as the ROADMAP consolidation row.) See ROADMAP for remaining workaround coproducts.
- Lens *bodies* (the `fn` definitions) in `src/v3/lenses/*.dag` — always positive-def; they import types from `std/` (and from `pipeline.dag` / `regen.dag` / lens-local carriers) rather than re-declaring them.
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
| SG-4b-4 (next infer helper tranche) | Aligned — infer_helpers imports types from substrate and `v3.std.lookup` |
| Algebra Quartet | Directly aligned — dissolves hand-rolled algebra ops into inhabitance pattern (fewer compiler-local types) |
| Pipeline Authority structural read | Directly aligned — `pipeline.dag` becomes authoritative, `pipeline_authority.rs` consumes the typed `PipelineStageBinding` carrier |

No lane in the wave introduces new dual-representation debt. Two (Algebra Quartet + Pipeline Authority) actively dissolve it. The wave is wave-neutral-to-positive on this end goal.

Against the deferred-debt section:

- `algebra.dag` signature/comment reconciliation — aligned
- `container_template_algebra_rows` dissolution — aligned (narrow form landed in #651; the renamed table remains as a bridge until `.dag` alias reflection derives it from the type aliases)
- `pipeline_authority` (in wave) — aligned
- Emitter render-helper consolidation — neutral
- Stale cross-refs — cosmetic
- SFP subdoc drift — cosmetic
- CI ratchet audit — not architectural
- Stale-receipt sweep — cosmetic
- Complexity v2/v3 comparison (narrowed to structural_depth rename+cement) — aligned (renames compiler-specific `cost` registry entry to match actual content)

## Ratchet definition

The consolidation can be measured:

**Primary ratchet:** count of `type` declarations in `src/v3/compiler/*.dag` **and `src/v3/lenses/*.dag`** that are NOT in the positive-definition set AND NOT exempted.

Positive-definition set (NOT counted against the ratchet):
- `pipeline.dag` types, `regen.dag` types
- Lens-local return-type carriers that represent the lens's published API (e.g., `Origin`, `UnusedParameter`, `CostEntry`) — **except** generic-Lookup-pattern duplicates (see below)
- Substrate reflection accessor declarations

Exempted (pending named trigger, not counted either direction):
- `parse_tables.dag` — 7 types, exempted pending SG-2c-proper per-row classification

Ratchet-tracked (**migrates to `std/`, counted against the baseline**):
- Compiler-side types that duplicate user-facing concepts (tokenize types moved; parse-surface family moved — remaining debt is lens Lookup/coproduct scaffolds listed in the inventory)
- Strict 2-variant `Missing | Found(T)` Lookup-pattern carriers declared **in** `src/v3/lenses/*.dag` — **none** on the tracked lens/compiler DAG surfaces (`complexity`, `cost`, and `infer_helpers` import `v3.std.lookup::Lookup`). Carriers with additional semantic variants (e.g., `VariantPayloadShapeLookup`'s `NotPayloadProduct`, `TemplateArgumentBinding = Conflict | NoOp | Append`) are lens-API, not in-ratchet.
- Lens-local workaround-shaped coproducts with named dissolution triggers (today: `TemplateArgumentsMatch`, `TemplateArgumentCursor` in `infer_helpers.dag`) also count against the ratchet; these are implementation scaffolds, not genuine lens-API carriers.

Baseline (2026-04-22, measured via `grep -cE "^type [A-Z]"` after the tokenizer migration; **primary ratchet updated 2026-04-22 — consolidation tranche 2** after `parse_surface` migration):

| File | Type decls | Disposition |
|---|---|---|
| `src/v3/compiler/parse_tables.dag` | 7 | exempted pending SG-2c-proper |
| `src/v3/compiler/operators.dag` | 0 | — |
| `src/v3/compiler/pipeline.dag` | 3 | positive-def |
| `src/v3/compiler/regen.dag` | 1 | positive-def |
| `src/v3/lenses/complexity.dag` | 1 | 1 positive-def (`CostEntry`); return surface is imported `v3.std.lookup::Lookup` (not a lens-local `type` decl) |
| `src/v3/lenses/cost.dag` | 1 | 1 positive-def (`SymbolicCostEntry`); return surface is imported `v3.std.lookup::Lookup<SymbolicCost>` (not a lens-local `type` decl) |
| `src/v3/lenses/idempotency.dag` | 0 | — |
| `src/v3/lenses/infer_helpers.dag` | 4 | 3 in-ratchet (`TemplateArgumentsMatch`, `TemplateArgumentCursor`, `NormalizedInstantiationArgs` — workaround-shaped coproduct scaffolds with named dissolution triggers) + 1 positive-def (`TemplateArgumentBinding = Conflict \| NoOp \| Append \| ReplaceAt` — semantic carrier); template-argument presence uses imported `Lookup<DeclarationId>` |
| `src/v3/lenses/lower_helpers.dag` | 0 | — |
| `src/v3/lenses/parallelism.dag` | 0 | — |
| `src/v3/lenses/provenance.dag` | 1 | positive-def (`Origin` — provenance-specific) |
| `src/v3/lenses/structural_resolution.dag` | 2 | both positive-def (`UnresolvedArrowBody` and `NameKeyedReference` — both are lens-API record types carrying the lens's findings, not Lookup patterns) |
| `src/v3/lenses/unused_parameters.dag` | 1 | positive-def (`UnusedParameter`) |
| `src/v3/lenses/variant_payload.dag` | 2 | both positive-def (`VariantPayloadShape` domain type + `VariantPayloadShapeLookup` — 3-variant carrier with `NotPayloadProduct` semantic distinction, not generic Lookup) |

**Primary ratchet count today: 3** (0 strict 2-variant Lookup-pattern carriers on lens surfaces — `complexity`, `cost`, and `infer_helpers` import `v3.std.lookup::Lookup` + 3 workaround-shaped infer-helper coproducts: `TemplateArgumentsMatch`, `TemplateArgumentCursor`, `NormalizedInstantiationArgs`). The tokenizer migration removed 6 compiler-local type declarations by moving them to `src/v3/std/tokenize.dag` (25 → 19). Consolidation tranche 2 removed 14 by moving the parse-surface family to `src/v3/std/parse_surface.dag` (19 → 5). The SG-4b callable-instantiation normalization cluster adds `NormalizedInstantiationArgs` as a tracked workaround carrier (Option-as-enum, paired to dissolve alongside `TemplateArgumentsMatch` when the lens→Rust emitter lands verified `T?` / `Bool` return mappings). All lens-local types now classified — `structural_resolution.dag`'s two record types (`UnresolvedArrowBody`, `NameKeyedReference`) are positive-def lens-API carrying the lens's findings.

End state: 0. Each migration lane reduces the count; positive-definition types track growth separately and are not bounded downward by this ratchet.

**`parse_tables.dag` exemption protocol.** The gap analysis above (§"From `src/v3/compiler/parse_tables.dag` → unclear") says some row *shapes* (`BinaryOpRow`, `TopLevelItemKwRow`, `SoftKeywordIdentRow`, `PrimaryPrefixRow`, `PrimaryAtomRow`, etc.) may legitimately stay compiler-API as parser-dispatch shapes, while the *data* they carry (operator precedence levels, keyword-to-item mappings) moves to `std/syntax.dag`. That per-row classification is deferred until SG-2c-proper parser cutover. While deferred, `parse_tables.dag` is **exempted from the primary ratchet count** — its 7 types neither count toward the migration target nor against it. When SG-2c-proper classifies:
- Rows classified as **compiler-API dispatch shapes** → formally added to the positive-definition set (same status as `pipeline.dag` / `regen.dag` types).
- Rows classified as **language-level data** → become ratchet-tracked migrations to `std/syntax.dag`.

The exemption dissolves at SG-2c-proper completion; both thesis doc and ROADMAP row update the ratchet formula in the same PR as SG-2c-proper lands.

**Secondary ratchet — v3-std tree consolidation.** Count of `type` declarations in `src/v3/std/*.dag`. Baseline: 162 types across 15 files after adding `src/v3/std/parse_surface.dag` (the post-tokenize 148 plus 14 migrated surface-schema types). These collapse to `dsl/std/*.dag` wholesale when the file-preference scaffold dissolves (ROADMAP: v2 retirement gate). This ratchet is gated by that dissolution, not per-lane moves.

**Tertiary ratchet — hand-Rust surface.** This doc does not duplicate the Pure Bootstrap ratchet; it anchors to it. The authoritative measure is **SG-0's** `(EXPECTED_HAND_AUTHORED_NON_TEST ∪ EXPECTED_HAND_AUTHORED_TEST ∪ EXPECTED_HAND_AUTHORED_FRAGMENTS) ∖ GENERATED_FILES` (live in `src/v3/compiler/tests/integration/sg0_census_test.rs`); [docs/design-pure-bootstrap.md](../design-pure-bootstrap.md)'s PB-0 ratchet is the tracking program. **The count is whatever the live ratchet test reads today** — any number in prose drifts stale. For a current snapshot, run the SG-0 census test directly; do not approximate via `grep` on the expected lists alone, because the ratchet subtracts `GENERATED_FILES` at runtime and splits non-test vs test paths — grep counts the pre-subtraction set, not the authoritative post-subtraction set. Target: **≤5** on the **non-test** subset per Pure Bootstrap's irreducible-shim goal. Concrete hand-Rust that still needs to dissolve: parse algorithm, lower algorithm body, infer algorithm body, emit backbone, bootstrap shim, and lens-adjacent Rust files (some of which are Band-C-STUB backs per the lens capability register — dissolving when substrate carriers and emit `match` capabilities land).

## How this claim composes with existing tracked work

- **File-preference scaffold dissolution trigger** (ROADMAP post-merge-debt) — the mechanism that unlocks most of the `src/v3/std/*` → `dsl/std/*` consolidation. When v2 retires or `dsl/std/` can load v3 grammar, the bulk of compiler-side type declarations move wholesale.
- **Pure Bootstrap design** — the algorithmic-emission capability program that unlocks moving compiler *logic* (not just types) out of hand-Rust. When v3 can emit `match`-on-user-sums and recursive list-folds, more algorithms move to `.dag`.
- **Lens capability register** — tracks whether each lens is behaviorally complete; a PROXY/STUB lens can't yet substitute for its v2 counterpart. Independent of this claim but complements it: both are "honesty about the current gap" tools.
- **`project_node_to_std` memory** — specific prior instance of this pattern (move `Node` to `std/`).

## Discipline

1. **Every new type declared in `src/v3/compiler/*.dag`, `src/v3/lenses/*.dag`, or `src/v3/std/*.dag` requires a home-check:** does it belong under the positive definition (pipeline / regen / lens-specific return-type carrier / accessor)? If yes, declare here. If no, declare in `dsl/std/` (or `src/v3/std/` with a migration trigger). **Lens-local types are positive-def only when genuinely lens-specific** — if the type matches a generalizable pattern (e.g., 2-variant `Missing | Found(T)` Lookup), it counts against the ratchet and the Lookup generic is the dissolution target.
2. **Every brief that introduces new compiler-side types must cite this doc** and classify each new type as (a) pure compiler API or (b) scheduled migration to `std/` with a named trigger.
3. **The ratchet-only-down rule applies:** the count of non-positive-definition types in compiler-side files can only decrease except during a tracked upstream substrate-extension lane.
4. **Downgrades are first-class:** a type that was declared in `std/` and then duplicated compiler-side for any reason needs an explicit justification and a dissolution trigger, not silent drift.

## Related docs

- [THESIS.md](../../THESIS.md) — parent thesis; this doc is a claim elaboration
- [docs/design-pure-bootstrap.md](../design-pure-bootstrap.md) — the algorithmic-emission side of the bootstrap program
- [docs/v3-lens-capability-register.md](../v3-lens-capability-register.md) — analysis-side honesty register (parallel discipline, different axis)
- [INVARIANTS.md](../../INVARIANTS.md) §P1, §P2, §P5 — the invariants this claim extends
- [epistemic-stacking.md](epistemic-stacking.md) — every concept grounds in primitives via an ontological DAG (the in-repo long-form of the grounding claim this doc extends to the compiler/user boundary)
