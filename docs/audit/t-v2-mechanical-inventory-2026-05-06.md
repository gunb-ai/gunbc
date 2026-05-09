# T-V2-Retirement — Mechanical Inventory / Consumer Census (2026-05-06 refresh)

**Status:** AUDIT artifact (docs-only; no code changes; no `src/v2/` deletion; no Cargo edge removal). Authored 2026-05-06 by PB Manager continuation per dispatch on inbox #1768 (sleek-eagle-514). Refreshes the per-surface inventory in [`docs/audit/t-v2-retirement-migration-matrix.md`](t-v2-retirement-migration-matrix.md) §2 against current `origin/main` HEAD.

**HEAD verified:** `2d26ed2b3` (initial inventory authoring against `origin/main` at refresh time). Methodology deltas applied during follow-up (regex coverage fix, C-data split extension to `compiler.dag:270`, line-completeness sweep) re-validated against `origin/main` after the #1848 squash-merge — line citations spot-checked at the post-merge tree (PB Mgr review confirmed `:53` / `:270` / `p0:L25` still resolve). Future refreshes should re-pin HEAD and re-run the unified grep in §"Search authority" before citing line numbers.

**This is an inventory refresh, not a decision.** It does not propose carrier shape, migration order, or G-1/G-2 sequencing — those remain S-1 / Substrate-Manager territory per [`docs/audit/t-v2-retirement-audit.md`](t-v2-retirement-audit.md) §1 STOP conditions and [`docs/briefs/r3-pb-tv2-s1-input-packet.md`](../briefs/r3-pb-tv2-s1-input-packet.md) §"Decision checklist". Initial cells cite live grep / file/line on `2d26ed2b3`; follow-up additions (per "HEAD verified" note above) re-validated against post-#1848-merge `origin/main`. Future refreshes should re-pin HEAD per §"Search authority" before citing line numbers — line citations may drift on busy `main`.

## Scope

Mechanical census: enumerate every workspace `v2_compiler` / `v2-compiler` / `src/v2` consumer in `src/` + `Cargo.toml` + `dsl/` at HEAD, attach grep receipts, and cross-link each row to the migration-matrix surface it refreshes. **Forbidden by dispatch:** deleting `src/v2/`, removing Cargo edges, mutating any v2 file. None of those are touched here.

## Search authority

