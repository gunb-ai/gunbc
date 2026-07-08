# S2 strategic lane — v2 emits v2 (worker brief)

**Status:** brief for a parallel worker · branches from `claude/roadmap-strategy-discussion-313pvj` (PR #6371) or from main once it merges.
**Coordination:** this lane runs concurrently with the *tactical* S2 lane (v1 seed emits v2 sources to cargo-green Rust, worked in the #6371 session). File ownership is disjoint — see §5.

## 1. Objective (the milestone)

v2's **own** emitter — `emit = serialize_target ∘ translate`, the one-grammar-read-backward machine (DESIGN §4) — covers v2's own language surface, proven module-by-module against the v1 seed's emit of the same sources, terminating in the **byte-fixed-point self-emit** that makes `regen_stage0` unnecessary. That is S2 proper: after it, the Rust seed is "one realization" of the `.dag` truth and S3 (delete `src/v1`) becomes mechanical.

## 2. What already exists (don't rebuild)

- **The kernel works end-to-end at MVP scale.** S0's receipt (landed, #6359): `emit(add-fn)` → real Rust workspace → `cargo build && run` → output **== eval** of the same tree, with a wrong-oracle RED. The pieces: `src/v2/compiler/emit_host.dag` (host transport, wet-mode), `rust_host_transport_mvp1_descriptor()` in `src/v2/extdeps/languages/rust.dag` (Cargo.toml + harness + 5-byte codec), `emit_host_run_transport` builtin in the v1 interpreter.
- **Translate rows live in** `src/v2/std/compilers/target_model.dag` (11.4k lines of rows) + `src/v2/extdeps/languages/rust.dag` (`rust_mvp1_target_model_staging()`). New coverage = **rows, never edits to the fold** (DESIGN §4: N rows, not N×M adapters; §7: a wall is a row).
- **The subject corpus is enumerated**: the parse pipeline's 40-file closure is data in `src/v2/test/claim/manual/s1_closure_receipt_test.dag` — the same list is this lane's module ladder.

## 3. The receipt ladder (work bottom-up, one module at a time)

For each module M in the closure, smallest first (`src/v2/std/witness.dag` is 16 lines; `logic`, `occurrence_id`, `artifact` are tiny):

1. **Parity receipt:** v2-emit(M) vs v1-emit(M) (`gunbc compile --entry <M> --target rust` gives the v1 side). Compare per DESIGN's rule — **normalized round-trip, not golden strings** (a byte-diff is the *terminal* receipt, not the per-module one; early modules should compare the normalized/parsed form so incidental formatting doesn't block).
2. **Behavioral receipt where cheap:** if M has pure leaf functions, an `emit==eval` witness in the S0 pattern (`emit_host_add_equals_eval_test.dag` is the template).
3. **Fail-closed on gaps:** an uncovered construct is a **typed, located refusal** naming the construct — never fabricated Rust, never a silent skip (DESIGN §5; the absorbing-fallback section is the review bar). The refusal list IS the worklist.

Terminal: v2-emit(all 40) builds cargo-green and byte-matches v1's emit → then self-emit fixed point.

## 4. Known landmines (measured, not guessed)

- **The Symbol carrier seam.** Fresh v1-emit of the v2 closure currently fails with 4,266 rustc errors; 3,560 are E0308 and **1,790 of those are `Symbol` vs `String`**: v2 declares `type Symbol` opaque → v1's emitter mints `pub struct Symbol(pub String)` but still emits `^atom` literals as `"...".to_string()`. The tactical lane is fixing this in v1's emitter — **coordinate the carrier decision**: whatever Rust representation v1's fix settles on (newtype vs alias), v2's rows must emit the *same* carrier or parity can never hold. Ask before assuming; the decision will be visible in `src/v1/05_emit_rust.dag` history on the #6371 branch.
- **Rope strings.** Emitted/carried text is Cons-chain `FreeMonoid`, not `Value::Str` — anything that assumes flat strings breaks (S0 hit this; `free_monoid_to_string` at the host boundary is the pattern).
- **`Value::Null` split** (CLAUDE.md open thread): `Optional`/`Witness`/miss all realize as `Value::Null` today. The emitter must pick a grounded Rust story for `Optional<T>`/`Witness<T>`; that choice should be written down as rows + a note, and will feed the eventual Null-split work rather than fight it.
- **Loop/body lowering** is an open thread (operator-held FLAG confirms). Emitting `Loop`-sugar bodies faithfully may block on it; if a module's refusal list hits loop constructs, park that module and move on — do not invent a lowering.
- **Interpreted iteration is slow** (~20s fixed + content-proportional per run) until the tactical lane lands the compiled v2 pipeline. Keep receipts module-scale (seconds-to-minutes); avoid whole-closure runs per iteration.

## 5. Coordination / file ownership (merge-conflict avoidance)

| Lane | Owns | Must not touch |
|---|---|---|
| **Strategic (this brief)** | `src/v2/std/compilers/target_model.dag`, `src/v2/extdeps/languages/rust.dag`, new test claims under `src/v2/test/claim/emit/` (new dir) | `src/v1/**` (emitter + regen outputs), `src/v2/compiler/0{1,2,3}_*.dag` |
| **Tactical (#6371 session)** | `src/v1/05_emit_rust.dag` + regenerated `src/v1/stage0/src/*.rs`, parse-pipeline perf | `target_model.dag` rows, `rust.dag` MVP descriptor |

Shared read-only: the 40-file closure list, DESIGN.md. If a change wants to cross the boundary (e.g., the Symbol carrier), it's a one-message sync with the operator, not a silent edit.

## 6. Definition of done for the first PR of this lane

One small module (suggest `src/v2/std/witness.dag`) with: v2-emit produces Rust, a parity receipt vs v1-emit (normalized), a RED (perturb the module → receipt fails), and every uncovered construct surfaced as a typed refusal with a count. Small, green, honest — the S0 shape, one level up.
