---
status: PM-authored worker brief (deep-wolf-155 direct dispatch)
authority_parent: Operator briansrls 2026-05-15 directive — "could we try to do path B for tokenize/parse (NO workarounds) now" + 2026-05-15 follow-on "lets attack DB-10 for confidence"
authoring_date: 2026-05-15
brief_set: docs/r3-m2-class-5-deferral-survey.md (recommended attack point: DB-10)
authoritative_design: docs/design-m2-feature-parity.md §DB-10 (lines 20-71)
worker_session: witty-moth-725 (continuing from Brief 1; PR #3142 awaiting merge in parallel)
reporting: directly to deep-wolf-155 via dashboard-message (no Director/Mgr intermediary)
---

# Path B Brief 4 — M2 / Class-5 Closure: DB-10 `data` value semantics

## Context

This is the FIRST attack against the M2 / class-5 deferral pit. Per the operator-requested survey at `docs/r3-m2-class-5-deferral-survey.md` (PR #3139), DB-10 was identified as the smallest tractable M2 closure with the highest leverage:

- Size S (smallest M2 item)
- Substrate carrier already exists (`Declaration.value_body: Option<ValueBody>` at `src/v3/compiler/src/dag.rs:122`)
- The gap is downstream **consumers** of the carrier, not new substrate
- Closes class-5 sub-gap #2 (record literals inside `data` bodies usable at compile time)
- Unblocks several hand-Rust trampolines pointing at this dissolution

**Confidence-building purpose**: M2 feature parity has had design ready since `docs/design-m2-feature-parity.md` was authored, but no DB item has landed. Closing DB-10 demonstrates we can close M2 deferrals; the path to DB-11/12/13 + class-5 brace-body becomes concrete after this lands.

**Worker continuity**: witty-moth-725 closed Brief 1 cleanly (PR #3142 promoted to ready; awaiting merge). Brief 4 dispatch reuses the same worker session for substrate-language work — they've demonstrated solid investigation-first discipline + understand the substrate well from Brief 1.

## The authoritative design

**Source**: `docs/design-m2-feature-parity.md` §DB-10 (lines 20-71). This brief CITES that design; it does not re-author it.

Key facts from the design:
- `Declaration.value_body: Option<ValueBody>` at `src/v3/compiler/src/dag.rs:122` ALREADY EXISTS
- `ValueBody::Structural { fields }` + `ValueBody::List` / `Map` / `Record` / `Variant` variants ALREADY EXIST
- Parser ALREADY lowers `data foo: T = v` into `ValueBody`
- **Nothing downstream reads `value_body` back**
- Net effect: `data answer: Int = 42` compiles but is unreachable at every use site; `data config: Config = { host: "h", port: 8080 }` supports `config.host` nowhere

**Architectural commitment (PR #496, 2026-04-17)**: inlining happens at LOWERING, not emission. The test ratchet `test_3a2_data_field_access_resolves_statically` asserts `cfg.host` lowers to `Value(Int(1))` AND no `FieldProject<host>` `Transform` exists in the DAG. **This is load-bearing architecture, not swappable implementation detail.** Brief honors this commitment.

## Scope

### Phase A — Investigation

Verify the design doc's facts hold at HEAD (per `feedback_corrections_must_grep_verify_source`):

1. **Confirm `Declaration.value_body: Option<ValueBody>` lives at `src/v3/compiler/src/dag.rs:122`** (or current location post-drift) — grep + cite line number.
2. **Confirm `ValueBody` variants** match the design doc (`Structural { fields }` / `List` / `Map` / `Record` / `Variant`).
3. **Audit existing consumers** of `value_body`: grep for `.value_body` reads + `data_value_at` calls. The design says "nothing downstream reads it back" — verify or correct.
4. **Find `SurfaceExpr::Var { name }` lowering** — locate the path where unresolved-identifier resolution happens (probably in `lower.rs`).
5. **Find `lower_field_path_expr`** — confirm location + current behavior (design references `lower.rs:1404`).
6. **Find or confirm the test ratchet** `test_3a2_data_field_access_resolves_statically` — grep for it. If it exists already and is gated/ignored, that's an important finding.

Surface findings to deep-wolf-155 via `dashboard-message send --to deep-wolf-155 --body "..."` BEFORE authoring fix.

### Phase B — Land the consumers (per design §"Consumed in two places")

**1. Identifier resolution**:
- `SurfaceExpr::Var { name }` lowering, when name is not in local scope and not a variant constructor, falls back to `symbols.get(name)`
- If the resolved `Declaration` carries `ValueBody::Scalar(bits)`, emit a `Value(bits)` node inline via `emit_literal_as_value_port` (or current equivalent)
- Record-valued data references OUT OF SCOPE for this brief (acceptance scopes to scalar)

**2. Static field access**:
- `lower_field_path_expr`, when head ident is not in local scope, falls back to `symbols.get(head)` and walks the resolved Declaration's `ValueBody::Structural { fields }` via `resolve_structural_field_path`
- Terminal scalar literals emit `Value(LiteralBits)` nodes
- Nested `FieldValue::Record` payloads recurse
- `List` / `Map` / `Variant` / `Reference` terminals fall through to unresolved diagnostic (OUT OF SCOPE for 3a.2 acceptance)

**3. Substrate contract accessor** (per design):
- `Dag::data_value_at(decl_id) -> Option<&ValueBody>` — one query function, single-line implementation, no new state
- Lives in `impl Dag` at `src/v3/compiler/src/dag.rs`
- Exposes raw `ValueBody` for any future lens (provenance / symbolic cost / reflection)

### Phase C — Test fixture + acceptance

Per design §Acceptance:
- `data answer: Int = 42` compiles; use of `answer` emits the literal `42` in all 3 targets
- `data config: Config = { host: "h", port: 8080 }` compiles; `config.host` resolves statically; emission produces target-native string literal `"h"` at use site (NOT a struct-field read)
- `fn f() -> Int = answer + 1` emits `42 + 1` (inlined) in all 3 targets
- Rejected case: `data x: Int = 42` + `x.foo` → diagnostic "Int has no field foo"

Plus: confirm `test_3a2_data_field_access_resolves_statically` is GREEN (currently gated/ignored, presumably). If the test exists and passes after Phase B, that's the completion signal.

## Deliverables (concrete)

1. **Investigation report** to deep-wolf-155: findings from Phase A, especially any drift from the design doc's cited line numbers / fn names.
2. **Substrate accessor PR** (or bundle with the next): `Dag::data_value_at` one-line accessor.
3. **Identifier resolution lowering PR**: `SurfaceExpr::Var` falls back to `value_body` for scalar inlining.
4. **Field path lowering PR**: `lower_field_path_expr` walks `ValueBody::Structural` for scalar terminals.
5. **Test fixture**: `data answer = 42 ; f() = answer + 1` round-trip emits `42 + 1` per all 3 targets.
6. **Trampoline retirement** (if scope permits): identify which `class-5 record-body lands` trampolines are now unblocked by DB-10 landing. Surface candidates to deep-wolf-155 but do NOT retire in this PR (separate scope).

## Acceptance criteria (substrate-fact-at-HEAD)

- `cargo test -p v3-compiler --test integration test_3a2_data_field_access_resolves_statically` passes (currently gated/ignored, presumably).
- `cargo test -p v3-compiler --test integration db_10_data_value_inline_demo` passes (new fixture name TBD).
- `.dag` fixture: `data answer: Int = 42` + `fn f() -> Int = answer + 1` emits `42 + 1` in all 3 target languages.
- `.dag` fixture: `data config: Config = { host: "h", port: 8080 }` + `config.host` emits target-native `"h"`.
- `grep -n "fn data_value_at" src/v3/compiler/src/dag.rs` returns the new accessor.

## Anti-paper-shrink check

Naive workarounds that DO NOT count:
- Adding a Rust trampoline `inline_data_value(decl) -> Option<LiteralBits>` outside the lower pipeline — parallel authority; data inlining must be at LOWERING per the architectural commitment
- Adding emission-time inlining (each emit_target reads value_body separately) — explicitly REJECTED by design §"Architectural commitment" + locked by test ratchet
- Re-authoring the existing `ValueBody` carriers as new shapes — substrate already exists; the gap is consumer-side

The discriminator: after Brief 4 lands, `cargo test test_3a2_data_field_access_resolves_statically` is GREEN, no `FieldProject<host>` Transform exists in the lowered DAG for `data`-resolved field accesses, and the 3-target emit shows inlined literals (not struct-field reads). Per design's locked test assertion.

## Risks + open questions to surface back

- **Drift from design line numbers**: design doc cites `lower.rs:1404`, `dag.rs:122` — these may have drifted. Phase A surfaces current locations.
- **`test_3a2_data_field_access_resolves_statically` state**: design references this test as a LOCKED assertion. Is it currently passing (and we just need to maintain), failing (the gap), or gated/ignored (the expected state)? Phase A surfaces.
- **`SurfaceExpr::Var` resolution path complexity**: identifier-fallback may have intricate ordering rules (local scope → variant constructor → symbols → fallback). Phase A audits.
- **`resolve_structural_field_path`**: design references this as the recursion entry point. May not exist yet; may need to be authored as part of Phase B.
- **Provenance debt**: design accepts that `Value(42)` inlined doesn't carry the fact that the author wrote `answer`. Source-map provenance is OUT OF SCOPE; named as bounded debt (`Value.span` walks back via source).
- **Diagnostic shape for OUT-OF-SCOPE terminals** (List / Map / Variant / Reference field access on data values): unresolved diagnostic with what message? Surface to deep-wolf-155 for ratification before authoring.

## Coordination

- **Report findings** to deep-wolf-155 via `dashboard-message send --to deep-wolf-155 --body "..."` after Phase A.
- **Pause for guidance** before Phase B if Phase A reveals substantial drift from the design doc.
- **Tag PRs** with title prefix `r3-path-b-brief-4: ...`.
- **Coordinate with sunny-tern-495 (Brief 2)** if `format` / conversion primitives become a dependency (unlikely for scalar inlining; possibly relevant if record-field paths interact with String conversions later).
- **Coordinate with bright-swift-668 (Brief 3)** if class-5 brace-body work overlaps. Brief 4 (DB-10) is a SEPARATE class-5 sub-gap (#2 record literals) from Brief 3's class-5 sub-gap (#1 brace-body fn). Should not block each other.
- **PR #3142 (your prior Brief 1 closure) in flight**: do not block on its merge; Brief 4 is a SEPARATE branch (`session/witty-moth-725-brief-4` or analogous). When PR #3142 merges, that work item closes; Brief 4 continues under a new work item.

## Estimated effort

1-2 weeks if the design doc is accurate and the gaps are downstream-consumer-only. Longer if Phase A reveals additional substrate-shape gaps (which would itself be a substantive finding to surface).

## Read first

- `docs/design-m2-feature-parity.md` §DB-10 (lines 20-71) — **authoritative design**
- `docs/r3-m2-class-5-deferral-survey.md` — context on why DB-10 is the chosen attack point
- `src/v3/compiler/src/dag.rs` around line 122 — `Declaration.value_body` + `ValueBody` variants
- `src/v3/compiler/src/lower.rs` — `SurfaceExpr::Var` lowering + `lower_field_path_expr` + `resolve_static_field_project` at design-cited line 1404
- `feedback_corrections_must_grep_verify_source` — verify design-doc line numbers before relying on them
- `feedback_template_relocation_paper_shrink_discriminator` — the substrate growth MUST land in `.dag`-facing consumers, not hand-Rust trampolines
