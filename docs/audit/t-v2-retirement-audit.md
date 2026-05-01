# T-V2-Retirement — Bounded Planning / Audit

**Status:** PROPOSAL (audit only). Authored 2026-04-30 (silent-boar-29) per Director dispatch via cool-stag-230 (R3 PB).
**Authority basis:** `docs/r3-structure.md` Lane structure §11 row "T-V2-Retirement" (added 2026-04-30) + `docs/r3-structure.md` §165 (the post-R3 → R3 move row). PR #1319 (commit `a83b58bbd`, "gunbc Director", 2026-04-30) is the ratifying merge; it carries the user directive *"nothing can be deferred past R3."* `ROADMAP.md` mentions "v2 retirement" inline at lines 366 and 421 as scope references, but does NOT carry a dedicated `§"v2 retirement"` section — `r3-structure.md` is the live anchor.
**Scope:** planning + audit only. **No code/test/build changes.** No deletions, no workspace edits, no v2/v3 import bridge, no PB-Runtime trampoline implementation.

---

## 1. STOP conditions for implementation

STOP conditions are hard preconditions, but they do **not** all gate the same work. Per-gate authority — single source for "may this work begin?":

- **G-1 (test-consumer dissolution) implementation may begin once S-1 is green.** S-2/S-3/S-4 are NOT prerequisites for G-1: replacing the two v2-oracle test consumers (§2.3) with v3 evaluator equivalents — or deleting them as redundant — needs only the PM-authored disposition (S-1). The v3 evaluator already exists; no FixedPoint/LensProducer/PB-Runtime work is structurally required to retire those two tests.
- **G-2 (workspace-member deletion + `src/v2/` removal) implementation may not begin until S-1 + S-2 + S-3 + S-4 + G-1 are all green** (per §3.2 prerequisites).
- **Audit work** (this doc, refinements, consumer-list updates) is unblocked at all times.

| # | STOP condition | State at audit time |
|---|---|---|
| S-1 | PM-authored `T-V2-Retirement` worker brief landed under `docs/briefs/` | **NOT MET** — `docs/briefs/` contains no `t-v2-retirement-*` or equivalent file as of `origin/main` HEAD `5e6b48b40`. PB manager standing brief (`docs/briefs/r2-pure-bootstrap-manager.md`, refreshed 2026-04-28) predates the 2026-04-30 lane absorption and does not yet enumerate T-V2-Retirement under §"R3 lanes". |
| S-2 | T-FixedPoint closed | NOT MET (R3 in flight; gate per `r3-structure.md` Lane 5). |
| S-3 | T-LensProducer-Retirement closed (all 3 sub-gates: `lens_apply.rs`, `lens_testgen.rs`, `regen_lens.rs`) | NOT MET (R3 in flight; sub-gates per `r2-pure-bootstrap-manager.md` row T-LensProducer-Retirement + `design-pb-runtime-interpreter.md` §5.1). |
| S-4 | PB-Runtime trampoline lands such that bootstrap no longer routes through `src/v2/stage0` | NOT MET (PB-Runtime interpreter-as-data is the gate per `design-pb-runtime-interpreter.md` §3). |

The §3 gate rows (G-1.STOP, G-2.STOP) are the single authority for which STOP rows block which work; this section's table records state only.

---

## 2. v2 footprint inventory (audit snapshot at HEAD `5e6b48b40`)

### 2.1 `src/v2/` source tree

- **79 `.rs` files** under `src/v2/` (`find src/v2 -name '*.rs' | wc -l`).
- **32 `.dag` files** under `src/v2/` (`find src/v2 -name '*.dag' | wc -l`).
  - r3-structure.md §11 cites "~28 `.dag`"; current count is 32. Audit-time correction; not load-bearing for either gate.

### 2.2 Workspace / Cargo references to v2

`Cargo.toml` (`/Cargo.toml:5-7`):
```
members = [
    "src/v2/stage0",   # v2 self-hosted compiler
    "src/v2/tests",    # v2 self-hosted compiler tests
    ...
]
```

`src/v3/compiler/Cargo.toml:32-33`:
```
v2-compiler        = { path = "../../v2/stage0" }
v2-compiler-tests  = { path = "../../v2/tests" }
```

