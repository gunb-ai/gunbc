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

### Substrate observations — Director pre-ratified dispositions (gunbc#828 #issuecomment-4394416049)

These were flagged as STOP-and-PING in canvas; Director pre-ratified the dispositions so worker can proceed without re-pinging unless evidence forces escalation.

1. **`DeclarationRef = String` alias at `dsl/std/serialization.dag:16`** — **Disposition: (b) accept alias for this slice + debt note.** Promoting `DeclarationRef` to structural typed shape mid-slice is cascade scope creep per `feedback_construction_over_ratchets`. Document the soft-typed nominal handle in `CoproductProjection` carrier comments; add a debt-paydown row pointing at future structural promotion of `DeclarationRef`. **Re-escalate to Mgr only if** worker surfaces evidence the alias actively breaks something same-slice (e.g., Anthropic #1702 wiring triggers a string-identity bridge per `feedback_opaque_strings_attract_heuristics`).

2. **`FieldRef` does not exist at HEAD; only `InputFieldRef` at `services.dag:34`** — **Disposition: grep-decide between two structural patterns.** Worker greps `services.dag` + adjacent surface for `InputFieldRef` consumers BEFORE deciding (per `feedback_emitter_workaround_is_gap_symptom` + `feedback_audit_adjacent_authority_first`):
   - If `InputFieldRef` carries input-specific structural context → introduce `FieldRef` as the general carrier; treat `InputFieldRef` as specialization (compose, not parallel-author).
   - If `InputFieldRef` is just narrowly-named for an input use-case → rename to `FieldRef` (per `feedback_naming_is_aliasing` — rename is structural-cheap).
   - **Do NOT** manufacture a parallel-authority second `FieldRef`.

3. **`InternallyTaggedObject.tag_field: String` asymmetry at `dsl/std/serialization.dag:49`** — **Disposition: typed introduction here + debt note for future migration with named dissolution trigger.** Per `feedback_parallel_representation_debt`, the typed/string asymmetry is recorded as known shape mismatch. **Dissolution trigger** (binding per P5 scaffold/debt discipline): when **(a)** `FieldRef` exists as a typed top-level carrier in the substrate AND **(b)** `InputFieldRef` (currently at `services.dag:34`) has been classified — either retained as input-scoped specialization or renamed to general `FieldRef` per substrate observation #2 above — THEN migrate `InternallyTaggedObject.tag_field: String` → `InternallyTaggedObject.tag_field: FieldRef` in a follow-on Substrate hygiene PR (analogous to Q2 `result_port` canonical rename per Substrate Mgr standing dispatch authority). Worker MUST add a debt-paydown row to `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` (or current authoritative debt ledger at HEAD) naming this carrier + dissolution trigger before merging the carrier-introduction PR. Do NOT fix in same slice unless evidence shows the asymmetry actively blocks a downstream consumer.

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
