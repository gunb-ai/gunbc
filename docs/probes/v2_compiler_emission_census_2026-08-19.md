# v2 compiler emission census — all 41 top-level modules at one pinned SHA

**Pin: `90b1e4e7ffc8baa6407cbe21d3b65aaf76fe4913`.**

**Main has moved past this pin.** #8570 merged as `52e1c4dc6c` after these measurements were taken, and
it touches `v2_compiler_infer` — which is on the work list below. This document describes the tree at
`90b1e4e7ff`, not current main. A reader who assumes it is current will misattribute the difference.

Every row was measured with `docs/probes/curated_cargo_probe_one.sh` under `CSSL_STD_SEED_LINK=1` with
no lane shim, dispatched remotely. `PROBE_EXPECT_BASE_SHA` asserted the pin in-run on every dispatch.

## What this is and is not

A **dated observation** at a pinned SHA. Nothing consumes it programmatically and nobody maintains it;
it is evidence for a plan, not a live authority. It is prose in `docs/` rather than a typed `.dag`
carrier precisely because a carrier would imply an authority someone keeps current, which would already
be false (DESIGN §4c — machine-consumed facts belong in typed carriers; nothing here is machine-consumed).

## Coverage — four states, never collapsed

| state | n | meaning |
|---|---|---|
| MEASURED | 40 | a board exists |
| EMIT_REFUSE | 1 | attempted; the emitter refused before cargo. No board exists and none ever will |
| IN_FLIGHT | 0 | — |
| HOLE | 0 | nobody attempted it |

`program_assembly` is the EMIT_REFUSE. The refusal is deterministic, so a retry would refuse
identically; it is **not a hole and not a zero**, and it carries no numeric board anywhere in this
document.

Total across the 40 boards: **9028 diagnostics**.

## Instrument limitations, stated at their real scope

**The import-reachability proxy under-approximates everywhere.** Reachability below is computed from
declared `import` lines, transitively. But `.dag` resolves names by namespace, so a module may use a
name it never imported — and this does **not** require a module to declare zero imports. A module with
fifteen declared imports can use a sixteenth unimported name, and the proxy is equally blind to that
edge. Zero-import modules are merely where the blindness is *total* and therefore where it was
guaranteed to become visible; elsewhere it is partial and silent.

So the 26 is a **lower bound** on reach. What corroborates the clean population is not the proxy but the
**outcome column**: all 14 non-reaching modules carry `E0063=0`, and a module that truly reached
`target_model` would carry the block. The measurement confirms what the proxy could not see.

**Determinism rests on exactly two receipts, both accidental.** deep-bat-35's infrastructure retry
re-measured three modules from a fresh clone and seed build — identical class counts across nineteen
classes. neat-owl-140's `self_host` was measured in two separate dispatches hours apart — identical
field for field. Two receipts, not eleven copies; an earlier apparent ten-module agreement was a derived
file that had re-entered its own population and was discarded.

## The root: `v2.std.compilers.target_model`

`E0063` takes exactly two values across all 40 boards — **16 or 0**, with no intermediate value anywhere.

| | E0063 = 16 | E0063 = 0 |
|---|---|---|
| reaches `target_model` | 26 | 0 |
| does not reach (per proxy) | 1 | 13 |

The single off-diagonal cell is `body_producer_forward`, the one module of 41 that declares no imports
at all; it uses `Symbol` unimported, emits 72 files, and carries the full signature. It is a false
negative of the proxy, not a counterexample.

**Closure size is not the discriminant.** Four modules sit at exactly 72 files emitted and split across
both outcomes:

| module | files | E0063 |
|---|---|---|
| `build_transport_admission` | 72 | 0 |
| `body_producer_forward` | 72 | 16 |
| `fold_lowering` | 72 | 16 |
| `namespace_graft` | 72 | 16 |

The E0063=16 population spans 67–179 files and the E0063=0 population spans 13–72. The ranges overlap,
so **no file-count threshold exists at any value**.

| | modules | combined board | mean |
|---|---|---|---|
| reaching `target_model` | 26 | 8108 | 312 |
| not reaching | 14 | 920 | 66 |

`target_model` is strictly upstream of `v2.std.runtime` (reaching it transitively; the reverse does not
hold), and across this population `runtime` is reached **only** under `target_model` — one inclusion
delivering two files. But **there is no edge to cut**: 14 of the 26 import `target_model` directly. It is
a deliberately used authority, so the work is in the file.

