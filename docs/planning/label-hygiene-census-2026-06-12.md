# Label-Hygiene Census — 2026-06-12

> **What this is.** A point-in-time census of deprecated task-ID jargon, milestone
> codenames, and dead bind anchors in code and comments, dispatched by operator
> directive 2026-06-12 ("i don't like putting tags like mvp1_ in actual code … i'm
> not a fan of jargon/nicknames in general, especially ones that are deprecated
> technically"). **Census only — this PR renames nothing.** This is a disposable
> worksheet that sequences the cleanup; the inline marks remain the authority
> (CLAUDE.md ledger standing principle). Delete this doc when the sweep closes.
>
> Measured against branch tip `f59cb7e284` (main parity, 2026-06-12). All counts
> are over the code areas `src/ dsl/ tools/ scripts/ .github/` unless noted;
> `docs/` prose is out of scope for renaming (dated design docs legitimately speak
> in the vocabulary of their time).

## Why these labels are debt

- `src/v4/TASKS.md` was deleted 2026-06-01 (#4192). Every `bind TASKS.md T-NN`
  mark now points at a ledger that does not exist; the IDs are unresolvable for a
  new reader.
- Codename prefixes (`mvp1_`, `comprep_`, `b3_`, `T6`, `wave1`) bake milestone
  scheduling jargon into durable identifiers — file names, witness names, atoms —
  that outlive the milestone.
- 🟡 marks require a **live** bind anchor (open issue or dashboard node). At least
  one cluster binds a **closed** issue whose dissolve-on condition never landed.

---

## Category A — deleted-ledger citations (`src/v4/TASKS.md`)

**151 cite lines across 68 files; 119 of them are inside 🟡 marks** (i.e. they are
bind anchors, not prose asides). Shapes observed:

| Shape | Example | Count class |
|---|---|---|
| `bind TASKS.md T-NN` / `bind src/v4/TASKS.md#t-NN-…` (mark anchor) | `00_compile.dag:71` (T-23), `05_eval.dag:225/345/388/526/645/888` (IRT-4 / T-22), `06_translate.dag:302` (T-11), `coverage.dag:490/705/706` (T-4/T-19) | ~119 |
| Scope/Status header citing the ledger | `extdeps/formats/{csv,toml,sql,openapi,json_schema}.dag` "TASKS.md T-4.6", `fidelity.dag:4` | ~20 |
| Test/harness doc-comments | `sg0_census_test.rs:501/515`, `v4_emit_host_harness_test.rs:8/43`, `v4_t15_self_host_fixed_point_harness_test.rs:2/190` | ~10 |
| **String literals compared at runtime** | `v4_test_bootstrap_infra_closeout_test.rs:184/584` — `"bound task: src/v4/TASKS.md#t-19-…"` asserted as data | 2 |
| CI-floor authority comment | `.github/ci-floor/v4-m1-rust-emit-probe.sh:8` (T-24) | 1 |
| Gate doc-comment | `src/v3/compiler/src/v4_hollow_alias_gate.rs:16` (T-30 "interim mirror paragraph") | 1 |

Top files by cite count: `std/verification.dag` (9), `extdeps/languages/python.dag`
(8), `extdeps/languages/ecmascript.dag` (7), `lens/coverage.dag` (6),
`compiler/05_eval.dag` (6), the five `mvp1_*_add_translate.dag` claims (5 each),
`std/node.dag` (5), `std/compilers/lexing.dag` (5), `extdeps/languages/dag.dag` (5).

Note: `INVARIANTS.md` and `docs/modeling-discipline.md` already carry the corrected
phrasing ("the `src/v4/TASKS.md` ledger is retired; tracked as dashboard work
items") — that is the target wording for repointing.

## Category B — task-ID tokens in code/comments

Token frequency in code areas (comments + feature tags + identifiers):

| Token | Hits | Token | Hits |
|---|---|---|---|
| T-22 | 297 | T-9 | 29 |
| T-4 | 142 | T-30 | 29 |
| T-10 | 74 | T-23 | 25 |
| T-11 | 68 | T4 / T6 (no hyphen) | 23 / 22 |
| T-6 / T-19 | 49 each | T-25 / T-24 | 21 / 20 |
| T-13 | 40 | T-17 | 18 |
| T-7 / T-8 | 35 / 34 | T22 (feature tags) | 12 |
| T-38 / T-21 | 33 each | W-T-* + IRT-4 | 60 lines / 21 files |

Identifier-position occurrences (harder than comment edits — these are feature-tag
strings and witness names):

- `feature:T22-EVAL-CACHE-HASHES` (6 marks in `05_eval.dag`)
- `feature:W-T-10-mvp1-exact-zip-closure` (cpp/go/python/rust/typescript extdeps —
  double jargon: ledger ID + codename)
- `feature:T-23-lens-application-migration`, `feature:T-11-typed-grammar-relation-row-items`,
  `SL-T11-GRAMMAR-FROM-TOKEN-ROW`, `SL-3229-T4-FORMAT-T6T7`
- `t_15_self_host_fixed_point` harness name (cited as a CI gate name)

Caveat: `ROADMAP.md`'s T-22 bridge row is a **live** roadmap row (its trigger is
real); the dissolve marks that cite "ROADMAP T-22 bridge row" are pointing at a doc
that still exists. Those cites are *resolvable*, unlike TASKS.md cites — lower
priority, rename only if the row itself is renamed.

## Category C — codename identifiers (`mvp1_*`, `comprep_*`, atoms, file names)

**2759 raw hits; 650 distinct `*mvp1*` identifiers; 241 distinct `*comprep*`
identifiers.** Distribution: src/v4/test (43 files), src/v4/extdeps (12),
src/v4/compiler (5), src/v3/compiler (3), plus workflow/std/lens/program.dag,
2 v2 test files, 2 tools crates, 1 CI-floor script. Zero hits in generated
bootstrap files (regen is not a coupling).

**23 jargon-named files** (paths are identifiers too):

- `src/v4/test/claim/manual/`: `mvp1_{cpp,go,python,rust,typescript}_add_translate.dag`,
  `mvp1_typescript_{pr3_typed_fn,record_task}_translate.dag`, `mvp1_dag_add_round_trip.dag`,
  `{go,kotlin,python}_mvp1_grammar_claim.dag`, `comprep_add_body_{producer,emit_typescript}.dag`,
  `comprep_{branch_,}eval_by_execution.dag`, `comprep_branch_lazy_arm_eval_acceptance.dag`,
  `comprep_value_expression_fold_typescript.dag`, `comprep_b3_ts_descriptor_node_run.dag`
  (b3 = phase codename **inside** a codename)
- `src/v4/test/claim/complexity_gate/comprep_wave1_{add,branch}_subject_producer.dag`
- `src/v4/test/fixture/dag_round_trip_mvp1.dag`
- `fixtures/v4-mvp1/add/add.dag` (whole directory)
- `docs/design-comprep-m0-branch-mapping.md` (doc — keep, dated artifact)

High-frequency identifier families (top of 650/241): `mvp1_rust_emitted_root` (26),
`dag_mvp1_type_atom_node` (26), `rust_mvp1_target_model` (23), `mvp1_{ts,python,go,cpp}_emitted_root`
(21 each), `comprep_eval_atom` (27), `comprep_ts_canonical_grounding` (25),
`comprep_source_bridged_add_arrow_with_body` (11). Atom literals exist too:
`^dag_mvp1_pick_lit_{one,two}`, `^dag_mvp1_source_literal`,
`^dag_mvp1_captured_flat_atom_binding_port` — atoms are *values*, so renames there
are semantic, not cosmetic (content hashes / witness expectations may move).

**Rename couplings found (must move in the same PR as any rename):**

1. `.github/ci-floor/v4-m0-ts-emit-probe.sh` pins both the entry path
   `manual/mvp1_typescript_add_translate.dag` **and** the witness name
   `mvp1_ts_emit_add_fn_accepts_holds`.
2. `tools/ci_affected_components/src/lib.rs:133` hardcodes the
   `fixtures/v4-mvp1/` path prefix; `src/v2/tests/src/pipeline.rs:868` joins it.
3. `body_producer_reason_mvp1_resolved_shape` (17 hits) lives in the
   `03_body_producer` pipeline stage — load-bearing file, higher escalation bar.
4. Witness names inside claim files are asserted by the claim runner; file renames
   alone are safe-ish (roster has no mvp1/comprep string coupling — discovery is
   glob-based), identifier renames move witness expectations.

## Category D — dead / questionable bind anchors

| Anchor | State | Live citers | Verdict |
|---|---|---|---|
| `gunbc#4674` (T-22 host-emission TargetModel dissolution) | **CLOSED 2026-06-12, dissolve-on NOT landed** (step 1 landed via #4718) | `emit_host.dag:216,289`; `tools/emit_host_runner/src/lib.rs:263,283` (2 marks); DORMANT notes in `comprep_b3_ts_descriptor_node_run.dag:8` **and `sg0_census_test.rs:347`** (the latter missed by the first census pass) | **Dead bind — repoint in flight (PR #4752, not yet merged)** to tracking issue **#4750**, which supersedes #4674 and restates the dissolve-on verbatim. Until #4752 lands, the inline marks still cite #4674 and remain the authority. |
| `PR #3971` (merged) | MERGED; its cited `design-rust-dag-leafmodel-instantiation.md` is no longer in the repo; named owner session archived | `test/claim/language_model/rust.dag:154` — `bind PR #3971 §5 …` | Dead bind — dissolve-on confirmed un-landed (no explicit `-C overflow-checks` runner support exists). **Repoint in flight (PR #4752, not yet merged)** to tracking issue **#4751**. |
| `#4553` (×111), `#4543`, `#4540`, `#4252`, `#4046`, `#3961`, `#3468`, `#4410` | all MERGED/CLOSED | various | **Fine as-is** — these are genealogy cites ("Consolidation #4553", "relocated by PR #4543"), recording provenance, not live binds. No action. |

The distinction that matters: `bind:` anchors assert "this mark dissolves when that
item closes" — those must be live. Provenance cites are history and are healthy.

## Category E — phase-name codenames (T6 / B3 / wave1 / pr3 / M0)

- `06_translate.dag:3661,3699-3700` — "T6 / E-9", "bind: PR #4627 T6 bodied-arrow
  skeleton — owner: T6 value-expression skeleton" (T6 = emit-ladder rung codename).
- `emit_host_eval.rs:184` "B3 omni-emission"; `sg0_census_test.rs:337` "B3
  `run_host_process`"; file name `comprep_b3_ts_descriptor_node_run.dag`.
- `comprep_wave1_*`, `eval_context_v4_evaluator_wave1`, `mvp1_typescript_pr3_*`
  (a PR number as a name component), `v4-m0/m1` probe names.
- v2-side `B3` cites in `src/v2/tests/` bind **ctrl#1476** phases — different
  ledger (ctrl repo, still resolvable); out of scope for this sweep.

## Complete pattern catalogue (operator round 2, 2026-06-12)

Operator follow-up: catalogue **all** known codename patterns (the original brief
named only the mvp1/comprep/task-ID families) and plan to rename/delete them all
ASAP. Counts over code areas; "ids" = identifier-position occurrences (names,
witnesses, types, env vars, file/crate names), which are the expensive ones.

| Pattern | Scale | Exemplars | Couplings | Disposition |
|---|---|---|---|---|
| `mvp1_*` (+files, `fixtures/v4-mvp1/`) | 650 distinct ids | `mvp1_rust_emitted_root`, `mvp1_ts_emit_add_fn_accepts_holds` | ci-floor m0 probe pins path+witness; `ci_affected_components:133` path prefix; `pipeline.rs:868` | rename → content names (canonical add-fn corpus) |
| `^dag_mvp1_*` atoms | ~60 distinct | `^dag_mvp1_pick_lit_one` | atoms are **values** — content hashes/witness expectations move | rename **last** |
| `comprep_*` | 241 distinct ids | `comprep_eval_atom`, `comprep_eval_by_execution` | still growing in #4741/#4747 | rename → body/eval domain names; convention gate now |
| `mvp2_` / `MVP-2` | 338 hits / 22 files | `eval_mvp2_add_subgraph`, `MVP2_RUNTIME_VALUE_FIVE_BYTES` | five-byte runtime-value contract comments in emit_host | rename → contract names (`runtime_value_five_bytes_*`) |
| `pilot` / `Pilot*` | 356 hits + a whole crate | `src/v3/grounding_pilot/` crate, `PilotRustPrimitive`, `rust_pilot_primitives` | workspace member; "T-Ground-Pilot toy probe" per its own header | **delete-candidate**: if the probe's question is answered, retire the crate; else rename to what it validates (inhabitance-search parity) |
| `wave[0-9]` / `wave2a` | 1429 hits / 100 files | `rust_language_model_wave1`, `ts_production_wave2a_type_annotation`, `dag_wave1_lex_token_symbol` | wave = grammar/coverage tier; lens-ci claim names | rename → coverage-descriptive tiers (e.g. `_core`, `_literals`) — **needs a naming table sign-off** |
| `sg[0-9]` | 1250 hits / 62 files (sg2 ×887, sg8 ×286, sg1/sg0 ×~110 each) | `sg0_census_test`, `sg2_pass_node`, `sg8_def/use/proxy` | `sg0_census_test` is the hand-Rust census gate (CI-wired) | **SG = Self-Generation** (see F1e appendix) — program is live; rename = spell it out, pending naming-table sign-off |
| `rung*` / `rung[0-9]+` | 711 hits / 37 files | `rung56_emit_vs_eval_run`, `BlockingForRung`, `nat_semiring_rung34_*` | ladder vocabulary is **live** in ROADMAP/design docs | needs operator decision: either the ladder vocabulary is blessed (then ids citing it are fine) or it renames everywhere incl. docs — don't rename ids unilaterally |
| `phase1` / `Phase-1` | 675 hits / 117 files | `V4_PHASE1_NAT_SEMIRING_*` env vars, `Rustc_ErrorCode_Phase1` | `scripts/v4-phase1-nat-semiring-*-gate.sh` (3 gate scripts) + env-var contract | rename ids+scripts+env vars together, per gate |
| task-IDs `T-NN`/`W-T-*`/`IRT-4` | T-22 ×297, T-4 ×142, … | `feature:T22-EVAL-CACHE-HASHES`, `feature:W-T-10-mvp1-exact-zip-closure` | 119 marks bind deleted TASKS.md | repoint/rewrite (F1a below) |
| `_b[0-9]_` / `_pr[0-9]_` | 190 hits / 9 files | `comprep_b3_*`, `mvp1_ts_pr3_*` (a PR number as a name) | nested inside mvp1/comprep families | rename with parent family |
| `W1`/`W2`/`W3` | 140 hits / 64 files | `emit_host_bridge.rs` "W3 bridge", `Cargo.toml` "W3 / T-38" | comments only | comment rewrite pass |
| `Tranche-N` | 26 hits / 11 files | `go/primitives.dag` "TRANCHE 1" | comments only | comment rewrite pass |
| `Lane X` / `Theme-A` | 33 hits | "Lane C format slice", "Theme-A #9" | comments only | comment rewrite pass |
| `m0`/`m1` probe names | 2 scripts | `.github/ci-floor/v4-m{0,1}-*-emit-probe.sh` | ci.yml job wiring | rename with workflow update |
| `T6`/`B3` phase names in comments | ~30 | `06_translate.dag:3661/3699` | comments only | comment rewrite pass |
| Borderline: `r[123][ab]?` leaf-model rows, `stage0/1/2` | `claim_rust_leaf_r2b_*`; `src/v2/stage0/` | R-rows defined by live design docs; stage-N is standard bootstrap terminology | — | **keep** stage-N; R-rows rename only if their design doc vocabulary changes |

Rough total: ~7,000 jargon-token hits in code areas; the identifier-position
subset (the real work) is on the order of 1,200 distinct names plus 25+ file/
script/crate names.

## In-flight collision map (why renames wait)

Open PRs touching census files as of 2026-06-12:

- **#4741** (§2 Branch emit/round-trip) — *adds new* `comprep_bind_lazy_arm_eval_acceptance.dag`,
  `comprep_branch_emit_if_then_else.dag`, `comprep_loop_lazy_arm_eval_acceptance.dag`,
  `mvp1_dag_pick_round_trip.dag`; touches `extdeps/languages/{dag,typescript}.dag`.
- **#4747** (Q-B2 infer Branch row) — *adds new* `comprep_branch_infer.dag`.
- **#4748 / #4745 / #4743** (§4 runtime_run lane) — touch `manual/runtime_run.dag`
  (descriptively named — the convention the operator wants is already appearing).

The jargon family is **still growing** in the §2/§4 lanes. Renaming under their
feet guarantees conflicts on the busiest files (extdeps language models, manual/
claims). Two consequences:

1. **Stop-the-bleeding is cheap and immediate:** new files/witnesses use
   descriptive names (`runtime_run.dag` is the existing good example;
   `branch_emit_if_then_else.dag` not `comprep_branch_emit_if_then_else.dag`).
   That's a review-convention note to the §2/§4 lanes, not a code change.
2. **Bulk renames sequence after the §2 wave-2 and §4c merges settle.**

## ASAP cleanup plan (operator round 2 — rename/delete everything, soonest safe order)

Operator directive 2026-06-12 (round 2): rename/delete **all** catalogued
patterns ASAP. This supersedes the earlier "park until §2/§4 settle" framing —
the only things that still wait are the specific *files* open PRs are touching,
and the few families that need a naming decision first. Everything else
dispatches now, in parallel (the families are mostly file-disjoint).

**Wave 0 — dead binds: in flight (PR #4752,** awaiting merge; issues #4750/#4751).

**F1 — dispatch immediately, parallel lanes, no inter-conflicts:**

- **F1a — comment-only jargon rewrite** (biggest single lane): TASKS.md cite
  repointing (119 marks: bind → dashboard node / open issue, or self-contained
  dissolve-on), plus the comment-only families (W1/W2/W3, Tranche-N, Lane/Theme,
  T6/B3 phase prose). Includes the two runtime string literals in
  `v4_test_bootstrap_infra_closeout_test.rs` (update assertion sites together).
  File-batched to dodge #4741's two extdeps files until it merges.
  **Operator ruling 2026-06-12 (during F1a review):** repointed marks carry only
  the live bind (e.g. `bind gunbc#4757`) — no historical/meta parentheticals
  like "(re-anchor of deleted TASKS.md T-9)" in code comments. The deleted-ledger
  genealogy lives in the tracking issues (#4750/#4751, #4757–#4766, #4773), not
  inline. Applies to all future repoints/renames.
- **F1b — pilot crate disposition:** decide delete-vs-rename for
  `src/v3/grounding_pilot/` ("T-Ground-Pilot toy probe" by its own header). If
  the probe's question (inhabitance-search reproduces table-lookup routing) is
  answered, retire the crate and its mirror-validation tests; else rename to
  what it validates. Self-contained, zero collisions.
- **F1c — phase1 gates:** rename `scripts/v4-phase1-nat-semiring-*` + their
  `V4_PHASE1_NAT_SEMIRING_*` env vars + consumers in one PR per gate.
- **F1d — ci-floor probe scripts:** `v4-m{0,1}-*-emit-probe.sh` → descriptive
  names, with ci.yml wiring in the same PR.
- **F1e — naming-table sign-off (blocking F3 only):** one short operator/parent
  review fixing replacement vocabulary for the decision-needing families:
  `wave[0-9]` tiers (proposal: coverage-descriptive — `_core`, `_literals`, …),
  `sg[0-9]` (SG = Self-Generation — resolved, see appendix), `rung*` (ladder
  vocabulary is live in ROADMAP — bless it or rename it everywhere, not just in
  ids). Without this, F3 renames would mint a second generation of guesses.

**F2 — file renames (the moment #4741/#4747 merge):** 23+ jargon-named claim/
fixture files + `fixtures/v4-mvp1/` directory, each with its couplings in the
same PR (ci-floor probe pins, `ci_affected_components:133` path prefix,
`pipeline.rs:868` fixture join). #4741/#4747 are near bar, so this is days not
weeks; their five new `comprep_*`/`mvp1_*` files join the rename list.

**F3 — identifier/witness renames (after F1e lands):** ~1,200 distinct names
across the mvp1/mvp2/comprep/b3/pr3/wave/sg/phase1 families, batched per
directory/file-cluster, suite-delta=0 per PR, repoint-then-delete in the same
PR (Jun-11 §0 rule). Feature tags rename here too
(`feature:T22-EVAL-CACHE-HASHES` → `feature:eval-cache-content-hashes`).

**F4 — atom renames, last:** `^dag_mvp1_*` and friends. Atoms are values —
content hashes and witness expectations move — so this batch carries the most
verification weight and goes after the cheap mass is gone.

**Backstop so the family never regrows:** once F2/F3 land, add a tiny
naming-hygiene check (grep gate over the catalogue's patterns for *new*
occurrences, ratcheted to current count → 0). Not before — a gate now would
fight the in-flight PRs.

Naming direction everywhere: name by **what the artifact is**, not when it was
scheduled — `add_translate_rust.dag` not `mvp1_rust_add_translate.dag`;
`eval_by_execution_add.dag` not `comprep_eval_by_execution.dag`;
`runtime_value_five_bytes` not `MVP2_RUNTIME_VALUE_FIVE_BYTES`.

## F1e appendix — `sg[0-9]` resolved: SG = Self-Generation

Defining cites:

- **Live**: `docs/design-pure-bootstrap.md:340` — "**SG program**
  (Self-Generation): SG retires specific Rust files by authoring their `.dag`
  counterparts. PB is the program that names the endpoint: SG-N deliverables
  feed PB-N stages."
- **Deleted** (#4192 purge), fuller definition:
  `docs/history/roadmap-active-deferrals.md:244` at `eceeed72d3^` — "The
  Self-Gen (SG) program is the 'zero hand-authored Rust in src/v3' track …
  every SG PR reduces the hand-authored Rust census in src/v3, ratchet only
  down. SG-6 specifically owns build.rs, bootstrap.rs, pipeline_authority.rs,
  lens_testgen.rs, and src/bin/regen_*.rs."

Per-lane meanings (v3 family — what the renamer needs):

| Lane | Surface | Cite |
|---|---|---|
| SG-0 | hand-Rust census + ratchet gate | `sg0_census_test.rs` header |
| SG-1 | tokenize.dag tokenizer authority | `docs/design-pure-bootstrap.md` |
| SG-2 / SG-2b | parser staging / .dag-owned parse logic | `src/v3/SELF_HOSTING.md:998` |
| SG-3 | lower | `docs/design-pure-bootstrap-zero.md` |
| SG-4 | infer dispatch | `docs/design-pure-bootstrap-zero.md` |
| SG-5 | substrate / runtime-mirror projections | `src/v3/compiler/build.rs:661` |
| SG-6 | build.rs / bootstrap.rs / regen binaries | deleted roadmap-active-deferrals.md |

Unlike mvp1/comprep, the SG program is **live** (it is the thesis — Rust
shrinks toward zero); the debt is only the cryptic abbreviation. Proposed F3
rule (pending operator sign-off): spell it out — `sg` → `self_gen`
(`sg0_census_test.rs` → `self_gen_census_test.rs`), with descriptive per-lane
suffixes where the surface is clear. Caveat for the renamer: witness fns
`sg2_arrow` / `sg2_mode2*` / `ts_sg2_conj` are in the known-false baseline
ledger (5 v4 claim witnesses false on main, pre-existing) — any rename must
carry that annotation along.

**Second, unrelated SG family (DEFERRED):** v4 uses SG-1/SG-1b/SG-2/SG-3/SG-5
as target-realization/grounding **catalog labels** in
`src/v4/std/compilers/target_model.dag`. Its header cites
`design-target-realization-canonical-home.md`, which no longer exists (likely
#4192); no expansion found — possibly not even the same acronym. That file is
skip-listed (in-flight #4741 + effect-emit PR1), so this family gets its own
naming decision later.
