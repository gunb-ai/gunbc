# Lane A PREFIX — **Interface-Freeze pin** (T-23 keystone — operator go 2026-05-18)

> **Broadcast point:** parallel Acceptance authoring + driver/registry work pins **here** until superseded by operator-signed amendment. **Authority:** `docs/design-lens-application-surface.md` §2 (carriers), §5.1 (default `IntrospectApplication<ComplexitySummary>` synthesis), `src/v4/lens/application.dag` header (D1 ops + advisory→fail-closed bridge).

## 1. Frozen carrier shapes (normative digest — do not drift)

**`SectionRef`** — disjoint sum:

- `DeclarationScope { declaration: DeclarationId }`
- `NodeScope { declaration: DeclarationId, node: NodeId }`

**`DiagnosticSeverity`** — single admitted steady-state variant: `Error` (C-8).

**`LensEnforcement<Output, Budget, Projected>`** — `{ project: fn(Output) -> Projected, violates: fn(Output, Budget) -> Bool }`.

**`EnforceableLens<Output, Budget, Projected>`** — `{ lens: Lens<Output>, enforcement: LensEnforcement<Output, Budget, Projected> }`.

**`EnforcedApplication<Output, Budget, Projected>`** — `{ enforceable_lens: EnforceableLens<Output, Budget, Projected>, section: SectionRef, budget: Budget, diagnostic_severity: DiagnosticSeverity, span: SourceSpan }`.

**`IntrospectApplication<Output>`** — `{ lens: Lens<Output>, section: SectionRef, span: SourceSpan }` (no budget, no enforcement metadata).

**Two top-level carriers** — not a sum; fold walks **two separate lists** (design doc §5).

**D1 operations** (operation home `src/v4/lens/application.dag`; data in `std/node.dag`): `subterm_at`, `apply_diff` — signatures and fail-closed discipline per that file’s header.

## 2. Synthesis default (complexity — operator dev-speed lever)

Per **`docs/design-lens-application-surface.md` §5.1**: for **unannotated** function declarations, the compiler **synthesizes** implicit **`IntrospectApplication<ComplexitySummary>`** during the lens fold (not stored in source). **Never** auto-synthesizes `Enforce` for unannotated functions. Acceptance batches that touch complexity **must** cite §5.1 when stating “always-on introspection.”

## 3. Registry + frozen argv (v0 — **substrate receipt today**; **executable deferred B2**)

**Invocation template (frozen string for Acceptance tables — intent until an in-tree consumer lands):**

```text
gunbc-prefix-lens-driver v0 <LENS_ID> --path <FILE.dag> [--mode enforce|introspect]
gunbc-prefix-lens-driver v0 <LENS_ID> --whole-corpus
```

**In-tree executable / interim Rust gate:** **None** — operator relay **2026-05-18** (*witty-cat-59* via *fierce-cat-31*, Lane A CP mgr): **no** standalone `tools/gunbc_prefix_lens_driver/` crate; **no** `gunbc_ci.rs` interim fold (**B1** off). **B2** only: a **shared v2 corpus-enumeration / filesystem-walk primitive** (**T-21/T-24**, owner coordination **#3322 §5**) consumed by **`.dag`**; PREFIX is the **first consumer**, not a second walker. **Accepted latency:** no CI step may claim a live `target/release/gunbc-prefix-lens-driver` receipt until that substrate ships.

**`LENS_ID` registry (v0 — extend only via operator-signed pin amend):**

Closed identifiers are **not** modeled as free `String` in substrate: **`LensIdV0`** in **`src/v4/lens/registry.dag`** is the closed sum (`Complexity` \| `Cost` \| … \| `StructuralResolution`) per **MODELING.md M4** (“Closed sets are enums, not strings”). The §3 table spells the same set in **argv / prose spelling** (what drivers and humans type); each row’s structural variant is the authoritative key inside `LensRegistryEntryV0 { lens_id: LensIdV0, module_path: LensModulePathV0 }`.

| `LENS_ID` (argv / table) | `LensIdV0` variant | Lens module (v4) | Notes |
|--------------------------|--------------------|------------------|--------|
| `complexity` | `Complexity` | `v4.lens.complexity` | Wave-1 **#1** (operator). |
| `cost` | `Cost` | `v4.lens.cost` | PREFIX reference / SymbolicCost algebra. |
| `parallelism` | `Parallelism` | `v4.lens.parallelism` | |
| `effect_enumeration` | `EffectEnumeration` | `v4.lens.effect` | |
| `idempotency` | `Idempotency` | `v4.lens.idempotency` | |
| `provenance` | `Provenance` | `v4.lens.ownership` | structural-readers batch; module name = ownership.dag today. |
| `unused_parameters` | `UnusedParameters` | `v4.lens.unused_parameters` | |
| `structural_resolution` | `StructuralResolution` | `v4.lens.structural_resolution` | T-13 mirror — InferredTree + dependency projection (BindsTo use edges, T-9 resolve stamps). |

**Substrate home-of-record (P2-staging — INVARIANTS §P2 / Practice 5):** **`src/v4/lens/registry.dag`** is the **canonical `.dag` surface** for closed rows: **`lens_id: LensIdV0`** (M4 closed set) plus **`module_path: LensModulePathV0`** as `Bound { path: … }` vs `Unbound`. A **landed** compiler **single authority** in the §P2 sense (declaration + realization + **generated** consumer) **does not exist yet** — same staging posture as `src/v4/std/fact_density.dag` until mechanical read; the paired **`v4_lens_registry_dag_smoke_test.rs`** receipt is **parse cleanliness only** plus bounded **source-text** receipts for the closed `LensIdV0` set and key `Bound` rows (INVARIANTS §P2 staging; **not** an inference-stage or `compile_to_dag` guarantee — post-#3503 P9 `Symbol`/`List` imports defer single-file lowering until M2 multi-module load, same posture as `v4_bin_main_dag_smoke_test`). The §3 markdown table is a **human mirror**; amend the `.dag` first, then align this table on operator-signed pin revision. Rows whose ratified v4 module is not yet fixed use **`Unbound`** in the substrate (same intent as *(TBD v4 module)* in the table); **never** encode that state as a fake `v4.lens.*` string or other string sentinel inside `path`.

**Runnable-AC column (today):** Rows requiring **closed `LENS_ID` names** cite **`src/v4/lens/registry.dag`**. Rows requiring **v4 parse bootstrap** cite the existing **v2 → v4** job in **`.github/workflows/ci.yml`** (`v2-compiler compile --source-root src/v4 …`). Rows that assert a **live CLI `LENS_ID` / whole-corpus compile gate** are **deferred** until **B2** — do not substitute an interim binary receipt. **Lens evaluation** (`Witness` / `DimensionOk` / `DimensionFail` on applied v4 `Lens<Output>` with **`--mode enforce|introspect`**) stays **`TBD …`** until the evaluation dispatch lands — do not treat compile-only receipts as evaluation substitutes.

## 4. Mechanical port backlog (explicit — not blocking witness authoring)

The following are **not** in this pin’s frozen **names** (they are **implementation debt** to wire carriers to a compiling `application.dag` body):

- **`Lens<Output>`** primitive in v4 `std` (v4 has no `v4.std.lens` yet; v3 uses `Dag`, `Behavior`, `LoopBound`, `OptionalDiagnostic`, …).
- **`DeclarationId` / `NodeId` / `SourceSpan`** as v4-first-class types (today: v3 `substrate_minimal` / design references).

**Acceptance-PR substance** (witness `.dag`, issue-class, clean counter) stays **interface-independent**; only rows that assert **v4 parse-tree field names** or **live driver argv** depend on §3 / mechanical port.

## 5. Amend discipline

Revision of §1–§3 requires **operator-signed** amendment (same discipline as PREFIX Acceptance PRs).
