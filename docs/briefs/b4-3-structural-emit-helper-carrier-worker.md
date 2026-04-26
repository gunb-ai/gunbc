# B4.3 — Structural emit-helper carrier `(M; B4 Phase 1 #3 of 4)`

> **Worker brief.** Reports through Substrate Manager (post-R2 spin-up) /
> Director (pre-spin-up). Sub-brief of the
> [B4 Identity-Carrier Substrate Pass program](b4-identity-carrier-substrate-pass.md)
> (merged via #814). Replaces §0.6 — `bind.span.file == "named_alias_emit_helper.v3"` /
> `branch.span.file == "match_emit_helper.v3"` checks — with a typed role marker
> on `Bind` / `Branch` nodes, attached at lowering time.

## Read first

- **[`docs/briefs/b4-identity-carrier-substrate-pass.md`](b4-identity-carrier-substrate-pass.md)** — parent program brief.
- **[`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag)** — live substrate authority for `BindNode` / `BranchNode` shape.
- **[`src/v3/compiler/src/emit.rs:3181, :3206`](../../src/v3/compiler/src/emit.rs)** — the §0.6 cited sites. Note: line numbers per synthesis #810; verify against current `main` at PR-author time.
- **[`src/v3/compiler/src/emit.rs::primitive_type_id_for_port_shared`](../../src/v3/compiler/src/emit.rs)** + **[`src/v3/compiler/src/emit.rs::walk_to_disj`](../../src/v3/compiler/src/emit.rs)** — production functions that may consult the helper-role markers.
- **`feedback_audit_adjacent_authority_first`** — grep before designing.
- **`feedback_no_metadata_markers`** — no `__is_X` string markers; model concepts structurally.
- **`feedback_no_textual_enforcement_bridges`** — no replacement sentinel strings.
- **`feedback_coproduct_dissolution`** — receipt for any new variant.

## Frame

`emit.rs:3181, :3206` dispatches via `bind.span.file == "named_alias_emit_helper.v3"` / `branch.span.file == "match_emit_helper.v3"` — file-equality string check, not structural. The §0.6 site from the B4 program. Substrate gap: emit helpers (named-alias emission, match emission) need a typed role marker on the `Bind` / `Branch` node attached at lowering time, not inferred from `span.file`.

The fix is a typed role enum field on `BindNode` / `BranchNode` — populated structurally by the lowering pass when it constructs the node for a helper role. Emission consumers ask "does this Bind/Branch carry a `UserCallable` / `UserMatch` role?" instead of "is the source file the magic helper file?".

## Pre-author authority audit (mandatory)

Before designing the new carrier, **grep `src/v3/std/` + `src/v3/spec/`** for any existing emit-role marker, lens-helper marker, or `EmitRole`-shaped carrier on `Bind`/`Branch` nodes. Search terms:

- `EmitRole`, `EmitParticipation`, `BindRole`, `BranchRole`
- existing optional fields on `BindNode` / `BranchNode` declaration in `src/v3/std/substrate.dag`
- existing emit-helper authority in `src/v3/std/emit_model.dag` or sibling files

**If audit reveals existing authority, reframe as consumer migration.**

## Slice (assume audit shows substrate gap)

1. **Land typed role enum carriers in `src/v3/std/substrate.dag`:**
   - `type BranchEmitParticipation = UserMatch` (extensible coproduct; document expected future variants in coproduct receipt)
   - `type BindEmitParticipation = UserCallable` (same)
   - Add optional `emit_participation: BranchEmitParticipation?` field to `BranchNode` declaration.
   - Add optional `emit_participation: BindEmitParticipation?` field to `BindNode` declaration.
2. **Mirror in Rust** (`src/v3/compiler/src/dag.rs`) — substrate.dag is canonical, Rust mirrors. Builder defaults to `None`.
3. **Lower the role markers** at every helper-role site:
   - User `match` Branch nodes get `Some(UserMatch)` at lowering time, not refinement / Bool-`if` Branches.
   - User `fn` / lambda Arrow body Bind nodes get `Some(UserCallable)`, not refinement-predicate / `let` / synthetic Binds.
4. **Replace the file-equality checks** at `emit.rs:3181, :3206` (and any sibling `span.file ==` checks for these helper roles) with structural queries against `bind.emit_participation == Some(UserCallable)` / `branch.emit_participation == Some(UserMatch)`.
5. **Audit `emit.rs::primitive_type_id_for_port_shared` + `walk_to_disj`** — confirm production code consumes the new markers correctly. If a sibling production site reaches for `span.file` for the same role, migrate it in this PR.
6. **Coproduct dissolution receipt** for the two new variants (`UserMatch`, `UserCallable`) per `feedback_coproduct_dissolution`. Receipt frame: "current taxonomy reflects the two production helper roles known at lower-time; open for extension when new helper roles arrive."
7. **Test migration** — rename test fixtures from magic strings (`named_alias_emit_helper.v3`, `match_emit_helper.v3`) to ordinary names; dispatch via the structural marker. Spoofing test: an ordinary fixture filename that matches the old magic string but lacks the role marker is NOT treated as a helper role. Existing tests in `emit.rs:3120+` `#[cfg(test)] mod tests` are precedent.

## Acceptance

- [ ] §0.6 `bind.span.file == "named_alias_emit_helper.v3"` and `branch.span.file == "match_emit_helper.v3"` sites in `emit.rs:3181, :3206` (and any siblings) removed.
- [ ] Substrate carriers `BranchEmitParticipation { UserMatch }` / `BindEmitParticipation { UserCallable }` land in `src/v3/std/substrate.dag` with Rust mirror parity.
- [ ] Lowering populates `Some(...)` at every user `match` / `fn` / lambda site; `None` for refinement / `let` / synthetic.
- [ ] Authority audit receipt recorded in PR body.
- [ ] Coproduct dissolution receipt for the two new variants.
- [ ] Test migration renames magic-filename fixtures and dispatches via structural marker.
- [ ] Spoofing regression test: ordinary fixture with magic-string filename does NOT receive helper-role treatment without the marker.
- [ ] Production consumer audit: confirm `primitive_type_id_for_port_shared` / `walk_to_disj` and any sibling site uses the structural marker, not `span.file`.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.
- [ ] No `--no-verify` push without explicit cargo-unavailable note in PR body.

## STOP-AND-ESCALATE

- **Audit reveals helper roles have additional structural dependencies** (e.g., `match_emit_helper` requires more than just role-tagging — needs context the lowering doesn't yet thread) — surface for substrate-scope call.
- **More than two helper roles surface during the production audit** — expand the coproduct or surface for design call (the receipt should remain honest).
- **DB-8 drifts** — STOP immediately.
- **A production consumer reaches for `span.file` for a role NOT cited in the synthesis** — surface; that's an undiscovered §0.X site.

## Non-goals

- Not extending the helper-role taxonomy beyond what production code currently dispatches on.
- Not replacing `span.file` checks for legitimate non-role purposes (e.g., diagnostic source-mapping).
- Not addressing other §0 sites (those are B4.1/B4.2/B4.4).

## Cross-program note

- **Producer:** Substrate Manager (T-Substrate / B4 Phase 1 #3).
- **Consumer:** emit.rs runtime — same-PR consumer migration.
- **No cross-program consumer signal** — emit-side only.

## Reporting

Single PR. Title: `feat(v3): B4.3 structural emit-helper carrier — replace span.file equality with typed BindEmitParticipation/BranchEmitParticipation roles`. Body cites this brief + B4 program brief + records the authority audit + coproduct receipt + production consumer audit findings.

On merge: signal Substrate Manager / Director; B4 Phase 1 carrier #3 of 4 lands.
