# Canvas — Substrate S5 Variant-aware projection metadata carrier

**Sub-issue**: gunbc#1947 (parented under #1939 Substrate Mgr lane).
**Authority**: `docs/r3-design-schedule-2026-05-06.md` §S5 (lines 101-106); `docs/r3-program-plan.md:979` Q-Anthropic-Variant-Aware (RATIFIED-by-default — all 3 paydowns); Substrate canvas C1.
**Status**: **canvas — Director-tier ratification needed on carrier shape before worker brief authoring**.

## Scope

Typed REST response projection carrier for **coproduct response bodies** — the substrate fact that lets a typed REST response `from`-path resolver dispatch on a sum-type response variant rather than collapsing into untyped JSON.

Closure: §1.8 gates #29-#30 (T-Anthropic-Wire 2 gates); unblocks #1702 Anthropic re-dispatch + 3 follow-up paydown PRs (Anthropic Messages 200 residual + 2 sibling coproduct slices already-briefed at `r3-coproduct-{2,3}-*.md`).

## Why a canvas not a worker brief

Carrier shape is a **substrate-fact-introduction (P1 procedure)** with non-trivial design space. Authoring a worker brief without Director ratification of shape risks rework. Surfacing 3 carrier-shape options for ratification.

## Carrier-shape options

### Option α — Variant-tag projection on `RestResponseProjection`

Extend the existing `RestResponseProjection` carrier (if extant; else introduce alongside `CoproductWireContract` at `dsl/extdeps/llm/anthropic.dag:20-30` precedent) with a per-variant projection field:

```dag
type RestResponseProjection
  = { request_path: String
    , response_variant_tag: String
    , response_body_field: List<FieldProjection>
    }
```

Indexed by `(method, response_variant_tag)`. Resolver picks the matching projection by parsing the response wire-tag (e.g., `type` discriminator in Anthropic's content-block coproducts) and dispatching to the variant's projection.

**Pro**: minimal new substrate; reuses `FieldProjection` shape; aligns with `CoproductWireContract::InternallyTaggedObject` (which Anthropic uses already at `:20-30`).
**Con**: tag-string IS a bridge — the rank-table risk of audit-row #14 family. Acceptable because tag strings are wire-protocol identity (not internal compiler identity), but worth flagging.

### Option β — Sum-type metadata on `Declaration`

Generalize: every coproduct `Declaration` carries optional `variant_projection_metadata: Option<Map<VariantId, FieldProjection>>`. The resolver finds the declaration via existing identity surface (declaration_by_name or DeclarationRef) and picks variant by structural variant-id, not wire string.

**Pro**: structural — no string bridge. Reuses sum-type variant identity already in `Declaration` (TypeBody::Sum variant constructors per VariantConstruct lowerer at memory/MEMORY.md:201).
**Con**: heavier substrate change; couples REST-response-specific concern to general `Declaration`. May leak to other coproduct uses without need.

### Option γ — Free-standing `CoproductProjection` carrier

New top-level carrier in `dsl/extdeps/llm/wire_contracts.dag` (or similar):

```dag
type CoproductProjection
  = { coproduct_decl: DeclarationRef
    , wire_tag_field: String     // e.g., "type"
    , variant_projections: Map<VariantTag, ResponseProjection>
    }
```

Lookup: `coproduct_projection_for(decl: DeclarationRef) -> CoproductProjection`. Wire-tag string lives in projection data, not in identity dispatch.

**Pro**: localizes the wire-coproduct-dispatch concern to one named carrier; avoids polluting `Declaration`. DeclarationRef-keyed (not span.file-keyed) — clean of audit-row-#14 concerns.
**Con**: new carrier type; some duplication if `RestResponseProjection` already carries a similar shape.

## Director ratification ask

1. **Pick α / β / γ** (or surface a fourth option). Provisional Mgr-tier recommendation: **γ** — DeclarationRef-keyed, localized to wire-protocol concern, doesn't leak into `Declaration` general surface, avoids the tag-string-as-identity concern of α.
2. Confirm scope: carrier-only at S5, or carrier + 1 of 3 paydowns bundled? Current default per Q-Anthropic-Variant-Aware = carrier + ALL 3 paydowns separately (already briefed). Confirm S5 dispatch is **carrier-only**; the 3 paydowns dispatch in their own briefs.

## On ratification — worker brief scope

Will author execution brief covering:
- New carrier definition in chosen location (`dsl/extdeps/llm/wire_contracts.dag` per γ, or alternate per ratification)
- Indexing API (`coproduct_projection_for(decl) -> CoproductProjection` or analog)
- Resolver wiring point in `from`-path resolution (typed REST response projection consumer)
- Acceptance: §1.8 gates #29-#30 advance from declared → carrier-landed; #1702 branch unparks
- Bootstrap regen + clippy + workspace tests green
- Cross-Mgr handoff: 3 coproduct paydown briefs already authored (`r3-coproduct-{1,2,3}-*-worker.md`) ready to dispatch in their own PRs post-S5 land

## Worker pin candidates (post-ratification)

valiant-ibex-312 (recently freed post-#1933 IntPlatform/UIntPlatform land — substrate-fact-introduction precedent owner) OR smart-ram-167. Final pin at dispatch.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director endorsement at gunbc#828 #issuecomment-4394293399 of authoring next 2-3 unbriefed items.
