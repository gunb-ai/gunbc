# SG-3f — Surface reflection (substrate-authoritative surface types) `(XXL)`

## Context

Two census-gaming debts trace back to the same root cause:

- `parse_parser_body.txt` (1350 LOC hand-authored recursive-descent) — can't become `parse.dag` until the substrate can describe `SurfaceModule` / `SurfaceItem` / `SurfaceExpr` / etc.
- `lowering_rust.authority` or equivalent (per #593 Option B) — `lower.dag` can't exist until the surface types it consumes are substrate-authoritative.

The thesis's self-inspection claim says the substrate describes its own shape. Today it describes `Dag`, `Behavior`, `Declaration`, `TypeConnective` (via `substrate.dag`). It does **not** describe the surface grammar: `SurfaceModule`, `SurfaceItem`, `SurfaceParam`, `SurfaceField`, `SurfaceVariant`, `SurfaceType`, `SurfaceExpr`, `SurfaceRecordField`, `SurfaceMatchArm`, `SurfacePatternField`, `SurfacePattern`, `SurfaceLiteral` — 12+ types living only as hand-authored Rust in `src/v3/compiler/src/parse.rs` (now `parse_parser_body.txt`).

**Until those surface types are reflected, parse / lower / any `.dag` consumer of the surface cannot be authored in `.dag`.** This lane is the foundational prerequisite.

## Read first

- `docs/substrate-reflection-design.md` — the reflection design canon. Notes this as the last untested load-bearing thesis claim. Read §1 Motivation, §3 File locations, §4 Schema-diff consumers.
- `src/v3/std/substrate.dag` — how reflection works for the existing substrate types (`Dag`, `Behavior`, etc.). Template for how the surface types should be declared.
- `src/v3/std/substrate_minimal.dag` — `NonEmptyList` / `NonSingletonList` precedent; how to bootstrap-split a reflection load.
- `src/v3/compiler/parse_parser_body.txt` — the 1350-LOC hand-authored parser; its `SurfaceModule`, `SurfaceItem`, etc. are the types that need reflection. Don't edit, but understand the shape.
- `src/v3/compiler/runtime_mirrors.dag` — already has partial Surface* declarations (from #589). Partial base to extend.
- `src/v3/spec/rust.dag` — realization pattern for reflected types; shows how Rust struct/enum is the realization of a reflected declaration.
- My review on #589 (comment 4281812427) — context on why this lane's prerequisite-status matters.

## Work

Staged into sub-lanes so an XXL scope stays dispatchable. Worker proposes final sequencing in first PR body; example split:

**SG-3f-a — Top-level surface carriers (S-M).**
Reflect `SurfaceModule`, `SurfaceItem`, `SurfaceParam`, `SurfaceField`, `SurfaceVariant` into `src/v3/std/surface.dag` (or extend `runtime_mirrors.dag` — pick the cleaner home based on bootstrap ordering). Add realization bindings in `src/v3/spec/rust.dag` so emission still produces the existing Rust struct/enum shapes. Round-trip test: a Rust module emitted from reflected `SurfaceModule` matches the pre-reflection handwritten definitions byte-for-byte.

**SG-3f-b — Expression + type trees (M).**
Reflect `SurfaceExpr`, `SurfaceType` — the recursive structures. Expression is the big one (17+ variants per `parse.rs`). Type is smaller but similarly recursive. Same realization + round-trip discipline.

**SG-3f-c — Patterns + literals (S).**
Reflect `SurfacePattern`, `SurfacePatternField`, `SurfaceRecordField`, `SurfaceMatchArm`, `SurfaceLiteral`. Smaller leaf types.

**SG-3f-d — Consumption proof (M).**
Author a minimal `.dag` consumer that reads the reflected surface — pilot options: "count SurfaceBind nodes in a module", "list all SurfaceVariant names across items", "assert SurfacePattern arities against SurfaceVariant arities". Must compile, emit Rust, rustc-link, and produce a correct result on a test fixture. This is the acid test that reflection is actually consumable, not just declarable.

## Acceptance

- `src/v3/std/surface.dag` (or equivalent new file) declares all 12+ Surface types as substrate declarations with realization bindings
- `src/v3/spec/rust.dag` (and `.go` / `.python` if relevant) carries realization rows for every reflected type
- Round-trip pilot: Rust emitted from reflected declarations matches the handwritten `parse_parser_body.txt` byte-identical on at least one canonical fixture
- SG-3f-d consumer: one `.dag` program reads reflected surface + emits + rustc-links + runs + produces correct output
- Bootstrap ordering works: `parse_parser_body.txt` still `include_str!`'s cleanly (no circular dep introduced by surface-reflection loading order)
- `docs/substrate-reflection-design.md` prerequisite marked satisfied for `SurfaceModule`, `SurfaceItem`, etc.
- Follow-on SG-2b-proper and SG-3b-proper become honestly dispatchable (named in ROADMAP as unblocked)

## STOP-AND-ESCALATE

- **If reflection of a specific surface type requires substrate extension** (a 7th connective or similar C1 stop-signal territory) — STOP. Surface the exact type + what substrate shape it needs. Do not extend the substrate inside this lane.
- **If bootstrap ordering reveals a circular dependency** (e.g., reflected Surface types need `parse_parser_body.txt` to bootstrap, but that fragment is loaded after `std/`) — STOP. This is the bootstrap-split problem. Propose whether the fix is a `substrate_minimal.dag`-style split or a build.rs ordering change.
- **If round-trip bytes don't match** on the pilot emission — STOP. The reflected + emitted shape should be identical to the handwritten one. Drift means the reflection is lossy or the realization is wrong; both need to be understood before proceeding.
- **If a worker proposes collapsing multiple Surface types** (e.g., "SurfaceParam and SurfaceField are the same shape, merge them") — STOP. That's a surface-grammar change, not reflection scope. Surface for separate decision.

## Non-goals

- **Not dissolving `parse_parser_body.txt`** — that's SG-2b-proper (next lane after SG-3f lands).
- **Not dissolving `lowering_rust.authority`** — that's SG-3b-proper (also after SG-3f).
- **Not touching parse semantics** — the recursive descent in `parse_parser_body.txt` stays exactly as-is during this lane.
- **Not changing emit-side Rust shapes** — the realizations produce the same Rust structs/enums, just from `.dag` declarations instead of hand-authored code.
- **Not authoring `parse.dag`** — that's the follow-on. This lane only makes `parse.dag` possible.

## Size

XXL. 4 sub-lanes. SG-3f-a + SG-3f-d are the riskiest (bootstrap ordering + consumption proof); SG-3f-b is the bulk. Multi-week scope.

Expected LOC delta this lane: ~+200 to +500 (new `.dag` declarations and realizations added; handwritten Surface struct/enum definitions may be deletable if round-trip proves equivalence, but safest to keep until follow-on lanes actually consume). The real dissolution (parse + lower) lands in follow-on lanes.

## Dispatch note

Director reviews each sub-lane PR. SG-3f-a gets the heaviest review (it sets the reflection pattern for the rest). STOP aggressively on bootstrap ordering — reflection-load-order bugs are hard to untangle once merged.
