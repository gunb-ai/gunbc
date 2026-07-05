# Witness subject-execution audit (spec-without-execution census)

> **Read-only audit** (2026-07-05, session tidy-deer-688). DESIGN §5: *a typecheck and a `.contains()` grep are not consumers* — done means a real consumer **green by execution**. This report measures how widespread the inverse is among floor-discoverable witnesses: tests whose **name claims** equivalence with a `.dag` authority but whose **body never interpreter-evaluates** that authority.

**Trigger receipt (PR 6231):** `import_closure_equivalence_tests` in `cli_run.rs` are named equivalence tests but call Rust `import_closure_live_paths` — never `module_graph.dag:import_closure_live` through the interpreter. That green masked **3 real `map_get`/`Outcome` bugs** in `v2.lens.module_graph` (matching `Present`/`Absent` directly against `Outcome<Optional<V>>` instead of unwrapping `Accepted`/`Rejected` first). The bugs surfaced only when PR 6274 ran the `.dag` source for the first time (deep-koi-309).

---

## Scope and method

**Corpus:** all `test fn … -> Bool` witnesses discoverable under floor witness roots:

- `dag/test/claim/**`
- `src/v2/test/claim/**` (including `manual/`, `execution/`)
- `src/v2/lens/**`
- `src/v2/test/manual/**`
- `src/v2/workflow/**`

**Total:** 1,789 witness test functions across 553 `*_test.dag` files.

**Classification** (each test fn, with transitive helper analysis):

| Class | Meaning |
|-------|---------|
| **(a)** | Executes its `.dag` subject through the interpreter (`claim_batch` / `gunbc run` path on the entry closure). |
| **(b)** | Executes only a Rust host-realization or shells to a host binary. `.dag` entry is a thin wrapper or absent. |
| **(b-danger)** | Class (b) where a **named `.dag` authority twin exists** and can **silently diverge** from the Rust path (the PR 6231 failure mode). |
| **(c)** | Asserts on source text / generated artifact substrings (`filesystem_read` + `string_contains`, or `string_contains` on emitted scripts) without executing the claimed subject. |

Additionally: **18** Rust `#[test]` equivalence modules outside floor discovery (`import_closure_equivalence_tests`, `wet_hermetic_equivalence_test`, etc.).

---

## Summary counts

| Class | Test fns | Share |
|-------|----------|-------|
| **(a)** interpreter | 1,652 | 92.3% |
| **(b)** host/shell transport | 11 | 0.6% |
| **(b-danger)** host reimpl with `.dag` twin | 27 | 1.5% |
| **(c)** grep-shaped | 102 | 5.7% |
| **Total** | **1,789** | |

Plus **18** Rust-only equivalence `#[test]` fns (not floor-discovered; all class (b), 12 are (b-danger)).

**Illusion rate (b + b-danger + c):** 140 / 1,789 = **7.8%** of floor witnesses never interpreter-evaluate their claimed `.dag` subject. Of those, **(b-danger)** is the silent-divergence subset: **27** floor fns + **12** Rust equivalence fns = **39** high-risk sites.

---

## Ranked dangerous subset (class b-danger)

Ordered by divergence risk. **Claims** = what the name/provenance implies; **Exercises** = what actually runs.

### Tier 1 — PR 6231 class (import closure)

| Location | Witness | Claims | Actually exercises |
|----------|---------|--------|-------------------|
| `src/v1/stage0/src/cli_run.rs:11894–12186` | 12× `import_closure_*` `#[test]` | `import_closure_live` equivalence to legacy BFS | Rust `import_closure_live_paths_with_facts` + `resolve_transitively_bfs_legacy`; **never** `v2.lens.module_graph:import_closure_live` |
| `src/v2/test/claim/intent_linearity/lens_unit/import_closure_completeness_test.dag:35–44` | 3× completeness fold tests | declared consumed-input closure = derived `import_closure_live` | Rust `import_closure_is_clean_live` only; `.dag` `import_closure_is_clean` not evaluated |
| `src/v2/test/claim/intent_linearity/lens_unit/import_graph_live_test.dag:42–58` | 3× live lens row tests | declared vs derived import closure | Rust `import_closure_is_clean_live` only |

