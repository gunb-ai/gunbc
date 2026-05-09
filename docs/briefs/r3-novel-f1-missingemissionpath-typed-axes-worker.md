# R3 Novel-Finding Worker Brief — F1 `MissingEmissionPath` typed-axes substrate

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope; worker dispatch via Substrate Mgr standing authority OR Debt-Paydown Mgr (gentle-newt-665 / gunbc#2062) standing authority per Brian directive 2026-05-09.
**Authority parent**: gpt-5-5-pro reflective analysis on `main@b09e0c8` Finding 1; PM dispatch at gunbc#846 c#4413701937 (operator authorized 2026-05-09; standing-authority confirmed).
**Priority**: HIGH — typed-carrier regression at diagnostic boundary; Rust mirror is MORE typed than substrate (inverted drift).

---

## §0. Problem statement

Substrate stringifies typed axes that the Rust mirror types correctly:

`src/v3/std/diagnostics.dag:219-223`:
```
| MissingEmissionPath {
    connective: String,
    behavior: String,
    target: String
  }
```

Rust mirror `src/v3/grounding_cross_target_meta/src/diagnostic.rs:26-30`:
```rust
MissingEmissionPath {
    connective: FormAxis,
    behavior: BehaviorAxis,
    target: ShapeATarget,
}
```

Substrate comments at `:215-218` admit: *"Each axis carries a string label (substrate-anchored discriminant; resolved against `TypeConnective` per #1229 anchor + `Behavior` per L1 model + Shape A targets per `r2-grounding-manager.md` portability set)."*

P1 Modeling Faithfulness violation: substrate authority weaker-typed than its consumer. Reverse-direction drift — the typed names exist (`TypeConnective`, `Behavior`, `ShapeATarget`); substrate just bypasses them via String labels.

## §1. Required outcome

Substrate axes typed against existing carriers; Rust mirror trivially aligns.

## §2. Fix options

**Option A (preferred)**: Type substrate fields directly:
```
| MissingEmissionPath {
    connective: TypeConnective,
    behavior: Behavior,
    target: ShapeATarget
  }
```

Update consumers (lower / emit / test_runner) to pass typed values. Rust mirror matches without conversion.

**Option B (pragmatic)**: Keep String axes; add typed-discriminant-roundtrip assertion in Rust constructor (`from_cell` at `:34-41`); document the substrate-weak-type as deliberate boundary.

PM-recommended: Option A — eliminates the drift class entirely. Option B preserves it as ratified-bridge.

## §3. Files

**Option A**:
- `src/v3/std/diagnostics.dag` (type fields)
- `src/v3/grounding_cross_target_meta/src/diagnostic.rs` (no change to Rust — already typed)
- consumers of `MissingEmissionPath` constructor in `.dag` (typecheck)
- new `.dag` `TestClaim` cementing typed roundtrip

## §4. Cross-cutting constraints

- No new hand-Rust tests; `.dag` `TestClaim` form.
- STOP-and-PING via Mgr inbox if `TypeConnective`/`Behavior`/`ShapeATarget` need substrate-shape changes.
- Cross-references Class C row 4 in `docs/audit/r3-debt-sweep-2026-05-06.md`.

## §5. Receipt

- Substrate-side fields typed; Rust mirror trivially aligns.
- Cementing `TestClaim` pinning typed roundtrip.
- ROADMAP / sweep-doc Class C row 4 updated to RETIRED.

---

**End of brief.**
