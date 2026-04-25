# PB-Substrate Pilot — Worker Brief `(S, pattern-proof slice)`

> **Pre-promotion Deliverable 4** for [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md)
> (PROPOSAL). Cited by the cascade promotion PR as the prototyped lane
> closure proving the migration pattern. Reports through the Zero-Floor
> Program Manager (parallel to R2 per
> [`docs/briefs/pure-bootstrap-zero-manager.md`](pure-bootstrap-zero-manager.md)).

## Read first

- [`docs/design-pure-bootstrap-zero-audit.md`](../design-pure-bootstrap-zero-audit.md) — lane-mapping authority; `dag.rs` family is in the PB-Substrate group.
- [`docs/design-pure-bootstrap-zero.md`](../design-pure-bootstrap-zero.md) §"New lanes" `PB-Substrate` — sized M-L; cementing test: generated Rust matches the structural facts the substrate model declares.
- [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag) — 398 LOC, TERMINAL-marked types mirroring the runtime enum surface. The generation source.
- [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) — current hand-authored substrate. Contains the `CardinalityBound` enum we'll round-trip through generation.
- [`src/v3/compiler/build.rs`](../../src/v3/compiler/build.rs) — current build script that handles `extdeps_generated` / `gunbc_generated`. Pattern for adding a new generation step.

## Frame

This is a **pilot, not a lane closure**. PB-Substrate proper migrates `dag.rs` + `dag/ports.rs` + `dag/effects.rs` (M-L scope). The pilot proves the *pattern* — one TERMINAL-marked type generated end-to-end with cementing test — so the cascade promotion PR can cite empirical evidence that the irreducible tier isn't structurally irreducible.

Pattern-proof scales to the rest of PB-Substrate post-cascade. **Do not expand pilot scope** to additional types or files; surface the gap and stop.

## Slice — `CardinalityBound` round-trip

`CardinalityBound` is the chosen first slice. From [`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag):

```
type CardinalityBound
  = Exact(Int)
  | AtMostOne
  | Unbounded
```

Three variants, single primitive dependency (`Int`). Smallest TERMINAL-marked type with a clean dep profile.

**Hand-authored counterpart** lives in [`src/v3/compiler/src/dag.rs`](../../src/v3/compiler/src/dag.rs) — find the existing `CardinalityBound` (or equivalent) Rust enum.

**Round-trip:**

1. **Read** the `CardinalityBound` declaration from `substrate.dag` via the existing Dag bootstrap path (it's already loaded into the bootstrapped Dag at compile time — that's what `bootstrap.rs::Dag::new()` does).
2. **Emit** Rust source for the enum — same shape as the existing hand-authored declaration in `dag.rs`. Land emission output in `OUT_DIR` via `build.rs`; e.g., `OUT_DIR/dag_substrate_generated.rs`.
3. **Cementing test** — assert the generated source matches the hand-authored declaration. Two acceptable forms:
   - **(a)** Byte-for-byte string match between generated output and the relevant slice of `dag.rs`.
   - **(b)** Both compile to the same Rust AST (more flexible to formatting). Prefer (a) for the pilot — exact-match is the strongest cementing claim.

The hand-authored `CardinalityBound` in `dag.rs` stays in place during the pilot; generated counterpart lives alongside under a different name (e.g., `CardinalityBoundGenerated`) so cementing is structural, not destructive. Retirement of the hand-authored version happens in PB-Substrate proper post-cascade.

## Acceptance

- [ ] Generation step lands in `build.rs`; runs at compile time without manual invocation.
- [ ] Generated file at `OUT_DIR/dag_substrate_generated.rs` (or equivalent) declares `CardinalityBoundGenerated` enum matching the hand-authored shape.
- [ ] Cementing test asserts the generated declaration matches the hand-authored `CardinalityBound` in `dag.rs` (form (a) or (b) above).
- [ ] `cargo test --workspace --exclude v2-compiler-tests` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` clean.
- [ ] `cargo fmt --all --check` clean (pre-push hook will enforce).
- [ ] DB-8 `self_host_fixed_point` still converges bit-identically (no substrate drift).
- [ ] No SG-0 census change — pilot is additive, not retiring `dag.rs`.

## STOP-AND-ESCALATE

Surface to Zero-Floor Manager (`stern-swift-335`); do not absorb scope.

- **If `substrate.dag`'s `CardinalityBound` declaration cannot be read by the existing bootstrap path** — STOP. Indicates the substrate model isn't actually evaluable today; the audit's evaluability claim was wrong; fall back to PB-1-a continuation per Director Q2 contingency.
- **If emission requires extending the spec authority** (e.g., Rust target spec doesn't know how to render a `Disj` type with `Conj` payloads) — STOP. That's a PB-6 prerequisite, not pilot scope.
- **If the cementing test reveals structural divergence** between substrate.dag's `CardinalityBound` and `dag.rs`'s — STOP. Either substrate.dag drifted from the runtime, or the runtime drifted from substrate.dag; both are findings the cascade PR needs to know about before locking the framing.
- **If pilot scope balloons** (more than `CardinalityBound` round-trip) — STOP. Pilot's job is pattern-proof, not breadth.
- **If DB-8 fixed-point drifts** — STOP immediately. The no-compromise gate.

## Non-goals

- **Not retiring `CardinalityBound` from `dag.rs`** — pilot is additive. Retirement is PB-Substrate proper post-cascade.
- **Not generating other types** (`LiteralBits`, `AtomPayload`, `PortState`, etc.) — pilot is one type, by design.
- **Not generating `dag/ports.rs` or `dag/effects.rs`** — pilot is one slice of `dag.rs`, by design.
- **Not amending the SG-0 census** — pilot is additive.
- **Not deciding the N=0 resolution shape** — orthogonal; design doc Open call.

## Reporting

- Single PR for the pilot. Title pattern: `feat(v3): PB-Substrate pilot — CardinalityBound round-trip via substrate.dag (Pre-promotion Deliverable 4 for #762)`.
- PR description cites this brief + the cementing-test result; states whether pattern-proof succeeded.
- On merge: Zero-Floor Manager (`stern-swift-335`) signals Director that Deliverable 4 is closed and cascade-eligible.
- On STOP-AND-ESCALATE: surface to Zero-Floor Manager; manager decides fallback (PB-1-a continuation) and re-dispatches.

## Cross-manager note

Grounding Manager (`crisp-seal-366`) has been heads-up'd at pilot start (per Zero-Floor Manager brief's bidirectional substrate-coordination protocol). First slice is additive — no shape-affecting changes to `dag.rs`. If subsequent slices (post-pilot) start altering `dag.rs` structure, the manager re-signals.