## The shared file set

File-level joins for `00_compile` (79 files, 691 sited, 0 unsited) and `emit_module` (35 files, 286
sited, 0 unsited). **34 files appear in both, and in 34 of 34 the diagnostic count is identical.**

| count | file | classes |
|---|---|---|
| 61 | `v2_std_compilers_target_model.rs` | E0063:16 E0277:15 E0308:11 unreachable:5 E0614:4 E0369:3 |
| 36 | `v2_std_runtime.rs` | E0277:30 E0369:6 |
| 25 | `v2_std_integer.rs` | E0308:18 E0282:3 E0392:2 E0369:2 |
| 22 | `v2_compiler_translate.rs` | E0597:10 E0609:7 E0071:3 unreachable:2 |
| 20 | `v2_compiler_infer.rs` | unreachable:11 E0308:4 E0597:3 E0560:2 |
| 19 | `v2_extdeps_languages_dag.rs` | E0308:14 E0560:3 E0609:1 unreachable:1 |
| 14 | `v2_std_grammar.rs` | E0609:4 E0308:4 unreachable:4 UNRESOLVED_CompilerError:1 E0597:1 |
| 13 | `v2_std_algebra.rs` | E0277:10 E0369:2 E0308:1 |
| 11 | `v2_std_logic.rs` | E0308:11 |
| 8 | `v2_compiler_resolve.rs` | unreachable:4 E0560:1 E0308:1 E0609:1 E0597:1 |
| 5 | `v2_std_node.rs` | E0308:3 unreachable:2 |
| 4 | `v2_std_symbol_index.rs` | E0599:4 |
| 4 | `v2_std_collection.rs` | E0308:3 E0560:1 |

That is **278 diagnostics in identically-counted shared files — 97% of `emit_module`'s board of 286**,
and 40% of `00_compile`'s 691.

### Own-code share

| module | own emitted file | board | own share |
|---|---|---|---|
| `00_compile` | 18 | 691 | 2.6% |
| `emit_module` | 8 | 286 | 2.8% |

A lane assigned to drive `emit_module` to zero owns **eight** diagnostics and is blocked by 278 it does
not own — and nothing in the board says so. This is why per-module packets kept producing roots that did
not generalise: they were roots of the shared closure, found through whichever module was assigned.

Reach of the top shared files, per the (under-approximating) proxy, over the 40 measured modules:

| file's module | reached by |
|---|---|
| `v2.std.compilers.target_model` | 26 |
| `v2.std.runtime` | 26 |
| `v2.compiler.infer` | 12 |
| `v2.compiler.translate` | 10 |

`v2_compiler_infer.rs`'s 11 `unreachable_pattern` therefore appear in 12 boards, not one. Note that
`unreachable_pattern` has several sources — modules that do not reach `infer` still carry it — so
infer's 11 is a subset of the class, and same class is not same defect.

## The matrix

**A board is a snapshot, not a score, and this table must not be read as a ranking of what to fix.**
A diagnostic count is not a value function: a fix that makes a silent wrong answer *loud* always looks
like a regression on a count, and a fix that makes a loud error *silent* always looks like progress —
and the second direction is the dangerous one. A module whose count **rises** because concealed
non-exhaustiveness became a hard error has **improved**. Rank by what a fix *converts*, not by board
size, and report any re-measurement as which classes converted into which — never as a net.

That is not hypothetical here. Re-measuring `emit_module` after gunbc#8570 (`52e1c4dc6c`, the emitter's
nested-constructor-pattern flattening) moved its board 286 → 276, a net of −10 that reads as a rounding
error. What actually happened: `unreachable_pattern` went 37 → **0** while `E0004` (non-exhaustive
match) went 1 → **28**, with all sixteen other classes byte-identical. Thirty-seven lint-shaped
concealments became twenty-eight hard errors — a climb from *mitigatable* to *loud refusal* (DESIGN
§4b), bought for +27 visible errors. The net conceals the entire event.

The conversion is also **heterogeneous**, which only a per-class reading shows: a control module that
does not reach `v2.compiler.infer` lost its 2 `unreachable_pattern` and gained **no** `E0004`. Where the
flattened arm was genuinely redundant it vanishes; where it concealed real non-exhaustiveness, that
surfaces as an error.

