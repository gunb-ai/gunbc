# Census: greens whose subject scope is unstated

**Subject:** every check that can report success in this repository, asked one question —
*what population did this green actually range over, and does the green say so?*

**Ref:** all measurements below are at `026a709a71636d60df325f9183b78febff22901d` (main,
2026-08-20) unless a run id is named. CI observations are named by run id.

**Class.** A green that is TRUE but answers a NARROWER question than its reader believes.
DESIGN §4b names the failure: a reported rung must equal the rung established by executed
evidence, *measured against a declared boundary*, and a class's rung is the **minimum across
its in-scope paths** — "citing the strongest path while another stays silent is inflation."
DESIGN §5 names two causes: **specification-without-execution** (a green from something that
did not run) and the **absorbing fallback**'s mirror, the **empty-observation narrow** (⊥-as-
answer conflated with ⊥-as-ignorance).

**House rule applied throughout.** A measurement is reported with its subject and its ref. An
inference is labelled. No zero is reported without a control showing the instrument could have
returned nonzero.

**What a row is.** Mechanism (module + symbol, per the DESIGN §3 cite-the-symbol rule) · what a
green there asserts · what a reader takes it to assert · whether the gap is stated anywhere ·
whether the narrow reading is reachable in practice. Ranked by how load-bearing the misreading
is, not by how easy a fix looks. **No mechanism is proposed here** — this pass measures.

**Live surface.** `.github/workflows/` holds exactly two files: `witnesses.yml` and
`fleet-converge.yml`. `witnesses.yml` is the only one on `push`/`pull_request`; everything a
merge is gated on is one of its five steps. That is the denominator for rows 1–5.

---

## Rank 1 — `witnesses.yml` step "Build the witness fold" is the tree's only compile gate, and it is bin-scoped

**Mechanism.** `.github/workflows/witnesses.yml`, step `build_witness_fold`:

    cargo build --release -p v1-compiler --bin claim_executor --bin gunbc --bin v1_src_dag_parse

**A green asserts:** those three binaries, and the dependency crates they pull, compile under
the release profile.

**A reader takes it to assert:** the repository compiles.

**What is outside the population, measured:**

| population | measured | in the green |
|---|---|---|
| `src/v1/stage0/src/bin/*.rs` | 31 | 3 |
| workspace members (root `Cargo.toml`) | 9 | reached only as deps of `v1-compiler` |
| `src/v1/tests` | a declared workspace member | never built |
| `src/v1/stage0_core` | has a `Cargo.toml`; named by **no** manifest in the tree — not a member, not in `exclude` (`[".stage1"]`), not a path dependency (grep over every `Cargo.toml` → 0; control: `stage0_runtime` matches three) | never built |
| `#[cfg(test)]` code | no `--all-targets` anywhere | never compiled |

**Grep control (house rule).** `cargo test`, `cargo clippy` and `--all-targets` return **zero
matches** across `.github/workflows/` and `.githooks/`. The instrument could have returned
nonzero: the same grep over the same paths matches `cargo build` once, in `witnesses.yml`.

**Is it stated?** *Partially, and not where it matters.* DESIGN's CI paragraph records two
operator rulings — the Rust test suite left CI 2026-07-11 and clippy left 2026-07-08 — and both
are about *running* those checks. Neither says that the remaining Rust **compile** is scoped to
three binaries. DESIGN's explicit "WHAT IS UNGUARDED IN THE MEANTIME" list (drift gates, heal,
the seven effect gates, fmt, merge-admission stamping, the falsifier cadence, the per-witness
eval deadline) does **not** include the Rust workspace compile.

**Reachable in practice?** Yes, and cheaply: a compile error in any of the 28 unbuilt bins, in
`src/v1/tests`, or in any `#[cfg(test)]` block merges green.

**Why rank 1.** Every other row on this census is a check *about* something. This is the row
where the thing checks are made of stops being checked, and the step is named "Build".

---

## Rank 2 — the step named "All witnesses" runs 9008 of 10498 discovered sites

