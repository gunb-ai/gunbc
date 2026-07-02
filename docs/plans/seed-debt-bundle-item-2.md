# Seed-only debt bundle item 2 — CLI dep-pool authority model (#5894)

**Status:** design-first (2026-06-28). Parent `stern-moth-225` phase-0 metrics bundle, **item 2**. **#5894 scope:** lands `gunbc.compile_source_model` + pure `.dag` floor witness only — no emitter flip, no exemption retirement, no `regen --verify` flip. Parse-cursor grounding deferred to emit-migration (jolly-cat / #5873 sequencing). Work-item `adhoc-8521769b-288`.

## 0. Verdict — HAND_MAINTAINED exemptions vs authority models

Both `main.rs` and `v1_compiler_parse.rs` are **`HAND_MAINTAINED_STAGE0_FILES`** on `origin/main` (`regen_stage0.rs:141`, `:148`) with explicit dissolution comments. Full dissolution = model + emit-wiring + move-to-`GENERATED` + `regen --verify` byte-identical (operator superset-diff playbook). **#5894 grounds fork B only; it does not retire either exemption.**

| seed file | committed behavior | #5894 | exemption retirement (follow-up) |
| --- | --- | --- | --- |
| `main.rs` | `DependencyPoolIndex`, `--dependency-pool-index`, `pool_fill_only` indexing | `gunbc.compile_source_model` + `compile_source_model_witnesses` (pure floor witness) | wire `05_emit_rust.dag` main-emit → move to `GENERATED_STAGE0_FILES` → `regen --verify` |
| `v1_compiler_parse.rs` | O(N) `TokenStream` cursor (#5864) | **deferred** — no `02_parse` edits, no floor sidecar | ground cursor in `02_parse.dag` + emit → move to `GENERATED` → `regen --verify` |

## 1. Fork A — parse cursor (deferred; not #5894)

**Symptom:** committed `v1_compiler_parse.rs` is cursor-based; `02_parse.dag` still models `List<Token>` + `skip(1)`. **Why not #5894:** `src/v1` is not a floor `witness_layer_root`. A bash sidecar would grow `realization_vocab_exception_roster` — avoided by deferring the whole fork.

**Honest dissolution trigger:** when `parse.rs` moves exempt→GENERATED and is emitted from `02_parse.dag`, `regen_stage0 --verify` (`RegenVerifyGate`, #5873) proves the cursor byte-identical by execution.

## 2. Fork B — CLI dep-pool model (`gunbc.compile_source_model`) — #5894

**Symptom:** `gunbc compile --source-root A --source-root B` semantics are load-bearing for tree-scoped builtin registry partition. The committed CLI implements two policies; the emitter authority does not.

**Construction direction (landed #5894):** `dag/gunbc/compile_source_model.dag` — `DependencyPoolIndex`, `SourceRootRole`, and pure policy fns (`source_root_role`, `pool_fill_only_for_role`, `skip_pool_module_when_indexed`, `duplicate_module_path_across_index_is_error`, `duplicate_module_path_within_root_scan_is_error`, `compile_entry_source_root_index`).

**Floor witness:** `dag/test/claim/compile_source_model_witness_test.dag` — executes in the floor with **no transport** (`dag/` is a witness-layer-root). RED if policy fns regress.

**Follow-on (emitter PR):** see §3 emit-wiring shape → regen cutover PR.

## 3. `main.rs` emit-wiring shape (for jolly-cat review — NEXT PR, not #5894)

**Goal:** `05_emit_rust.dag` `emit_main_rs` reproduces committed hand `main.rs:98-188` dep-pool semantics from `gunbc.compile_source_model` — then `main.rs` moves `HAND_MAINTAINED` → `GENERATED` and `regen --verify` is byte-identical.

1. **CLI field:** extend `Commands::Compile` clap struct with `#[arg(long, default_value = "primary-precedence")] dependency_pool_index: String`.
2. **Parse helper:** emit `parse_dependency_pool_index` matching `dependency_pool_index_from_flag` (strict | primary-precedence | exit 1).
3. **Index helpers:** replace strict-only `emit_build_module_index_fn` with `build_module_index(source_roots, pool_index)` + `index_source_root(..., pool_fill_only)` + `insert_module_path` — inline emit matching hand `main.rs`, model is authority checklist.
4. **Compile arm:** `emit_compile_match_arm` passes parsed pool index into `build_module_index`; entry-root scan unchanged.
5. **Registry flip:** `main.rs` from `HAND_MAINTAINED_STAGE0_FILES` → `GENERATED_STAGE0_FILES`.
6. **Oracle:** `regen_stage0 --verify` byte-identical vs hand `main.rs`; `gunbc compile --source-root src/v1 --source-root dag --dependency-pool-index primary-precedence` green.

**Sequencing:** after cool-hawk-908 cargo-green `05_emit_rust` merges; no concurrent seed PRs in `05_emit_rust`/`02_parse`. jolly-cat slots merge order. **parse.rs emit authors LAST** (operator sign-off on intent; shape review before authoring; strong oracle: regen --verify vs #5864 O(N) seed + corpus green + cursor witness RED on broken advance).

## 4. RegenVerifyGate coordination

While both files remain HAND_MAINTAINED, faithful regen diverges from committed seed. **Drift 5 (main)** unblocks on emitter follow-up. **Drift 4 (parse)** unblocks on deferred emit-migration — not via roster-growing shell witnesses.

**Explicit non-goals (#5894):** no `regen --verify` flip; no `02_parse` edit; no `05_emit_rust` emitter edit; no stage0 regen commit; no `realization_vocab_exception_roster` growth.

## 5. Discriminating witness (cutover PR)

- **Dep-pool:** `gunbc compile --source-root src/v1 --source-root dag --dependency-pool-index primary-precedence` succeeds with entry=`src/v1` modules after emit-wiring.

## Dissolution trigger (DESIGN §6)

`main.rs` exemption retired by construction: `05_emit_rust` emits from `gunbc.compile_source_model`, file moves to `GENERATED_STAGE0_FILES`, and `regen --verify` is byte-identical. Parse cursor / drift 4 is a separate emit-migration track.
