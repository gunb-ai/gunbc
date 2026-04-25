# PB-Substrate next slice — `AtomPayload` round-trip via existing regen pattern `(genuinely-S)`

> **Worker brief** for the next PB-Substrate slice after the proven
> [#780 ArithmeticOp/etc bundle](https://github.com/gunb-ai/gunbc/pull/780).
> Reports through Zero-Floor Program Manager (`stern-swift-335`).
> Authored 2026-04-25 against the post-cascade-prep main.

## Read first (the proven pattern)

- [`docs/briefs/pb-substrate-pilot-v2-arithmeticop.md`](pb-substrate-pilot-v2-arithmeticop.md) — the v2 pilot brief that proved the pattern. This brief inherits its structure; differences called out below.
- [`docs/design-pure-bootstrap-zero-audit.md`](../design-pure-bootstrap-zero-audit.md) §"Substrate generation is already proven and shipping" — the lane-context reframe + survey.
- [PR #780](https://github.com/gunb-ai/gunbc/pull/780) — the merged predecessor. Read the diff and PR description; this slice is the same shape with a different target.

## Read first (the artifacts you'll touch)

- [`scripts/regen_runtime_mirrors.py`](../../scripts/regen_runtime_mirrors.py) `:758` — `render_dag_scalar_module`. Extend with one more `render_sum` call (#780's pattern).
- [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) `:35-41` — `AtomPayload` declaration (5 variants: `Literal(LiteralBits)`, `UnresolvedIdentifier(String)`, `ResolvedByStructure(DeclarationId)`, `ResolvedByName(DeclarationId)`, `TypeParam(String)`). Source-of-truth.
- [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) `:455-493` — current hand-authored counterpart (enum at `:457` + dissolution-ledger doc block at `:415-455` + `impl AtomPayload { resolved_id }` at `:486-493`).
- [`src/v3/compiler/src/dag_scalar_generated.rs`](../../src/v3/compiler/src/dag_scalar_generated.rs) — generated output (current shape; you'll regenerate it with `AtomPayload` added).
- [#781 tracked-debt entry](https://github.com/gunb-ai/gunbc/pull/781) — generator doc-comment propagation gap. **AtomPayload triggers this entry's dissolution path** — see "Documentation handling" below.

## Slice — `AtomPayload`

5 variants. All payload types already mapped by the existing generator:
- `LiteralBits` — already generated (`dag_scalar_generated.rs:5`); flows by name.
- `String` — Rust primitive; existing pattern handles.
- `DeclarationId` — defined in `substrate_minimal.dag`; generator already references it for other types.

No new type-mapping pattern required. Operator-types pilot's fold (3) precedent applies: STOP-AND-ESCALATE if a new mapping surfaces.

**Hand-authored counterpart**: `dag.rs:457-485` (enum body) + `dag.rs:486-493` (`impl AtomPayload { resolved_id }`).

## Round-trip mechanics — three differences from #780

The mechanics are #780's pattern with three explicit deltas:

**1. `impl` block must be preserved.** `dag.rs:486-493` defines `AtomPayload::resolved_id`. The generator emits the enum decl only; the impl block is not generator output. Worker's job: retire the enum decl from `dag.rs`, leave the `impl AtomPayload` block in place at the same site, referencing the generated type. Rust's coherence rules permit impl blocks anywhere in the same crate; consumers compile unchanged.

**2. Derive set differs from the operator types.** Inspect `dag.rs:456` — likely `#[derive(Debug, Clone)]` (no `Copy`/`PartialEq`/`Eq`/`Hash` because variants carry non-`Copy` payloads like `String`, `LiteralBits`). Match the hand-authored derives exactly via `render_sum`'s third argument.

**3. Documentation handling — triggers #781's tracked-debt closure.** The hand-authored block at `dag.rs:415-455` is a substantial dissolution-ledger doc (4-pattern check, M1(2.6) review-round-7 history, per-variant rationale). substrate.dag's `🟢 TERMINAL` annotation at `:34-35` is short — *"durable atom connective surface the lowered declaration graph carries"* — and does **not** carry the per-variant rationale or the historical context. Per #781: this is the first PR whose rationale lives only in the retired hand-Rust block.

   Two acceptable paths to close the #781 gap (worker picks):
   - **(a) Propagate rationale into substrate.dag first.** Add a `//` block above the `AtomPayload` type declaration in substrate.dag carrying the dissolution-ledger context. Then proceed with the generator extension. Worker authors a substrate.dag commit + a regen commit on the same PR.
   - **(b) Extend the generator to propagate `//` doc-comments.** Modify `render_sum` / `render_record` (or a new helper) to capture leading `//` blocks above each type declaration in substrate.dag and emit them as `///` doc-comments on the rendered Rust. This closes the gap structurally for **every** future migration, not just AtomPayload.

   **Lean: (a) for AtomPayload specifically.** (b) is the durable structural fix but is itself genuinely-S → S++ scope; the slice should not absorb it. (a) is a small per-type rationale propagation; (b) lands as its own follow-up PR (named in #781's dissolution trigger). Surface in PR description which path you took.

   If (a)'s rationale-propagation reveals that substrate.dag's annotation system can't express the dissolution-ledger context (e.g., 4-pattern checks need a structured form), STOP-AND-ESCALATE — that's a substrate-annotation modeling decision that belongs upstream.

## Acceptance

- [ ] `sums["AtomPayload"]` populated by existing parser; verified in worker first-action grep.
- [ ] `regen_runtime_mirrors.py` `render_dag_scalar_module` extended with `render_sum("AtomPayload", ...)` matching the hand-authored derive set.
- [ ] `dag_scalar_generated.rs` regenerated; new `AtomPayload` enum present.
- [ ] `dag.rs` enum decl at `:457-485` retired; **`impl AtomPayload { resolved_id }` block at `:486-493` retained** in place referencing the generated type.
- [ ] `dag.rs:415-455` dissolution-ledger doc block: rationale propagated into `substrate.dag` (path (a)) OR generator extended (path (b)) — closing #781's tracked-debt entry for this row.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean.
- [ ] DB-8 `self_host_fixed_point` converges bit-identically.
- [ ] SG-0 census: `dag.rs` partition unchanged (file stays); `dag_scalar_generated.rs` partition unchanged (REGEN_OUTPUTS member). Partition shift is automatic cementing.

No SG-0 census array edits required (no file added or retired).

## STOP-AND-ESCALATE

Surface to Zero-Floor Manager (`stern-swift-335`).

- **If `sums["AtomPayload"]` is missing from the existing parser dict** — STOP. Same shape as v2 brief's first STOP — parser support gap.
- **If migrating `AtomPayload` requires a new variant-payload type-mapping pattern** beyond what the generator's existing override system handles — STOP.
- **If consumers of `AtomPayload` (resolve passes, lowering, infer) break on the migrated type** — STOP. Indicates derive set or visibility contract drifted.
- **If `impl AtomPayload { resolved_id }` block can't stay in `dag.rs` adjacent to the migration site** (e.g., orphan-rule blocker, but unlikely in same crate) — STOP. Indicates the slice needs a new module split, which is scope balloon.
- **If path (a) rationale-propagation reveals substrate.dag's annotation system can't express dissolution-ledger context** — STOP. Substrate-annotation modeling decision belongs upstream.
- **If pilot scope balloons beyond `AtomPayload`** — STOP. Genuinely-S; preserve.
- **If DB-8 fixed-point drifts** — STOP immediately.

## Non-goals

- **Not extending the pattern beyond AtomPayload.** Other 10 hand-authored substrate types (ArrowBody, TypeConnective, FieldValue, ValueBody, Declaration, BindNode, BranchNode, TransformNode, ValueNode, LoopNode) get separate worker dispatches per refined survey.
- **Not implementing path (b) generator-comment-propagation here.** That's #781's own dissolution trigger; lands as separate follow-up if AtomPayload picks (a).
- **Not retiring `dag.rs` itself** — only the AtomPayload enum slice.
- **Not amending `substrate.dag` declarations** beyond the rationale propagation in path (a).

## Reporting

- Single PR. Title pattern: `feat(v3): PB-Substrate next slice — AtomPayload via existing regen pattern (closes #781 tracked-debt for this row via path X)`.
- PR description: cite this brief; cite path (a) vs (b) chosen + reason; cite #781 closure for the AtomPayload row.
- On merge: Zero-Floor Manager confirms the slice + #781 partial-closure to Director.
- On STOP-AND-ESCALATE: surface to Zero-Floor Manager.

## Cross-manager note

No cross-manager signal needed. AtomPayload migration is internal to PB-Substrate scope; no shape change reaches Grounding or other managers.