Boards sorted descending. `program_assembly` is carried as its own row with **no numeric board**.

| module | files | board | E0308 | E0277 | ~unreachable_pattern | E0369 | E0063 | E0560 | E0609 | E0597 | E0599 | E0282 | E0425 | E0614 | E0061 | E0615 | E0392 | E0071 | E0004 | ~UNRESOLVED_CompilerError | E0310 | E0631 | E0433 | E0728 | E0573 | E0223 | E0533 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `03_ingest` | 179 | **692** | 292 | 102 | 41 | 45 | 16 | 18 | 19 | 27 | 32 | 14 | 24 | 7 | 17 | 4 | 2 | 3 | 1 | 1 | 4 | 10 | 8 | 2 | 1 | 1 | 1 |
| `00_compile` | 177 | **691** | 292 | 102 | 41 | 45 | 16 | 18 | 19 | 27 | 32 | 14 | 24 | 6 | 17 | 4 | 2 | 3 | 1 | 1 | 4 | 10 | 8 | 2 | 1 | 1 | 1 |
| `03_name_resolve` | 143 | **495** | 233 | 65 | 24 | 25 | 16 | 13 | 12 | 8 | 31 | 14 | 19 | 4 | 17 | 4 | 2 |  | 1 | 1 | 2 | 1 | 1 |  | 1 | 1 |  |
| `source_authority` | 132 | **439** | 183 | 65 | 38 | 17 | 16 | 13 | 18 | 21 | 17 | 12 | 12 | 4 | 6 | 4 | 2 | 3 | 1 | 1 | 2 |  |  | 2 | 1 | 1 |  |
| `emit_host` | 126 | **425** | 152 | 98 | 42 | 36 | 16 | 13 | 13 | 18 | 5 | 6 | 8 | 5 | 1 | 3 | 2 | 3 | 1 | 1 | 2 |  |  |  |  |  |  |
| `ingested_fixture_arrows` | 116 | **376** | 169 | 65 | 24 | 16 | 16 | 11 | 11 | 8 | 16 | 12 | 8 | 4 | 5 | 4 | 2 |  | 1 | 1 | 2 |  |  |  | 1 |  |  |
| `05_eval` | 94 | **339** | 121 | 78 | 37 | 33 | 16 | 12 | 6 | 8 | 5 | 5 | 3 | 5 | 1 | 3 | 2 |  | 1 | 1 | 2 |  |  |  |  |  |  |
| `05_emit_orchestration` | 106 | **292** | 98 | 55 | 37 | 16 | 16 | 11 | 13 | 15 | 5 | 6 | 5 | 4 | 1 | 3 | 2 | 3 | 1 | 1 |  |  |  |  |  |  |  |
| `emit_produced` | 96 | **292** | 102 | 55 | 37 | 16 | 16 | 9 | 13 | 15 | 5 | 5 | 4 | 4 | 1 | 3 | 2 | 3 | 1 | 1 |  |  |  |  |  |  |  |
| `program_partition` | 100 | **291** | 90 | 55 | 38 | 16 | 16 | 9 | 13 | 15 | 5 | 11 | 4 | 4 | 1 | 3 | 6 | 3 | 1 | 1 |  |  |  |  |  |  |  |
| `02_parse` | 102 | **291** | 120 | 60 | 15 | 15 | 16 | 10 | 10 | 5 | 5 | 11 | 4 | 4 | 5 | 4 | 2 |  | 1 | 1 | 2 |  |  |  | 1 |  |  |
| `emit_module` | 95 | **286** | 96 | 55 | 37 | 16 | 16 | 9 | 13 | 15 | 5 | 5 | 4 | 4 | 1 | 3 | 2 | 3 | 1 | 1 |  |  |  |  |  |  |  |
| `emit_semantic_decl` | 96 | **281** | 89 | 55 | 38 | 16 | 16 | 9 | 13 | 15 | 5 | 6 | 4 | 4 | 1 | 3 | 2 | 3 | 1 | 1 |  |  |  |  |  |  |  |
| `06_translate` | 93 | **278** | 88 | 55 | 37 | 16 | 16 | 9 | 13 | 15 | 5 | 5 | 4 | 4 | 1 | 3 | 2 | 3 | 1 | 1 |  |  |  |  |  |  |  |
| `05_emit` | 94 | **278** | 88 | 55 | 37 | 16 | 16 | 9 | 13 | 15 | 5 | 5 | 4 | 4 | 1 | 3 | 2 | 3 | 1 | 1 |  |  |  |  |  |  |  |
| `04_infer` | 87 | **252** | 87 | 55 | 34 | 15 | 16 | 9 | 6 | 5 | 5 | 5 | 3 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `03_resolve` | 82 | **230** | 81 | 55 | 23 | 15 | 16 | 7 | 6 | 2 | 5 | 5 | 3 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `03_normalize` | 81 | **225** | 80 | 55 | 20 | 15 | 16 | 6 | 5 | 1 | 5 | 5 | 5 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `normalized_tree` | 80 | **222** | 80 | 55 | 19 | 15 | 16 | 6 | 5 | 1 | 5 | 5 | 3 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `body_lowering_fold` | 79 | **220** | 80 | 55 | 19 | 15 | 16 | 6 | 5 | 1 | 5 | 5 | 1 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `symbol_index_fill` | 74 | **216** | 78 | 55 | 17 | 15 | 16 | 6 | 5 | 1 | 5 | 5 | 1 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `body_producer_forward` | 72 | **211** | 78 | 55 | 16 | 15 | 16 | 6 | 5 | 1 | 1 | 5 | 1 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `fold_lowering` | 72 | **210** | 78 | 55 | 15 | 15 | 16 | 6 | 5 | 1 | 1 | 5 | 1 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `namespace_graft` | 72 | **210** | 78 | 55 | 15 | 15 | 16 | 6 | 5 | 1 | 1 | 5 | 1 | 4 | 1 | 3 | 2 |  | 1 | 1 |  |  |  |  |  |  |  |
| `wrap_decision` | 67 | **195** | 79 | 55 | 10 | 15 | 16 | 2 |  |  | 1 | 5 | 1 | 4 | 1 | 3 | 2 |  | 1 |  |  |  |  |  |  |  |  |
| `07_target_carriers` | 69 | **194** | 75 | 55 | 11 | 16 | 16 | 2 |  |  | 1 | 5 | 2 | 4 | 1 | 3 | 2 |  | 1 |  |  |  |  |  |  |  |  |
| `trait_derive_completeness` | 68 | **188** | 71 | 55 | 11 | 15 | 16 | 2 |  |  | 1 | 5 | 1 | 4 | 1 | 3 | 2 |  | 1 |  |  |  |  |  |  |  |  |
| `build_transport_admission` | 72 | **139** | 56 | 56 | 4 | 14 |  | 3 |  |  |  | 4 |  |  |  |  | 2 |  |  |  |  |  |  |  |  |  |  |
| `native_selected_bundle_process` | 51 | **108** | 45 | 40 | 4 | 12 |  | 2 |  |  |  | 3 |  |  |  |  | 2 |  |  |  |  |  |  |  |  |  |  |
| `01_tokenize` | 26 | **98** | 66 | 15 | 2 | 4 |  | 1 |  | 2 | 7 | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `inferred_tree` | 58 | **75** | 46 | 10 | 4 | 6 |  | 2 |  |  |  | 4 |  |  | 1 |  | 2 |  |  |  |  |  |  |  |  |  |  |
| `self_host` | 59 | **73** | 44 | 10 | 4 | 6 |  | 2 |  |  |  | 4 |  |  | 1 |  | 2 |  |  |  |  |  |  |  |  |  |  |
| `build_workspace_grant` | 47 | **45** | 11 | 26 | 2 | 4 |  | 1 |  |  |  | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `host_run_boundary_admission` | 48 | **45** | 11 | 26 | 2 | 4 |  | 1 |  |  |  | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `materialization_carriers` | 52 | **43** | 15 | 13 | 2 | 2 |  | 1 |  |  | 4 | 1 | 3 |  |  |  |  |  |  |  | 2 |  |  |  |  |  |  |
| `parse_diagnostic` | 21 | **33** | 18 | 10 | 2 | 2 |  | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `use_site_verdict` | 19 | **22** | 7 | 10 | 2 | 2 |  | 1 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `03_body_producer` | 15 | **18** | 4 | 10 | 2 | 2 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `discovery_enumeration` | 14 | **5** | 3 |  | 2 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |
| `parse_engine_hooks` | 13 | **5** | 3 |  | 2 |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |  |

| `program_assembly` | — | *EMIT_REFUSE* | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — | — |
