---
status: QUEUED — substrate-prereq-blocked; NOT ready-to-dispatch
owning_manager: Pure Bootstrap Manager (R3)
lane: T-LensProducer-Retirement — driver+entry retirement (lens-producer subset residual decrement: 3 → 1)
authored: 2026-05-15 (zesty-dove-792 — Release PM, on operator-direct dispatch; spike receipt: vivid-crab-154 — operator-personal worker)
depends_on:
  - PB-1 BinShim Rust emitter `.dag` program (row #3 of `docs/briefs/r3-pb-binshim-blocker-ledger.md` — NOT LIVE at HEAD)
  - `dsl/std/runtime/**` loader gap closure (read/write/env/process carriers reachable from `regen_bootstrap`)
  - `compile_to_dag` + `emit_rust_module` callable-from-`.dag` story (compiler-compiles-compiler boundary)
parent_brief: docs/briefs/r3-pb-binshim-retirement-worker.md (template + locked substrate pattern; this brief inherits its STOP-AND-PING discipline)
sibling_brief: docs/briefs/r3-pb-regen-lens-consumer-audit.md (consumer-side audit; informs but does not gate)
---

# R3 PB — `regen_lens_driver.rs` + `regen_lens_entry.rs` retirement

**Status: QUEUED.** Substrate prerequisites enumerated in §"Substrate prerequisites" are NOT LIVE at HEAD as of this brief's authoring date. **Do NOT dispatch a worker** until the worker's first-step grep (per parent brief §"Dispatch preconditions") confirms ALL prereqs land. This brief exists so the work is named, sequenced, and ready to pick up — not so it ships.

## Why this brief exists

Verified during 2026-05-15 spike (worker `vivid-crab-154`):

- The lens-producer subset gate `lens_producer_files_remaining` (§1.8 #66, `src/v3/compiler/tests/integration/r3_lens_producer_retirement_executable_witness_test.rs:1-15`, `CURRENT_RESIDUAL_COUNT: i64 = 3`) names three residual files: `lens_declaration_apply.rs`, `regen_lens_driver.rs`, `regen_lens_entry.rs`.
- `lens_declaration_apply.rs` is sequencing-held on PB-Runtime / Row-4 per its own docstring (`src/v3/compiler/src/lens_declaration_apply.rs:1-9`); covered.
- `regen_lens.rs` (the legacy `src/bin/` shim) was retired by gate #7 (PR #3083, commit `6affbe75c`, 2026-05-14), with `regen_lens_driver.rs` + `regen_lens_entry.rs` deliberately ADDED to the residual count per Codex REQUEST_CHANGES on PR #3083 (commit message: *"retiring src/bin/regen_lens.rs must not shrink lens_producer_files_remaining — count regen_lens_driver.rs and regen_lens_entry.rs alongside lens_declaration_apply.rs (live residual 3)"*).
- **No existing brief targets driver + entry retirement.** Parent `r3-pb-binshim-retirement-worker.md` and follow-on `r3-pb-binshim-batch-followon-worker.md` cover the `src/v3/compiler/src/bin/**` subset (the 9-shim BinShim chain). The driver and entry live OUTSIDE `src/bin/` (per `test_runner.rs:6747` `is_bin_shim_census_path` body) and require a distinct substrate-emit story.

The dissolution path is named verbatim in the census authority. `src/v3/compiler/tests/integration/sg0_census_test.rs:78-82`:

> "R3 gate #7 (`regen_lens_dot_rs_retired`) requires `{RETIRED_REGEN_LENS_BIN_RS}` to stay retired. The `regen_lens` Cargo bin delegates through `src/regen_lens_entry.rs` into `regen_lens_driver.rs` **until PB-1 emits the shim from `.dag`**."

This brief names what "PB-1 emits the shim from `.dag`" requires, in citable detail.

## Scope

**IN:**
- Atomic retirement of `src/v3/compiler/src/regen_lens_driver.rs` (266 LOC, 8 fns) AND `src/v3/compiler/src/regen_lens_entry.rs` (23 LOC, 1 fn) in a single PR.
- Both census entries (`src/v3/compiler/tests/integration/sg0_census_test.rs:352-353`) removed from `EXPECTED_HAND_AUTHORED_NON_TEST` in the same PR.
- Lens-producer residual count witness strings updated in `r3_lens_producer_retirement_executable_witness_test.rs` (currently `CURRENT_RESIDUAL_COUNT: i64 = 3`; this PR reduces to 1).
- `gate #7 stays-retired` assertion message in `sg0_census_test.rs:78-82` updated (its language references the about-to-retire files as the delegation chain; the message must change when they're gone).
- `regen_lens` Cargo `[[bin]]` target (`src/v3/compiler/Cargo.toml:62-64`) continues to build and run — `cargo run -p v3-compiler --bin regen_lens` / `--lens cost` produce byte-identical output to the pre-retirement run (BEHAVIORAL equivalence, NOT byte-identity of intermediate Rust; see acceptance §"Behavioral equivalence").

**OUT:**
- Authoring of `dsl/std/runtime/**` carriers (read/write/env/process). That work is upstream substrate, owned by Substrate Mgr.
- Authoring of the BinShim Rust emitter `.dag` program (row #3 of `r3-pb-binshim-blocker-ledger.md`). Also upstream.
- Retirement of `lens_declaration_apply.rs` (separate, PB-Runtime/Row-4 gated).
- Retirement of the 8 remaining `src/v3/compiler/src/bin/**` shims (covered by `r3-pb-binshim-batch-followon-worker.md`).
- Hand-Rust bridges or interim shims of any kind. If the worker is tempted to add ANY hand-Rust to make this PR ship, the gap is the substrate prereq. STOP and ping PB Mgr.

## Substrate prerequisites — must be observably LIVE at dispatch time

Each row cites HEAD state as of 2026-05-15. The worker MUST re-verify by grep at dispatch time per `feedback_substrate_grep_before_authoring`.

| # | Prerequisite | HEAD state 2026-05-15 | Citation |
|---|---|---|---|
| 1 | PB-1 BinShim Rust emitter `.dag` program under `dsl/extdeps/**` mirroring `dsl/extdeps/languages/rust/emit.dag` discipline | **NOT LIVE** — `rg -n "BinShim" dsl/extdeps` returns no hits | `docs/briefs/r3-pb-binshim-blocker-ledger.md:35` row #3 |
| 2 | `emit_rust_bin_shim.rs` resolves `BinShim.entry` to a DeclarationRef (currently hand-authored shell text only) | **NOT LIVE** — module docs say "does not resolve `BinShim.entry`" | `docs/briefs/r3-pb-binshim-blocker-ledger.md:35` row #1; `src/v3/compiler/src/emit_rust_bin_shim.rs:1-12` |
| 3 | `dsl/std/runtime/**` substrate carriers for file read/write/env-args/process-spawn (rustfmt invocation) reachable from `regen_bootstrap` full-bootstrap glob | **LOADER GAP** — `dsl/std/runtime/` contains only `bin_shims/`; no `fs`/`env`/`process` `.dag` files; per `r3-pb-binshim-blocker-ledger.md`: *"`dsl/std/runtime/**` is **not** in the `regen_bootstrap` full-bootstrap glob"* | `dsl/std/runtime/` directory listing (HEAD); `r3-pb-binshim-blocker-ledger.md` loader-gap note |
| 4 | `compile_to_dag` callable from `.dag` (i.e., a substrate carrier `compile_to_dag(source: String, path: FilePath) -> Dag` invokable from a `.dag` program body, not just from the test-runner host) | **PARTIAL** — `compile_to_dag` is referenced in `src/v3/std/verification.dag:297` for DB-11 alias-RHS testing and in `src/v3/std/t_ci_workflow_as_data_demo.dag:169` as a payload-bind boundary, but no `.dag`-callable carrier surface exists | grep `compile_to_dag` across `dsl/`, `src/v3/std/` |
| 5 | `emit_rust_module` callable from `.dag` (substrate carrier; emits Rust source from a Dag) | **NOT LIVE** — `emit_rust_module` is the compiler-internal terminal emit pass at `src/v3/compiler/src/emit_rust.rs`; no `.dag` carrier wraps it | grep `emit_rust_module` |

**Cumulative implication:** the driver does compile + emit + format + write, four operations none of which currently have a `.dag`-callable substrate surface. Prereq #3 is the deepest gap (it's a loader-glob issue blocking ALL `dsl/std/runtime/**` consumption, not specific to this lane). Prereqs #4 and #5 are about whether the compiler can call itself from a `.dag` program — a bootstrap-loop question whose answer may legitimately be "this driver does not self-host until the compiler self-hosts more broadly," in which case this brief queues indefinitely. **That outcome is acceptable.** The brief's value is naming the dependency, not forcing a PR.

## Substrate landings — what the prereq authors must produce

These belong to Substrate Mgr / BinShim Rust emitter lane, not to this worker. Listed for awareness:

1. **`dsl/std/runtime/fs.dag`** with `read_to_string`, `write` carriers; loader-glob extension so `regen_bootstrap` picks them up.
2. **`dsl/std/runtime/env.dag`** with `args` carrier (for `--lens <name>`).
3. **`dsl/std/runtime/process.dag`** with `spawn` / `wait_with_output` carriers (for rustfmt invocation). Note: `src/v3/std/process.dag` already mirrors `ProcessExit`; this is the orthogonal "spawn other processes" carrier.
4. **BinShim Rust emitter `.dag` program** under `dsl/extdeps/languages/rust/` (or similar location consistent with `dsl/extdeps/languages/rust/emit.dag` precedent) that resolves a `data <shim>_shim: BinShim { entrypoint_name, description, entry }` record to an emitted `src/v3/compiler/src/bin/<shim>.rs` file via `emit_rust_bin_shim::format_main_shell(...)` (the existing hand-authored shell helper). Row #1 + row #3 of `r3-pb-binshim-blocker-ledger.md` together.
5. **A `regen_lens_driver_main` entry function authored in `.dag`** — either by extending `src/v3/compiler/regen.dag` with a top-level `fn regen_lens_main() -> ProcessExit = ...` that reads its own registry + dispatches to runtime carriers, or by a sibling `src/v3/compiler/regen_lens.dag`. This is the actual lift this brief tracks; the substrate landings above (#1-#4) are precondition substrate for it.

## Retirement shape — once prereqs LAND

Single PR. The worker, in order:

1. **Re-grep all 5 prereqs.** Each row in §"Substrate prerequisites" must be observably LIVE. STOP-AND-PING PB Mgr if any is missing.
2. **Confirm `regen_lens_main` is emitted from `.dag`** — `cargo build -p v3-compiler --bin regen_lens` produces a binary whose `main` function is reachable from a `// AUTO-GENERATED from <regen_lens.dag or regen.dag> ...` header file, not from the hand-Rust `regen_lens_entry.rs`. (Mechanism per substrate landing #4 + #5.)
3. **Run `cargo run -p v3-compiler --bin regen_lens`** against a clean working tree. Verify every entry in `src/v3/compiler/regen.dag` (11 entries at HEAD: `lens_cost_entry` through `lens_lower_helpers_entry`) regenerates its declared `generated_file` byte-identical to its pre-PR contents.
4. **Run `cargo run -p v3-compiler --bin regen_lens -- --lens cost`** to verify the `--lens <name>` selector works through the emitted code path.
5. **Delete hand-Rust files atomically in the same PR:**
   - `src/v3/compiler/src/regen_lens_driver.rs`
   - `src/v3/compiler/src/regen_lens_entry.rs`
6. **Update `src/v3/compiler/Cargo.toml:62-64`** — the `[[bin]] name = "regen_lens"` entry's `path` field must point at the emitted file (e.g., `src/v3/compiler/src/regen_lens_generated.rs` or wherever the emitter places it).
7. **Update `src/v3/compiler/src/lib.rs`** — remove `pub mod regen_lens_driver;` declaration (line TBD at dispatch time; grep at retirement-PR time, NOT now).
8. **Census + witness updates in same PR:**
   - Remove both paths from `EXPECTED_HAND_AUTHORED_NON_TEST` in `sg0_census_test.rs:352-353`.
   - Add the emitted file path to `GENERATED_FILES` partition (`generated_files.rs`).
   - Update `CURRENT_RESIDUAL_COUNT` in `r3_lens_producer_retirement_executable_witness_test.rs:21` from `3` to `1`.
   - Update the lens-producer subset enumeration in `r3_lens_producer_retirement_executable_witness_test.rs:8-15` docstring (remove `regen_lens_driver.rs` + `regen_lens_entry.rs` from the named-residual list).
   - Update gate #7 stays-retired message in `sg0_census_test.rs:78-82` — the language about "delegates through `src/regen_lens_entry.rs` into `regen_lens_driver.rs` until PB-1 emits the shim from `.dag`" becomes obsolete; rewrite to reflect the now-emitted path.
9. **Run the gate-test suite** locally and confirm `r3_gate_66_lens_producer_retirement_claim_executes_against_live_census` reports the new count (Fail → reason contains "lens-producer subset observed 1"; or Pass if §1.8 #66 closure threshold is `≤ 1`; check threshold at dispatch time).

## SG-0 PR-window net-shrink discipline

Per ROADMAP §"SG-0 PR-window net-shrink discipline" (course-correction 2026-05-05; tightened 2026-05-09):

- **SG-0 hand-path delta: -2** (strict net-remove; both `regen_lens_driver.rs` and `regen_lens_entry.rs` removed from `EXPECTED_HAND_AUTHORED_NON_TEST`).
- **No pairing class needed** — strict net-remove.
- **No `EXPECTED_HAND_AUTHORED_FRAGMENTS` change** — neither file is a `.txt` scaffold.

Until this brief is dispatched, the queued state itself counts as pairing-class (c) deferral evidence for any UPSTREAM net-add PR that wants to cite "BinShim/PB-1 chain has a queued pre-authored brief targeting `regen_lens_driver.rs` retirement at `docs/briefs/r3-pb-regen-lens-driver-retirement-worker.md`." That is THIS brief's secondary value: it lets the substrate-prereq lanes cite a real downstream consumer instead of paper-trailing without dispatch evidence.

## Acceptance criteria

PR closes when:

- Both files (`regen_lens_driver.rs`, `regen_lens_entry.rs`) absent from the tree.
- `cargo build -p v3-compiler --bin regen_lens` succeeds against the emitted source.
- `cargo run -p v3-compiler --bin regen_lens` regenerates all 11 entries in `regen.dag` byte-identical to pre-PR.
- `cargo run -p v3-compiler --bin regen_lens -- --lens cost` succeeds.
- `cargo test --workspace` green.
- `cargo clippy --all-targets -- -D warnings` green.
- `cargo fmt --all --check` green.
- `sg0_census_test` passes with updated `EXPECTED_HAND_AUTHORED_NON_TEST`.
- `r3_gate_66_lens_producer_retirement_claim_executes_against_live_census` reports residual count 1.
- PR body carries `SG-0 hand-path delta: -2` (machine-checkable per `scripts/check-pr-sg0-net-shrink-discipline.sh`).
- Zero new hand-Rust in the PR diff (no shims, no bridges; if substrate fell short at PR-author time the worker MUST stop, not paper over).

## STOP-AND-PING conditions

Worker MUST STOP and ping PB Mgr when:

- **Any prereq row in §"Substrate prerequisites" reads NOT LIVE at dispatch-time grep.** Includes the loader-glob gap (row #3) being unresolved even if individual `dsl/std/runtime/*.dag` files exist — the substrate must be REACHABLE from `regen_bootstrap`, not just present.
- **Byte-identity check fails** in step 3 of §"Retirement shape" — the emitted regen output differs from pre-PR generated files. Investigate; do NOT silently update the generated files to match. Behavioral equivalence is the bar, but for THIS retirement specifically (the driver IS the regen producer), byte-identity of its outputs is the cleanest acceptance test.
- **Emit-pattern divergence from `r3-pb-binshim-batch-followon-worker.md` precedent** — if the BinShim emit pattern was used for the 8-shim batch and works there but this retirement's worker finds itself authoring a parallel emit story, STOP — that's a sign substrate landing #4 isn't general enough yet.
- **Census-test wording drift** — gate #7 stays-retired message at `sg0_census_test.rs:78-82` cites specific file names; if the worker can't find a non-fudge rewrite of that message reflecting the new state, the substrate emit may not actually be replacing the delegation chain. Verify before patching the message.
- **Test that names `regen_lens_main` symbol breaks** — `regen_lens_main` is `pub` and may have external callers; grep before deleting (`rg -n "regen_lens_main" src/`) and surface any unexpected hits.

## Cross-refs

- **Census authority**: `src/v3/compiler/tests/integration/sg0_census_test.rs:78-82` (gate #7 stays-retired with PB-1 dissolution path named); `:352-353` (target files in `EXPECTED_HAND_AUTHORED_NON_TEST`).
- **Lens-producer subset gate**: `src/v3/compiler/tests/integration/r3_lens_producer_retirement_executable_witness_test.rs:8-15` (residual enumeration); `:21` (`CURRENT_RESIDUAL_COUNT`).
- **Substrate authority for what gets regenerated**: `src/v3/compiler/regen.dag` (11 `LensRegistryEntry` records at HEAD).
- **Substrate-side carriers already in place**: `src/v3/std/bin_shim.dag:19` (`type BinShim`); `src/v3/std/process.dag` (`ProcessExit` mirror); `src/v3/std/verification.dag:75-77` (`BinShimFilesSubsetPredicate`).
- **Parent brief**: `docs/briefs/r3-pb-binshim-retirement-worker.md` (substrate-landings locked shape; STOP-AND-PING discipline this brief inherits).
- **Sibling brief (batch follow-on)**: `docs/briefs/r3-pb-binshim-batch-followon-worker.md` (8-shim retirement; same emit pattern this brief's substrate landing #4 produces).
- **Consumer audit**: `docs/briefs/r3-pb-regen-lens-consumer-audit.md` (consumer-side surface enumeration; informational).
- **Blocker ledger**: `docs/briefs/r3-pb-binshim-blocker-ledger.md` (rows #1 and #3 are this brief's hard prereqs).
- **Roadmap row**: `ROADMAP.md` §"Nine lanes" T-PB-A; §1.8 #66 (lens-producer subset gate).

## P0 5-gate self-check (per `feedback_p0_applies_to_pm_authored_briefs`)

Authoring discipline receipt — applied to this brief BEFORE it was queued:

| Gate | Evidence |
|---|---|
| grep-anchor | Every NOT LIVE / LOADER GAP / PARTIAL claim in §"Substrate prerequisites" has a file path + line number or named grep producing the cited result. |
| test-consumer | Gate #66 references the executable witness test by name + line numbers; SG-0 census references `sg0_census_test.rs` by name + line numbers; gate #7 retirement traced to PR #3083 commit `6affbe75c` with quoted commit-message language. |
| CI-receipt | Cycle-4 retirement revert (PR #3048 → revert commit `ed5352b98`) is named in the §"Why this brief exists" framing so future-worker knows what NOT to copy. |
| surface-match | Scope §"IN" / "OUT" matches the actual file footprints (266 + 23 LOC); enumerated steps cite line numbers in target files. |
| audience-bs-detector | A worker reading this in 2 months CAN: grep the 5 substrate prereqs to confirm whether they've landed; cite this brief in an upstream PR-body for pairing-class (c); STOP at any of the named conditions without speculation. |

---

— Authored by `zesty-dove-792` (Release PM, on operator-direct dispatch 2026-05-15). Spike receipt: `vivid-crab-154` (operator-personal worker; pushback on 2026-05-15 operator-self-brief that mis-categorized `complexity_lattice.rs` as a lens-producer surface and mis-named the retirement mechanism; redirect-to-spike-on-driver flagged this brief as the missing artifact). Per the memory line memorialized 2026-05-15: *"P0 applies to PM-authored briefs"* — this brief's 5-gate self-check is the receipt that the discipline was applied this time.