**Counterexample (class a, correct pattern):** `src/v2/test/claim/module_graph/import_closure_live_test.dag:55–60` calls `.dag` `import_closure_live(...)` through the interpreter and compares to declared conformance closure. This is the witness that would have caught the `map_get`/`Outcome` bugs.

### Tier 2 — Lens scanner walls (`*_live` bypasses `.dag` lens)

| Location | Witness | `.dag` authority bypassed |
|----------|---------|---------------------------|
| `src/v2/test/claim/realization_vocabulary_containment/clean_tree_test.dag:10` | `realization_vocab_clean_tree_holds` | `v2.lens.realization_vocabulary_containment` |
| `src/v2/test/claim/realization_vocabulary_containment/scanner/planted_leak_test.dag:15` | `realization_vocab_planted_leak_detected_holds` | same (+ `realization_vocab_leak_count_live`) |
| `src/v2/test/claim/realization_vocabulary_containment/roster_soundness_test.dag:30` | `realization_vocab_roster_soundness_holds` | same (+ roster `*_live`) |
| `src/v2/test/claim/medium_structure_containment/clean_tree_test.dag:11` | `medium_structure_clean_tree_holds` | `v2.lens.medium_structure_containment` |
| `src/v2/test/claim/medium_structure_containment/scanner/*.dag` | 2× planted/perturb tests | same |
| `src/v2/test/claim/layering_imports/clean_tree_test.dag:11` | `clean_tree_no_wrong_direction_imports_holds` | `v2.lens.layering_imports` (`layering_imports_clean_holds` vs `layering_imports_clean_holds` host) |

### Tier 3 — Repo-wide / syntactic host audits

| Location | Witness | `.dag` authority bypassed |
|----------|---------|---------------------------|
| `src/v2/test/claim/complexity_linearity/syntactic_audit_witness_test.dag:13–41` | 7× syntactic audit fns | `v2.lens.complexity_linearity` (entire lens is Rust `complexity_linearity_*_live`) |
| `src/v2/test/claim/enforcement_live_witness_test.dag:34–38` | 2× gate fns | `v2.lens.enforcement`, `v2.lens.complexity` |
| `src/v2/lens/wiring_liveness_corpus_test.dag:153` | `wiring_liveness_corpus_witnesses` | `v2.lens.wiring_liveness` (`fn_arrow_decl_facts_live`) |
| `src/v2/test/claim/host_language_transport_script/corpus/facts_readback_verdict_perturb_test.dag:38` | perturb witness | `v2.lens.host_language_transport_script` |
| `src/v2/test/claim/manual/test_migration_debt_test.dag:28` | `test_migration_delete_guard_holds` | `v2.lens.test_migration_debt` |
| `dag/test/claim/ci_budget_tree_witness_test.dag:142` | `ci_budget_tree_holds` | three-way conservation (host `witness_three_way_conservation_live`) |

---

## Class (b) — intentional host physics (lower divergence risk)

Shell-transport witnesses: `.dag` entry resolves but body is `run_*_witness()` → host binary. Documented as interim until substrate migration.

| File | Test fn | Transport |
|------|---------|-----------|
| `src/v2/test/claim/infer_semantics_witness_test.dag:8` | `infer_semantics_witness_keystone_holds` | `infer_semantics_witness` host binary |
| `src/v2/test/claim/bootstrap_test.dag:8` | `bootstrap_witness_keystone_holds` | `bootstrap_witness` host binary |
| `src/v2/test/claim/auth_declared_but_unwired_witness_test.dag:8` | `auth_declared_but_unwired_witness_keystone_holds` | auth host binary |
| `dag/test/claim/interp_recorded_fixture_witness_test.dag:8` | `interp_recorded_fixture_keystone_holds` | recorded-fixture host |
| `dag/test/claim/floor_skip_discovery_witness_test.dag:8` | `floor_skip_discovery_keystone_holds` | floor-skip host |
| `dag/test/claim/effects_rest_transport_parse_witness_test.dag:8` | `effects_rest_transport_parse_keystone_holds` | REST transport host |
| `dag/test/claim/diagnostics_test.dag:14–30` | 5× diagnostics holds | `diagnostics_witness` host (5 suites) |
| `dag/test/claim/srv3_os_install_actuate_witness_test.dag:78` | shell runner dissolution | direct `shell.Exec.Run` |

