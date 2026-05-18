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

## 3. Driver / registry CLI — **v0 slot** (stub until binary lands)

**Invocation template (frozen string for Acceptance tables):**

```text
gunbc-prefix-lens-driver v0 <LENS_ID> --path <FILE.dag> [--mode enforce|introspect]
```

**`LENS_ID` registry (v0 — extend only via operator-signed pin amend):**

| `LENS_ID` | Lens module (v4) | Notes |
|-----------|------------------|--------|
| `complexity` | `v4.lens.complexity` | Wave-1 **#1** (operator). |
| `cost` | `v4.lens.cost` | PREFIX reference / SymbolicCost algebra. |
| `parallelism` | `v4.lens.parallelism` | |
| `effect_enumeration` | `v4.lens.effect` | |
| `idempotency` | `v4.lens.idempotency` | |
| `provenance` | `v4.lens.ownership` | structural-readers batch; module name = ownership.dag today. |
| `unused_parameters` | *(TBD v4 module)* | placeholder — ratify module path before impl. |
| `structural_resolution` | *(TBD v4 module)* | placeholder — ratify module path before impl. |

**Runnable-AC column:** until the Rust binary exists, use **`TBD — gunbc-prefix-lens-driver v0 …`** or interim **`v2-compiler compile --source-root src/v4`** + v3 fold per worker brief Fork A — **do not** claim CLI is live without receipt.

## 4. Mechanical port backlog (explicit — not blocking witness authoring)

The following are **not** in this pin’s frozen **names** (they are **implementation debt** to wire carriers to a compiling `application.dag` body):

- **`Lens<Output>`** primitive in v4 `std` (v4 has no `v4.std.lens` yet; v3 uses `Dag`, `Behavior`, `LoopBound`, `OptionalDiagnostic`, …).
- **`DeclarationId` / `NodeId` / `SourceSpan`** as v4-first-class types (today: v3 `substrate_minimal` / design references).

**Acceptance-PR substance** (witness `.dag`, issue-class, clean counter) stays **interface-independent**; only rows that assert **v4 parse-tree field names** or **live driver argv** depend on §3 / mechanical port.

## 5. Amend discipline

Revision of §1–§3 requires **operator-signed** amendment (same discipline as PREFIX Acceptance PRs).