**Mechanism.** `witnesses.yml` step `All witnesses (one prepared subject, one fold)` →
`claim_executor --required-floor` → `v2.workflow.required_floor` `run_required_floor`.

**Measured, run `32203661157` (main, green):**

    [floor-disposition] 1490 of 10498 discovered site(s) are NOT required-floor and NOT RUN
    HERE: 733 long-home-declined (68 of them also declare ReadsLiveTree), 757 live-tree-declined
    — each needs an executing consumer on another cadence, and none exists yet on this branch
    required-floor: subject=6aa2207bcb59abd7 modules_resolved=3650 modules_excluded=2
    required-floor: planned=9008 executed=9008 terminal=9008 passed=8702 known_red_held=306
    failed=0 stale_quarantine=0 budget_refused=0 host_tool_unresolved=0

**Is it stated?** *In the log, exemplarily.* That `[floor-disposition]` line is the best
scope statement in the repository: it gives the denominator, the numerator, the split by cause,
and — the part most such lines omit — that no other cadence picks up the remainder. `cli_run.rs`
`prepare_repository_from_source_roots` deliberately computes the subject digest and both module
counts **before** the gate that can reject, so a refusal states its subject too. The current
tree adds `offered= / routed= / declined_long= / declined_live=` on its own line, and
`[floor-route-gap] N enrolled identity(ies) held as route-gapped` for witnesses that never
reached their subject. This machinery is not the finding.

**The finding is the surface mismatch.** The GitHub checks UI, the PR page, and every
`gh run list` consumer show the **step name**, not the log. The step name is *"All witnesses"*.
The honest statement is ~13 minutes into a step log that a reader must know to open. 14.2% of
discovered sites are not run, and the only surface that says so is the one nobody reads on the
way to merging.

**Reachable in practice?** It is the normal case, on every run.

---

## Rank 3 — `--required-regen`'s population is 128 of the 203 `.rs` files under `src/v1/stage0/src`, and the receipt names only the 128

**Mechanism.** `v1_compiler::required_regen_host` `committed_generated_basenames`, consumed by
`run_required_regen`; surfaced by `witnesses.yml` step *"Regen fixed point: first generation
matches committed candidate"*.

**A green asserts:** the 128 committed top-level `.rs` files that are not on
`HAND_MAINTAINED_STAGE0_FILES` are byte-equal, after rustfmt normalization, to what the `.dag`
authority emits for them this pass.

**A reader takes it to assert:** the seed regenerates from the `.dag` authority.

**Measured, and the instrument validated against itself:**

- run `32397225590`: `required-regen: elapsed_ms=392959 first_generation_equal=false planned=128 executed=128`
- at `026a709a716`: `ls src/v1/stage0/src/*.rs` → **164**; `HAND_MAINTAINED_STAGE0_FILES` → **36** entries; 164 − 36 = **128**. The measured `planned` and the derived denominator agree exactly, so the reading below is over a validated instrument.
- `find src/v1/stage0/src -mindepth 2 -name '*.rs'` → **39**, split `bin` 31, `cli_run` 5, `module_path_index` 3.

**So the population is 128 of 203 `.rs` files (63%), and the receipt prints only `planned=128
executed=128` — a self-consistent pair with no denominator beside it.**

**The sharp half.** `HAND_MAINTAINED_STAGE0_DIRS` names `cli_run` and `module_path_index`, so
8 of the 39 nested files are excluded *by declaration*. The other **31 — all of `bin/`,
including `claim_executor.rs`, the binary that runs this very gate and the entire floor — are
excluded by nothing but the fact that `committed_generated_basenames` calls `fs::read_dir` and
does not recurse.** There is no row anywhere asserting that `bin/` is hand-maintained. The
comparison is also basename-keyed throughout (`emit_path_basename` takes `file_name()`;
`lookup_emitted` probes `src/{basename}` then `{basename}`), so the population is structurally
flat: an emitted file at a nested path could not be represented in the candidate tree even if
the walk found it.

**Is it stated?** No. Not in the receipt line, not in the step name, not in DESIGN. The
declared-exclusion list (`HAND_MAINTAINED_STAGE0_FILES`/`_DIRS`) reads as *the* exclusion
authority, and it is not — the walk's arity is a second, silent one.

**Reachable in practice?** Yes: a generated surface authored under a subdirectory leaves the
regen population with no refusal. Note the contrast with `v1_src_dag_parse` (row 6), whose
in-file comment records this exact defect being found and fixed in the *other* recursive walk:
"a walk that finds no files in a directory it never opened is indistinguishable from a clean
one." The lesson was applied there and not here.

---

## Rank 4 — `verify_hand_maintained` verifies that rustfmt ran, not that anything matches

**Mechanism.** `v1_compiler::required_regen_host` `verify_hand_maintained`, whose report feeds
`first_generation_equal = sync.matches && hand.unverifiable.is_empty()`.

**What the body does.** For each of the 36 `HAND_MAINTAINED_STAGE0_FILES`:

- if the emitter produced no candidate for it → `continue`, uncounted;
- otherwise normalize both sides and evaluate `if committed_norm != candidate_norm { }` — **an
  empty block**, with the comment *"drift expected on clean tree for some hand files; not a sync
  refusal."*

The **only** path that can push onto `unverifiable`, and so the only path by which this function
can influence `first_generation_equal`, is `normalize_with_workdir` returning `Err` — i.e.
rustfmt failing to execute.

**A green asserts:** rustfmt ran successfully on both texts of every hand-maintained file for
which the emitter produced a candidate.

**A reader takes it to assert:** it is called `verify_hand_maintained`, it gates a boolean called
`first_generation_equal`, and it sits under a step called *"first generation matches committed
candidate"*. A reader takes it to assert that the hand-maintained files match.

**Is it stated?** Only in an inline source comment. Not in the receipt, not in the step name, and
the aggregation into `first_generation_equal` actively contradicts the comment.

**Reachable in practice?** It is the steady state — the comment says drift there is *expected*.
The 36 files include `cli_run.rs`, `v1_interpreter.rs` and `main.rs`.

**Note on the adjacent counter, which is honest.** `claim_executor --verify-build-artifacts`
(used by `fleet-converge.yml`, not by `witnesses.yml`) checks `is_file`, the executable bit, and
non-zero length — and its success line says exactly *"declared release binar{y|ies} present +
non-empty"*. That is the shape row 4 is missing: the receipt states the predicate, so the name
cannot outrange it.

---

## Rank 5 — `lifecycle_totality_witness_observation`: the axis is its own input (confirmed, owned elsewhere)

**Mechanism.** `gunbc.stage0_rust_lifecycle_totality` `lifecycle_totality_witness_observation`;
consumers in `dag/test/claim/stage0_rust_lifecycle_totality_witness_test.dag`,
`…_maintenance_census_report_witness_test.dag`, `…_honest_frontier_integration_witness_test.dag`.

**Confirmed by reading, not re-measured** (a separate child owns the repair): every enrolled
consumer supplies the observation's tracked set as *the classified set concatenated with the
test's own `extra_paths`*, so the derived `unclassified_paths` axis is identically `extra_paths`.
The witness greens on a subject constructed from its own answer. The live arm — which would put
the axis against every tracked `.rs` in the repo — is excluded from hermetic discovery in
`gunbc.ci_layer_roots`.

This is the class in its purest form and is the calibration specimen for this census: not a weak
check, an unfalsifiable one. **Routed to the manager, not to the owning child, per instruction.**

---

## Rank 6 — `tools.ci_gates` and its members have no invocation on either workflow

**Mechanism.** `tools.ci_gates` dispatches `ProseRowIntroductionGate` →
`tools.prose_row_introduction_gate` `run_prose_row_introduction_gate`;
`tools.extdeps_scope_placement_gate` `scope_placement_gate_verdict` and
`manifest_remove_only_verdict` are the same shape.