These are **honest spec-without-execution** (provenance strings say "host physics until modeled"). Risk is **enrollment**: they gate the floor green while the `.dag` model is still absent or unexecuted.

---

## Class (c) — grep-shaped witnesses (102 test fns, 41 files)

Largest clusters:

| File | # fns | Pattern |
|------|-------|---------|
| `dag/test/claim/v1_source_audit_witness_test.dag` | 40 | `filesystem_read` + `string_contains` over v1 pipeline `.dag`/`.rs` sources (provenance admits "Grep-style") |
| `dag/test/claim/node_http_server_emit_test.dag` | 8 | `string_contains` on emitted Node server source |
| `src/v2/test/claim/self_host_realized_comparison_floor_test.dag` | 6 | `string_contains` on folded file lists (generated vs hand-maintained) |
| `src/v2/test/claim/ci_spec_witness_test.dag` | 1 | 19 helper fns with `string_contains` on `gunbc_ci_*` generated scripts (aggregator `ci_spec_witnesses`) |
| `dag/test/claim/ci_yaml_serializer_witness_test.dag` | 1 | serialized CI YAML substring checks |
| `dag/test/claim/srv3_*` emit/witness files | ~15 | emitted install-media / cloud-config script substring checks |

These are **legible interim** where the subject is intentionally pre-model (emit output, CI script shape). They are still §5 spec-without-execution for any claim that the underlying `.dag` logic is correct.

---

## Bonus: `map_get` / `Outcome` unwrap hazard

`v2.std.collection:map_get` (`src/v2/std/collection.dag:86`) returns `Outcome<Optional<V>>` and must be unwrapped via `Accepted`/`Rejected` before matching `Present`/`Absent`. The v1-layer `map_get` builtin has a **different shape** — ~15 bare `Present`/`Absent` matches in `dag/std/*`, `dag/extdeps/**`, and some `dag/test/claim/*` import **no** `v2.std.collection` and are **not** this bug class.

**Same-name-different-shape across layers is a hazard amplifier:** the bug is easy to write and hard to grep for.

| Scan | Count |
|------|-------|
| Correct `Accepted`/`Rejected` unwraps in `src/v2/**` importing `v2.std.collection` | 32 |
| **Hazard sites** (`match map_get → Present/Absent` without `Accepted`) | **3** — all in `src/v2/lens/module_graph.dag:83,85,150` |

All other `src/v2` consumers (`affected_set`, `frontier_observation`, `03_resolve`, `witness_option_bridge_test`, etc.) unwrap correctly. The 3 hazard sites are exactly the PR 6274 fix locus; they remain in tree at audit time (interpreter execution of `import_closure_live_test` would go red on unfixed code).

---

## Recommendation (one paragraph)

**Make class (b-danger) unwritable by construction:** extend witness enrollment (`BoolWitnessClaim` / floor discovery) with a **dual-oracle gate** — when a witness body (transitively) calls a host `*_live` builtin that reimplements a `.dag` function with the same contract, enrollment is **refused** unless the same witness entry also contains an interpreter call to the `.dag` authority (or a dedicated paired RED control that executes the `.dag` fn on a discriminating fixture). The gate is a row in the existing construction-justification / `StandingIntent` machinery, not a new ad-hoc grep. **Dissolve-on:** `gunbc#5364` node-tree readers eliminate the host-bridge seam; until then, Rust equivalence modules like `import_closure_equivalence_tests` should be demoted from "equivalence" to "host perf receipt" and must not be the sole green signal for a load-bearing `.dag` lens. Class (c) grep witnesses remain honest interim for emit/CI-shape subjects but should carry an explicit `DecodeFidelity: Lossy` / `WitnessExecutionClass: GrepShaped` stamp so affected-set skip and enrollment optics do not treat them as behavioral coverage.

---

## Audit artifact

Classification script: one-shot Python over witness roots (transitive helper analysis + provenance overrides). Re-run after roster changes; no checked-in ratchet (this is a census, not a gate — the gate is the recommendation above).
