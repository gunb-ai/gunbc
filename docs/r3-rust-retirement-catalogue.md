# R3 Hand-Rust Retirement Catalogue

**Status**: DRAFT — format under operator review (only `emit.rs` entry populated; rest pending format ratification).

**Purpose**: catalogue every hand-authored Rust file in `src/v3/compiler/` and, for each, document (a) what semantic it owns, (b) what `.dag` substrate would absorb that semantic, and (c) the substantive risk that "retirement" collapses into paper-shrink (file moved to `tools/*.rs.in` template with no substrate growth).

**This doc is NOT a ratchet**. The SG-0 file-path ratchet is structurally compromised (rewards file-path manipulation; passes paper-shrink). This catalogue is the *planning artifact* — it tracks the SEMANTIC question "what does this file do?" against "what substrate would own that semantic?" The retirement check is substrate growth, not file-list shrinkage.

**Anti-paper-shrink stance**: every entry's "Retirement plan" section must answer two questions explicitly:
- **(R1)** What `.dag` substrate authors the behavior this file currently owns?
- **(R2)** Does the codegen-driver READ that `.dag` substrate to produce its output? Or does it read a Rust template at a different path?

If R2 answers "Rust template", the retirement is paper-shrink and **does not count**. Per `feedback_template_relocation_paper_shrink_discriminator` (2026-05-14 from cycles 4/5/6 reverts at PR #3059).

---

## 5-phase plan (canonical authority for Phase enum referenced below)

Per the operator discussion 2026-05-15 — replace the SG-0 file-path ratchet with this catalogue + a hand-Rust monotonicity invariant. Phases ordered cheapest-first (largest retirement count first).

1. **Phase 1 — Hand-Rust monotonicity invariant** (new P-rule): every PR's net hand-authored Rust delta ≤ 0 across the **FULL hand-Rust surface — `src/v3/compiler/src/` AND `src/v3/compiler/tests/**`**. Per `INVARIANTS.md` Dispatch-Discipline Mechanisms (b) line 323: "**Hand-Rust includes Rust tests** (`src/v3/compiler/tests/**`) — these are the T-PB-B test subset of the SG-0 census; the gate applies the same way it applies to T-PB-A non-test files." Forecloses the growth pattern where gate-closure PRs add Rust escorts (tests + cementing receipts + lens-as-Rust). After this lands, every PR must either retire ≥1 Rust file or land purely in `.dag`. **Per briansrls INLINE BLOCKING PR #3139 2026-05-15 at catalogue.md:21**: prior framing scoped only `src/v3/compiler/src/` which created a loophole — gate-closure PRs adding tests would skirt the invariant. Corrected.

2. **Phase 2 — Test harness layer dissolution**: 122 of 177 hand-Rust entries are `tests/integration/*.rs` files asserting one structural claim each. Replacement: `.dag` fixture programs + `TestClaim` literals consumed by ONE Rust test runner. Net retirement: ~120 files. Depends on Gap 11 TestClaim infrastructure (partially landed per gate #85).

3. **Phase 3 — Lens-as-Rust dissolution**: ~7-10 Rust files implement lenses (`complexity_lattice.rs`, `lens_declaration_apply.rs`, `lens_t_las_carrier.rs`, `enforced_lens_application.rs`, etc.). Replacement: `data lens_X: Lens<C>` declarations per the live `Lens<C>` substrate at `src/v3/std/lens.dag:70`, consumed via `fold_lens<C>`. Net retirement: -7 to -10 files. The complexity-tightness-lens PR #3067 was preparing this shape.

4. **Phase 4 — Substrate-mirror generation**: ~5-8 Rust files are parallel-representations of `src/v3/std/substrate.dag` types (`dag.rs`, `dag/{builder,cardinality_payload,effects,ports}.rs`, possibly `diagnostics.rs`, `dimension.rs`, `complexity_lattice.rs`). Replacement: regen pipeline reads `substrate.dag` + generates these files byte-identically; hand-editing forbidden. Per `feedback_isomorphism_or_generation_for_mirrors`.

5. **Phase 5 — Pipeline-stage retirement** (meta-circular bootstrap): the remaining ~12-15 files (`infer.rs`, `lower.rs`, `emit.rs` + variants, `bootstrap*.rs`, `regen_*.rs`, `pipeline_authority.rs`, `post_emit_verifier.rs`, etc.). Retires when `.dag`-authored stages compile to byte-identical Rust through SELF_HOSTING.md §7 fixed point. Requires resolving Decision 3.A + 3.B + 3.C substrate prereqs.

**End state target**: **zero hand-authored Rust in `src/v3/` source tree.** Per the live authority `docs/design-pure-bootstrap-zero.md` (LIVE 2026-04-25 cascade promotion):
- Line 41: "Goal: zero hand-authored files in v3's source tree. Better than v2's 1-residual."
- Line 95: "the in-tree floor target is 0 regardless of which N=0 resolution lands."
- Line 191: "If first-time bootstrap (N=0) resolution requires hand-Rust in v3's source tree — STOP. The resolution is supposed to live outside v3's source tree (install script, gunbc-runtime crate, or rustc macro)."

Files that LOOK irreducible (bootstrap drivers, the rustc-handoff seed, etc.) retire by **moving OUTSIDE v3's source tree** (per design-pure-bootstrap-zero.md's N=0 resolution paths — install script, gunbc-runtime crate, or rustc macro), NOT by accepting a permanent in-tree residual. Per Phase 5 trajectory: all `src/v3/**` hand-authored Rust dissolves; what's left is either (a) generated from `.dag`, (b) emitted by the substrate-driven pipeline, or (c) moved to an out-of-tree resolution.

**Per codex BLOCKING PR #3139 2026-05-15**: this catalogue earlier described an "irreducible hand-Rust seed of ~10-15 files" — that framing CONTRADICTED `docs/design-pure-bootstrap-zero.md`'s 0-floor target and is RETRACTED. The catalogue's planning artifact-status does not have authority to redefine the canonical target away from 0.

---

## Format spec (per entry — references Phase numbers from §"5-phase plan" above)

```
### `<path>` (<lines> lines, <bytes> bytes)

**Role**: <1-2 sentence behavioral role — what semantic does this file OWN for the rest of the codebase?>

**Public surface**: <enumerate the pub fn/struct/enum that other code depends on>

**Inputs**: <what types this file consumes>

**Calls into**: <other Rust files this file imports/depends on>

**Called by**: <other Rust files that import/depend on this file>

**Existing `.dag` substrate**: <links to relevant `.dag` types that already model the input/output domain>

**Retirement target** (one of — all converge on 0-floor per `docs/design-pure-bootstrap-zero.md`):
- Generated from `.dag` substrate (existing or planned carrier at `src/v3/std/<file>.dag:<line>`)
- Dissolved entirely (semantic absorbed by another `.dag` carrier; file unnecessary)
- Moved out-of-tree (per design-pure-bootstrap-zero.md N=0 resolution: install script, gunbc-runtime crate, or rustc macro — documented WHICH out-of-tree home is the target)

**(N.B.)** No "irreducible in-tree seed" option. Per codex BLOCKING PR #3139 2026-05-15 + `docs/design-pure-bootstrap-zero.md` line 191: if a file genuinely cannot retire in-tree, its retirement target is "move out-of-tree" — NOT "permanent residual." A planning artifact does not have authority to introduce a permanent in-tree carve-out against the live 0-floor design.

**Replacement plan**:
- **(R1)** `.dag` substrate that owns the semantic: <path:line, or NEW with proposed shape>
- **(R2)** Codegen-driver / consumer that reads (R1) and produces equivalent behavior: <path or NEW>
- **(R3)** Parity invariant (semantic, not textual): <what executes against R1 to verify behavioral equivalence>

**Anti-paper-shrink check**: <explicit assertion that the replacement plan does not template-clone this file's contents into `tools/*.rs.in`. If risk exists, name the discriminator that catches it.>

**Phase**: one of Phase 1–5 per §"5-phase plan" above. (Definitions are NOT duplicated here — the §"5-phase plan" section is the single authority.)

**Substantive retirement risks**: <name the specific things that could go wrong>
```

---

## Catalogue

### `src/v3/compiler/src/emit.rs` (3,992 lines, ~160 KB)

**Role**: Single dispatch surface for code emission across all three targets (Go/Rust/Python). Routes `(dag, target, mode)` → either `emit_go_with_mode` (inline in this file), `rust_target::emit_rust_with_mode`, or `python_target::emit_python_with_mode`. Owns the cross-target shared scaffolding (`VariantPayloadBinding`, `VariantPayloadFieldAccessRuleBinding`) that all targets consume. Asserts determinism invariant D-1 (byte-identical re-emit per `tests/determinism_test.rs`).

**Public surface**:
- `pub enum EmitTarget { Go, Rust, Python }` (line 1060)
- `pub enum EmitMode { Program, Module }` (line 1067)
- `pub struct EmittedSource { text, target, mode }` (line 1073)
- `pub enum EmitDispatchError { Core, Python }` (line 1080)
- `pub fn emit(dag: &Dag, target: EmitTarget) -> Result<EmittedSource, EmitDispatchError>` (line 1097)
- `pub fn emit_module(dag, target) -> Result<EmittedSource, EmitDispatchError>` (line 1101)
- `pub fn emit_go_text / emit_go_module_text` (lines 1105, 1115) — convenience wrappers
- `pub fn emit_rust_text / emit_rust_module_text` (lines 1125, 1135) — convenience wrappers
- `pub fn emit_python_text / emit_python_module_text` (lines 1145, 1155) — convenience wrappers

**Inputs**: `&Dag` (substrate model from `crate::dag`); `EmitTarget` + `EmitMode` enum tags.

**Calls into**:
- `crate::emit::rust_target` (sibling module, 6,845 lines, target-monolithic Rust emission)
- `crate::emit::python_target` (sibling module, 2,232 lines, target-monolithic Python emission)
- `crate::emit::collection_ops_method_contract` (sibling module, 183 lines, shared helper for collection ops)
- Inline `emit_go_with_mode` (Go emission body lives in this file — no separate `go_target` module yet; ~3,000 lines of the 3,992)
- `crate::dag::*` for Dag/Declaration/Behavior/TypeConnective walks
- `crate::diagnostics` for `EmitError` shape

**Called by**:
- `crate::bin::regen_v3` (regen entrypoint)
- `crate::bin::regen_bootstrap` (bootstrap regen)
- `crate::emit_rust_bin_shim` (CLI shim for stage0)
- `crate::emit_rust_roundtrip_fixtures` (roundtrip test harness)
- `crate::post_emit_verifier` (post-emit verification)
- Integration tests in `tests/integration/` (m1_3_emit_*, m1_4_emit_*, m1_5_emit_*, boundary/m1_*, etc.)

**Existing `.dag` substrate**:
- `src/v3/std/emit_model.dag` (534 lines) — `type TypeRealization`, `type TargetIntegerTypeInhabitance`, `type LanguageSpec` (line 430), per-target inhabitance bound carriers
- `src/v3/std/clean_emission.dag` (146 lines) — `VariantPayloadFieldAccessRule` (Rust counterpart at `emit.rs:14`)
- `src/v3/spec/{rust,python,go}.dag` — per-target `LanguageSpec` data values
- `src/v3/compiler/pipeline.dag` — pipeline declaration that names `emit(dag, spec)` as a stage
- `docs/design-emission-model.md` — design doc

**Retirement target**: Generated from `.dag` substrate. The dispatcher logic (`emit_with_mode` match on `EmitTarget` → per-target call) is mechanical projection from the live `LanguageSpec`-driven model at `emit_model.dag`. The cross-target scaffolding (`VariantPayloadBinding`) duplicates `clean_emission.dag` types and should be a generated mirror, not hand-Rust.

**Replacement plan**:
- **(R1)** `.dag` substrate: `src/v3/std/emit_model.dag` is the authority for what an emitter IS. The dispatcher specifically would be a new `emit.dag` (NOT YET EXISTS — design-tier per `docs/design-emission-model.md`; awaiting PB-6 worker dispatch per the L2.5 model that landed in PR #3066). Per Decision 4.Q2 ratified, ONE `emit.dag` with LanguageSpec-driven per-target projection, not per-target sibling files.
- **(R2)** Consumer: a `.dag`-authored emit interpreter (or codegen-driver that consumes `emit.dag`). Per Decision 4.Q3 ratified, the meta-circular bootstrap Step 4 has Rust stage0 emit loading `emit.dag` + parity vs `emit.rs` + simultaneous deletion. The CODEGEN-DRIVER must read `emit.dag` substrate (not a `tools/emit.rs.in` template) to satisfy R2 honestly.
- **(R3)** Parity invariant: byte-identical determinism (D-1) on a corpus of test programs, where each test program is emitted (a) by the live `emit.rs` and (b) by the new `emit.dag`-interpreted emitter, and the byte outputs match. This is **semantic parity via behavioral equivalence**, not textual parity via source-clone.

**Anti-paper-shrink check**: The naive paper-shrink shape would be: `mv src/v3/compiler/src/emit.rs tools/emit.rs.in`, add codegen-driver that copies the template through, register the new emitted Rust file at the original path, mark `emit.rs` retired. **THIS DOES NOT COUNT** because the `.dag` substrate at `emit_model.dag` did not grow by the dispatch-logic constructs (EmitTarget routing, EmitMode dispatch, VariantPayloadBinding etc.). Discriminator: the retirement PR must include diff hunks ADDING constructs to `emit_model.dag` (or sibling `.dag` files) that account for every behavior `emit.rs` currently owns; merely deleting `emit.rs` while adding `tools/emit.rs.in` is paper-shrink per `feedback_template_relocation_paper_shrink_discriminator`.

**Phase**: 5 (pipeline-stage retirement via meta-circular bootstrap). Cannot retire before:
- Decision 3.A InferredDag/PreInferDag carrier shape lands in substrate (currently surfaced back to operator/Director for amendment-PR disposition)
- PB-6 emit worker dispatch (Step 2 of SELF_HOSTING.md §2.2 4-step; Step 1 ratified at PR #3066)
- `emit.dag` substrate authored (Step 3)
- Meta-circular bootstrap fixed-point landed (SELF_HOSTING.md §7)

**Substantive retirement risks**:
1. **Paper-shrink via per-target template-cloning**: emitting `tools/{rust,python,go}_target.rs.in` separately, codegen copies through. Discriminator: substrate-growth check on `emit_model.dag`.
2. **Inline Go emitter body** (~3,000 lines of `emit.rs` are the Go target itself, NOT shared dispatch): retirement order matters — these should move to a `go_target.rs` sibling FIRST (refactor, not retirement) before any `.dag` rewrite, otherwise the retirement PR has to swallow 3K lines of Go-specific logic alongside the dispatcher abstraction. Counter-suggestion: precondition retirement on the inline Go body being extracted to a sibling module.
3. **Cross-target `VariantPayloadBinding` is shared scaffolding**, not target-specific. It models `std.clean_emission.VariantPayloadFieldAccessRule` (line 14 comment confirms this). The Rust struct is duplication of the `.dag` carrier — should be GENERATED from `clean_emission.dag`, not hand-Rust. This is Phase 4 (substrate-mirror generation), not Phase 5; consider retiring this layer SEPARATELY first.
4. **`tests/integration/` consumers**: 6+ tests import `crate::emit::emit_rust_text` directly. Phase 2 (test harness dissolution) must precede or accompany this retirement, otherwise the tests block deletion.

---

## (Remaining ~176 entries — to be filled per format-ratified spec)

### `src/v3/compiler/src/infer.rs` (7,262 lines)
*Entry pending format ratification.*

### `src/v3/compiler/src/lower.rs` (11,894 lines)
*Entry pending format ratification.*

[... 174 more files ...]

---

## Open questions for operator before format is ratified

1. Is the format per-entry too verbose? Should "Calls into" / "Called by" be collapsed into a single dependency-graph diagram at the end of the doc rather than per-entry?
2. Is the "Anti-paper-shrink check" section per-entry useful, or should it live ONCE in the doc framing + each entry just cite the framing?
3. Should the catalogue group by Phase (1-5) instead of alphabetical, so retirement order is visible at a glance?
4. For TEST files (122 entries), is a per-file entry useful, or should we batch them as a single "Phase 2 group" with a manifest listing only the test names + which `.dag` fixture replaces each?
5. Should the catalogue have a counter-example section listing files that **look retireable but aren't** (e.g., the irreducible bootstrap seed), so retirement plans don't waste cycles trying to retire genuinely irreducible files?