**Measured.** Neither gate module declares a `test fn` (`grep -c "test fn"` → 0 on both;
control: the same grep over `dag/test/claim/stage0_rust_lifecycle_totality_witness_test.dag`
matches), so neither is reachable through floor discovery. `grep -rn "ci_gates"` over
`.github/` returns zero (control: the same directory matches `cargo build`). So these gates
have no executor at all.

**Why it is on a census of greens.** These are per-PR *merge* gates, described in
`gunbc.extdeps_scope_frontier` `extdeps_scope_frontier_law` in the present tense as "per-PR
MERGE-GATED", with a declared coverage-narrowing note (`scope_placement_base_split_note`) that
reasons carefully about *which base* they read. All of that prose describes a mechanism that no
longer runs. A witnesses.yml green is read as "the merge gates passed"; these did not run.

**Is it stated?** DESIGN's unguarded list names seven things and does not name either of these.

**Rank.** Below the four above because the misreading is confined to readers of those two
carriers, and because it is a floor-cut consequence rather than a scope statement that is wrong
on its own terms.

---

## Declared, and therefore not findings — recorded so the next census does not re-open them

Per the instruction that a declared gap is not a finding, these were examined and are honest:

- **`executed` in the floor receipt.** DESIGN states plainly that it counts a witness *reaching*
  the fold, not its assertion running, and that a witness hermetically refused at
  `shell.Test.IsExecutable` is counted `executed` — with the reason (mocking the refusal would
  pass the witness against a fabricated exit status).
- **`[floor-disposition]`, `offered/routed/declined_*`, `[floor-route-gap]`, and the
  subject-before-the-gate refusal in `prepare_repository_from_source_roots`.** All state their
  populations. The only complaint is row 2's: which *surface* carries them.
- **`ClaimOutcome::HostEffectRefused` handling in `cli_run.rs`.** A witness that never reached its
  subject goes to `route_gap`, a separate blocking collection — explicitly *not* `passed` and
  explicitly *not* `failures`, with the reasoning inline. The expected-red roster and the
  route-gap roster are joined in both directions, so a repaired route cannot leave a stale row
  counting a debt that was paid.
- **`gunbc.ci_layer_roots` `bin_witness_wet_entries`.** Measured **94** `bin_wet(` rows
  (`grep -c "^  bin_wet("`; control: `bin_wet_NOPE(` → 0). Their executing consumer was the wet
  job in the deleted `ci.yml`. `gunbc.witness_floor_workflow` declares this in its own prose:
  *"`bin_witness_wet_entries` was the declared per-PR wet batch for this whole class, and it died
  with the old floor. The class has had no executor since."* Declared — **except** for the
  residue below, which is not.
- **`v1_src_dag_parse`.** States its subject (`src/v1` `.dag` files), prints its count
  (`N file(s) parse-clean`), refuses on an empty walk rather than passing, and its in-file
  comment names exactly what it claims and does not claim: *"NOTHING about resolution, types, or
  whether those modules mean anything."* This is the model the other four rows are measured
  against.
- **`fleet-converge.yml`'s sccache arm.** Prints `CiSccacheProviderReceipt: provider=SKIPPED …
  the release build runs UNCACHED — a counted degradation, not a supported mode`.

## One undeclared residue inside a declared gap

Several hermetic-discovery **exclusion** rows justify themselves in prose by naming
`bin_witness_wet_entries` as their real per-PR executing consumer — for example the row for
`stage0_rust_host_observation_live_witness_test.dag`. The *class's* lack of an executor is
declared (above); the fact that these individual exclusions now rest on a dead lane is not. Each
such row is a green (the witness is absent from the floor and nothing reds) resting on a stated
scope that has ceased to be true. Inference, not measurement: I read the exclusion prose and
confirmed the lane is dead; I did not enumerate every exclusion row that cites it.

## Routing

Nothing here is emission or semantic correctness (`smart-ram-730`), and nothing is convergence
or mirror-vs-authority agreement (`wise-boar-649`). Row 5 belongs to the child repairing
`stage0_rust_lifecycle_totality` and is reported to the manager rather than to them, as
instructed.
