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
type CoproductVariantProjection {
  field_projection:  FieldProjection   // payload field projection for this variant
  wire_tag_value:    WireTagValue       // typed wire-tag for this variant
}

type CoproductProjection {
  declaration:        DeclarationRef
  wire_tag_field:     {String|FieldRef}  // shape decided by substrate observation #2 STOP-and-PING; see disposition
  variant_projections: Map<VariantId, CoproductVariantProjection>
}
```

**Single-keyed-fact discipline (P2 boundary)**: per-variant payload + tag-value are one keyed fact, not two parallel maps. Director's binding constraint #2 originally enumerated them as separate fields ("refine in implementation as ergonomics demand"); the consolidated `CoproductVariantProjection` shape preserves the substantive constraints (typed `WireTagValue` leaf, structural per-variant projection) while making illegal states unrepresentable — a variant cannot have a field projection without a wire-tag value or vice versa. If "variant has no payload fields" is legitimate (e.g., Anthropic's empty-payload variants), encode that explicitly via a `FieldProjection::Empty` constructor or analog inside `CoproductVariantProjection.field_projection` rather than as absence from one map.

`WireTagValue` MUST be a typed leaf (sum type or named record), **not** `String`. If Anthropic + REST both serialize as string at the wire boundary, encode the typed-on-our-side / serialized-at-emit pattern: `WireTagValue` stays typed in substrate; serialization adapter handles `String <-> WireTagValue` at the wire boundary.

### Substrate observations — Director pre-ratified dispositions (gunbc#828 #issuecomment-4394416049)

**Pre-ratification scope** — Director pre-ratified observations **#1 (DeclarationRef alias)** and **#3 (tag_field asymmetry conditional)** at gunbc#828 #issuecomment-4394416049; worker proceeds on those without re-pinging unless evidence forces escalation. **Observation #2 was re-opened** after codex BLOCKING at gunbc#2079 corrected an `InputFieldRef` mis-read; #2's `wire_tag_field` shape decision (path (a) vs (b)) **remains an open STOP-and-PING** — worker MUST escalate to Mgr before choosing, NOT proceed silently.

1. **`DeclarationRef = String` alias at `dsl/std/serialization.dag:16`** — **Disposition: (b) accept alias for this slice + debt note with named dissolution trigger.** Promoting `DeclarationRef` to structural typed shape mid-slice is cascade scope creep per `feedback_construction_over_ratchets`. Document the soft-typed nominal handle in `CoproductProjection` carrier comments. **Dissolution trigger** (binding per P5 scaffold/debt discipline): promote `DeclarationRef` from `String` alias to structural typed shape (e.g., `(module: ModuleId, name: DeclarationName)` record, or `DeclarationId` with bootstrap-module witness) when **EITHER (a)** audit-row #14 (`declaration_name_preference_rank` / `declaration_by_name` rank-table at `bridge-retirement-audit-sourcespan-family.md:93` row 14) closes — i.e., dsl/std ↔ src/v3/std module convergence makes name-keyed identity unambiguous and the substrate has structural module identity available; **OR (b)** any `DeclarationRef`-typed consumer surfaces a string-identity bridge per `feedback_opaque_strings_attract_heuristics` (e.g., heuristic patching on the alias' string contents, naming-convention dispatch, suffix/prefix matching). Worker MUST add a debt-paydown row to authoritative debt ledger naming `DeclarationRef` + this trigger before merging carrier-introduction PR. **Re-escalate to Mgr** if neither (a) nor (b) is satisfied but evidence emerges mid-slice that the alias actively breaks something same-slice (e.g., Anthropic #1702 wiring forces a string-identity dispatch).

2. **No `FieldRef`-shaped carrier exists at HEAD** — corrected per codex BLOCKING at PR #2079. Earlier framing referenced "`InputFieldRef` at `services.dag:34`" — that line is a **comment explicitly REJECTING** the wrapper: *"No separate `InputFieldRef` carrier is introduced here, since `ParamToken.name` already carries the same shape and adding a wrapper would duplicate without strengthening the structural invariant"* (`src/v3/std/services.dag:28-36`). The services.dag precedent: parameter-input identity lives in `Map<String, InputField>`'s key invariant + `ParamToken.name`; no parallel typed-handle wrapper.

   **Disposition (revised)**: `CoproductProjection.wire_tag_field` faces the same shape question — does it need a typed wrapper, or does the structural invariant already exist elsewhere? Two paths, **STOP-and-PING the Mgr** before choosing:
   - **(a) Follow services.dag precedent (no wrapper)**: `wire_tag_field: String`, with structural identity carried by the consolidated `Map<VariantId, CoproductVariantProjection>` key invariant on `variant_projections` (each variant's `wire_tag_value` is the per-variant typed authority) + a fail-closed check at fixture load that the `String` value names a real field on the coproduct's variant payloads. Adds NO new carrier; aligns with `feedback_construction_over_ratchets`. Asymmetry with `WireTagValue` (typed leaf, binding per Director constraint #2) is acceptable — values and field-handles are distinct concerns.
   - **(b) Introduce typed `FieldRef`**: new top-level structural carrier; ergonomic when consumers need to compose field-identity across coproduct + record + tagged-object surfaces. Stronger structural invariant than (a) but introduces a new carrier — must justify per services.dag precedent's "duplicate without strengthening" test.
   - **Do NOT** manufacture an `InputFieldRef`-shaped wrapper (services.dag:28-36 explicitly rejected that shape).

3. **`InternallyTaggedObject.tag_field: String` asymmetry at `dsl/std/serialization.dag:49`** — **Disposition: conditional debt with named dissolution trigger.** Asymmetry only exists if substrate observation #2 resolves to **path (b)** (introduce typed `FieldRef`). If observation #2 resolves to **path (a)** (no wrapper, follow services.dag precedent), then `wire_tag_field: String` matches `InternallyTaggedObject.tag_field: String` — no asymmetry, no debt. Per `feedback_parallel_representation_debt`, recorded as shape-question rather than known mismatch.

   **Dissolution trigger** (binding per P5 scaffold/debt discipline; applies ONLY if observation #2 path (b) is chosen): when **(a)** typed `FieldRef` exists as top-level carrier (introduced via observation #2 path (b)) AND **(b)** evidence emerges that a downstream consumer benefits from typed field-identity in `InternallyTaggedObject` (per `feedback_emitter_workaround_is_gap_symptom`), THEN migrate `InternallyTaggedObject.tag_field: String` → `FieldRef` in a follow-on Substrate hygiene PR. Worker MUST add a debt-paydown row to authoritative debt ledger at HEAD naming this carrier + trigger before merging the carrier-introduction PR. Do NOT fix in same slice unless evidence shows the asymmetry actively blocks a downstream consumer.

## Acceptance gates (same-slice, all must pass)

1. **Carrier landed** in `src/v3/std/coproduct_projection.dag` (or chosen location post-grep) with the shape above (modulo STOP-resolved decisions on items 1-3).
2. **Anthropic #1702 re-dispatch wires through `CoproductProjection`** — NOT through any `response_variant_tag: String` shim. The `from`-path resolver consumes `CoproductProjection` to dispatch on coproduct response variants. (Anthropic #1702 work is preserved on branch `codex/cc1-target-integer-structural-fold` per Grounding G5; merge unblocks on this carrier.)
3. **At least one of the 3 already-briefed coproduct paydowns consumes `CoproductProjection`** — proof of multi-consumer composability. Refutes "carrier with single consumer is just an interface" anti-pattern. Worker picks the easiest of `r3-coproduct-{1,2,3}-*-worker.md` to thread through; documents which one in the PR description. The other two paydowns dispatch in their own PRs after.
4. **No `Option<>` wrapping on `Declaration`** — verify via grep at acceptance time that no `variant_projection_metadata: Option<...>` field was added to `Declaration` (per Director's β-rejection rationale; `feedback_node_not_god_struct` analog).
5. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean (carrier is added; no existing surface should perturb).
6. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.

## STOP / PING criteria

- **STOP** before proceeding if:
  - Substrate observation #1 (`DeclarationRef` alias): evidence emerges mid-slice that the alias actively breaks something (e.g., string-identity bridge on Anthropic #1702 wire) — re-escalate; default disposition (b) otherwise applies without re-ping.
  - Substrate observation #2 (`FieldRef` introduction): cascades into emit/typecheck surfaces beyond `coproduct_projection.dag` + the chosen consumer paydown — surface scope-creep.
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
