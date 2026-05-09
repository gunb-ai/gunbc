# R3 Novel-Finding Worker Brief — F2 `ShapeATarget` closed enum vs `LanguageSpec` extensibility

**Owner**: Grounding Mgr (sunny-koi-893 / gunbc#2063) lane scope; coordinate with Substrate Mgr (warm-wolf-698 / gunbc#2068) on carrier consumption shape.
**Authority parent**: gpt-5-5-pro reflective analysis Finding 2; PM dispatch at gunbc#846 c#4413701937 (operator authorized 2026-05-09).
**Priority**: HIGH — Class F ontology duplication; closed enum constrains thesis extensibility claim.

---

## §0. Problem statement

`ShapeATarget = Rust | Python | Go` is a closed sum type used at `src/v3/grounding_cross_target_meta/src/cells.rs` and consumed across grounding-related crates. In parallel, `LanguageSpec` data declarations at `src/v3/spec/{rust,python,go}.dag` are extensible — adding a fourth language is "add `.dag` data + retypecheck."

But `ShapeATarget` is closed at the Rust enum boundary. Adding a fourth target requires editing the enum, the discriminant, and every match-arm — Rust-side change instead of `.dag`-data change.

P1 Modeling Faithfulness: thesis says target extensibility is data-driven; the enum says it's closed. The `LanguageSpec` carrier IS the source of truth; `ShapeATarget` is a parallel-authority shadow.

## §1. Required outcome

`ShapeATarget` consumes from `LanguageSpec` data declarations; closed enum dissolves to a typed-id reference.

## §2. Fix options

**Option A (proper dissolution)**: Replace `ShapeATarget = Rust | Python | Go` with `ShapeATarget = LanguageSpecRef { spec_id: DeclarationId }` carrying a typed reference to the `LanguageSpec` data declaration. Consumers dispatch via reflected spec lookup; adding a target = `.dag` data only.

**Option B (pragmatic ratchet)**: Keep closed enum + add cementing test that asserts `ShapeATarget` variants partition the registered `LanguageSpec` declarations (any new `LanguageSpec` without a matching enum variant fails closed). Test fails if the enum drifts behind data.

PM-recommended: Option A — extensibility is the thesis claim; the enum negates it. Option B ratchets the drift but doesn't dissolve the boundary.

## §3. Files

**Option A**:
- `src/v3/grounding_cross_target_meta/src/cells.rs` (replace closed enum with typed ref)
- `src/v3/grounding_cross_target_meta/src/diagnostic.rs:29` (consumer)
- `src/v3/compiler/tests/integration/cross_target_coverage_carrier_test.rs` (substrate read)
- All Rust match-arm consumers across grounding crates

## §4. Cross-cutting constraints

- Substrate-grep before authoring (Mgr-cited discipline): consumer-side breadth audit before changing the Rust enum.
- STOP-and-PING if substrate-side reflection plumbing isn't ready (`LanguageSpec` declaration-id lookup via `declaration_by_name` α).
- Cross-references Class F row 3 in sweep doc.

## §5. Receipt

- `ShapeATarget` consumes typed `LanguageSpec` reference (Option A); OR cementing test pins enum ⊇ registered specs (Option B).
- All consumers updated.
- Cementing `.dag` `TestClaim` for the partition-or-typed-ref invariant.
- Sweep-doc Class F row 3 updated.

---

**End of brief.**
