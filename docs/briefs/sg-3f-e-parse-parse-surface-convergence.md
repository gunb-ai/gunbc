# SG-3f-e — `parse` / `parse_surface` convergence `(M-L)`

## Context

PR #612 (SG-3g) proved `lower_helpers.dag` can generate a correct `expr_span`
helper, but wire-in (SG-3g-b) is blocked because `lower.rs` consumes
`parse::SurfaceExpr` while the generated helper targets
`parse_surface::SurfaceExpr`. Today those are duplicate generated Rust types
from the same authority (`runtime_mirrors.dag` "Surface carriers"), with only
derive-surface differences — `parse_surface` adds `PartialEq, Eq`.

This is parallel-representation debt, not a lowerer problem. The available
`From<&crate::parse::SurfaceExpr>` bridge in `parse_surface_generated.rs`
recursively deep-clones the full expression tree; using it at 16 `expr_span`
call sites (or even once per lowering at the `compile_to_dag` entry) regresses
the lowerer rather than improving it. SG-3g-b was parked on this brief.

## Read first

- `src/v3/compiler/src/parse_generated.rs` — defines `parse::SurfaceExpr` and
  family; emitted by `regen_parse`.
- `src/v3/compiler/src/parse_surface_generated.rs` — defines
  `parse_surface::SurfaceExpr` and family, plus the deep-cloning `From` bridge.
- `src/v3/compiler/runtime_mirrors.dag` — shared authority for the Surface
  carriers both modules derive from.
- `src/v3/compiler/src/bin/regen_parse.rs` — the regen flow that emits
  `parse_generated.rs`.
- `src/v3/compiler/src/lib.rs` — module wiring (`parse`, `parse_surface`) and
  `compile_to_dag` / `compile_parse_surface_std_authority_dag` entry points.
- PR #612 body — explicit framing of the parked wire-in and its prerequisites.
- `docs/briefs/sg-3g-b-lower-helpers-wire-in.md` — the parked wire-in lane;
  this brief unblocks it.

## Work

Make `parse` and `parse_surface` use the **same Rust `Surface*` types**.

**Emission ownership (directional):** `parse_surface` owns Surface carrier
type emission; `parse` re-exports. Rationale: `parse_surface_generated.rs`
already carries the richer derive surface (`PartialEq, Eq`) and is the module
the `lower_helpers` lens (and future `.dag`-authored lenses) already target
via `v3.compiler.runtime_mirrors` → `parse_surface::…`. Flipping direction
(having `parse` own, `parse_surface` re-export) would force the lenses to
re-target, which is out-of-scope churn. Do not revisit this direction in the
execution PR without escalation.

- **Preferred path**: stop emitting a second `SurfaceExpr` family in
  `parse_generated.rs`; have the `parse` module re-export the `parse_surface`
  carriers (`SurfaceExpr`, `SurfaceLiteral`, `SurfaceModule`, `SurfaceItem`,
  `SurfaceField`, `SurfaceVariant`, `VariantPayload`, `SurfaceParam`,
  `SurfacePattern`, `SurfacePatternField`, `SurfaceType`, `SurfaceRecordField`,
  `SurfaceMatchArm`, …) instead. The parser body in `parse_generated.rs`
  continues to construct values, but it constructs the re-exported (single)
  type.
- Update `regen_parse` to skip emitting Surface carrier type declarations —
  only emit parser functions — and to generate imports / `pub use` lines
  against `crate::parse_surface::…`.
- **Relocate inherent and trait impls that live only on `parse::Surface*`
  today.** `parse_generated.rs` currently defines
  `impl SurfaceType { pub fn span(&self) -> &SourceSpan }` (and may carry
  similar helpers); under the convergence direction those impls collide with
  or are shadowed by the re-exported type. Move them onto
  `parse_surface::Surface*`. The execution PR must pick and document one of:
  (a) port to the generating lens / regen so they land in
  `parse_surface_generated.rs` alongside the type, or (b) a sibling
  hand-written `impl` block in a non-generated module targeting the
  re-exported type. (a) is preferred when the impl is mechanical; (b) is
  acceptable when the impl carries logic the lens doesn't express.
  **Audit via grep before cutting the `pub use`**: any `impl … for
  parse::Surface…` (trait impls, including auto-derive-dependent ones), any
  inherent `impl parse::Surface…`, and any `parse::Surface…::method` call
  site must still resolve post-convergence — the grep must cover both
  inherent and trait impl surfaces, not just the inherent side.
