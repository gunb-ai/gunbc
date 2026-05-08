# R3 V7 — `ValueBody` variant inventory (Rust ↔ substrate `.dag`)

**Status:** **DESCRIPTIVE READINESS** — Verification-authored input for Substrate Mgr (warm-wolf-698 #2068) D1 mirror-parity-vs-codegen-generation choice. **Non-normative.** No prescription on dissolution mechanism.

**Authority:** Worker brief `docs/briefs/r3-v-valuebody-substrate-mirror-isomorphism-v1-worker.md` §Dependencies — D1 (mirror completion or single-authority generation) is **Substrate** authority. This doc reports current-state asymmetry only.

**Sources at HEAD:**
- Rust live carrier: `src/v3/compiler/src/dag.rs:436` — `pub enum ValueBody`
- `.dag` mirror: `src/v3/std/substrate.dag:179` — `type ValueBody`

## Variant census

| # | Rust variant (`ValueBody`) | Payload | Substrate variant (`ValueBody`) | Payload |
| --- | --- | --- | --- | --- |
| 1 | `Unparsed(SourceSpan)` | `SourceSpan` | `ValueBodyUnparsed(SourceSpan)` | `SourceSpan` |
| 2 | `Structural { fields }` | `Vec<(String, FieldValue)>` | `ValueBodyStructural { fields }` | `List<FieldEntry>` |
| 3 | `Scalar(LiteralBits)` | `LiteralBits` (Int / Bool / String) | — *(absent)* | — |
| 4 | `List(Vec<FieldValue>)` | `Vec<FieldValue>` | — *(absent)* | — |
| 5 | `Map(FieldMap)` | `FieldMap` (ordered, dup-key rejected at constructor) | `ValueBodyMap(List<FieldEntry>)` | `List<FieldEntry>` (no constructor-level dup-key rejection at HEAD) |

**Rust variants:** 5. **Substrate variants:** 3. **Drift class:** real and asymmetric.

## Asymmetry table

| Asymmetry | Side(s) | Detail |
| --- | --- | --- |
| **Rust-only** | `Scalar(LiteralBits)` | DB-10 (Lane 3 Stage 3a.2) introduced scalar-valued `data` declarations (e.g. `data answer: Int = 42`). No substrate constructor at HEAD. |
| **Rust-only** | `List(Vec<FieldValue>)` | Top-level structural list bodies (`data xs: List<T> = [...]`). No substrate constructor at HEAD. |
| **Payload-shape parity** | `Structural` | Rust `Vec<(String, FieldValue)>` ↔ substrate `List<FieldEntry { label: String, value: FieldValue }>` — structurally equivalent (named struct vs. tuple, same fields). |
| **Payload-shape parity (with semantic gap)** | `Map` | Rust `FieldMap` enforces dup-key rejection at the constructor boundary (`FieldMap::from_entries → Result<_, DuplicateFieldMapKey>`); substrate `List<FieldEntry>` payload at HEAD has no constructor-level uniqueness invariant. Same surface shape, weaker substrate-side invariant. |
| **Both-sides present** | `Unparsed`, `Structural`, `Map` | Naming convention diverges (`ValueBody*` prefix on substrate side vs. unprefixed Rust variants); structural pairing intent is clear. |

## Source citations

- Rust `ValueBody` enum and per-variant 4-pattern justifications: `src/v3/compiler/src/dag.rs:436-504`.
- Rust `FieldMap` constructor with dup-key rejection: `src/v3/compiler/src/dag.rs:511-538`.
- Substrate `type ValueBody`: `src/v3/std/substrate.dag:179-182`.
- Substrate `type FieldEntry`: `src/v3/std/substrate.dag:158-161`.
- ROADMAP debt row: `ROADMAP.md` — *"`ValueBody` Rust↔.dag mirror drift; no isomorphism gate"*.

## Non-normative observations (for Substrate Mgr's D1 input only)

- **Mirror-parity path** would require substrate to add `ValueBodyScalar(LiteralBits)` + `ValueBodyList(List<FieldValue>)` constructors and (optionally) a substrate-level dup-key invariant on `Map`. Maintains two hand-edited authorities — exposed to the same drift class that produced this row.
- **Codegen-generation path** would derive the substrate side (or the Rust side) from a single authority, retiring the dual-taxonomy class entirely (per `feedback_isomorphism_or_generation_for_mirrors`).
- **Choice not made here.** Substrate Mgr authority per worker brief §Dependencies D1.

## Downstream consumer (this doc's role in the gate)

**Canonical gate**: §1.8 **#96** `value_body_substrate_mirror_isomorphism_executable` (assigned by PM deep-wolf-155 #846 issuecomment-4404054226; durable on branch HEAD via PR #2217).

This inventory is **input to D1**, not a Slice 2 deliverable. Once Substrate lands D1 (mirror parity or codegen) + D2 (CI hook-point agreed: build-time enum walk / boot-time structural check / `.dag` `TestClaim`), Verification authors Slice 2 (the §1.8 #96 consumer harness) against the substrate-side carrier shape.

## Footer

Descriptive only; D1 dissolution choice is Substrate Mgr authority per brief §Dependencies. STOP+PING items per brief §Scope (out) are NOT in scope here: this doc does not hand-edit `substrate.dag`, does not introduce a `TestPredicate` variant, and does not claim any Pattern-A TC1–TC3 disposition.
