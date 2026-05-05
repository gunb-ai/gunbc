# R3 PB — `regen_lens.rs` consumer audit (carrier-independent, docs-only)

**Status:** AUDIT artifact (docs-only, carrier-independent). Authored 2026-05-01 by PB Manager continuation per Director follow-up on inbox #1149 — bounded post-#1347 planning slice that maps `src/v3/compiler/src/bin/regen_lens.rs`'s consumer / build / CI / call surfaces and the exact handoff points for the future `BinShim` carrier + instance + emitter + §7.2 equivalence fixture, **without** waiting on or inventing the carrier.

**Parent authorities:**
- [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4 (Item 5 emit pattern), §4.3 (dissolution path), §5.1 (sub-gate decomposition), §7.2 (BinShim equivalence fixture).
- [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md) — parent BinShim retirement program.
- [`docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md) — sub-gate 3 skeleton.
- [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md) — instance-declaration framework (PR #1347).
- [`docs/briefs/r3-pb-regen-lens-first-binshim-target-retirement-readiness.md`](r3-pb-regen-lens-first-binshim-target-retirement-readiness.md) — quick-newt's retirement-readiness checklist (parallel work; non-overlap by directive).

This audit does NOT introduce authority. Every cell cites a live file/line on origin/main HEAD.

## Scope

Carrier-independent dependency map for the future sub-gate-3 retirement worker. Names every consumer / build / CI / call / test / doc surface that participates in the cutover from the hand-Rust `regen_lens.rs` to the `.dag`-emitted equivalent. **Does NOT** propose carrier shape, emit semantics, or implementation order — those are quick-newt's territory + Substrate Manager's territory + future carrier-landing PR's territory.

## Non-overlap reminder

- **This audit:** consumer / handoff map. Docs-only.
- **quick-newt** (separate session): retirement-readiness implementation attempt path → see `r3-pb-regen-lens-first-binshim-target-retirement-readiness.md` referenced above.
- **Substrate Manager**: `BinShim` carrier-type shape (not yet on main; STOP+PING tracked in #1347).
- **PB framework PR (#1347):** `dsl/std/runtime/bin_shims/` framework directory + naming convention. Authored.

## Source surface (the file itself)

| Property | Value (verified on origin/main HEAD) |
|---|---|
| Path | `src/v3/compiler/src/bin/regen_lens.rs` |
| Size | 9405 bytes (~9 KB) |
| Header docstring | "Unified lens-regen driver. Narrow host shim for `src/v3/compiler/regen.dag`: reads every `data <name>_entry: LensRegistryEntry` record out of the bootstrap Dag, compiles the referenced `.dag` lens, and writes the `emit_rust_module` projection to the declared output path." (line 1-6) |
| Entry-point shape | `fn main() -> ExitCode` (line 33) — already returns a `std::process::ExitCode`, structurally close to the design-doc §4.2 `entry: () -> std.process.ProcessExit` contract. Re-declaring it as a `BinShim.entry` does NOT require evolving the substrate `ProcessExit` shape. |
| `LENS_REGISTRY_ENTRY_TYPE` | `&str = "LensRegistryEntry"` (line 26) — string-keyed lookup against `regen.dag`'s `data <name>_entry: LensRegistryEntry` records. |
| Authority source | `src/v3/compiler/regen.dag` — the bin reads 11 `LensRegistryEntry` records here (verified `grep -c "_entry: LensRegistryEntry" src/v3/compiler/regen.dag`). |

## Internal crate imports (the bin's PB-Runtime-equivalent surfaces)

These imports are what the future `BinShim` emitter must end up calling from emitted Rust. The shape of each call site is what the equivalence fixture (§7.2) will compare on. Verified on `src/v3/compiler/src/bin/regen_lens.rs:14-23`:

| Import | Role in the shim | What the future emitter must reproduce |
|---|---|---|
| `v3_compiler::dag::{Dag, FieldValue, LiteralBits, ValueBody}` | Walk the bootstrap Dag to enumerate `LensRegistryEntry` records | Same Dag-walk surface. The emitted shim invokes the same dag-traversal API. |
| `v3_compiler::emit_rust::emit_rust_module` | Project the compiled lens into a Rust module | Same emit-rust call. The §7.2 fixture compares behavioral output of this call between hand-Rust and emitted forms. |
| `v3_compiler::generated_files::GENERATED_FILES` | Cross-check that `generated_file` paths in `regen.dag` match the producer-owned manifest in `build.rs` | Emitted shim reads the same constant — `GENERATED_FILES` is a build-time manifest, not a lens-side surface, and is unchanged by retirement. |
| `v3_compiler::{compile_to_dag, CompileError}` | Compile each registered lens .dag source on the fly | Same compile-to-dag call. The emitter doesn't re-implement compilation; it calls the existing surface. |
| `std::collections::HashMap`, `env`, `io::{self, Write}`, `path::{Path, PathBuf}`, `process::{Command, ExitCode, Stdio}` | Standard host-shim plumbing (CLI args, stdout/stderr, file paths, exit codes) | The emitted shim still needs an analogous host-shim layer per design-doc §4.2: "the host (cli_run.rs / emitted bin-shim) checks `type_name=="ProcessExit"` AND `variant_name=="ExitFailure"` to set exit code." |

**Implication for the equivalence fixture:** the §7.2 acceptance compares behavioral equivalence (per design-doc §7.2: "**not** byte-identical"). The five import groups above are the structural surface the emitted form invokes; the fixture's "produces same output for same input" assertion runs over (i) the per-lens regeneration step and (ii) the `--lens <name>` CLI selector path.

## Build / packaging surfaces

| Surface | Live location (verified) | Cutover delta |
|---|---|---|
| **Cargo bin entry** | `src/v3/compiler/Cargo.toml`: `[[bin]] name = "regen_lens" path = "src/bin/regen_lens.rs"` | `path` updates to the emitted location once the file is generated. The `name = "regen_lens"` and CLI invocation surface stay stable so the 8-bin Cargo manifest's authority surface is unchanged. |
| **`build.rs` `REGEN_OUTPUTS`** | `src/v3/compiler/build.rs:479-516` (25 entries on origin/main HEAD at audit time) | Add the emitted shim's path to `REGEN_OUTPUTS` so SG-0's `sg0_generated_partition_is_producer_owned` invariant counts the file as generated. The current `REGEN_OUTPUTS` lists every per-lens `lens_*_generated.rs` (the bin's *outputs*) but NOT the bin itself — `regen_lens.rs` is the producer, not a generated artifact. After retirement, the bin moves from "hand-authored producer" to "generated artifact" and the manifest grows by 1. |
| **`GENERATED_FILES` runtime constant** | Built from `REGEN_OUTPUTS` per `build.rs:520-526`, exposed as `v3_compiler::generated_files::GENERATED_FILES`. | Auto-updates when `REGEN_OUTPUTS` grows; no separate edit. The bin imports this exact constant (line 22), so its own re-emission produces a consistent self-reference. |
| **Package `v3-compiler` declares eight `[[bin]]` targets**: `regen_bootstrap`, `regen_lens`, `regen_parse`, `regen_parse_tables`, `regen_tokenize`, `regen_v3`, `self_host_fixed_point`, `r1c_e_emit_gates` | `src/v3/compiler/Cargo.toml` `[[bin]]` blocks | Other bins follow the same template after `regen_lens` lands as the canonical first slice (per design-doc §4.3 + BinShim brief §"First slice"). They are NOT in scope for sub-gate 3 — but they ARE in scope for the broader BinShim retirement program once the carrier and pattern stabilize. |

## SG-0 census surface

| Authority | Live location (verified) | Cutover delta |
|---|---|---|
| `EXPECTED_HAND_AUTHORED_NON_TEST` | `src/v3/compiler/tests/integration/sg0_census_test.rs:174` lists `"src/v3/compiler/src/bin/regen_lens.rs"` | Remove the entry. SG-0 census decreases by 1; `REGEN_OUTPUTS` grows by 1 (above). Both updates land atomically in the same retirement PR. |
| Census comment trail | `sg0_census_test.rs:50-67` documents the 2026 SG-6 cutover that folded 4 per-lens regen bins + `regen_infer_helpers` into the unified `regen_lens` shim. | Future entry: comment annotation that `regen_lens.rs` retired via Item-5 BinShim emit pattern + `data regen_lens_shim` instance under `dsl/std/runtime/bin_shims/regen_lens.dag`. |

## Test surface (consumers that exercise the bin or its registry)

| Test | Live location | What it asserts | Cutover impact |
|---|---|---|---|
| `lens_register_correspondence_test::every_regen_lens_entry_has_a_capability_register_row` | `src/v3/compiler/tests/integration/lens_register_correspondence_test.rs:163-164` | Every `LensRegistryEntry` in `regen.dag` has a corresponding row in `docs/v3-lens-capability-register.md` | Unaffected by retirement — reads `regen.dag` directly, not the bin. The dispatch chain `regen.dag → emitter` survives the bin's retirement. |
| `lens_register_correspondence_test::regen_lens_file_basenames` (helper) | Same file, line 66 | Helper iterating `regen.dag` filenames | Same — helper is registry-side, not bin-side. |
| `sg6_hand_authored_census_test::sg6_regen_lens_cli_smoke_regenerates_named_entry_without_drift` | `src/v3/compiler/tests/integration/sg6_hand_authored_census_test.rs` (cited in `scripts/slow-test-exemptions.txt:78`) | End-to-end CLI smoke: shells out to `regen_lens` and asserts no drift in regenerated entry | **Direct cutover dependency.** Test invokes the bin's CLI; once retired, this test exercises the emitted shim. The §7.2 equivalence fixture's "behavioral equivalence" claim is what makes this test still pass post-retirement. |
| Stale-snapshot error messages naming `cargo run -p v3-compiler --bin regen_lens -- --lens <name>` | `lane2_stage_2d_symbolic_cost_test.rs:656`, `sg7_prep_variant_payload_freshness_test.rs:73` | Diagnostic strings telling devs to regenerate when generated module is stale | The bin's CLI invocation string remains valid post-retirement (binary name preserved); diagnostic text needs no update unless the worker chooses to add an `(emitted)` annotation. |
| `t_impossiblebugs_unenumerated_effects_test.rs:5` | Same path | Comment: "migration test once `regen_lens` can run in this environment" | Comment-only reference; informational. |
| `sg6_hand_authored_census_test.rs:6,9,147,154` | Same path | Multiple comment references to "the unified `regen_lens` driver" / "the lens-registry shape that `regen_lens` enumerates" | Comment-only references; the test itself focuses on the SG-6 census shape, not the bin. Update comments at retirement to read "emitted regen_lens shim" if desired. |

## CI / call surface

| Surface | Live location (verified) | Cutover impact |
|---|---|---|
| GitHub Actions workflows | `.github/workflows/` (no direct `regen_lens` reference; `grep` returned 0 hits in `.github/workflows/`) | None. CI does not invoke the bin directly; consumption is via the SG-6 CLI smoke test (above) which runs as part of `cargo test`. |
| `scripts/slow-test-exemptions.txt:78` | exemption row for `sg6_regen_lens_cli_smoke_regenerates_named_entry_without_drift` | Exemption persists post-retirement (test still shells out). Optional follow-up: paydown lane updates the exemption note to mention the emitted form. |
| Doc invocation strings | `docs/history/roadmap-active-deferrals.md:153`, `docs/db-history/db-3.md:9`, `docs/history/lens-wire-in-audit-v2-comparison.md:76` | Stable: all three say `cargo run -p v3-compiler --bin regen_lens` which keeps working post-retirement (binary name preserved). |

## Documentation surface (authority chain)

| Authority | Live location | Cutover impact |
|---|---|---|
| `docs/design-pure-bootstrap-zero-audit.md:265` row | `src/v3/compiler/src/bin/regen_lens.rs` retiring under "PB-Runtime" | Row's resolution stays valid; once retired, mark as RESOLVED. |
| `docs/r3-structure.md:26, :52, :107` | T-LensProducer-Retirement sub-gate 3 acceptance | Sub-gate 3 row's `regen_lens_dot_rs_retired` ledger predicate goes green; status row marks closed. |
| `docs/design-reflection-completeness.md:139` | "`regen_lens.rs` — bin-shim retirement, gated separately on PB-Runtime + bin-shim spec (Items 4+5)" | No edit needed; the gating language stays accurate. |
| `docs/audit/t-v2-retirement-audit.md:21` | S-3 entry (`regen_lens.rs` retirement) — "NOT MET" | Flips to "MET" when retirement lands. |
| `docs/briefs/t-pb-a-lens-producer-priority-slice-first-retirement-worker.md:73, :103` | Earlier T-PB-A scoping ("Tier-1 / regen: regen_lens.rs — depends on PB-1 + emit") | Already names the dependency chain; informational. |

## Handoff points for the future `BinShim` carrier + instance + emitter + §7.2 fixture

This is the load-bearing audit deliverable: a per-handoff row naming exactly what each future PR consumes from this audit.

| Handoff | Owner | Consumes from this audit | What that PR adds |
|---|---|---|---|
| **`type BinShim` carrier landing** | Substrate Manager (per design-doc §5.4 + #1347 STOP+PING) | Field requirements named in design-doc §4.2's example `data regen_lens_shim: BinShim = { name: "regen_lens", entry: regen_lens, ... }`: a `name` field whose value is the **Cargo `[[bin]]` target name** (e.g. `"regen_lens"` per `src/v3/compiler/Cargo.toml`'s `[[bin]] name = "regen_lens"` block), and `entry: () -> std.process.ProcessExit` whose structural model is the bin's existing `fn main() -> ExitCode` (line 33). Whatever pipeline-composition fields the §4.2 sketch settles on are Substrate Manager's call. **Not derivable from `LENS_REGISTRY_ENTRY_TYPE` at line 26** — that constant is the registry-entry meta-tag string the bin uses to look up `LensRegistryEntry` records inside `regen.dag`, not a property of the bin-shim itself. | `type BinShim { ... }` declaration in `dsl/std/runtime/bin_shim.dag` (or wherever Substrate picks). |
| **`data regen_lens_shim: BinShim` instance authoring** | PB Manager (R3 worker, dispatched by PB on carrier landing) | Field values from the audit's "Source surface" + "Internal crate imports" tables. The `entry` field references a `regen_lens_main` declaration that fold-mirrors the current `fn main` body via the locked PB-Runtime emit pattern. | New file `dsl/std/runtime/bin_shims/regen_lens.dag` per the framework convention in PR #1347. |
| **BinShim emit pattern (the `.dag` emitter)** | PB Manager (R3 worker) | The bin's import surface (the 5 internal crates listed above) — the emitter must produce emitted Rust that calls these same crate APIs, since they are NOT being retired (they're producer-side authority). | New `.dag` emitter program analogous to `dsl/extdeps/languages/rust/emit.dag` per design-doc §4.2 + anti-bridge invariant #4. |
| **§7.2 equivalence fixture authoring** | PB Manager (R3 worker) — same retirement PR | The behavioral spec from the bin's docstring + `sg6_regen_lens_cli_smoke_regenerates_named_entry_without_drift` test (which IS the runtime evidence base). The fixture asserts: emitted-Rust + hand-Rust produce identical regenerated `lens_*_generated.rs` output for every `LensRegistryEntry` in `regen.dag`. | TestClaim `regen_lens_bin_shim_emits_behaviorally_equivalent_to_hand_rust` per design-doc §7.2 locked name. |
| **Cargo `path` flip** | PB Manager (same retirement PR) | The Cargo `[[bin]]` block at `Cargo.toml` (above). | `path = "src/bin/regen_lens.rs"` updates to whatever absolute path the emitted file lands at (or stays the same if the emitter writes to the same path). |
| **`REGEN_OUTPUTS` extension** | PB Manager (same retirement PR) | The list at `build.rs:479-516` (25 entries on origin/main HEAD at audit time). | Add `"src/v3/compiler/src/bin/regen_lens.rs"` (the now-emitted path). |
| **SG-0 census drop** | PB Manager (same retirement PR) | `sg0_census_test.rs:174`. | Remove the entry. |
| **Doc status flips** | PB Manager (same retirement PR) | The 4 doc references in §"Documentation surface" above. | Mark RESOLVED/MET/closed as appropriate. |

## STOP / report-instead-of-invent

Per dispatch directive — STOP if:

- **Carrier-shape pressure surfaces during the audit.** Did not surface; the bin's existing `fn main() -> ExitCode` already structurally matches the design-doc §4.2 sketch `entry: () -> std.process.ProcessExit`. No carrier-field invention.
- **Emit-semantic invention surfaces.** Did not surface; the bin's import surface (5 internal-crate imports) is the literal call surface the emitter must reproduce — no semantic that's not already on main.
- **Existing docs already cover this.** `r3-pb-binshim-retirement-worker.md` §"First slice" sketches the dissolution path verbally but does NOT enumerate consumers. `r3-pb-regen-lens-first-binshim-target-retirement-readiness.md` (quick-newt's parallel work) is implementation-readiness focused; this audit is consumer/handoff focused. No duplication.

Audit conclusions: **no missing authority, no contradictions, no carrier/schema invention.** Every cell traces to a live file/line on origin/main HEAD.

## Delta — 2026-05-05 refresh vs current `origin/main`

Refresh command surface:

- `rg -n "regen_lens" .github/workflows scripts docs src/v3/compiler/tests/integration src/v3/compiler/Cargo.toml src/v3/compiler/build.rs dsl/std/runtime/bin_shims/README.md src/v3/std/bin_shim.dag src/v3/compiler/src/emit_rust_bin_shim.rs src/v3/compiler/src/process_exit.rs`
- `rg -n "BinShim|ProcessExit|REGEN_OUTPUTS|GENERATED_FILES|EXPECTED_HAND_AUTHORED_NON_TEST" src/v3/compiler src/v3/std dsl/std/runtime docs/briefs/r3-pb-regen-lens-consumer-audit.md scripts .github/workflows docs/design-pb-runtime-interpreter.md docs/r3-structure.md docs/audit/t-v2-retirement-audit.md docs/design-reflection-completeness.md`
- `rg -n "^data .*_entry: LensRegistryEntry" src/v3/compiler/regen.dag`

Observed deltas:

| Surface | Current anchor(s) | Delta vs original audit |
|---|---|---|
| **`BinShim` carrier** | `src/v3/std/bin_shim.dag:19-23`; `dsl/std/runtime/bin_shims/README.md:7`, `:19-27`, `:35` | The carrier is now live, not "not yet on main". Its live fields are `entrypoint_name: NonEmptyStr`, `description: String`, `entry: DeclarationRef`; the older audit handoff text that spoke generically about `name` is superseded by the README's `entrypoint_name` convention. This is a carrier-state refresh only; this audit still does not invent or edit carrier shape. |
| **Per-shim instance blocker** | `dsl/std/runtime/bin_shims/README.md:38-45`; `docs/briefs/r3-pb-binshim-emitter-readiness.md:40-54`; `docs/briefs/r3-pb-binshim-row-1-decision-packet.md:7`, `:12`, `:24-29`, `:49` | The live blocker moved from carrier landing to the missing `.dag` entry function: no `regen_lens_main` / `*_main -> ProcessExit` target exists for `entry: DeclarationRef`. The row-#1 decision packet is a new planning consumer for this audit's map; it explicitly does not author `regen_lens_main`, `data regen_lens_shim`, emitter wiring, §7.2, `REGEN_OUTPUTS`, or retirement. |
| **Host shell helper consumer** | `src/v3/compiler/src/emit_rust_bin_shim.rs:1-12`, `:20-63`, `:70-85`; `src/v3/compiler/tests/integration/sg0_census_test.rs:259`, `:1073-1076` | New hand-authored Rust consumer exists for PB-1 bin-shim shell text. It formats the generated `main.rs` wrapper around an entry function returning `ProcessExit`; it does not resolve `BinShim.entry`, wire Cargo, edit `build.rs`, or retire `regen_lens.rs`. It is SG-0 hand-authored today and therefore a consumer/handoff surface for future emitted shims. |
| **Host `ProcessExit` mirror** | `src/v3/compiler/src/process_exit.rs:1-18`; `src/v3/compiler/tests/integration/sg0_census_test.rs:281-282` | New hand-authored host mirror exists for generated bin-shim shells to match on `ExitSuccess` / `ExitFailure`. This mirrors `dsl/std/process.dag` for emitted shell plumbing; it does not alter `regen_lens.rs`'s current `fn main() -> ExitCode` surface. |
| **Cargo bin census** | `src/v3/compiler/Cargo.toml:47-85` | `v3-compiler` now declares **nine** explicit `[[bin]]` targets, not eight: `emit_method_template_projection` was added at `Cargo.toml:72-74`. `regen_lens` remains stable at `Cargo.toml:52-54` (`name = "regen_lens"`, `path = "src/bin/regen_lens.rs"`). |
| **`REGEN_OUTPUTS` / `GENERATED_FILES` anchors** | `src/v3/compiler/build.rs:479-513`, `:514-526` | Line anchors moved, but semantics are unchanged: `REGEN_OUTPUTS` remains the producer-owned generated manifest, and `GENERATED_FILES` is still emitted from it. Current list has 34 entries by literal row count in `build.rs:480-512`; it still does **not** include `src/v3/compiler/src/bin/regen_lens.rs`. Retirement still adds the emitted shim path here atomically with the SG-0 census drop. |
| **SG-0 census anchor** | `src/v3/compiler/tests/integration/sg0_census_test.rs:211-242`; `:237`; `:259`; `:281-282` | The line anchor for `EXPECTED_HAND_AUTHORED_NON_TEST` moved from the older `:174` citation to `:211`, and `regen_lens.rs` is now at `:237`. New adjacent hand-authored bin-shim support files (`emit_rust_bin_shim.rs`, `process_exit.rs`) are also listed and should not be confused with generated outputs. |
| **Registry row count** | `src/v3/compiler/regen.dag:44`, `:50`, `:56`, `:62`, `:68`, `:74`, `:90`, `:96`, `:106` | The live registry has **nine** `data ..._entry: LensRegistryEntry` rows when counted with `rg -n "^data .*_entry: LensRegistryEntry"`. A broad text search/count for `_entry: LensRegistryEntry` returns 11 because `regen.dag:5` and `:13` are comments describing the pattern; future refreshes should use the anchored `^data` query for data-row count. |
| **Cementing registry consumer** | `src/v3/compiler/tests/integration/cementing/cementing_lens_registry_dispatch_test.rs:340-395` | Newer cementing tests consume the same `LensRegistryEntry` authority to derive `regen_lens --lens <name>` keys required by the capability register. This is a registry-side consumer, not a direct bin implementation consumer, but it is part of the call/test map because the CLI selector namespace must remain stable. |
| **CI wiring** | `.github/workflows/ci.yml:129-130`, `:195-204`, `:215-268`, `:274-310`; `scripts/slow-test-exemptions.txt:78` | Still no direct `regen_lens` invocation in `.github/workflows/`; the path remains indirect through `cargo test -p v3-compiler` in the v3 job and the SG-6 smoke test exemption. CI now also runs lens-surface gates in the same v3 job (`L-7`, `L-8`), but those scan lens files / generated lens wrappers, not `regen_lens.rs` directly. |
| **Doc / planning consumers** | `dsl/std/runtime/bin_shims/README.md:49-61`; `docs/briefs/r3-pb-binshim-emitter-readiness.md:23-24`, `:32-37`, `:40-54`; `docs/briefs/r3-pb-binshim-row-1-decision-packet.md:24-29`; `docs/briefs/r3-pb-regen-lens-first-binshim-target-retirement-readiness.md:32-37`, `:80-83` | The surrounding planning docs now split the chain more finely: row #1 `regen_lens_main` entry function, row #2 `data regen_lens_shim`, row #3 emitter/shell, row #4 §7.2 equivalence, then `REGEN_OUTPUTS` + SG-0 retirement. This audit remains the consumer/build/CI/call map only. |

No carrier/emitter implementation is implied by these deltas. The current cutover blocker is not "find the `BinShim` carrier"; it is "provide a non-fabricated `regen_lens_main` entry target and loader story, then author the instance/emitter/§7.2/SG-0 changes in their assigned slices."

## Cross-refs

- Parent design lock: [`docs/design-pb-runtime-interpreter.md`](../design-pb-runtime-interpreter.md) §4 (Item 5 emit pattern), §4.3 (dissolution path), §5.1 (sub-gate decomposition), §7.2 (BinShim equivalence fixture).
- Parent BinShim retirement program: [`docs/briefs/r3-pb-binshim-retirement-worker.md`](r3-pb-binshim-retirement-worker.md).
- Sub-gate 3 skeleton: [`docs/briefs/r3-pb-t-lensproducer-sub3-regen-lens-retirement.md`](r3-pb-t-lensproducer-sub3-regen-lens-retirement.md).
- Instance-declaration framework (PR #1347): [`dsl/std/runtime/bin_shims/README.md`](../../dsl/std/runtime/bin_shims/README.md).
- Quick-newt's parallel readiness checklist: [`docs/briefs/r3-pb-regen-lens-first-binshim-target-retirement-readiness.md`](r3-pb-regen-lens-first-binshim-target-retirement-readiness.md).
- PB Manager brief: [`docs/briefs/r2-pure-bootstrap-manager.md`](r2-pure-bootstrap-manager.md) line 37 (BinShim instances + emit pattern + retirement dispatch).
- Substrate-fact-introduction procedure: [`INVARIANTS.md`](../../INVARIANTS.md) §P1.
- Live source paths cited (anchor for future workers): `src/v3/compiler/src/bin/regen_lens.rs`, `src/v3/compiler/regen.dag`, `src/v3/compiler/Cargo.toml`, `src/v3/compiler/build.rs:479-516`, `src/v3/compiler/tests/integration/sg0_census_test.rs:174`, `src/v3/compiler/tests/integration/lens_register_correspondence_test.rs`, `src/v3/compiler/tests/integration/sg6_hand_authored_census_test.rs`, `scripts/slow-test-exemptions.txt:78`, `dsl/std/process.dag:39` (`type ProcessExit`).
