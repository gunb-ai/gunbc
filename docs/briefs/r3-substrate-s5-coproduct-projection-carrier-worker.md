# Worker brief — Substrate S5 `CoproductProjection` carrier (γ)

**Sub-issue**: gunbc#1947 (parented under #1939 Substrate Mgr lane).
**Authority**: Director ratification of **option γ** at gunbc#828 #issuecomment-4394369848 (2026-05-07); supersedes the canvas at `docs/briefs/r3-substrate-s5-variant-aware-projection-carrier-canvas.md` (canvas may be deleted after this brief lands).
**Closure predicate**: §1.8 gates #29-#30 (T-Anthropic-Wire 2 gates); unblocks Anthropic #1702 re-dispatch + 3 already-briefed coproduct paydowns (`r3-coproduct-{1,2,3}-*-worker.md`).

## Scope

Substrate-fact-introduction (P1 procedure): typed REST response projection carrier for coproduct response bodies. Free-standing carrier keyed by `DeclarationRef`; wire-tag in DATA, not identity.

## Carrier shape (binding per Director)

**Location**: `src/v3/std/coproduct_projection.dag` (verify-via-grep at HEAD that this file does not yet exist; if a near-neighbor already hosts compatible projection types, fold in cleanly). **NOT** `dsl/std/` — this is compiler-internal projection substrate, not user-facing language vocabulary. Contrast: `dsl/std/serialization.dag::CoproductWireContract` lives in `dsl/std/` because users declare wire contracts; `CoproductProjection` is for internal projection-resolver dispatch.

**Initial shape** (refine in implementation as ergonomics demand):

```dag
type CoproductProjection {
  declaration:              DeclarationRef
  variant_field_projections: Map<VariantId, FieldProjection>
  wire_tag_field:           FieldRef       // which field carries the wire-tag discriminator
  wire_tag_values:          Map<VariantId, WireTagValue>
}
```

`WireTagValue` MUST be a typed leaf (sum type or named record), **not** `String`. If Anthropic + REST both serialize as string at the wire boundary, encode the typed-on-our-side / serialized-at-emit pattern: `WireTagValue` stays typed in substrate; serialization adapter handles `String <-> WireTagValue` at the wire boundary.

### Substrate observations the worker must surface (do not silently work around)

1. **`DeclarationRef = String` alias at `dsl/std/serialization.dag:16`** — Director's ratification framing emphasizes "typed key, not string identity (avoids audit row #14 collision)". The current `DeclarationRef` IS a String alias. If `CoproductProjection.declaration: DeclarationRef` is to be a real typed key, either (a) `DeclarationRef` itself must promote to a structural typed shape (e.g., `(module: ModuleId, name: DeclarationName)` record) — substantial substrate change, surface as scope question; OR (b) accept the alias as a soft-typed nominal handle for this carrier slice and note the future-promotion debt. **STOP-and-PING the Mgr** before proceeding; don't choose silently.

2. **`FieldRef` does not exist at HEAD** as a standalone type (only `InputFieldRef` mentioned in `services.dag:34`). The brief asks for a typed `FieldRef` carrier. Either reuse `InputFieldRef` if its semantics generalize, or introduce `FieldRef` alongside `CoproductProjection`. Choose pragmatically; don't introduce a parallel-authority second `FieldRef`.

3. **`InternallyTaggedObject.tag_field: String`** at `dsl/std/serialization.dag:49` already uses a String tag-field. The new typed `FieldRef` shape will create a typed/string asymmetry between the two carriers. Either (a) `CoproductProjection.wire_tag_field` is `FieldRef` and the asymmetry is documented as tracked debt (eventually `InternallyTaggedObject` migrates), or (b) accept `String` for `wire_tag_field` to match. Director's binding constraint #2 is explicit: typed leaf `WireTagValue`; the `wire_tag_field` field shape is less hard-binding. Recommend (a) — typed introduction here, debt note for `InternallyTaggedObject` migration.

## Acceptance gates (same-slice, all must pass)

1. **Carrier landed** in `src/v3/std/coproduct_projection.dag` (or chosen location post-grep) with the shape above (modulo STOP-resolved decisions on items 1-3).
2. **Anthropic #1702 re-dispatch wires through `CoproductProjection`** — NOT through any `response_variant_tag: String` shim. The `from`-path resolver consumes `CoproductProjection` to dispatch on coproduct response variants. (Anthropic #1702 work is preserved on branch `codex/cc1-target-integer-structural-fold` per Grounding G5; merge unblocks on this carrier.)
3. **At least one of the 3 already-briefed coproduct paydowns consumes `CoproductProjection`** — proof of multi-consumer composability. Refutes "carrier with single consumer is just an interface" anti-pattern. Worker picks the easiest of `r3-coproduct-{1,2,3}-*-worker.md` to thread through; documents which one in the PR description. The other two paydowns dispatch in their own PRs after.
4. **No `Option<>` wrapping on `Declaration`** — verify via grep at acceptance time that no `variant_projection_metadata: Option<...>` field was added to `Declaration` (per Director's β-rejection rationale; `feedback_node_not_god_struct` analog).
5. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean (carrier is added; no existing surface should perturb).
6. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.

## STOP / PING criteria

- **STOP** before proceeding if:
  - `DeclarationRef` typed-promotion question (substrate observation #1) cannot be answered locally — surface to Mgr (warm-wolf-698 / inbox #2068).
  - `FieldRef` introduction cascades into emit/typecheck surfaces beyond `coproduct_projection.dag` and the chosen consumer paydown — surface scope-creep.
  - At carrier-shape implementation time, `WireTagValue` typed-leaf shape forces a String<->typed adapter that has nontrivial bootstrap-regen impact — surface.
- **PING** PB Mgr (#2074 / `warm-dove-618`) at carrier-landing time per Director's cross-Mgr coordination note: PB owns `T-LensProducer-Retirement` which may consume similar projection shapes. They may have downstream needs constraining the carrier. **This is a heads-up, not a same-slice blocker** — PB's input refines future iterations.

## Cross-Mgr coordination

- **Anthropic #1702 (Grounding G5)**: re-dispatch unblocks at carrier-landing. Heads-up to Grounding Mgr (#1944 / `clever-otter-128` if active) at PR-open time.
- **PB Mgr (#2074)**: see PING above.
- **Verification Mgr (#2075)**: ratchet authoring for `bridge_*_carrier_landed`-shaped gates is Verification's standing concern; no specific same-slice handoff expected here unless ledger row needs to advance (which on a carrier-introduction is normal — §1.8 gates #29-#30 advance to `CONSUMER_LANDED` per closure predicate).

## Worker pin (Mgr disposition)

**valiant-ibex-312** preferred — substrate-fact-introduction precedent owner (delivered IntPlatform/UIntPlatform via PR #1933). Fallback: smart-ram-167. Final pin at dispatch.

## Auto-spawn caveat

Per Director's note 2026-05-07: worker auto-spawn from Mgr-context is currently bug-affected (PM investigating); manual dispatch via PR creation under Mgr session branch is acceptable for low-risk shapes but L+ workers should hold until fix. This carrier is L-sized (substrate-fact-introduction + 1 multi-consumer paydown thread-through); recommend **HOLD dispatch until auto-spawn fix lands** or operator-directed manual route. Brief is dispatch-ready regardless.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director γ-ratification at gunbc#828 #issuecomment-4394369848.
