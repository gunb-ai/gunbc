# Rc-ownership wrap-decision predicate — design + deep-emitter gate derisk

Status: LANDED (predicate + translate routing + UseSiteVerdict enrollment), 2026-07-20 verify-by-execution (valiant-dove-723). Original design: 2026-07-16 (bold-seal-166). Plan carrier (`v1_deletion_plan.dag` lane_state rows) is batched in #6909 — not this PR; the mark on the carrier is the plan file, this doc is the design receipt only.
Parent: sharp-bee-290 (Weak → Strong Self Host, Wave 1→4).
Displaced cost: unblocks self-emit for Rc-heavy core compiler modules (`04_infer`, `06_translate`, `05_emit*`, … — each seed-emitted with 100+ `Rc<` sites today) without the latent §5 fail-open that silently wraps every `shared_types` member in `Rc<T>`.

Related lanes (orthogonal axes — do not conflate):
- [emitter-ownership-defork.md](emitter-ownership-defork.md) — clone-vs-move at **value** use sites (`UseSiteVerdict`, `make_decision`).
- [s2-v2-self-emit-direction.md](s2-v2-self-emit-direction.md) Track C4 — `Rc<T>` **type** wrapping derived from the model.
- `program_partition.dag` `partition_derive_target_for_emit` — per-module **user-type** catalog augmentation (`ReferenceLayerOwned` default).

## Problem (§5 construct finding)

Wave 2 `use_site_verdict` behavioral pilot recorded `ssuv_ownership_rc_default_fail_open_finding` in `dag/tools/self_host_use_site_verdict_behavioral_transport.dag`:

> The Rust emitter silently applies default `Rc<>` wrapping on `Node` / `UseSiteVerdict` / fn params and returns with **no typed ownership refusal** (`target_use_site_ownership_lookup_miss` unwired).

Benign for immutable-value pilot modules (zero emit diagnostics). **Latent fail-open** for mutation/aliasing-bearing compiler modules in the wide fan-out: a carrier that should stay owned at a site, or a missing catalog row that should refuse, instead gets the v1 `shared_types` heuristic (`render_rust_shared_type_if_needed` in `05_emit_rust.dag`).

Band A flip policy (`frontier_band_a_emit_readiness.dag`) already blocks `parse_engine_hooks` and `discovery_enumeration` until this gate lands.

## The fork (§3)

Three independent opinions on whether to wrap a type in `Rc<T>` / `Box<T>`:

| Authority | Location | Predicate | On miss |
|---|---|---|---|
| **A — v1 shared_types** | `05_emit_rust.dag` `render_rust_shared_type_if_needed` | `set_contains(shared_types, type_name)` | **silent bare type** (no wrap) — but any shared type silently wraps |
| **B — v2 translate catalog** | `06_translate.dag` `translate_apply_use_site_ownership_*` | `target_use_site_ownership_lookup_in_catalog_node` keyed by `(carrier, use_site)` | **typed `Rejected`** (`^target_use_site_ownership_lookup_miss`) |
| **C — partition derive** | `program_partition.dag` `partition_derive_target_for_emit` | user semantic types → `ReferenceLayerOwned` rows appended to catalog | N/A (synthetic rows) |

A and B disagree today on compiler std carriers (`Node`, `Diagnostics`, …): B has explicit per-site rows in `rust_sg_rc_use_site_ownership_catalog` (return/binding → `Rc`, param → owned); A wraps whenever `build_shared_types` names the carrier, regardless of use site.

**Single authority target:** B's catalog lookup, gated by bundle readiness, exposed as one total function `wrap_decision_gate`.

## Wrap-decision predicate (the model)

### Types (`target_model.dag`)

```dag
type WrapDecision
  = WrapByValue
  | WrapByReference { layer: TargetReferenceLayer }

type WrapDecisionGate
  = WrapGateInapplicable
  | WrapGateDecided { decision: WrapDecision }
```

`WrapByValue` = emit the inner type shell with no reference layer.
`WrapByReference { layer }` = apply `target_reference_layer_apply_type_emitted` / `target_reference_layer_apply_value_expression` with the given layer (`Rc` or `Box` only — catalog `ReferenceLayerOwned` rows normalize to `WrapByValue` in `wrap_decision_from_carrier_ownership`).

### Core lookup (`wrap_decision_lookup_in_catalog_node`)

Thin rename over the existing authority — no new semantics:

```
wrap_decision_lookup_in_catalog_node(catalog, value_semantics_carriers, carrier, use_site)
  = target_use_site_ownership_lookup_in_catalog_node(...)
    |> map CarrierOwnership → WrapDecision
```

`value_semantics_carriers` short-circuit (already in `target_use_site_ownership_lookup_in_catalog_node`): atom carriers enrolled as value-semantics bypass the catalog and return `WrapByValue`. Used by `program_partition` for user types and structural surfaces.

### Bundle gate (`v2.compiler.wrap_decision` `wrap_decision_gate`)

Mirrors `translate_sg_rc_bundle_apply_disposition` (the SG-RC readiness check):

| `has_catalog` | `has_tokens` | Result |
|---|---|---|
| false | false | `Accepted(WrapGateInapplicable)` — no SG-RC opinion; legacy v1 path may still run (retire in implementation PR) |
| true | true | run `wrap_decision_lookup_in_catalog_node` → `WrapGateDecided` or `Rejected` |
| xor | | `Rejected(^translate_sg_rc_bundle_partial)` — partial bundle is fail-closed |

**Deep emitter ownership gate** = every v2 Rust emit path that today calls `translate_apply_use_site_ownership_*` **or** would have fallen through to v1 `shared_types` wrapping must instead call `wrap_decision_gate` and:

