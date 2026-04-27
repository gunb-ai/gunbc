# PR #825 (B4.4 extdeps bootstrap fixture carrier) — pre-merge review vs `main`

**Purpose.** Checkable receipt: compare automated and human PR #825 review findings against the tree that merged to `main` (merge commit `5fb84cf2d`, message `B4.4 (#825)`), so later readers can confirm blockers were resolved or explicitly superseded.

**Authority on `main` (spot-check paths).**

| Area | Location |
|------|----------|
| Substrate fixture set | `src/v3/std/extdeps_bootstrap_fixtures.dag` — `extdeps_bootstrap_fixture_authority` |
| Regen filter keys | `src/v3/compiler/src/bootstrap.rs` — `EXTDEPS_BOOTSTRAP_PATH_KEYS` |
| Regen-time resolution check | `src/v3/compiler/src/bootstrap_regen_fresh.rs` — `assert_extdeps_bootstrap_keys_resolve_against_extdeps_files` |
| Runtime snapshot ↔ keys lockstep | `src/v3/compiler/src/dag.rs` — `assert_extdeps_bootstrap_fixture_paths_match_regen_keys`, `Dag::extdeps_bootstrap_fixture_virtual_paths` |
| Generated bootstrap bodies | `src/v3/compiler/src/bootstrap_generated.rs` and `bootstrap_generated_without_parse_surface.rs` — include `extdeps_bootstrap_fixture_authority` |

**Mechanical verification (run locally when `cargo` is available).**

```bash
cargo run -p v3-compiler --features bootstrap-regen-fresh --bin regen_bootstrap -- --verify
```

Expect **success** (DB-8 snapshot discipline). CI on the merged PR used the same gate.

---

## Review findings → disposition

### 1. Codex (`59160671`) — **BLOCKING**: committed bootstrap snapshots out of date

- **Claim.** New substrate declaration without matching `bootstrap_generated*.rs`; `regen_bootstrap --verify` would fail; runtime loads snapshots, not fresh `.dag` alone.
- **On `main`.** Snapshots contain `extdeps_bootstrap_fixture_authority` with the rust primitives virtual path (see grep in `bootstrap_generated.rs` around `extdeps_bootstrap_fixture_authority`).
- **Verdict.** **Landed.** Matches worker follow-up on the PR thread (full regen + verify green).

### 2. Claude Opus (`59160671`) — parallel `.dag` vs Rust const without enforced bridge

- **Claim.** Substrate carrier and `EXTDEPS_BOOTSTRAP_PATH_KEYS` could drift; comments implied tests that did not compare substrate to const.
- **On `main`.** `assert_extdeps_bootstrap_fixture_paths_match_regen_keys` runs at committed-bootstrap `Dag` init and compares ordered `virtual_path` list from the snapshot to `EXTDEPS_BOOTSTRAP_PATH_KEYS`; panic on mismatch. Regen path asserts keys resolve into `EXTDEPS_FILES`.
- **Verdict.** **Landed** as explicit lockstep (fail-closed panic), not silent drift.

### 3. Cursor / Composer (`f035183b`, `0462d1c9`) — **APPROVE**

- **Claim.** No invariant violations on changed lines; lockstep documented.
- **Verdict.** **Consistent with `main`** (same wiring as above).

### 4. OpenAI-pro (`f035183b`) — **REQUEST_CHANGES**: dual authority + missing named dissolution trigger for the const mirror

- **Claim.** One fact in `.dag` and Rust const; pre-promotion ordering may justify the bridge, but debt should name a **dissolution trigger** for eliminating hand-authored `EXTDEPS_BOOTSTRAP_PATH_KEYS` (e.g. generate keys from substrate once regen can read the carrier first).
- **On `main`.** Bridge remains by design (`extdeps_bootstrap_fixtures.dag` still notes this file + parallel const). **Ordered parity is enforced** at runtime and regen asserts key membership in `EXTDEPS_FILES`.
- **Verdict.** **Partial.** Mechanical **non-drift** requirement is satisfied. A **standalone ROADMAP row** dedicated to “derive/delete `EXTDEPS_BOOTSTRAP_PATH_KEYS`” was not added as a named dissolution trigger in that PR; treat as **optional follow-up** if Substrate Manager wants an explicit ledger line separate from the generic “filename / sentinel bridges” pattern row (which still names legacy `EXTDEPS_BOOTSTRAP_FIXTURES` phrasing in places — B4.4 superseded that constant name in code).

### 5. Codex (`6484954a`, post-merge) — live calls vs request-wire gap

- **Claim.** Typed `messages` while request-wire serde gap documented — boundary concern.
- **Disposition on PR.** Addressed by director merge approval with residues carried as structural-coverage-gap data; **not a #825 scope item** (see PR #901 / `rest_request_wire_serde_alignment` in `dsl/extdeps/llm/*.dag`).

### 6. Infra-only “BLOCKING” / sandbox diff-fetch failures

- **Verdict.** **Not code findings** — no audit action.

---

## Summary table

| Source | Severity | Topic | Landed on `main`? |
|--------|----------|--------|-------------------|
| Codex | BLOCKING | Snapshot / DB-8 verify | Yes |
| Claude | P2 | Substrate ↔ const parity | Yes (assertions) |
| OpenAI-pro | BLOCKING | Named dissolution trigger for const mirror | Partial (parity enforced; explicit ROADMAP dissolution line optional) |
| Others | — | APPROVE / infra | N/A |

**Bottom line.** The **merge-blocking** codex snapshot finding and the **parity** finding are **verified addressed** in the merged shape. The **openai-pro “name a dissolution trigger”** ask is **best-effort satisfied** by enforced lockstep + comments; add a dedicated ROADMAP debt row only if the program wants that trigger spelled in the ledger verbatim.
