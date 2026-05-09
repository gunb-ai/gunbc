# T-V2-Retirement — G-2 Deletion Plan & Guardrails

**Status:** PROPOSAL (planning only). Authored 2026-05-02 (silent-boar-29) per Director dispatch via cool-stag-230 (R3 PB).
**Parents:** `docs/audit/t-v2-retirement-audit.md` (#1338, merged), `docs/audit/t-v2-retirement-migration-matrix.md` (#1346/#1379 corrections, merged).
**Authority basis:** parent audit §3.2 (G-2 STOP condition `S-1 + S-2 + S-3 + S-4 + G-1`); migration matrix §2.1 (Population A — 13 internal `src/v2/tests` files + 2 non-test workspace `lib.rs`/`bug_sentinel_ratchet.rs`), §3.3 (Cargo edges drop with G-1, not G-2), §4 (legacy emit chain G-2 prerequisite), §5 (verification.dag convergence routed to Substrate).
**Scope:** docs-only deletion plan + guardrails. **No `src/v2/` deletion. No workspace-member removal. No code changes. No CI wiring. No baseline JSON.**

This artifact captures the bounded shape the eventual G-2 deletion PR must take so the deletion lands as one reviewable atomic step rather than an unbounded sweep. It is a guardrail packet, not a dispatch order.

---

## 1. STOP conditions (deletion may not begin until ALL green)

Re-stated from parent audit §1 + §3.2. The §1 table also includes three explicit G-2 prerequisite rows: G-2-prereq-emit (legacy emit chain retirement, derived from migration matrix §4.2) and G-2-prereq-verif (`verification.dag` convergence, derived from migration matrix §5) — both lifted from the parent matrix into row form here. G-2-prereq-ci (CI workflow no longer invoking v2-compiler) is newly surfaced by this plan on 2026-05-02. Verified at HEAD before deletion PR opens:

| # | Prereq | What "green" looks like at deletion time |
|---|---|---|
| S-1 | PM-authored T-V2-Retirement worker brief | File exists under `docs/briefs/`; references `T-V2-Retirement`; brief PR merged — verified via `gh pr view <pr-number> --json state,mergedAt --jq '.state == "MERGED"'` returning `true` (the `--state` flag belongs to `gh pr list`, not `gh pr view` — use the JSON form to make the check executable). |
| S-2 | T-FixedPoint closed | `pb_self_compile_fixed_point` predicate green under R3 elevated bar; closure ledger receipt. |
| S-3 | T-LensProducer-Retirement closed | All three sub-gates green (`lens_apply.rs`, `lens_testgen.rs`, `regen_lens.rs` retired); files do not exist under `src/v3/compiler/src/`. |
| S-4 | PB-Runtime trampoline live as bootstrap | `cargo build --workspace` succeeds without `src/v2/stage0` invocation in the bootstrap chain. Verified by removing **both** `src/v2/stage0` AND `src/v2/tests` from `Cargo.toml` workspace `members` in a *throw-away* check (NOT committed) and confirming compile + run works through PB-Runtime alone. **Both must be removed**: `src/v2/tests/Cargo.toml:9` path-depends on `../stage0` (`v2-compiler = { path = "../stage0" }`), so removing only stage0 leaves tests pulling it in transitively (fail-open). |
| G-1 | `v2_oracle_no_remaining_test_consumers` green | Per migration matrix §3 (single authority): `grep -rEn 'v2[_-]compiler(_tests|-tests)?' src/` excluding `src/v2/` returns zero substantive matches; Cargo edges in `src/v3/compiler/Cargo.toml:32-33` deleted (§3.3). G-1 closure does NOT include the legacy emit chain or verification.dag convergence — those are separate G-2 prerequisites tracked in the next two rows. |
| G-2-prereq-emit | Legacy emit chain retired (G-2 prerequisite per migration matrix §4.2) | `{rust,python,go}_simple_method_specs` / `*_method_templates` / `*_method_wraps_result` deleted from `dsl/extdeps/languages/{rust,python,go}/emit.dag` — per-target, all three families. Per matrix §4.2 STOP/green criteria. |
| G-2-prereq-verif | `verification.dag` convergence landed (G-2 prerequisite per migration matrix §5) | Substrate-led design call ratified; v2-era `dsl/std/verification.dag` surface either dissolved into v3's `src/v3/std/verification.dag` (`TestPredicate`/`TestClaim { ..., requires: List<ResourceReference> }`/`TestSuite`/`TestObligation`) or moved to a renamed module path; no surviving authority depends on the v2-era surface. Routed to Substrate Manager per matrix §5.2. |
| G-2-prereq-ci | CI workflow no longer invokes v2-compiler (G-2 prerequisite, surfaced 2026-05-02) | `.github/workflows/ci.yml` has no `v2-compiler` references. At surfacing time the `ci` job ran `cargo build -p v2-compiler --release`, `cargo run -p v2-compiler --release -- run --source-root dsl --function run_ci_pipeline`, and a cache key hashing `src/v2/stage0/src/**` (line numbers omitted — workflow churns; identify by symbolic anchor via the grep below). All three must be retired or replaced (with v3-side equivalents under PB-Runtime) in a **separate prior PR** before the deletion PR opens — otherwise the deletion PR's `ci` job fails on "package `v2-compiler` not found". Green criterion: `grep -nE 'v2[_-]compiler|src/v2/stage0' .github/workflows/ci.yml` returns zero matches. |

If any row is not green, deletion PR MUST NOT open. STOP+PING with the unmet row.

---

## 2. Deletion PR shape (single atomic PR)

The G-2 deletion is **one PR with one commit** that does exactly the operations below. No surrounding refactor; no opportunistic cleanup that isn't structurally required by deletion.

### 2.1 File-system operations

```sh
# Run in the workspace root, on a fresh branch from origin/main, only when §1
# is verified green. Order is structural: workspace members removed before the
# tree is deleted so cargo doesn't index stale member paths.

# Step 1: workspace member removal in Cargo.toml.
#   Edit: remove the two member lines for "src/v2/stage0" and "src/v2/tests"
#         (and any v2-only comment block adjacent to them; do not touch other members).

# Step 2: tree deletion.
git rm -r src/v2/

# Step 3: regen / lockfile refresh — let cargo update Cargo.lock to drop
#         v2-compiler / v2-compiler-tests entries; do NOT hand-edit Cargo.lock.
cargo build --workspace
```

### 2.2 What MUST NOT be in the deletion PR

- ❌ No edits to `src/v3/` code (G-1 already removed v2-* deps from `src/v3/compiler/Cargo.toml` per migration matrix §3.3).
- ❌ No edits to `dsl/extdeps/languages/{rust,python,go}/emit.dag` (the G-2-prereq-emit row in §1 — legacy emit chain — already retired before this PR).
- ❌ No edits to `dsl/std/verification.dag` (the G-2-prereq-verif row in §1 — Substrate-led convergence — already landed before this PR; v2-era surface dissolved or renamed).
- ❌ No new tests, no fixture changes, no comment-cleanup sweeps in unrelated files.
- ❌ No CI workflow edits.
- ❌ No SG-0 census expansion. (`src/v2/` files were never on `EXPECTED_HAND_AUTHORED_*` because the SG-0 census root is `src/v3/compiler` per `sg0_census_test.rs`. Verify at deletion time; if drift placed any v2 file on the list, removal is in-scope for the same PR.)

### 2.3 Cosmetic Population C cleanup (in scope per migration matrix §6.4)

Inside the deletion PR, sweep doc-comment / string-literal `src/v2/` references whose continued presence is misleading post-deletion:

```sh
grep -rln 'src/v2/' src/v3/ dsl/ docs/ | xargs -I {} echo "review: {}"
```

Per-tree disposition (must be consistent with Gd-1, which fail-closes on zero matches in `src/`, `dsl/`, `.github/`):

- **`src/v3/`, `dsl/`, `.github/`** — match must be **deleted or rephrased to remove the trigger**. "Keep" is NOT permitted in these trees because Gd-1 scans them and would fail-open if a match survived.
- **`docs/`** — "keep" permitted for genuine historical references (Gd-1 does not scan `docs/`); delete rotted comments; rephrase general statements that no longer need v2 context.

Per migration matrix §6.4: "do not pre-empt; sweep at G-2." This sweep is bounded and listed explicitly so it doesn't expand into a broader churn.

---

## 3. Pre-merge guardrails (verified inside the deletion PR's CI)

These checks MUST hold on the deletion PR before merge. Two enforcement classes:

- **Gd-1..Gd-5: mechanical CI / grep gates.** Existing CI invocations + scriptable greps; fail-closed by construction (no human discretion required).
- **Gd-6..Gd-7: human / process guardrails.** Gd-6 requires PB Manager reviewer sign-off on the receipt-table (per §6); Gd-7 is standard repo discipline enforced by the operator running `git push`. Neither is a CI check; both are reviewer-enforced.

| # | Guardrail | Verification |
|---|---|---|
| Gd-1 | No remaining v2 references in `src/`, `dsl/`, `.github/` | See verification command below the table; returns zero matches that aren't already in the PR's deletion diff. (Repo has no top-level `tests/` dir; v3 tests live under `src/v3/compiler/tests/` and are reached via the recursive `src/` scan.) |
| Gd-2 | Workspace builds | `cargo build --workspace` exit 0; `cargo test --workspace` exit 0. (No more `--exclude v2-compiler-tests` flag needed; the crate is gone.) |
| Gd-3 | `Cargo.lock` no longer references v2 | `grep -n 'v2-compiler' Cargo.lock` returns zero matches. |
| Gd-4 | SG-0 census still passes | `cargo test -p v3-compiler --test integration sg0_census_test` green; the census root `src/v3/compiler` is unchanged by this PR. |
| Gd-5 | `fmt`, `ci`, `v3`, `self_host_ratchet` all green | Standard CI matrix on the deletion PR. |
| Gd-6 | PR description includes the §1 STOP-condition green-receipt table | Each of the 8 §1 rows — S-1, S-2, S-3, S-4, G-1, G-2-prereq-emit, G-2-prereq-verif, G-2-prereq-ci — links to the merged PR or closure-ledger receipt that took it green. Reviewer rejects the PR if any row is unevidenced. |
| Gd-7 | No `--no-verify` push, no force-push to deletion branch | Standard repo discipline. |

Gd-1 verification command (kept outside the table to avoid markdown pipe-escape pitfalls; in `grep -E`, `\|` is a literal `|` not alternation, so the in-table form silently misses one of the alternatives):

```sh
# Scan live source / dsl / workflow trees (no `tests/` — repo has no
# top-level tests dir; tests live under src/v3/compiler/tests/ which
# is reached via the recursive src/ scan). Catches both Rust path-style
# (v2_compiler / v2_compiler_tests, underscore) AND Cargo dep-style
# (v2-compiler / v2-compiler-tests, hyphen). Catches src/v2 whether
# followed by `/`, `"`, or word-boundary (e.g. `path: "src/v2"` in
# dsl/gunbc/compiler.dag:53 — no trailing slash).
grep -rEn 'v2[_-]compiler(_tests|-tests)?|src/v2($|/|"|[^a-zA-Z0-9_])' src/ dsl/ .github/
```

Green: returns zero matches that aren't already in the PR's deletion diff. **Live authorities the prior pattern would have missed** (verified at HEAD): `dsl/gunbc/compiler.dag:53` (`path: "src/v2"`); `dsl/gunbc/compiler.dag:266` (`"v2-compiler-tests"` hyphenated); `dsl/std/node.dag:9`, `dsl/std/syntax.dag:79`, `dsl/std/constructors.dag:52`, `dsl/extdeps/llm/openai.dag:90`, `dsl/extdeps/languages/python/syntax.dag:43`, `dsl/tools/purity_check.dag:157`, `dsl/gunbc/tools/review_codex.dag:75`, `dsl/gunbc/tools/ci_runner.dag:16`. These each need their own disposition before Gd-1 can pass — most are doc-comments or string literals (Population C cleanup per §2.3); `dsl/gunbc/compiler.dag` is live authority and gets retired/repointed via the v3 compiler-source migration before deletion.

---

## 4. Rollback plan

`git rm -r src/v2/` is reversible up to the merge. After merge:

- **If a regression surfaces within 7 days:** revert the deletion PR with `git revert <merge-sha>` (keeps history of why deletion happened); restore workspace members in `Cargo.toml`; re-run `cargo build --workspace`. Net: full rollback in one revert PR.
- **If a regression surfaces after 7 days:** treat as a fresh feature reintroduction, not a revert — re-import the relevant code from the pre-deletion commit, with explicit director approval and a receipt naming the regression class. The 7-day window matches the typical PR-review SLA; beyond it, divergence in main makes a clean revert unsafe.
- **Post-merge `Cargo.lock`:** `Cargo.lock` will be regenerated by `cargo build` after revert; do not hand-edit.

---

## 5. What this plan deliberately does NOT do

- ❌ Does not specify the deletion timeline or schedule. Timing is governed by §1 STOP conditions; this plan is shape, not schedule.
- ❌ Does not enumerate the v2 file count or per-file disposition. The migration matrix §2.1 already named the 13 substantive `v2_compiler`-importing test files in `src/v2/tests/src/` (15 `.rs` files total in the crate when counting `lib.rs` + `bug_sentinel_ratchet.rs`) and 124 total files under `src/v2/`; the deletion is `git rm -r src/v2/`, no per-file plan needed.
- ❌ Does not propose new substrate, new tests, or new authorities. This is a removal plan only.
- ❌ Does not include the Substrate-Mgr `verification.dag` convergence call. That work belongs to migration matrix §5 / Substrate Manager and is the §1 G-2-prereq-verif row, not part of this PR.
- ❌ Does not include capture of any baseline data, perf gates, or CI-runner changes. Orthogonal lane (C1).

---

## 6. Routing question (single, for PB Manager)

**Who signs off on Gd-6?** Standing review cadence (CI + scheduled-review providers) covers Gd-1..Gd-5 and Gd-7. Gd-6 (STOP-condition green-receipt table) requires a human sign-off that all 8 receipts (S-1, S-2, S-3, S-4, G-1, G-2-prereq-emit, G-2-prereq-verif, G-2-prereq-ci) are valid. **Single-reviewer rule: PB Manager owns Gd-6 sign-off** on the deletion PR; ambiguous-ownership reviews are not acceptable. Director involvement is escalation-only — if PB Manager finds the receipt-table *format itself* under-specified at deletion time, escalate format ratification to Director once and apply the ratified format on subsequent reviews; Gd-6 sign-off authority remains PB Manager.

---

## 7. Acceptance summary

This plan is intentionally minimal:

- §1: STOP conditions re-stated; deletion PR may not open until all 8 (S-1..S-4 + G-1 + G-2-prereq-emit + G-2-prereq-verif + G-2-prereq-ci) are green (verified at HEAD).
- §2: Deletion PR shape — single atomic PR, three structural file-system operations, explicit out-of-scope list, bounded Population C sweep.
- §3: 7 guardrails total — Gd-1..Gd-5 mechanical CI/grep gates; Gd-6..Gd-7 human/process guardrails (PB Manager sign-off and repo discipline).
- §4: 7-day rollback window via `git revert`; beyond that, treat as fresh reintroduction.
- §5: Explicit non-goals.
- §6: Single PB-Manager routing question (reviewer for Gd-6).

**No deletion has been performed.** This plan stands ready for the eventual deletion PR; STOP conditions remain the gate.