- **Delete** the recursive `From<&crate::parse::SurfaceExpr>` (and sibling
  `From` impls on `parse_surface::Surface*`) deep-clone bridge. INVARIANTS
  "no bridges" forbids keeping it in-tree at all; leaving it as tracked debt
  weakens this lane into partial convergence. If any call site still needs
  it post-convergence, that's a signal the convergence is incomplete —
  surface the consumer and fix upstream, don't retain the bridge.
- Keep parser behavior unchanged. Parser tests must stay bit-identical on
  output. "Bit-identical" means constructed-value equality (same variants,
  same field contents, same spans) — not trait-surface equality. After
  convergence `parse::SurfaceExpr` values transitively gain
  `parse_surface`'s `PartialEq, Eq` derives; that's additive and expected,
  not a bit-identity violation.

## Acceptance

- `parse::SurfaceExpr` and `parse_surface::SurfaceExpr` are no longer parallel
  Rust types — one is a re-export of the other (or equivalent single-type
  convergence).
- Same convergence for the rest of the Surface carrier family
  (`SurfaceModule`, `SurfaceItem`, `SurfaceType`, `SurfaceLiteral`,
  `SurfacePattern`, `SurfaceRecordField`, `SurfaceMatchArm`,
  `SurfaceParam`, `SurfaceField`, `SurfaceVariant`, `VariantPayload`,
  `SurfacePatternField`).
- No `From<&crate::parse::Surface…>` (or sibling) deep-clone bridge remains
  **anywhere in-tree** between `parse` and `parse_surface` — the impls are
  deleted, not merely unreferenced. Per INVARIANTS "no bridges": a dormant
  adapter is still a second authority waiting to be re-used.
- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green.
- `cargo clippy --all-targets -- -D warnings` clean.
- Regen ratchet (`lower_helpers.dag` → `lower_helpers_generated.rs`) stays
  green.
- SG-3g-b wire-in is unblocked: the generated `expr_span` helper can be called
  from `lower.rs` without cross-type cloning. (Demonstrate in the PR body by
  showing the would-be call site compiles against the converged type; actual
  wire-in stays in SG-3g-b.)

## STOP-AND-ESCALATE

- If convergence requires changing the **semantic shape** of `Surface*` (new
  or removed variants, field renames, representation swaps) rather than just
  deduplicating generated ownership — STOP. Semantic changes belong in
  separate lanes.
- If `regen_parse` cannot re-export shared types without a wider parser
  codegen redesign — STOP and surface the specific codegen constraint. Don't
  paper over with a hand-maintained shim.
- If the parser's constructed values diverge bit-for-bit from pre-convergence
  (e.g. derive differences change behavior via `PartialEq` paths) — STOP.
  Parser output must be identical.
- If a downstream consumer is found that relies on the `parse::Surface…` types
  being **distinct** from `parse_surface::Surface…` (e.g. a trait impl bound,
  an inherent `impl` block the re-export would collide with) — surface it and
  diagnose the conflict before forcing through.

## Non-goals

- **Not wiring `lower_helpers::expr_span` into `lower.rs`** — that's SG-3g-b,
  re-dispatched after this lands.
- **Not expanding the lens** — no new helpers in `lower_helpers.dag`.
- **Not retiring `parse_generated.rs`** — it still houses the parser body;
  only the carrier type declarations move.
- **Not touching the `infer_helpers.dag` / `lower_helpers.dag` lenses** — this
  is a regen / module-wiring lane, not a lens lane.

## Size

M-L. Carrier-type dedup touches `parse_generated.rs`, `regen_parse`, and any
call sites that referenced the `parse::Surface…` variants via disambiguating
paths. Bulk of the work is mechanical, but the surface area is wide (every
`use crate::parse::{…}` that imports a Surface carrier needs to keep working;
every construction call site must still compile against the converged type).

Expected LOC delta: net negative (deletes duplicate type declarations + the
deep-clone `From` bridge; adds `pub use` re-exports and regen tweaks).

## Dispatch note

Director reviews. Key acceptance signal: the parallel-representation debt is
gone — one Rust type per Surface carrier — and parser / compiler test suites
stay green. After this ships, SG-3g-b is re-dispatched as the straightforward
wire-in it was originally intended to be.
