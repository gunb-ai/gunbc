# T-V2-Retirement — Per-Surface Migration Matrix

**Status:** PROPOSAL (audit/planning only). Authored 2026-05-01 (silent-boar-29) per Director dispatch via cool-stag-230 (R3 PB).
**Parent:** `docs/audit/t-v2-retirement-audit.md` (#1338, merged). This doc is the next planning slice — per-surface mapping for gates `v2_oracle_no_remaining_test_consumers` (G-1) and `v2_directory_deleted` (G-2).
**Authority basis:** `docs/r3-structure.md` Lane structure §11 + §165 (PR #1319, 2026-04-30) + `docs/audit/t-v2-retirement-audit.md` §3 (gate authority) + `docs/design-test-infra.md:10-14` (dual `verification.dag` framing).
**Scope:** docs-only mapping. **No code/test rewiring; no v2 deletion; no workspace member changes; no v2/v3 import bridge; no decision on `verification.dag` convergence (routed to Substrate / Director).**

---

## 1. Definitions reused from #1338

- **G-1** counts only test surfaces *outside* `src/v2/` that substantively reference `v2_compiler` or `v2_compiler_tests` crates (substantive `use` / call, not doc-comment / string literal).
- **G-2** owns workspace-member removal of `src/v2/stage0` + `src/v2/tests` and full `src/v2/` deletion; prerequisites include G-1 green + S-1..S-4.
- **STOP S-1** (PM-authored T-V2-Retirement worker brief) gates G-1 implementation. Until S-1 lands, this matrix is mapping only — no per-surface migration may be executed.

Search command (corrected per #1338 review): `grep -rEln 'src/v2/|\bv2_compiler(_tests)?\b' src/ tests/` excluding `src/v2/`.

---

## 2. Surface inventory at HEAD (`origin/main` 662567645)

The r3-structure.md §11 phrase "~13 v2-using test files" resolves to **two distinct populations** that must not be conflated:

### 2.1 Population A — internal to `src/v2/tests` crate (13 files)

These import `v2_compiler::*` from inside the v2 tests crate. They are **internal to v2** and fall with **G-2** (workspace member removal of `src/v2/tests`). They do **NOT** count against G-1.

```
src/v2/tests/src/bootstrap.rs
src/v2/tests/src/derive_bound_fail_closed_test.rs
src/v2/tests/src/diagnostics.rs
src/v2/tests/src/effects.rs
src/v2/tests/src/helpers.rs
src/v2/tests/src/infer_semantics.rs
src/v2/tests/src/int_pow_bounded_test.rs
src/v2/tests/src/parse.rs
src/v2/tests/src/peano_materialization_cap_test.rs
src/v2/tests/src/pipeline.rs
src/v2/tests/src/render_repeat_test.rs
src/v2/tests/src/source_audit.rs
src/v2/tests/src/sub_value_lattice_factor_test.rs
```

(Plus `src/v2/tests/src/lib.rs` and `src/v2/tests/src/bug_sentinel_ratchet.rs` which do not directly import `v2_compiler` but are part of the same crate; total 15 files in `src/v2/tests/src/`.)

**Migration disposition for Population A:** none individually. The crate retires as a unit when G-2's workspace-member removal fires. Per-test coverage migration (if any v2-tests-crate behavior is not already covered by v3-side tests) is the responsibility of the PM-authored worker brief — flagged as routing question §6.1.

### 2.2 Population B — substantive G-1 consumers (2 test files + 2 Cargo edges)

These are the surfaces *outside* `src/v2/` that substantively reference v2 crates. Each is enumerated in §3 with disposition. **Per #1338 §3.1 green criteria, G-1 closure requires both the test-file dispositions AND deletion of the two Cargo edges below**; treating only the test files as Population B would let the matrix declare consumers "migrated" while `v2-compiler` / `v2-compiler-tests` remain live workspace dependencies of `src/v3/compiler` — exactly the parallel-authority residue INVARIANTS §P2 (Boundary Discipline / single authority) forbids.

Test consumers (2 files):
```
src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs
src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs
```

Cargo edges (2 lines):
```
src/v3/compiler/Cargo.toml:32  v2-compiler        = { path = "../../v2/stage0" }
src/v3/compiler/Cargo.toml:33  v2-compiler-tests  = { path = "../../v2/tests" }
```

The Cargo edges have no other consumer in `src/v3/compiler` — they exist solely to support the two test files in §3.1 / §3.2. Their deletion is mechanical once both tests dissolve, but it is **part of G-1 closure, not a downstream cleanup**.

### 2.3 Population C — non-G-1 references (cosmetic at deletion)

Doc-comments, string literals, README mentions. **Not** G-1 consumers; cleaned up cosmetically during G-2.

| File | Kind |
|---|---|
| `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs:140` | string literal `"src/v2/complexity.dag (5488L)"` |
| `src/v3/compiler/src/dag.rs` (lines 526, 983, 1562, 1598, 1602, 3102) | doc-comments |
| `src/v3/lenses/complexity.dag:13` | doc-comment |
| `src/v3/std/anthropic_operations.dag:20` | doc-comment |
| `src/v3/std/rust_method_template_contracts.dag:21` | doc-comment + the legacy emit chain note (§4) |
| `src/v3/SELF_HOSTING.md` | doc reference |

---

## 3. Population B — per-file G-1 disposition

### 3.1 `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs`

| Field | Value |
|---|---|
| Current dependency | `use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult}; use v2_compiler::v2_interpreter::{self, Value}; use v2_compiler_tests::helpers::resolve_imports_transitively;` |
| Role | Behavior oracle for the `repeat_string(s:, n:)` lower-time fold. Compiles a small program through the v2 pipeline and runs the v2 interpreter to assert output value. |
| Counts against G-1? | **Yes** (substantive `use` of both v2 crates). |
| Owner | PB Manager (per `docs/audit/t-v2-retirement-audit.md` §3.1). |
| Proposed migration | **Replace with a v3 evaluator equivalence-corpus row**, per `r3-pb-runtime-equivalence-corpus-seed-audit.md`. The property under test (lower-time fold of `repeat_string` to a string literal) is independently expressible against the v3 evaluator surface; the v2 interpreter is only the *current* oracle, not a structural requirement. **Alternate disposition:** if the property is structurally guaranteed by v3 lower-time evaluation rules (e.g., the fold is a typed primitive composition), delete the test as redundant with a receipt linking to the structural guarantee. |
| Prerequisite | S-1 (PM brief authorizes the disposition choice: replace vs. delete). |
| STOP condition | S-1 unmet → no migration. |
| What green looks like | File no longer matches `\bv2_compiler(_tests)?\b`; the `repeat_string` lower-time-fold property has a v3-side receipt (either an equivalence-corpus row or a structural-guarantee citation), not a silent test deletion. |

### 3.2 `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`

| Field | Value |
|---|---|
| Current dependency | `fn v2_profile_to_v3(p: v2_compiler::std_algebra::AlgebraProfile) -> AlgebraProfile { use v2_compiler::std_algebra::AlgebraProfile as V2; ... }` (line 992-993) and `let v2_map = v2_compiler::std_algebra::kernel_algebra_profile();` (line 1005), inside `v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` (line 991). |
| Role | Drift ratchet: asserts v3 `dag::kernel_algebra_profile` matches v2 stage0's `std_algebra::kernel_algebra_profile()` row-for-row, treating v2 stage0's table as the authority for the kernel algebra profile. |
| Counts against G-1? | **Yes** (substantive 3 references). |
| Owner | PB Manager + Substrate Manager (cross-program — the authority migration is Substrate-shape; the test retirement is PB). |
| Proposed migration | **Migrate the authority** to `dsl/std/algebra.dag` (or its successor as v3 substrate inhabitance is consolidated), then **retire the parity test** entirely — once the v2 mirror is no longer the authority, parity is structurally meaningless. Per `feedback_isomorphism_or_generation_for_mirrors.md`: hand-maintained Rust↔.dag mirrors must be replaced by generation or by an isomorphism test, not by hand-maintenance + parity assertion. The end state is a single `kernel_algebra_profile` authority on the v3 side, with no v2 mirror to compare against. |
| Prerequisite | S-1 (PM brief routes the authority migration between PB + Substrate per §P1 substrate-fact-introduction); landing of v3-side single-authority `kernel_algebra_profile` (Substrate continuation work). |
| STOP condition | S-1 unmet → no migration. Substrate-side authority migration not landed → parity test cannot be safely retired (would lose drift detection). |
| What green looks like | `kernel_algebra_profile` has a single v3-side authority (under `dsl/std/algebra.dag` or named successor); `m2_substrate_inhabitance_test.rs` no longer matches `\bv2_compiler\b`; the rest of `m2_substrate_inhabitance_test.rs` (other tests in the file) continue to pass under `cargo test --workspace --exclude v2-compiler-tests`. |

### 3.3 `src/v3/compiler/Cargo.toml:32-33` — `v2-compiler` + `v2-compiler-tests` path deps

| Field | Value |
|---|---|
| Current dependency | `v2-compiler = { path = "../../v2/stage0" }` and `v2-compiler-tests = { path = "../../v2/tests" }`. |
| Role | Workspace-internal Cargo edges; exist solely to support §3.1 + §3.2. No other consumer in `src/v3/compiler`. |
| Counts against G-1? | **Yes** — per #1338 §3.1 green criteria, G-1 closure requires these edges deleted alongside the test-file dispositions. INVARIANTS §P2 (Boundary Discipline / single authority): leaving the deps live while the only callers are gone leaves a parallel-authority residue. |
| Owner | PB Manager (per `docs/audit/t-v2-retirement-audit.md` §3.1; same lane as §3.1 + §3.2). |
| Proposed migration | Mechanical deletion of both lines from `src/v3/compiler/Cargo.toml` once both §3.1 + §3.2 are green. No replacement; `src/v3/compiler` does not need v2 crates after the test consumers retire. |
| Prerequisite | §3.1 green + §3.2 green. (Pre-emptive deletion would break the live tests.) |
| STOP condition | Either §3.1 or §3.2 still has substantive `\bv2_compiler(_tests)?\b` references → cannot delete without breaking the build. |
| What green looks like | `grep -n 'v2-compiler\|v2_compiler' src/v3/compiler/Cargo.toml` returns no matches; `cargo build -p v3-compiler` and `cargo test -p v3-compiler` both pass. The `src/v2/stage0` and `src/v2/tests` workspace members remain (G-2 owns their removal). |

---

## 4. Legacy emit chain — `rust_method_template_contracts.dag` header note

### 4.1 Surface

Per `src/v3/std/rust_method_template_contracts.dag:1-22` header: v2's emit pipeline still consumes the legacy authorities

```
dsl/extdeps/languages/rust/emit.dag:53  data rust_simple_method_specs: List<SimpleMethodSpec>
dsl/extdeps/languages/rust/emit.dag:66  fn rust_method_templates() -> Map<String, String>
dsl/extdeps/languages/rust/emit.dag:72  fn rust_method_wraps_result() -> Map<String, Bool>
```

because v2 has no bootstrap-Dag consumer infrastructure to read the v3-side `MethodTemplateContract` rows. Header note explicitly defers full retirement to "Pure-Bootstrap-Zero / v2 retirement scope."

Parallel chains exist for python and go targets (per `dsl/extdeps/languages/{python,go}/emit.dag`); the same disposition applies.

### 4.2 Disposition

| Field | Value |
|---|---|
| Counts against G-1? | **No** (not a test consumer; it is an authority surface in `dsl/extdeps/`). |
| Counts against G-2? | **Yes** as a prerequisite — the chain consumes v2 emit infrastructure; deleting `src/v2/` requires that no surviving authority depends on v2-routable emit. |
| Owner | PB Manager. Cross-ref `r3-pb-binshim-retirement-worker.md` and the T-Ground-LanguageSpec scope-E lineage. |
| Proposed migration | Once PB-Runtime trampoline is the live bootstrap (S-4) and v3-side `MethodTemplateContract` rows are consumed by the v3 emitter end-to-end, **delete `rust_simple_method_specs` + `rust_method_templates()` + `rust_method_wraps_result()` from `dsl/extdeps/languages/rust/emit.dag`** and the parallel python/go chains. The `MethodTemplateContract` rows under `src/v3/std/{rust,python,go}_method_template_contracts.dag` already exist as the single-authority replacement. |
| Prerequisite | S-2 (T-FixedPoint) + S-3 (T-LensProducer-Retirement) + S-4 (PB-Runtime trampoline) + v3 emitter consumes `MethodTemplateContract` end-to-end **for every emitted target (Rust + Python + Go)**, not just one (this last is partly out-of-scope of T-V2-Retirement; flagged §6.2). |
| STOP condition | Per-target gate. If the v3 emitter does not consume `MethodTemplateContract` rows end-to-end for **target T**, the **target-T** legacy authority (the `{rust\|python\|go}_*` chain in `dsl/extdeps/languages/T/emit.dag`) cannot be deleted. Targets are independent: it is NOT acceptable to delete all three legacy chains because end-to-end consumption is only proven for one. THESIS cross-target drift prevention + INVARIANTS §P2 (Boundary Discipline) forbid leaving any target's legacy authority load-bearing while its sibling is deleted. |
| What green looks like | The legacy authority symbols are different per target; check **all three target families** explicitly. Rust: `rust_simple_method_specs`, `rust_method_templates`, `rust_method_wraps_result`. Python: `python_method_templates` (and any sibling `python_method_wraps_result` / `python_simple_method_specs` if introduced before deletion). Go: `go_method_templates` (and siblings if introduced). See verification command below the table. The v3 emitter's targets all compile + emit identical output (bit-identical artifacts ratchet from T-FixedPoint, applied per target). |

Verification command (kept outside the table to avoid markdown pipe-escape pitfalls; in `grep -E`, `\|` is a literal `|`, not alternation, so the in-table form would silently match nothing):

```sh
grep -rEn '\b(rust|python|go)_(simple_method_specs|method_templates|method_wraps_result)\b' dsl/ src/v3/
```

Green: returns no matches under `dsl/extdeps/`.

---

## 5. Dual `verification.dag` convergence — *routed, not decided*

### 5.1 Surface

Per `docs/design-test-infra.md:10-14` and `src/v3/std/verification.dag` header comment:

- **`dsl/std/verification.dag`** (v2-era): `AssertKind`, `TestClaim { kind, label }`, `TestCase { name, claims, ignored }`. Older behavioral-assertion model.
- **`src/v3/std/verification.dag`** (v3, extended by DB-15): `TestPredicate`, `TestClaim { name, source, file_name, predicate, requires: List<ResourceReference> }`, `TestSuite`, `TestObligation { resources: List<ResourceReference> }`. Structural authority for generated tests. Per `src/v3/std/verification.dag:290`: **`TestClaim.requires` is the single authority for `ResourceReference` edges attached to a claim** — `claim_obligation_resources` (L325) and `materialize_test_obligations` (L333) walk this field for obligation materialization. Any convergence call (§5.2) MUST address whether v2-era `TestCase { name, claims, ignored }` carries an equivalent dependency-edge fact or whether v2 retirement strands the requires/obligation surface; INVARIANTS §P2 (Boundary Discipline) forbids silently dropping the edge during convergence (modeling-discipline Practice 3 — facts flow forward).

The v3 file's own header reads: *"`dsl/std/verification.dag` remains the older v2-era behavioral-assertion model; it is not silently superseded here. Convergence trigger: once v2 retires and the shared std tree can host the v3 verification surface directly, dissolve the duplicate definitions back to one `std.verification`."*

### 5.2 Disposition

| Field | Value |
|---|---|
| Counts against G-1? | **No.** Not a test consumer crate. |
| Counts against G-2? | **Yes** as a prerequisite — `src/v2/` cannot be deleted while the v2-era `dsl/std/verification.dag` surface is load-bearing for any surviving authority. |
| Owner | **Substrate Manager** for the convergence design call (which surface wins, or whether both are kept under disjoint module paths); PB Manager for the v2-side cleanup once the call lands. |
| Proposed migration | **Routed to Substrate / Director per dispatch non-goals.** This doc records the surface and ownership; it does not propose a convergence shape. The header on `src/v3/std/verification.dag` already names the trigger ("once v2 retires and the shared std tree can host the v3 verification surface directly"); whether v3's `TestPredicate`/`TestSuite` model fully replaces v2's `AssertKind`/`TestClaim`/`TestCase` model, or whether some v2 surface continues under a renamed module path, is a Substrate Manager design call. |
| Prerequisite | Substrate convergence design call (open). |
| STOP condition | No design call → no migration. PB Manager must NOT unilaterally delete `dsl/std/verification.dag` v2 surface as part of G-2 without the design call landing first. |
| What green looks like | A single canonical `std.verification` (or Substrate-ratified disjoint coexistence) such that `src/v2/` deletion does not strand any surviving authority that referenced the v2-era surface. Receipt: link to the Substrate Manager design-call resolution. |

---

## 6. Open routing questions (for PM brief author / Substrate Manager)

These are NOT decisions — they are routing questions surfaced by the migration-mapping work. Resolution belongs to the PM-authored T-V2-Retirement worker brief and (for §6.3) the Substrate Manager design call.

1. **Population A coverage migration (§2.1):** are any behaviors covered by the 13 internal `src/v2/tests/src/*.rs` files NOT already covered by v3-side tests? If yes, which ones, and where do they migrate? If no, the entire crate retires under G-2 with no per-test work. Quick-audit recommendation: spot-check `derive_bound_fail_closed_test.rs`, `int_pow_bounded_test.rs`, `peano_materialization_cap_test.rs`, `sub_value_lattice_factor_test.rs` (the named-property tests; the others are pipeline/parse coverage which v3 likely subsumes).
2. **Legacy emit chain end-to-end consumer (§4.2):** does the v3 emitter currently consume `MethodTemplateContract` rows end-to-end for any target, or only at the row-population layer (Phase 1 per the header)? If end-to-end consumption is incomplete, that completion is a prerequisite for §4.2 migration and may be a sibling sub-lane rather than part of T-V2-Retirement.
3. **`verification.dag` convergence (§5.2):** routed to Substrate Manager. Specifically: (a) does v3's `TestPredicate`/`TestClaim { name, source, file_name, predicate, requires }`/`TestSuite`/`TestObligation` cover all behavior expressible under v2's `AssertKind`/`TestClaim { kind, label }`/`TestCase`, or does any surviving authority require continuation of the v2 surface under a renamed module path? (b) Does v2's `TestCase { name, claims, ignored }` carry any equivalent of v3's `TestClaim.requires: List<ResourceReference>` dependency-edge fact? If not, the convergence must explicitly account for whether v2-side claims have implicit dependency edges that need promotion to `requires` before v2 retirement, or whether v2-side claims are dependency-free and the requires field simply does not apply on the v2 surface. INVARIANTS §P2 (Boundary Discipline) forbids silently dropping the edge.
4. **Cosmetic Population C cleanup ordering (§2.3):** these are doc-comment / string-literal references to `src/v2/`. Cleanest moment to do the cleanup is alongside G-2 deletion, since rewording before deletion is wasted churn (the comments are accurate descriptions of current state). Recommendation: do not pre-empt; sweep at G-2.

---

## 7. Acceptance summary

This matrix is intentionally bounded:

- §2 enumerates **Population A** (13 internal files, fall with G-2), **Population B** (2 test files + 2 Cargo edges — all 4 surfaces are G-1 closure work per §2.2), **Population C** (cosmetic).
- §3 gives per-surface G-1 disposition for all 4 Population B surfaces — §3.1 + §3.2 (test files) and §3.3 (Cargo edges) — each with current dependency / role / G-1? / owner / proposed migration / prerequisite / STOP / green.
- §4 maps the legacy emit chain (G-2 prerequisite, not G-1).
- §5 maps the dual `verification.dag` surface as routed to Substrate / Director — no convergence decision proposed.
- §6 routes 4 open questions to the PM brief author.

**Implementation remains gated on `t-v2-retirement-audit.md` §1 STOP conditions, foremost S-1.** This matrix is suitable input for the PM-authored worker brief.