All counts use the audit-sanctioned search shape from [migration matrix §1](t-v2-retirement-migration-matrix.md) (corrected per #1338 review), broadened to also catch the hyphenated Cargo dep names (`v2-compiler` / `v2-compiler-tests`) and bare-string `src/v2` literals without a trailing slash — `\bv2_compiler\b` only matches the underscored module-path form, and `src/v2/` requires a trailing slash so it misses values like `path: "src/v2"`. The matrix-cited path set was `src/ tests/`, but the repo has no top-level `tests/` directory — Rust integration tests live under `src/<crate>/tests/`. Single unified census for current tree:

```sh
grep -rEn 'src/v2\b|\bv2[-_]compiler(_tests|-tests)?\b' src/ Cargo.toml dsl/
```

(`-rEn` not `-rEln`: `-l` would print filenames only and cannot reproduce the line-numbered receipts in §1 — the matrix-cited `-rEln` is the file-level form, this inventory's line-level form needs `-rEn`.)

The `src/v2\b` form (word-boundary, no trailing-slash requirement) matches `src/v2/...` paths, `"src/v2"` string literals, `src/v2.dag` / `src/v2-…` references, etc. without false-positives on `src/v23` or `src/v2something`. This single command catches every surface enumerated below in one pass: `src/v2` paths in any context, `v2_compiler` / `v2_compiler_tests` Rust module-path uses, AND `v2-compiler` / `v2-compiler-tests` Cargo dep names. Earlier inventory passes used narrower patterns and produced split receipts; this unified form is the canonical reproduction command for future refreshes.

**Regex evolution log** (for matrix-doc reconciliation): matrix `\bv2_compiler(_tests)?\b` → inventory v1 same → inventory v2 (commit `7442c9265`) `src/v2/|\bv2[-_]compiler(_tests|-tests)?\b` (added Cargo-dep-name coverage) → **inventory v3 (this commit) `src/v2\b|\bv2[-_]compiler(_tests|-tests)?\b`** (drops trailing-slash requirement so the `path: "src/v2"` value is rediscovered by the canonical command, addressing review).

Substantive vs cosmetic split per #1338 §3.1: substantive = `use` / call / function-signature / type reference; cosmetic = doc-comment / string literal / README mention.

---

## 1. Population census at HEAD `2d26ed2b3`

### 1.1 Population A — internal to `src/v2/tests` crate (16 files)

These import `v2_compiler::*` from inside the v2 tests crate. Internal to v2 → fall with **G-2** (workspace member removal of `src/v2/tests`). Do **NOT** count against G-1.

```
src/v2/tests/src/bootstrap.rs
src/v2/tests/src/bug_sentinel_ratchet.rs
src/v2/tests/src/derive_bound_fail_closed_test.rs
src/v2/tests/src/diagnostics.rs
src/v2/tests/src/effects.rs
src/v2/tests/src/helpers.rs
src/v2/tests/src/infer_semantics.rs
src/v2/tests/src/int_pow_bounded_test.rs
src/v2/tests/src/lib.rs
src/v2/tests/src/parse.rs
src/v2/tests/src/pb_method_template_projection_consumability.rs
src/v2/tests/src/peano_materialization_cap_test.rs
src/v2/tests/src/pipeline.rs
src/v2/tests/src/render_repeat_test.rs
src/v2/tests/src/source_audit.rs
src/v2/tests/src/sub_value_lattice_factor_test.rs
```

**Receipt:**
```sh
$ ls src/v2/tests/src/ | wc -l
16
$ ls src/v2/tests/src/
(16 files as listed above)
```

**Delta vs migration matrix §2.1:** matrix listed 13 importers + 2 non-importing (lib.rs, bug_sentinel_ratchet.rs) = 15 files. **HEAD = 16.** New file: `pb_method_template_projection_consumability.rs` (added post-`66edec52`). No matrix update is required for G-1 disposition — Population A retires as a unit under G-2 regardless of file count — but the matrix's "13 v2-using test files" / "15 total" framing is now stale.

**Migration disposition (unchanged from matrix §2.1):** none individually. Crate retires as a unit when G-2's workspace-member removal fires. Per-test coverage migration (if any) is PM/S-1 territory; flagged in matrix §6.1.

### 1.2 Population B — substantive G-1 consumers outside `src/v2/`

#### B.1 Test files (2)

Both files appear in matrix §3.1 / §3.2; cited construct still present at HEAD.

| File | Substantive references at HEAD | Drift vs matrix |
|---|---|---|
| `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs` | L9: `use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};`<br>L10: `use v2_compiler::v2_interpreter::{self, Value};`<br>L11: `use v2_compiler_tests::helpers::resolve_imports_transitively;`<br>L25: `.map(\|d\| v2_compiler::v2_std_core::diagnostic_to_message(d.diagnostic.clone()))` | Matrix §3.1 "Current dependency" cell lists L9–L11 use-decls and is still accurate; **L25 is an additional substantive call site** (single body-of-test invocation of `v2_compiler::v2_std_core`) not enumerated in the matrix's `use`-only excerpt. Same disposition (Decision 1: replace vs delete) covers all four lines together — they retire as a unit when the file's v2 dependency lifts. |
| `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs` | L1209: `fn v2_profile_to_v3(p: v2_compiler::std_algebra::AlgebraProfile) -> AlgebraProfile {`<br>L1210: `use v2_compiler::std_algebra::AlgebraProfile as V2;`<br>L1222: `let v2_map = v2_compiler::std_algebra::kernel_algebra_profile();` | Line numbers drifted: matrix §3.2 cited L991/L992-993/L1005; HEAD = L1209 / L1210 / L1222. Construct unchanged (still `v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` test). |

**Receipt:**
```sh
$ grep -rEn '^use v2_compiler|::v2_compiler::|let .* = v2_compiler::|fn .*v2_compiler::|v2_compiler::[a-z_]+::' src/v3/ \
    | grep -v 'src/v2/'
src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs:9:use v2_compiler::v2_compiler_compile::{compile_to_resolved, ResolvedPipelineResult};
src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs:10:use v2_compiler::v2_interpreter::{self, Value};
src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs:11:use v2_compiler_tests::helpers::resolve_imports_transitively;
src/v3/compiler/src/dag.rs:1793:/// `v2_compiler::std_algebra::kernel_algebra_profile` remains only as a   ← doc-comment, NOT substantive
src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs:1209:    fn v2_profile_to_v3(p: v2_compiler::std_algebra::AlgebraProfile) -> AlgebraProfile {
src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs:1210:        use v2_compiler::std_algebra::AlgebraProfile as V2;
src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs:1222:    let v2_map = v2_compiler::std_algebra::kernel_algebra_profile();
```

The dag.rs L1793 hit is a single doc-comment (drifted from matrix §2.3's listing of L526/L983/L1562/L1598/L1602/L3102 — none of those line numbers carry `v2_compiler` references at HEAD; the doc-comment now lives at L1793 only). Population C, not B.

#### B.2 Cargo path-dep edges (2)

| Edge | Location at HEAD | Drift vs matrix |
|---|---|---|
| `v2-compiler = { path = "../../v2/stage0" }` | `src/v3/compiler/Cargo.toml:37` | Matrix §3.3 cited L32; HEAD = L37 (5-line drift). Edge unchanged. |
| `v2-compiler-tests = { path = "../../v2/tests" }` | `src/v3/compiler/Cargo.toml:38` | Matrix §3.3 cited L33; HEAD = L38 (5-line drift). Edge unchanged. |

**Receipt:**
```sh
$ grep -nE 'v2-(compiler|compiler-tests)' src/v3/compiler/Cargo.toml
37:v2-compiler = { path = "../../v2/stage0" }
38:v2-compiler-tests = { path = "../../v2/tests" }
```

#### B.3 G-1 closure surface count

**Population B at HEAD = 4 surfaces** (2 test files + 2 Cargo edges) — same shape as matrix §2.2; only line-number drift in citations.

### 1.3 Population C — cosmetic references (not G-1; cleaned up at G-2)

Doc-comments, string literals, README mentions. **NOT** G-1 consumers.

#### C.1 Files inventoried in matrix §2.3 (refreshed citations)

| File | Kind | Matrix-cited lines | HEAD lines | Delta |
|---|---|---|---|---|
| `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs` | string literal `"src/v2/complexity.dag (5488L)"` | L140 | L140 | unchanged |
| `src/v3/compiler/src/dag.rs` | doc-comments | L526/L983/L1562/L1598/L1602/L3102 | **L1793 only** (single ref at HEAD; cited lines no longer carry `v2_compiler`) | **drift — matrix line set stale** |
| `src/v3/lenses/complexity.dag` | doc-comment | L13 | L13 | unchanged |
| `src/v3/std/anthropic_operations.dag` | doc-comment | L20 | L20 | unchanged |
| `src/v3/std/rust_method_template_contracts.dag` | doc-comment + legacy emit chain note | L21 | L21 | unchanged |
| `src/v3/SELF_HOSTING.md` | doc reference | (unspecified) | L25, L27, L1178, L1216, L1217, L1218, L1219, L1220 | inventoried (L1178 added in 2026-05-06 follow-up: `cargo test -p v2-compiler-tests ci_freshness` doc reference) |

#### C.2 Files NOT in matrix §2.3 — surfaces added since `66edec52`

These are cosmetic-only references not yet listed in the matrix; surfacing here so S-1 / G-2 cleanup sweep has a complete map.

| File | Kind | Lines |
|---|---|---|
| `src/v3/compiler/src/pb_method_template_projection_dag_emit.rs` | doc-comment | L29 |
| `src/v3/compiler/tests/integration/pb_method_template_projection_dag_emit_test.rs` | doc-comments | L24, L28, L54 |
| `dsl/tools/purity_check.dag` | shell-script literal in `concat(...)` invocation | L157 |
| `dsl/std/node.dag` | doc-comment | L9 |
| `dsl/std/constructors.dag` | doc-comment | L52 |
| `dsl/std/syntax.dag` | doc-comment | L79 |
| `dsl/extdeps/llm/openai.dag` | doc-comment | L90 |
| `dsl/extdeps/languages/python/syntax.dag` | doc-comment | L43 |
| `dsl/gunbc/compiler.dag` | doc-comment | L29 |
| `dsl/gunbc/tools/review_codex.dag` | reviewer-prompt string mentioning `src/v2` | L75 |
| `dsl/gunbc/tools/ci_runner.dag` | doc-comment containing example `cargo run -p v2-compiler` invocation | L16 |

#### C.3 `.dag` configuration-data references — NOT cosmetic

These are **typed `data` declarations** in `.dag` modules (substrate/config rows), distinct from doc-comments / string literals. They are load-bearing values consumed by the compile pipeline; treating them as "cosmetic Population C" would let G-2 retirement strand the value or silently retarget a config field. Each row gets a named disposition rather than the C-class "sweep at G-2" default.

| Surface | Construct | Role | Population | G-1 / G-2 routing |
|---|---|---|---|---|
| `dsl/gunbc/compiler.dag:53` | `data compiler_source: SourceRoot = { path: "src/v2" }` | Configuration row pointing the compiler-source `SourceRoot` at the v2 source tree. Consumed as compile input; not a Rust import. | **C-data** (not C-cosmetic) | **G-2 prerequisite**, not G-1. Disposition is **routed to S-1** (PM-authored worker brief): does `compiler_source.path` retarget to a v3-side root when v2 retires, or does `compiler.dag`'s self-compile target itself refactor under T-FixedPoint / SG-0 = 0 closure? **No disposition proposed here**; flagged so S-1 enumerates it explicitly rather than letting it slip into a Population C cleanup pass that would silently drop the path value. |
| `dsl/gunbc/compiler.dag:270` | `data test_package: NonEmptyStr = "v2-compiler-tests"` | Configuration row naming the Cargo test package the compiler-pipeline test driver invokes (`cargo test -p v2-compiler-tests …`). Consumed as a typed string constant; not a Rust import. | **C-data** (not C-cosmetic) | **G-2 prerequisite, not G-1.** This row points at a Cargo *package name*, which remains valid until the package itself is removed (root-workspace member removal, G-2 territory — see §1.4 root `Cargo.toml` L8). It does **NOT** become stale when `src/v3/compiler/Cargo.toml`'s dev-dep edge is deleted (§B.2 Decision 3, G-1 closure): G-1 deletes the consumer-side dependency edge, not the published `v2-compiler-tests` package. Disposition routed to S-1 as a separate G-2 decision (CI test package retargeting): when `src/v2/tests` is removed at G-2, does `test_package` retarget to a v3-side test crate, or does the test-driver authoring change shape? **No disposition proposed here**; flagged for S-1 scope coverage as a G-2 decision distinct from B.2/Decision 3. |

**Why this matters:** Population C's "sweep at G-2" default assumes refs are doc-comments / string literals where deletion or rewording is purely cosmetic. A `data` declaration's path field is a typed value the bootstrap consumes — silently rewriting it during a cosmetic sweep would either point the field at nothing (`compiler_source` becomes invalid) or quietly retarget the compile root without an authoring-time decision. Flagged here as a C-data row with named G-2 routing so S-1's scope coverage (input-packet Decision 6) can include it explicitly.

**Receipt:**
```sh
$ grep -rEn 'src/v2' src/v3/ dsl/ | grep -v Binary
(output as inventoried in C.1 + C.2 above)
```

### 1.4 Workspace-level Cargo.toml (root)

The workspace `Cargo.toml` (repo root) declares both v2 crates as workspace members and carries a v2-compiler-specific dev-profile override:

| Line | Construct |
|---|---|
| L6 | `"src/v2/stage0",` (workspace member) |
| L8 | `"src/v2/tests",` (workspace member) |
| L58 | comment: `# debug_assert! in v2-compiler, not just ownership checks — acceptable` |
| L61 | `[profile.dev.package.v2-compiler]` (dev-profile override section header) |

**Receipt:**
```sh
$ grep -nE 'src/v2|v2-(compiler|compiler-tests)' Cargo.toml
6:    "src/v2/stage0",
8:    "src/v2/tests",
58:# debug_assert! in v2-compiler, not just ownership checks — acceptable
61:[profile.dev.package.v2-compiler]
```

**Migration mapping:**
- L6, L8 — **G-2 (workspace member removal)**, not G-1. Matches matrix §2.1 "crate retires as a unit when G-2's workspace-member removal fires".
- L58, L61 (and the `[profile.dev.package.v2-compiler]` section body that follows) — cosmetic Population-C-equivalent at workspace scope. Cleaned up alongside G-2 deletion. **No edit proposed here.**

The matrix does not currently inventory the root `Cargo.toml` profile section; flagged as a minor gap (G-2 sweep, not G-1).

---

## 2. Cross-reference table

Each Population B / C row above maps back to a migration-matrix surface and (where applicable) a decision in [`r3-pb-tv2-s1-input-packet.md`](../briefs/r3-pb-tv2-s1-input-packet.md).

| HEAD surface | Matrix § | Input-packet decision | Population |
|---|---|---|---|
| `p0_std_render_repeat_string_test.rs` | §3.1 | Decision 1 (replace vs delete) | B |
| `m2_substrate_inhabitance_test.rs::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` | §3.2 | Decision 2 (cross-program routing) | B |
| `src/v3/compiler/Cargo.toml:37-38` | §3.3 | Decision 3 (atomic deletion mechanics) | B |
| Population A (16 files in `src/v2/tests/src/`) | §2.1 + §6.1 | (G-2 territory; no G-1 decision) | A |
| `dsl/extdeps/languages/{rust,python,go}/emit.dag` legacy chain | §4 | Decision 4 (gate timing) | (G-2 prereq, not G-1) |
| `dsl/std/verification.dag` (v2-era) vs `src/v3/std/verification.dag` | §5 | Decision 5 (Substrate-routed) | (G-2 prereq, not G-1) |
| `src/v3/compiler/src/dag.rs` doc-comments | §2.3 | (cosmetic — sweep at G-2) | C |
| `src/v3/compiler/src/pb_method_template_projection_dag_emit.rs:29`, `pb_method_template_projection_dag_emit_test.rs:24,28,54` | (not in matrix) | (cosmetic — sweep at G-2) | C (gap-fill) |
| `dsl/tools/purity_check.dag:157`, `dsl/std/{node,constructors,syntax}.dag`, `dsl/extdeps/{llm/openai,languages/python/syntax}.dag`, `dsl/gunbc/compiler.dag:29`, `dsl/gunbc/tools/{review_codex,ci_runner}.dag` | (not in matrix) | (cosmetic — sweep at G-2) | C (gap-fill) |
| **`dsl/gunbc/compiler.dag:53`** `data compiler_source: SourceRoot = { path: "src/v2" }` | (not in matrix) | **Decision 6 scope coverage (S-1 routes)** | **C-data** (G-2 prereq, named disposition) |
| **`dsl/gunbc/compiler.dag:270`** `data test_package: NonEmptyStr = "v2-compiler-tests"` | (not in matrix) | **Decision 6 scope coverage (S-1 routes)** as a **G-2** decision (CI test package retargeting tied to root-workspace `src/v2/tests` removal at §1.4 / Cargo.toml L8). NOT tied to §B.2 Decision 3 (G-1 dev-dep edge deletion). | **C-data** (G-2 prereq, named disposition) |
| Root `Cargo.toml` L6/L8/L58/L61 | (not in matrix) | (G-2 — workspace removal + profile section sweep) | A-equivalent at workspace scope |

---

## 3. Summary deltas vs migration matrix `66edec52`

The matrix's structural map (Populations A/B/C, G-1 vs G-2 split, ownership routing) is **unchanged**. Only refresh-class deltas at HEAD `2d26ed2b3`:

1. **Population A:** 15 → 16 files (added `pb_method_template_projection_consumability.rs`). Same disposition (G-2 unit retirement).
2. **Population B test-file line citations drifted:** `m2_substrate_inhabitance_test.rs` L991/L992-993/L1005 → L1209/L1210/L1222. Same construct (`v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`); structurally unchanged.
3. **Population B Cargo edge line citations drifted:** `src/v3/compiler/Cargo.toml:32-33` → `:37-38`. Edges unchanged.
4. **Population C dag.rs line set drifted:** matrix listed 6 lines (L526/L983/L1562/L1598/L1602/L3102); HEAD has a single doc-comment at L1793 only. Net Population C count for that file decreased from 6 → 1.
5. **Population C gap-fill:** 11 additional files carry cosmetic `src/v2` references (per §1.3 C.2 + `dsl/gunbc/tools/ci_runner.dag:16` added in 2026-05-06 follow-up). All doc-comment / string-literal; no substantive Rust consumer added.
6. **Population C-data split (2026-05-06 follow-up):** two typed `data` declarations reclassified out of C-cosmetic into a new **C-data** sub-class (§1.3 C.3) with named G-2 routing to S-1, since they're typed config values not doc-comments: `dsl/gunbc/compiler.dag:53` (`compiler_source.path = "src/v2"`) and `dsl/gunbc/compiler.dag:270` (`test_package = "v2-compiler-tests"`).
7. **Inventory line-completeness follow-up (2026-05-06):** Population B's `p0_std_render_repeat_string_test.rs` substantive references expanded from 3 (L9–L11 use-decls) to 4 (added L25 body-of-test call to `v2_compiler::v2_std_core::diagnostic_to_message`); Population C `pb_method_template_projection_dag_emit_test.rs` lines expanded from L28/L54 to L24/L28/L54; Population C `SELF_HOSTING.md` line set added L1178. Earlier inventory used file-level enumeration and missed in-file additional references; the unified single-grep census in §"Search authority" now produces line-level enumeration.
8. **Workspace-Cargo.toml inventory gap:** root `Cargo.toml` v2 lines (L6/L8/L58/L61) not previously inventoried. Surfaced here.
9. **Regex coverage fix (2026-05-06 follow-up):** unified census widened from `src/v2/|\bv2[-_]compiler…` → `src/v2\b|\bv2[-_]compiler…` so `path: "src/v2"` (no trailing slash) is matched by the canonical command. Without this fix the C-data row at `dsl/gunbc/compiler.dag:53` would not be rediscovered by the cited unified grep — which is the surface the C-data split was created to flag.

**G-1 closure surface count: 4 (unchanged from matrix §2.2).** No new substantive `v2_compiler` consumer has surfaced since `66edec52`; the retirement work scope is unchanged.

---

## 4. Reproducibility note

The migration-matrix §1 cites `grep -rEln '...' src/ tests/`. The `tests/` argument is a no-op on current tree (no top-level `tests/` dir exists; integration tests live under `src/<crate>/tests/`, which `src/` already covers recursively). The grep emits a warning but its match set is unaffected. This inventory's counts were produced with the `src/`-only form; matrix-cited form yields the same counts modulo the spurious warning. Flagged as a minor doc-hygiene cleanup for the matrix on its next touch (not in scope for this PR — separate doc).

## 5. Constraints honored (verbatim from dispatch)

- ✅ No `src/v2/` deletion.
- ✅ No `v2-compiler` / `v2-compiler-tests` Cargo edge removal.
- ✅ No code touched. The original PR (#1848) added one new docs-only file at this path; the follow-up PR (#1850) edits that file in place to address codex BLOCKING review findings — no additional files added or removed.
- ✅ No migration disposition decided (all routings remain matrix / S-1 / Substrate-Manager territory).

## Cross-refs

- Parent audit: [`docs/audit/t-v2-retirement-audit.md`](t-v2-retirement-audit.md) (#1338).
- Per-surface migration matrix (the doc this inventory refreshes): [`docs/audit/t-v2-retirement-migration-matrix.md`](t-v2-retirement-migration-matrix.md) (#1346/#1379).
- G-1 readiness receipt: [`docs/briefs/r3-pb-tv2-g1-readiness-receipt.md`](../briefs/r3-pb-tv2-g1-readiness-receipt.md) (#1446).
- S-1 input packet (decision routing PB cannot make): [`docs/briefs/r3-pb-tv2-s1-input-packet.md`](../briefs/r3-pb-tv2-s1-input-packet.md).
- PB Manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](../briefs/r2-pure-bootstrap-manager.md).
- Lane authority: [`docs/r3-structure.md`](../r3-structure.md) §"Lane structure" T-V2-Retirement row.