These are the only Cargo edges from non-`src/v2/` crates into v2. **Both edges drop with G-1** (§3.1) — they exist solely to support the two test consumers in §2.3, so once those dissolve the deps are dead and must be deleted as part of G-1 closure (§3.1 green criteria explicitly require this). G-2 then removes the `src/v2/stage0` and `src/v2/tests` workspace members themselves.

### 2.3 Tests / consumers that import or read v2

Search: `grep -rEln 'src/v2/|\bv2_compiler(_tests)?\b' src/ tests/` excluding `src/v2/` itself. (Earlier drafts used `v2_compiler\b`, which fails to match `v2_compiler_tests` because `_` is a word character; the corrected pattern covers both crates explicitly.) After filtering doc-comment-only matches, **substantive consumers are**:

| Consumer | Kind of dependency | Removal class |
|---|---|---|
| `src/v3/compiler/Cargo.toml` (lines 32-33) | Cargo `path =` deps on `v2-compiler` + `v2-compiler-tests` | drops with G-1 (after all callers below dissolve) |
| `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs` | Uses `v2_compiler::v2_compiler_compile::compile_to_resolved`, `v2_compiler::v2_interpreter`, `v2_compiler_tests::helpers::resolve_imports_transitively` as **behavior oracle** for `repeat_string` lower-time fold | **Test consumer** — directly counts against `v2_oracle_no_remaining_test_consumers`. Owner: PB Manager (per lane). Disposition: replace v2 oracle with v3 evaluator equivalence-corpus row, OR delete the test if the property is structurally guaranteed. |
| `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs` | `v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority` (line 991) compares v3 `dag::kernel_algebra_profile` against `v2_compiler::std_algebra::kernel_algebra_profile()`. Dual-authority drift ratchet. | **Test consumer** — counts against G-1. Disposition: once v3 substrate inhabitance is the single authority (via `dsl/std/algebra.dag`), retire the v2-mirror parity test; v3-side authority continues under the existing inhabitance suite. Cross-ref `feedback_isomorphism_or_generation_for_mirrors.md`. |
| `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs` (line 140) | String literal `"src/v2/complexity.dag (5488L)"` only — **not a code dependency** | **NOT a G-1 consumer**; cosmetic update at deletion time. |
| `src/v3/compiler/src/dag.rs` (lines 526, 983, 1562, 1598, 1602, 3102) | Doc-comment references to v2 only; no `use v2_compiler` | **NOT a G-1 consumer**; comment hygiene at deletion time. |
| `src/v3/std/anthropic_operations.dag:20`, `src/v3/std/rust_method_template_contracts.dag:21`, `src/v3/lenses/complexity.dag:13` | Doc-comment references | **NOT a G-1 consumer**; comment hygiene. |
| `src/v3/SELF_HOSTING.md` | Doc reference | **NOT a G-1 consumer**. |

**Test-consumer count for `v2_oracle_no_remaining_test_consumers`: 2** (`p0_std_render_repeat_string_test.rs`, `m2_substrate_inhabitance_test.rs`). Plus the entire `src/v2/tests` crate (15 test files), which is internal to v2 and falls with G-2 (workspace member removal).

### 2.4 Legacy emit chain

