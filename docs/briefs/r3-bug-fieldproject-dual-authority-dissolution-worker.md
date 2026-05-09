# R3 Bug-Fix Worker Brief — FieldProject dual-authority dissolution

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope; worker dispatch via Substrate Mgr standing authority.
**Authority parent**: gpt-5-5-pro reflective analysis on `main@b09e0c8` Finding 2; PM dispatch at gunbc#846 #issuecomment-4413207527.
**Priority**: HIGH — concrete soundness bug; illegal-state-representable.

---

## §0. Problem statement

`TransformTarget::FieldProject` carries both a string label and an optional typed child declaration:

```rust
// src/v3/compiler/src/dag.rs:1890-1892
FieldProject {
    field_label: String,
    field_child: Option<DeclarationId>,
},
```

Substrate mirror at `src/v3/std/substrate.dag:311-316` repeats the dual shape.

The intended semantics (per code comments at `src/v3/compiler/src/dag.rs:1880-1888`): inference walks the input through `Instantiation`/`ResolvedIdentifier` edges, looks up `field_label` on the reached `Conj`, and populates `field_child` post-resolution.

**Concrete soundness bug**: actual code paths disagree on which field is authoritative:

- **Inference** (`src/v3/compiler/src/infer.rs:4198-4204`): trusts `field_child` directly when present
- **Builder** (`src/v3/compiler/src/dag/builder.rs:300-310`): only checks `field_child` exists, not that it matches `field_label` on the input's record type
- **Emission** (`src/v3/compiler/src/emit/rust_target.rs:3571-3594`): renders from the string label

So an inconsistent `FieldProject { field_label: "x", field_child: y_decl }` lets:
- inference resolve output as `y_decl` (uses `field_child`)
- emission project `"x"` (uses `field_label`)

Illegal state representable; consumers can disagree on what the projection means.

## §1. Required outcome

Split lifecycle OR collapse authority such that label and field_child cannot disagree.

## §2. Fix options

PM recommends **Option A** (split lifecycle) — explicit pre/post-resolution states; alignment enforced structurally.

### Option A — split lifecycle (preferred)

Replace single variant with:

```rust
FieldProject = UnresolvedFieldProject { field_label: String }
             | ResolvedFieldProject { field_ref: FieldDeclRef }
```

Lifecycle:
- Builder constructs `UnresolvedFieldProject { field_label }`
- Inference transitions `Unresolved → Resolved` via lookup against parent Conj; constructs `ResolvedFieldProject { field_ref }` where `field_ref` is a typed witness
- `transform_output_shape` requires `Resolved` (panics on `Unresolved` — invariant: post-infer, all FieldProjects are Resolved)
- Emission consumes `field_ref` directly; label retained only on `Unresolved` for diagnostic provenance OR extracted via `field_ref.label()` accessor

If `field_ref` needs to expose label (for diagnostic spans), make label a derived view, not a stored authority.

### Option B — collapse to single authority

Keep single `FieldProject` variant; drop `field_label` entirely after lookup. `field_child` becomes the single authority. Diagnostic provenance preserved via `SourceSpan`.

Smaller PR but loses diagnostic-quality info (debug error messages can't say "field 'x' not found" without re-walking declaration).

## §3. Files (expected scope)

**Option A**:
- `src/v3/std/substrate.dag` (TransformTarget shape — substrate authority)
- `src/v3/compiler/src/dag.rs` (Rust mirror at line 1890-1892)
- `src/v3/compiler/src/dag/builder.rs` (builder at line 300-310 + 762-766; constructs Unresolved)
- `src/v3/compiler/src/infer.rs` (inference at line 4198-4208; transitions Unresolved → Resolved)
- `src/v3/compiler/src/emit/rust_target.rs` (emission at line 3571-3594; consumes Resolved.field_ref)
- Cementing test pinning shape (`.dag` TestClaim form preferred)

**Option B**:
- Same files as Option A but smaller diffs (variant payload change only, not split)

## §4. Cross-cutting constraints

- **No new hand-Rust tests** — `.dag` TestClaim form preferred per locked design `docs/design-tests-as-data-completeness.md` §C5 / §C1.
- **STOP-and-PING via PM inbox (#846)** if substrate-shape change introduces variants that parser cannot lower (data-body limitations).
- **Substrate authority canonical**: `src/v3/std/substrate.dag` is canonical; Rust mirror at `dag.rs` reflects.

## §5. Receipt

When work lands:
- `FieldProject` cannot represent inconsistent label/child state (illegal state unrepresentable structurally)
- Cementing test pinning that inference + emission agree on field identity for any constructed FieldProject
- Substrate + Rust mirror synchronized
- SG-0 census: any new test entries marked with dissolution-trigger comment

## §6. Dispatch trigger

PM-authored brief; awaiting worker dispatch. Bug remains live on main until fixed.

## §7. Risk note

This is a **substrate-shape change** — touches `substrate.dag` + Rust mirror + 3 consumers. Coordinate with Substrate Mgr; brief authoring + dispatch likely needs cross-Mgr sign-off (Substrate authoring + Verification consumer-wiring) for safety. Single PR feasible if scope contained.

---

**End of brief.**