1. `WrapGateInapplicable` → pass shell through unchanged (no silent `Rc` default).
2. `WrapGateDecided WrapByValue` → pass shell through unchanged.
3. `WrapGateDecided WrapByReference` → apply reference layer (existing `target_reference_layer_apply_*`).
4. `Rejected` → propagate diagnostic; emit aborts for that site.

This is **construction over validation**: the emitter cannot state a second wrap opinion.

## Compiler-module carrier census

Std carriers with explicit SG-RC rows today (`rust.dag` `rust_sg_rc_use_site_ownership_catalog`):

| Carrier | Return / binding | Param | Struct field |
|---|---|---|---|
| `Diagnostics` | `Rc` | owned | — |
| `Node` | `Rc` | owned | `Box` |
| `TestClaim` | `Rc` (instantiation head) | — | — |
| `FreeMonoid` | `Rc` | — | — |
| `Outcome` | `Rc` (instantiation head) | — | — |
| `ModelCore` | `Rc` | — | — |
| `AlgebraInhabitanceDecl` | `Rc` | — | — |
| `ProbeHeap` | `Rc` | — | — |

**Not yet in catalog:** `UseSiteVerdict` (pilot module), and most compiler-local types. For self-emit of `04_infer` / `06_translate` / `05_emit*`:

- Std carriers above: covered by static `rust.dag` rows.
- Module-local `data`/`type` aliases: `partition_derive_target_for_emit` appends `ReferenceLayerOwned` rows at emit time (value-semantics enrollment + owned-at-all-sites default).
- **Gap to close in implementation:** `UseSiteVerdict` needs a catalog row (or value-semantics enrollment) before flip; missing row must **reject**, not inherit v1 `shared_types` default.

### Rc>100 modules (seed census, 2026-07-16)

| Seed module | `Rc<` count | Blocker until gate |
|---|---|---|
| `v1_compiler_emit_rust.rs` | ~1527 | emit surface (Track B + gate) |
| `v1_compiler_infer.rs` | ~1023 | gate + body producer |
| `v1_compiler_resolve.rs` | ~74 | namespace lane |

The gate does not alone flip these — Track B body emit and namespace resolution remain — but **without** the gate, a cargo-green emit attempt on these modules risks silent wrong `Rc` shapes (fail-open), masking aliasing bugs that behavioral receipts would not catch.

## Implementation sequence (for the follow-on PR)

1. **Land predicate + witnesses** — LANDED (#6776): `wrap_decision_lookup_in_catalog_node`, `wrap_decision_gate`, `wrap_decision_predicate_test.dag` green.
2. **Rewire `06_translate`** — LANDED (#6775): inline `translate_sg_rc_bundle_apply_disposition` + lookup chains replaced with `wrap_decision_gate` (behavior-preserving refactor; `sg_rc_layering_test` stays green).
3. **Retire v1 `shared_types` wrap for v2 emit entry** — LANDED (v2 path): zero `shared_types` / `render_rust_shared_type_if_needed` references under `src/v2/`; v1 `gunbc compile --target rust` still uses seed `shared_types` until v1 delete (S3).
4. **Enroll `UseSiteVerdict` carrier** — LANDED: `target_carrier_use_site_verdict` + owned-at-all-sites rows in `rust.dag` catalog; witnesses in `wrap_decision_predicate_test.dag`.
5. **Frontier flip** — unblock `parse_engine_hooks`, `discovery_enumeration` per `frontier_band_a_emit_readiness` (still gated on rustc-green behavioral receipts + Track B body emit).

## Witness / RED controls (`wrap_decision_predicate_test.dag`)

| Witness | Proves |
|---|---|
| `wrap_decision_diagnostics_return_is_rc` | catalog row `(Diagnostics, Return) → WrapByReference Rc` |
| `wrap_decision_diagnostics_param_is_owned` | `(Diagnostics, Param) → WrapByValue` |
| `wrap_decision_probe_heap_param_miss_rejects` | missing row → `Rejected` (fail-closed) |
| `wrap_decision_bundle_absent_inapplicable` | target without SG-RC edges → `WrapGateInapplicable` |
| `wrap_decision_bundle_partial_rejects` | catalog without tokens → `Rejected` |

Discriminating RED (both directions):
- **OMIT** — delete `Diagnostics` param row from catalog → param witness must flip from `WrapByValue` to `Rejected`.
- **WIDEN** — add v1-style silent wrap fallback → test `wrap_decision_bundle_absent_inapplicable` must fail (gate must not default-wrap).

## Non-goals (this design)

- `UseSiteVerdict` clone-vs-move (`emitter-ownership-defork`) — orthogonal axis.
- v1 seed `build_shared_types` deletion — follows v1 retirement (S3).
- Auto-generating catalog rows from a corpus census — `program_partition` derive covers user types; std carriers stay explicit in `rust.dag` (§3: cite upstream, no nickname).

## Open question (escalate if implementation stalls)

**Arc migration** (`floor_materialization.dag` P2 note): cross-process `ResolvedGraph` sharing wants `Arc`, not `Rc`. The wrap-decision predicate is layer-agnostic (`ReferenceLayerRc` is a modeled enum, not hardcoded `Rc` in translate). If Arc lands, only `rust.dag` token rows and `ReferenceLayer` inhabitants change — the predicate shape is stable.

## Probe receipts

- [Emitter residual site map (post-#6981)](../probes/emitter_residual_site_map_2026-07-21.md) — crisp-fox-839 E0425/E0255 class breakdown from emitted deep-module histograms (MAP ONLY; owner quiet-bee #6924).