Per `src/v3/std/rust_method_template_contracts.dag:1-22` (header note): the v2 emit pipeline still consumes the legacy `rust_simple_method_specs` / `rust_method_templates()` / `rust_method_wraps_result()` chain in `dsl/extdeps/languages/rust/emit.dag` because v2 has no bootstrap-Dag consumer infrastructure. **Full retirement of those legacy authorities is part of T-V2-Retirement scope** (already named in the file's header comment). When G-2 fires, the `dsl/extdeps/languages/{rust,python,go}/emit.dag` legacy chain becomes deletable (audited at deletion time, not now).

### 2.5 Dual `verification.dag` convergence

Per ROADMAP.md §"File-preference rank is a ratified-parallel-authority scaffold": `module std.verification` exists at both `dsl/std/verification.dag` (v2 surface: `AssertKind / TestClaim / TestCase`) and `src/v3/std/verification.dag` (v3 surface: `DiagnosticKind / DiagnosticReference / PortStateExpectation`). The two surfaces are **disjoint today**; convergence requires a design call on which surface wins (or whether both are kept under disjoint module paths).

**Audit position:** convergence of `verification.dag` is a **prerequisite for G-2** (deleting `src/v2/` requires that no surviving authority depends on the v2 `verification.dag` surface). It is **NOT** a prerequisite for G-1 (test-consumer dissolution does not require the convergence). Owner: Substrate Manager (continuation) for the design call; PB Manager for the v2-side deletion once the call lands.

### 2.6 Bootstrap path

Bootstrap currently routes through `src/v2/stage0`. Until PB-Runtime trampoline lands (S-4), G-2 is structurally blocked: removing `src/v2/stage0` from the workspace breaks the build chain. PB-Runtime trampoline is the **structural prerequisite** that converts `src/v2/` from "active bootstrap dependency" to "pure historical reference."

---

## 3. Gate-by-gate disposition

### 3.1 Gate G-1 — `v2_oracle_no_remaining_test_consumers`

| Field | Value |
|---|---|
| **Definition** | No test outside `src/v2/` references `v2_compiler` or `v2_compiler_tests` crates (substantive `use` / function call, not doc-comment / string-literal). |
| **Current consumers (count: 2)** | `src/v3/compiler/tests/integration/p0_std_render_repeat_string_test.rs`; `src/v3/compiler/tests/integration/m2_substrate_inhabitance_test.rs`. Plus the Cargo edges in `src/v3/compiler/Cargo.toml:32-33` which become removable once both tests dissolve. |
| **Owner** | PB Manager (R3 continuation); cross-ref Substrate Manager for `kernel_algebra_profile` authority migration. |
| **Prerequisites** | (a) v3-side replacement for the `repeat_string` lower-time-fold oracle (equivalence-corpus row backed by v3 evaluator, OR proof that the property is structurally guaranteed and the test is redundant). (b) v3-side single-authority for `kernel_algebra_profile` such that the v2-mirror parity test is no longer load-bearing. |
| **STOP condition for G-1 work** | S-1 (PM worker brief). Without the brief, the per-test disposition (replace vs. delete) is not authorized. |
| **What counts as green** | A grep over `src/v3/compiler/tests/` and `tests/` for `v2_compiler\b` or `v2_compiler_tests\b` returns zero matches in non-doc-comment positions. Cargo edges in `src/v3/compiler/Cargo.toml:32-33` deleted. `cargo test --workspace --exclude v2-compiler-tests` still passes (i.e., test coverage was not silently dropped). Receipt artifact: short closure note linking to the two test dispositions. |

### 3.2 Gate G-2 — `v2_directory_deleted`

| Field | Value |
|---|---|
| **Definition** | `src/v2/` directory removed; `Cargo.toml` workspace `members` no longer lists `src/v2/stage0` or `src/v2/tests`; bootstrap chain routes through PB-Runtime trampoline only. |
| **Current consumers** | The build itself (workspace members), via `src/v2/stage0` as bootstrap source-of-truth; the legacy emit chain (`dsl/extdeps/languages/{rust,python,go}/emit.dag` — `rust_simple_method_specs`, `rust_method_templates()`, `rust_method_wraps_result()`); the `dsl/std/verification.dag` v2 surface. |
| **Owner** | PB Manager (R3 continuation). |
| **Prerequisites** | (a) **G-1 green** (no test consumers). (b) **S-4 green** — PB-Runtime trampoline is the live bootstrap. (c) Legacy emit chain (§2.4) retired or migrated to v3 authorities. (d) `verification.dag` convergence design-call landed (§2.5) and v2 surface no longer load-bearing for any surviving authority. |
| **STOP condition for G-2 work** | S-1 + S-2 + S-3 + S-4 + G-1 (matches §1 per-gate authority). S-2 (T-FixedPoint) and S-3 (T-LensProducer-Retirement) are explicit prerequisites because their closure is what allows S-4 (PB-Runtime trampoline) to be the live bootstrap; without S-2+S-3, removing `src/v2/stage0` from the workspace breaks the build chain even if PB-Runtime is technically present. |
| **What counts as green** | `find src/v2 -type f` returns empty (or directory does not exist). `Cargo.toml` workspace `members` array does not contain any `src/v2/...` entry. `cargo build --workspace` + `cargo test --workspace` both pass with no remaining `v2-compiler*` references. `dsl/extdeps/languages/{rust,python,go}/emit.dag` legacy chain deleted or audited as no-consumer. PR description includes an explicit checklist mapping each prerequisite (a)-(d) to a closed receipt. |

---

## 4. Consequence chain (why the gates are in this order)

```
S-2 T-FixedPoint  ──┐
S-3 T-LensProducer ─┼──> S-4 PB-Runtime trampoline live ──> G-2 prerequisite (b)
                    │
S-1 PM brief ───────┴──> G-1 dispositions authorized ──> G-1 green ──> G-2 prerequisite (a)
                                                                       │
                  legacy emit chain retired (§2.4) ───────────────────►├──> G-2 green
                                                                       │
                  verification.dag convergence (§2.5) ────────────────►┘
```

**Consequence:** T-V2-Retirement is structurally a *consequence* of T-FixedPoint + T-LensProducer-Retirement + the legacy-emit / verification convergence work — not a parallel program. Pulling it into R3 (per Director directive) is structurally cheap precisely because the dependency edges are already named; the lane carries auditing + sequencing, not new substrate.

---

## 5. Non-goals (audit-time reaffirmation)

- **No code deletion** as part of this audit. All deletions wait on §3 prerequisites.
- **No workspace member removal.** Both `src/v2/stage0` and `src/v2/tests` remain workspace members until G-2 fires.
- **No test rewiring.** The two G-1 consumer tests are catalogued, not modified.
- **No v2/v3 import bridge.** Per `feedback_no_textual_enforcement_bridges.md` + `feedback_construction_over_ratchets.md`: do not introduce a re-export shim or compatibility module to "ease" deletion. Either the consumer dissolves to a v3-native equivalent, or the test is deleted with a receipt.
- **No PB-Runtime trampoline implementation** in this lane — that work belongs to T-LensProducer-Retirement / T-FixedPoint and is a *prerequisite*, not a sub-task, of T-V2-Retirement.
- **No claim that T-V2-Retirement implementation can start before its STOP conditions are green.**

---

## 6. Open audit questions (for PM brief author)

These are NOT decisions — they are routing questions the PM-authored worker brief should resolve:

1. **G-1 disposition for `p0_std_render_repeat_string_test.rs`:** replace with v3 evaluator row, or delete as redundant (if `repeat_string` lower-time fold is structurally guaranteed)? Cross-ref `r3-pb-runtime-equivalence-corpus-seed-audit.md`.
2. **G-1 disposition for `m2_substrate_inhabitance_test.rs::v3_kernel_algebra_profile_mirror_matches_v2_stage0_authority`:** which v3-side authority replaces `v2_compiler::std_algebra::kernel_algebra_profile()` as the single source? Owner of the migration: PB or Substrate?
3. **§2.5 `verification.dag` convergence:** which surface wins — v2's `AssertKind/TestClaim/TestCase`, v3's `DiagnosticKind/DiagnosticReference/PortStateExpectation`, or both under disjoint module paths? Substrate Manager design call.
4. **§2.4 legacy emit chain:** retire under T-V2-Retirement scope, or split into a sibling sub-lane? Header note in `rust_method_template_contracts.dag` already names retirement as deferred to "Pure-Bootstrap-Zero / v2 retirement scope" — confirm scope boundary.

---

## 7. Acceptance summary

This audit is intentionally bounded:

- **Inventory:** §2 (v2 footprint, consumers, legacy emit chain, dual verification surface, bootstrap path).
- **Gate dispositions:** §3 (G-1, G-2 — each with consumers, owner, prerequisites, STOP condition, green criteria).
- **Sequencing:** §4 (consequence chain).
- **Non-goals:** §5.
- **Routing for PM brief:** §6.

**Implementation remains gated on §1 STOP conditions, foremost S-1 (PM worker brief).** This document is suitable input for that brief.
