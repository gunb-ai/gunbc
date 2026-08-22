# Corpus-wide unconsumed-module census

**Status: census only. Nothing is deleted by this document.** It exists to be reviewed
before any uprooting, per the dispatch: *"337 is too many to delete on one session's
judgment."*

Operator directive it serves (2026-08-21, verbatim): *"yes please make sure to clean up
anything without consumers that we don't need, or get them actually consumed."* Both arms
are live — a module that **should** be consumed and is not is a missing-consumer defect,
not residue.

## 1. The defensible number

| quantity | value | unit |
| --- | --- | --- |
| `.dag` modules under `dag/` + `src/v2` | 3816 | DistinctModuleCount |
| consumed by discovery, not by import (`/test/`, `*_test.dag`, `/lens/`, `/manual/`, `/fixture*`) | 1928 | DistinctModuleCount |
| additional roots: named by an entry row (argv `--entry` or an `*entry*:` path field) | 48 | DistinctModuleCount |
| additional roots: carrying a v1 seed mirror in `src/v1/stage0/src/<module_with_underscores>.rs` | 79 | DistinctModuleCount |
| reachable from those roots through imports **and qualified calls** | 3518 | DistinctModuleCount |
| **unreachable — the census population** | **298** | DistinctModuleCount |

Every number in this document carries its unit, and two units are deliberately never
interchangeable: a **DistinctFileCount** (how many files name a thing) and a
**SourceOccurrenceCount** (how many times it is named). Reporting one as the other is what
produced the false escalation in §4b, and the discipline is cheap: name the unit.

**298, not 337.** The inherited figure was an upper bound over a different question
(*zero importers*), and it was correct to label it that way. Two independent corrections
move it, in opposite directions, and they do not cancel:

- *Zero-importer is too narrow.* It cannot see an **island** — a cluster whose members
  import each other and nothing outside imports any of them. `extdeps/colo/` is exactly
  that: 18 of its 19 modules have zero importers, and the nineteenth (`extdeps.colo.types`)
  has eighteen — all from inside the island. Reachability from roots catches all 19.
- *Zero-importer is also too wide.* See the instrument defect in §2.

Re-derived here from scratch; the inherited number was not reused for anything.

## 2. Instrument: the universe it decodes, and the five defects found in building it

Attribution matters here, because a lesson that reads as one session's mistakes gets
discounted. **Defects 1 and 2 were the dispatching lane's, self-reported** in the brief
that commissioned this census. **Defect 3 was in both instruments** — theirs and this
one's first pass. **Defects 4 and 5 were this census's own**, and neither was caught by
its author: 4 by review of the first revision, after it had already produced a false
escalation (§4b), and 5 by an architecture ruling that asked which call surfaces the
number covered. Two of five were found by the measurer; three were found by someone
reading the measurement.

1. **Module name is not derivable from the path.** Names come from the declared `module`
   line. All 3816 files carry one (checked; zero missing).
2. **Test modules are consumed by discovery, not by imports.** Floor discovery is by
   **file suffix**, not module prefix — `cli_run.rs` `floor_discovery` selects
   `rel.ends_with("_test.dag")`. So the exclusion is by path/filename, and a module named
   `v2.workflow.ci_materialization_emit_test` living in `ci_materialization_emit_test.dag`
   *is* discovered even though its name is not under a `test.` namespace.
3. **NEW — qualified calls are consumption without an import.** `.dag` admits a
   fully-qualified call with no import statement:
   `dag/test/claim/accelerator_demo_gpu_witness_test.dag` calls
   `gunbc.accelerator_demo_gpu.witness_m5_gpu_execution_lane_count_grounded()` and never
   imports the module. An import-line census scores that module dead. Adding
   qualified-reference edges (over string- and annotation-stripped source, so prose
   mentions do not create false edges) moved the population 333 → 303 and returned the
   whole `gunbc.accelerator_demo_*` family to *reachable*. **Any earlier orphan number in
   this repo built from import lines alone over-reports by roughly 10%.**

**No instrument is committed with this document, deliberately.** A hand-authored census
script beside a substrate that already reads the module graph is the §6 manual-application
tell — the durable form is a lens over the same `Node` tree, and that is a separate,
larger piece of work than this census. The method above is stated at the grain needed to
re-derive the population independently, and §2's three defects are what a re-derivation
must reproduce to agree.

4. **NEW — a bare substring is not a reference, and one module name is a suffix of
   another.** `v2.std.verification` *contains* `std.verification`. A substring mention
   scan therefore reported `std.verification` as cited by 128 files; boundary-anchored
   (not preceded by `[A-Za-z0-9_.]`, not followed by an identifier character) the true
   count is **6 occurrences, one of which is the module's own `module` line**. This is the
   same class as defect 3 from the other side — an instrument reading text where it should
   read structure — and it was caught by review, not by me. Every mention count in §3 and
   §6 is boundary-anchored; the census document itself is excluded from its own scan, which
   is not pedantry: including it silently reclassified 40 rows out of RESIDUE-UNMENTIONED
   by naming them. **Rule for whoever measures next: never a bare substring for a module
   name.** Re-measuring the whole population this way moved only 4 rows, so the class was
   narrow — but it was fatal on exactly the row the census had escalated.

5. **NEW — an entry row is not always spelled `--entry`.** The first revision decoded argv
   `--entry` and a narrow `entry_file` form. It did not decode the `entry: "dag/…"` /
   `<name>_entry_file: String = "…"` field rows that `gunbc.ci_spec` and the emitted
   workflows actually use — 764 SourceOccurrenceCount of that shape against 116 of the
   argv shape, so the undecoded surface was the *larger* one. Decoding it moved the
   population 303 → 298 and returned three modules to consumed, two of which this document
   had classified FROZEN-PENDING-RE-ADD on the strength of a re-add anchor:
   `tools.floor_effect_gate_witness` and `tools.ci_heal_dispatch` are not frozen, they are
   **invoked**. A carve-out argued from a named anchor was still wrong, because the
   instrument had not looked at the surface that would have settled it.

6. **NEW — whole-pool unqualified resolution is consumption with neither an import nor a
   qualified name.** v2 resolves a bare symbol against the **whole module pool**, not
   against a containment scope, so a consumer may use a declaration while naming neither
   the declaring module nor its file. `src/v2/lens/extdeps_shape_transport_policy/`
   `module_refs.dag` is the worked case: **26 consumer files, all of them floor-discovered
   `*_test.dag`, and all 26 name neither `v2.lens.extdeps_shape_transport_policy.module_refs`
   nor its path** — they write `extdeps_cargo_build_module` bare and the resolver finds it.
   All 9 of the module's declarations are consumed this way. **Scope of that 26, stated
   because a narrower probe gets a smaller number and the two must not be reconciled by
   splitting the difference:** it counts consumers of *any* of the module's 9 declarations.
   A probe of the single symbol `extdeps_cargo_build_module` returns **13 consumer files,
   none of which names the module or the path**. Both figures are correct at their own
   scope; the class is identical either way. *One caveat, because it recurred here:* run
   that probe after this document names the module and it returns 14 with 1 naming it —
   the fourteenth is **this file**. §2's defect 4 already set the rule (the census is
   excluded from its own scan); it is repeated at the receipt because the contamination
   arrives the moment the finding is written down, not when it is measured. This class is invisible to a
   module-name surface and a file-path surface **by construction**, not by an oversight in
   either: neither string occurs in any consumer. Defect 3 is its near neighbour and does
   not cover it — a qualified call at least spells the module name, which is exactly what
   makes defect 3 detectable by a name scan and this one not.

   *Direction, and what is NOT claimed.* Like every surface in §2 this one can only move
   modules **out** of the population. It was measured on one module, reached by re-deriving
   this census independently; **the whole 298 has not been re-scored against it**, so how
   many rows it moves is unmeasured and no estimate is offered — §6's `LowerBoundOnly`
   standing already covers exactly this and is the reason it does not need revising here.
   **Rule for whoever measures next, beside the defect-4 rule:** a bare declaration name is
   a reference too, and in `src/v2/lens/` it is the *normal* one. Restrict such a scan to
   declaration names that are **unique corpus-wide**, or it reports collisions on common
   words (`Stage`, `Review`, `Permission`) as consumption — measured, and the reason this
   receipt counts unique-owner declarations only.

**The universe the 298 is over.** Call surfaces decoded: `import` lines; fully-qualified
`module.symbol` references (string- and annotation-stripped); argv `--entry` path literals;
`*entry*:` path-field rows in `.dag`, `.yml`, `.rs`, `.md`, and shell; v1 seed mirrors by
filename. Surfaces **not** decoded, each of which can only make the population smaller:

- **Dynamically composed argv.** `gunbc.roadmap_serve` is invoked with
  `--function ", svc.serve_function` — the function is a field read, not a literal. Any
  entry whose *path* is likewise composed is invisible to this instrument.
- **Host-side invocation in Rust that names neither the path nor the module.**
- **An entry row declared inside a module that is itself unreachable** — admitted here as a
  root, which over-admits consumption. `v2.workflow.product_receipt_stage` is rooted this
  way by a dead declarer.

**Controls, run on every pass** (a zero is readable only beside a nonzero):
`v2.compiler.compile`, `gunbc.spark.serving_desired`, `gunbc.clock_read`, `v2.std.node`
all score *reachable*; `gunbc.accelerator_demo_gpu` scores *reachable* only after defect 3
was fixed, and it is retained as the standing discriminating control for that arm.

**Known limits of this instrument, stated rather than left to be found.** It cannot see a
module invoked by a path assembled at runtime, and it treats an `--entry` argv literal
anywhere in the tree — including in a doc — as a root, which is deliberately generous:
this census over-admits consumption, so the population is a floor, and every row still
needs the mention check in §3 before deletion.

## 3. Dispositions

Assigned mechanically from evidence, then read. The rule for each class is stated so the
row can be re-derived and disagreed with.

| disposition | count | rule | what it means |
| --- | --- | --- | --- |
| RESIDUE-EMPTY | 8 | ≤5 lines — a `module` line and nothing else | Delete. No content to strand. |
| CITED-AUTHORITY | 103 | declares an `ExternalAuthority` anchor | **Do not sweep.** The value may be the citation (DESIGN §3 extdeps duty), not a call. Needs a per-island decision, §4. |
| PROSE-NAMED | 79 | named by live (reachable) `.dag`, `.rs`, or `.yml` source, boundary-anchored | Deleting strands a citation. Each needs the mention read before it moves: superseded, missing-consumer, or delete-with-citation-repair. |
| FROZEN-PENDING-RE-ADD | 13 | the `.dag` side of a capability whose invoker a cut removed, where the re-add is named | **Not residue.** DESIGN §3 frozen-X: deleting these deletes what the re-add queue exists to re-attach. Each row names its anchor, §4d. |
| RESIDUE-DOC-ONLY | 28 | named only in `.md`, receipts/TSVs, or other dead modules | Delete; repair the doc citation in the same diff. |
| RESIDUE-UNMENTIONED | 67 | not named anywhere in the tree outside itself | Delete. Highest-confidence residue. |

Per-module rows: appendix, §6.

**ENTRY-INVOKED is zero here by construction.** The brief's first job was to build the
entry index and subtract; it is built (48 entry-row roots + 79 seed mirrors) and
subtracted *before* the population is formed, so every entry-invoked module is already
outside the 298. The first revision's index was smaller (34 roots) because it decoded only
the argv spelling; §2's defect 5 is what that cost, and it is the reason this section now
reports the index by *surface* rather than as one number.

## 4. Named findings

These are the results worth a decision, as opposed to a row.

**a. `extdeps/colo/` — a 19-module island, zero consumers.** Real colocation vendors
(Equinix, CoreSite, Iron Mountain, QTS, …) with cited authority anchors, importing
`extdeps.colo.types` and each other, consumed by nothing. This is one decision, not
nineteen: does a siting consumer exist or is it planned? If not, the island is the largest
single deletion in the census. Same shape, smaller: `extdeps/formats/elf/` (7),
`extdeps/container/oci/` (5), `extdeps/boot/` (5), `extdeps/ebay/` (6),
`extdeps/tcgplayer/` (5), `extdeps/llm/` (7).

**b. `std.verification` — delete it; the escalation this census originally raised was an
instrument artifact.** The first revision reported 120 citing files and asked for an
operator decision. That count was a substring match against `v2.std.verification` (§2,
defect 4). Boundary-anchored and verified independently of the review that caught it, the
real citation surface is small and the deletion is ordinary:

- **5 genuine name references, in 2 files**, both plan carriers:
  `gunbc.plans.resolver_type_name_collision_wall` and
  `gunbc.plans.realization_measurement_loop`.
- **1 path reference the name-based scan could not see** — `src/v2/test/fixture/`
  `frontier_probe_elision_boundary_overlay.dag` pins `"dag/std/verification.dag"` by path
  **with a content hash**. A fixture pinned by content hash is a consumer: the deletion
  must move it, not just the prose. This site was not in the review's count either; a
  name-only census would have found it during the deletion instead of before it.

The second-authority half is measured, not inferred: `dag/std/verification.dag` is 604
bytes with 0 importers (`AssertKind`, `AssertionClaim`, `TestCase`); `src/v2/std/
verification.dag` is 13,813 bytes with 118 importers (`TestgenTier`,
`TestClassification`, …). Same name, same subject space, and the small one has no
consumers — the §3 second-authority shape. Delete the 604-byte module and repair all three
sites in the same PR: if a plan's claim is still true it points at `v2.std.verification`,
and if it is not, the claim retires with the module. A pointer to a deleted authority is
not an acceptable landing state.

**c. `src/v2/extdeps/formatters/` (9) and `typecheckers/` (2) — cited config models, no
consumer, no mention anywhere.** rustfmt, prettier, gofmt, black, ktfmt, clang-format,
swift-format, google-java-format, lean4-format, mypy, pyright. Modeled upstream config
surfaces with zero readers. Note the standing irony: DESIGN's fixed-point rule for the
emitted mirror is *about* rustfmt, and `v2.extdeps.formatters.rustfmt` is not what
implements it. Missing-consumer or residue — not a mechanical call.

**d. `dag/tools/` — 13 FROZEN, 5 residue, 2 that turned out to be invoked, and the split is per-module.** These are the
`.dag` sides of capabilities whose invokers the floor cut (2026-08-15) and the regen root
cut (2026-08-18) removed. DESIGN's CI paragraph names a **re-add queue** with a restoration
trigger, and a module the queue exists to re-attach is §3 frozen-X, not residue — deleting
it means re-authoring it worse from memory later. So the carve-out is granted **per module
against a named anchor**, never to the group:

- **Eight witness transports** — `parse`, `bootstrap`, `dag_collect_fingerprint`,
  `infer_semantics`, `interp_recorded_fixture`, `effects_rest_transport`,
  `auth_declared_but_unwired`, `v1_dag_parse`. Anchor: each wraps a Rust binary that
  **`gunbc.ci_release_bins` (live, reachable) still declares as a CI release artifact**.
  The binary is retained and the `.dag` invoker is the unattached half — the definition of
  frozen.
- **`tools.merge_admission_capture_transport`** and **`tools.merge_admission_current_context`**
  (merge-admission stamping). Anchor: the gate is named on DESIGN's own unguarded list.
- **NOT frozen after all: `tools.floor_effect_gate_witness` and `tools.ci_heal_dispatch`.**
  The first revision froze them on the unguarded-list anchor. Decoding the `entry:` field
  surface (§2, defect 5) shows both are named by live entry rows — they are **invoked**,
  and were never in the population. Recorded rather than quietly dropped: an anchor can be
  real and the conclusion still wrong, when the instrument has not read the surface that
  settles it.
- **`tools.dag_compile_clean_seam`, `_seam_transport`, `_shard_transport`.** Anchor:
  DESIGN's compile-clean entry-point trigger — **the weakest of the three anchors, and
  flagged as such**: prose, no binary, no queue line. If that trigger is judged not to be a
  queue entry, these three separate out as residue.

**Not on any queue, therefore residue:** `tools.build`, `tools.readme`,
`tools.roadmap_dispatch`, `tools.codegen` (empty), and `tools.gunbc_ci` — the last is
**superseded**, not merely unreferenced: it shells out to the old `gunbc ci` generate-and-run
script, and `gunbc.witness_floor_workflow` → `.github/workflows/witnesses.yml` is what
actually runs CI now. That is the §3 second-authority find in this cluster.

Related, and the reason this cluster is worth naming at all: **12 of the 298 declare
`main()`** — an entry shape with no argv anywhere that names it. Four are in `tools/`; the
rest are `examples/` and three `extdeps` witnesses. An entry that lost its invoker is the
exact residue a delete-first cut is supposed to surface loudly, and it did not surface,
because nothing downstream could refuse.

**e. Eight empty modules.** `std.list`, `std.containers`, `std.import`, `std.rational`,
`tools.codegen`, `v2.bin.main`, `v2.std.inhabitant_bridge`,
`v2.std.type_expr_projection_row_schema` — a `module` line and whitespace. `v2.bin.main`
is the one to look at twice: an empty `main` is a name reserving a seat.

**f. `src/v2/extdeps/language_model/` — 15 unmentioned rung modules** (`*_r2a`, `*_r2b`,
`*_r3_external` across Go/Python/Rust/TypeScript) beside `*_r1_test.dag` siblings that
*are* discovered. The `_test`-suffixed rungs run; their non-suffixed peers do not, and
nothing names them. Likely a ladder that stopped being climbed — but "the rung above the
one we execute" is a claim about intent, so it is flagged, not classified.

**g. Three modules that landed AFTER this census and are unconsumed — none of them
residue.** Surfaced by a second census run independently on 2026-08-22 (population: every
`.dag` module scored zero on module name *and* file path; the 14 it returned were 11
instrument artifacts of defects 3 and 6 plus these three). All three postdate #8803, which
is why §6's appendix does not carry them. Dispositions, one per module:

*Why a naive re-run disagrees with the 298, recorded so the next reader does not
re-derive it.* That second run scored 14 modules unreachable; **none of the 14 are in this
document's 298**, and there is no contradiction to resolve. Eleven were reachable all
along, through surfaces this document already names: ten by fully-qualified reference
(§2, defect 3 — the run required an *exact* match against the module name, so
`extdeps.bmc.access.redfish_rbac_policy` matched nothing because the reference string is
*longer* than the module name), and one, `module_refs`, by whole-pool resolution (§2,
defect 6 — the surface that run contributed). The `gunbc.accelerator_demo_*` family it
returned as a four-module cluster is precisely the family §2's defect 3 records as having
moved back to *reachable*, and `gunbc.accelerator_demo_gpu` is retained in §2's controls
as the standing discriminator for that arm. The remaining three are the rows below, which
postdate this document. **The general shape, which is the part worth keeping:** an
import-line or exact-name census over this corpus does not merely under-count, it reports
a **structural zero that is indistinguishable from a true zero** — deleting `module_refs`
on such a reading would have taken 26 live floor-discovered witnesses with it while the
census showed nothing. Before building a census instrument, search `docs/plans` for the
census.

- **`gunbc.empty_decl_file_checkpoint_bypass`** (`183e50a3469`) and
  **`gunbc.generic_binder_field_projection_deficit`** (`aecb1fed927`) — **KEEP, and being
  unconsumed is their correct state, not a defect.** Both are DESIGN §4b error-class
  filings: declared rung found-at, ceiling with reason, next-rung trigger, dissolution
  condition. §4b(2) *requires* a class below its ceiling to carry that row, and nothing in
  §4b makes a code consumer part of the requirement — the row is the filing. Deleting them
  deletes the safety ledger, and a future census that scores them residue is re-deriving a
  question this row answers. **The ending event is each carrier's own stated dissolution
  condition, and both are bounded events rather than an unbounded "later":** for the first,
  `lookup_checkpoint` refusing on an empty `decl_file` with every production call site
  threading identity; for the second, v2 inference resolving a generic coproduct's type
  argument into the arm binder. Note for a reader who checks whether these should carry
  witnesses: `generic_binder_field_projection_deficit` states in its own header why it does
  not, and the reason is §5 construction-over-validation — its discriminating RED was
  *refused at resolve* because the invalid state has no constructor, so the predicate and
  its witness were deleted rather than kept as a green that cannot go red.

- **`gunbc.scm.commit_closure_store`** (#8807) — **the one real finding in the batch: a
  replacement-migration leftover, not residue and not correctly standing.**
  `gunbc.scm.repository_envelope` (#8820, one day later) is by its own header the layer
  *above* this module's grain, and the store's header anticipates it by name. The envelope
  imports `gunbc.scm.image` and `gunbc.scm.object_store` and **does not import
  `commit_closure_store`** — so the envelope took the grain and left the `Filesystem`
  save/load half unattached, with the store's only tree-wide mention being a prose line in
  `image.dag`. This is DESIGN §3's replacement-migration shape caught mid-cutover: X still
  standing after Y claimed its root. **Not dispositioned here**, because the two arms —
  the envelope grows save/load and this module deletes, versus this module is the
  persistence layer the envelope should consume — differ on the #8820 author's intent, and
  guessing is how a second authority gets ratified. Raised for that author.

**A hazard for whoever measures next, cheap to hit and silent.** A fresh worktree of this
repo can be **shallow-grafted** — the 2026-08-22 run found its clone rooted at a single
4608-file import commit dated six days earlier, so `git log <file>` reported that graft
commit as the first commit of every older module. Any census arm that asks *was this
consumed until a recent cut* (the question that separates a severed consumer from §6
experimental residue) silently answers from a truncated history. `git rev-parse
--is-shallow-repository` before trusting a history claim; `git fetch --unshallow` fixes it.

## 5. Proposed sequencing (for approval, not execution)

One red in a 298-file deletion blocks everything, so: small coherent batches, each its own
PR, each verified by `claim_executor --required-ci --source-root dag --source-root src/v2`
with `failed=` read and PASS counted against the roster.

1. **B1 — RESIDUE-EMPTY (8).** Nothing to strand; a pure control batch that proves the
   deletion pipeline and the floor both behave.
2. **B2 — RESIDUE-UNMENTIONED, non-extdeps (53 of 67).** No citation to repair by
   construction. This is the batch to establish the mechanics on: lowest chance of an
   argument, per the review ruling.
3. **B3 — RESIDUE-DOC-ONLY (28).** Deletion plus the doc/receipt citation repair in the
   same diff.
4. **B4 — the five `tools/` rows that are NOT on the re-add queue (finding d)**, including
   the superseded `tools.gunbc_ci`. The 13 FROZEN-PENDING-RE-ADD rows are in no batch: they
   stay until their queued gate is re-derived, and their disposition is recorded here so a
   future census does not re-derive the question and answer "residue".
5. **B5 — `std.verification` (finding b)** alone: delete the module, repoint the two plan
   carriers, and move the content-hash fixture pin, all in one PR.
6. **B6+ — the extdeps islands (finding a, c)**, one island per PR, each gated on whether
   the citation is the deliverable.

PROSE-NAMED (79) does not get a batch until each row's mention has been read; several will
resolve to *missing consumer* and be wired up rather than deleted, which is the directive's
second arm.

**Excluded from every batch:** `dag/gunbc/spark/` — `fierce-lynx-647` owns that area and is
mid-census there. Two of the 298 (`gunbc.spark.provisioning`,
`gunbc.spark.managed_access_apply`) fall in it and are reported here for their benefit
only. `gunbc.spark.provisioning` is a live instance of the dangling-annotation hazard:
`extdeps/systems/nvidia_dgx_spark_setup.dag` names it as a fact's home.


## 6. Completeness standing

**`LowerBoundOnly`.** 298 resolved-unconsumed modules over the universe declared in §2.
Not `CompleteForDeclaredUniverse`, and the difference is not modesty: three call surfaces
are named as undecoded in §2, and every one of them can only *remove* modules from the
population, never add. Two prior revisions of this document each lost modules to a newly
decoded surface (333 → 303 → 298), which is the empirical case for the standing rather
than an argument for it.

What the instrument can support: *298 resolved consumers-of-none over universe U;
standing LowerBoundOnly; blind spots as named.* What it cannot support, and what this
document does not claim anywhere: *there are exactly 298 unconsumed modules.* The second
sentence dies the moment someone decodes a fourth surface; the first survives it, and
tells the reader what would change it.

This standing is why §5's batches are ordered as they are. RESIDUE-UNMENTIONED first is
not merely the lowest-argument batch — it is the class least exposed to the blind spots,
because a module named nowhere in any surface is not waiting on a surface to be decoded.

### Two broken entry strings, confirmed

Decoding the entry-row surface also surfaced consumption that is *declared and broken* —
neither consumed nor residue, and invisible to the import graph. Confirmed by reading both
sites, not by the count:

- **`dag/gunbc/spark/grant_install.dag`** — `gunbc.ci_spec` `gunbc_ci_spark_grant_install_invoke`
  names it as an entry with function `spark_grant_install_ci_wet`. **The file does not
  exist.**
- **`dag/gunbc/spark/serving_durability.dag`** with function
  `"spark_serving_durability_ci_wet"` — named by `gunbc.ci_spec` *and* by
  `.github/workflows/fleet-converge.yml`. The file exists; **that function is not in it**
  (its `fn`s are `spark_serving_boot_id_probe_argv`, `..._from_probe`,
  `spark_serving_reboot_transition`, `spark_serving_durability_verdict`, `..._is_proven`,
  `..._wire`).

Both are in `dag/gunbc/spark/`, which is `fierce-lynx-647`'s area — reported, not touched.
A sibling lane found the same two independently; this is a second instrument agreeing, not
a second report of one measurement.

**Not defects, and named so a future sweep does not "fix" them:** the raw scan also flags
`dag/a.dag`, `dag/mini.dag`, `dag/gunbc/live_deploy/WRONG_ENTRY.dag`,
`dag/test/claim/__no_such_entry_zzz.dag` and similar. Those are *deliberate negative
controls* inside witness tests — an entry that must fail to resolve. An instrument that
reports them is producing false positives, which is exactly what §7's last case exists to
catch.

## 7. Calibration benchmark for the next audit instrument

Today's failures, written as cases rather than as prose, so a future instrument calibrates
against a suite instead of rediscovering all five. Each case names a real subject in this
tree, the wrong answer, and the answer that is honest.

| # | case | subject | wrong answer | required answer |
| --- | --- | --- | --- | --- |
| 1 | **cardinality** | `std.verification` | one number for "how cited" | **both** units: SourceOccurrenceCount 6 **and** DistinctFileCount 3 — they are different questions |
| 2 | **exact identity** | `std.verification` vs `v2.std.verification` | substring match folds them | boundary-anchored: the two stay separate, and the suffix relation never merges them |
| 3 | **representation vs subject** | any modeled capability | "spelling absent ⟹ capability absent" | a typed model present with zero occurrences of its shell spelling means capability PRESENT, spelling ABSENT — **no absence conclusion** |
| 4 | **qualified use** | `gunbc.accelerator_demo_gpu` ← `dag/test/claim/accelerator_demo_gpu_witness_test.dag` | orphan (no import line) | caller edge PRESENT; an import-line census must declare itself incomplete |
| 5 | **string-bound entry** | `dag/gunbc/spark/serving_durability.dag` + `spark_serving_durability_ci_wet` | consumed (path resolves) | **resolution REFUSED** — path present, function absent; neither consumed nor residue |
| 6 | **positive no-finding** | `dag/gunbc/live_deploy/WRONG_ENTRY.dag`, `dag/a.dag`, `__no_such_entry_zzz.dag`; and any healthy imported module | flagged as broken/orphaned | **no finding** — deliberate negative controls and ordinary live modules must come back clean |

Case 6 is the one most likely to be skipped and the one that catches a broken audit: a
suite built only from defects is passed by an instrument that reports *everything* as
suspicious. Cases 1, 2, 4, 5 each have a live specimen in this tree, so the suite is
executable against the real corpus rather than against a fixture someone has to maintain.

## 8. Appendix — the 298 rows

### RESIDUE-EMPTY — 8 modules

| module | path | lines | live-source mentions |
| --- | --- | --- | --- |
| `std.containers` | `dag/std/containers.dag` | 2 | —  |
| `std.import` | `dag/std/import.dag` | 2 | {'dag': 1} `dag/gunbc/commit_workflow.dag` |
| `std.list` | `dag/std/list.dag` | 3 | —  |
| `std.rational` | `dag/std/rational.dag` | 5 | {'dag': 2} `dag/gunbc/doc_graph_roots.dag` `dag/gunbc/econ/acquisition.dag` |
| `tools.codegen` | `dag/tools/codegen.dag` | 5 | —  |
| `v2.bin.main` | `src/v2/bin/main.dag` | 4 | —  |
| `v2.std.inhabitant_bridge` | `src/v2/std/inhabitant_bridge.dag` | 4 | —  |
| `v2.std.type_expr_projection_row_schema` | `src/v2/std/type_expr_projection_row_schema.dag` | 4 | —  |

### RESIDUE-UNMENTIONED — 67 modules

| module | path | lines | live-source mentions |
| --- | --- | --- | --- |
| `config.codegen_paths` | `dag/config/codegen_paths.dag` | 20 | —  |
| `examples.html_markup_smoke` | `dag/examples/html_markup_smoke/html_markup_smoke.dag` | 66 | —  |
| `examples.js_site` | `dag/examples/js_site/js_site.dag` | 185 | —  |
| `examples.js_site_emit` | `dag/examples/js_site/js_site_emit.dag` | 76 | —  |
| `examples.nominal_distinctness_twin` | `dag/examples/nominal_distinctness_witness/twin.dag` | 15 | —  |
| `examples.nominal_distinctness_witness` | `dag/examples/nominal_distinctness_witness/witness.dag` | 19 | —  |
| `gunbc.assimilate.bmc_wif_canary_bootstrap` | `dag/gunbc/assimilate/bmc_wif_canary_bootstrap.dag` | 126 | —  |
| `gunbc.ci_oom_reclassify` | `dag/gunbc/ci_oom_reclassify.dag` | 87 | —  |
| `gunbc.cursor_sdk_secure_api_key` | `dag/gunbc/cursor_sdk_secure_api_key.dag` | 73 | —  |
| `gunbc.devboot.vertical_receipt` | `dag/gunbc/devboot/vertical_receipt.dag` | 68 | —  |
| `gunbc.host_converge_delta` | `dag/gunbc/host_converge_delta.dag` | 119 | —  |
| `gunbc.install_media` | `dag/gunbc/install_media.dag` | 43 | —  |
| `gunbc.parse_allowlist` | `dag/gunbc/parse_allowlist.dag` | 20 | —  |
| `gunbc.plans.wave2_prep_design` | `dag/gunbc/plans/wave2_prep_design.dag` | 205 | —  |
| `gunbc.provider_standing_live_probes` | `dag/gunbc/provider_standing_live_probes.dag` | 194 | —  |
| `gunbc.srv4_seeded_install_media_artifact` | `dag/gunbc/srv4_seeded_install_media_artifact.dag` | 47 | —  |
| `gunbc.tools.bmc_onboard_validate` | `dag/gunbc/tools/bmc_onboard_validate.dag` | 22 | —  |
| `gunbc.tools.ebay_listing` | `dag/gunbc/tools/ebay_listing.dag` | 143 | —  |
| `gunbc.tools.roadmap_spawn_request` | `dag/gunbc/tools/roadmap_spawn_request.dag` | 36 | —  |
| `std.binding` | `dag/std/binding.dag` | 7 | —  |
| `std.syllogism` | `dag/std/syllogism.dag` | 82 | —  |
| `tools.build` | `dag/tools/build.dag` | 34 | —  |
| `tools.readme` | `dag/tools/readme.dag` | 70 | —  |
| `tools.roadmap_dispatch` | `dag/tools/roadmap_dispatch.dag` | 17 | —  |
| `v2.extdeps.formats.csv` | `src/v2/extdeps/formats/csv.dag` | 136 | —  |
| `v2.extdeps.formats.openapi` | `src/v2/extdeps/formats/openapi.dag` | 459 | —  |
| `v2.extdeps.formats.toml` | `src/v2/extdeps/formats/toml.dag` | 132 | —  |
| `v2.extdeps.formatters.black` | `src/v2/extdeps/formatters/black.dag` | 87 | —  |
| `v2.extdeps.formatters.clang_format` | `src/v2/extdeps/formatters/clang_format.dag` | 893 | —  |
| `v2.extdeps.formatters.gofmt` | `src/v2/extdeps/formatters/gofmt.dag` | 14 | —  |
| `v2.extdeps.formatters.google_java_format` | `src/v2/extdeps/formatters/google_java_format.dag` | 22 | —  |
| `v2.extdeps.formatters.ktfmt` | `src/v2/extdeps/formatters/ktfmt.dag` | 81 | —  |
| `v2.extdeps.formatters.lean4_format` | `src/v2/extdeps/formatters/lean4_format.dag` | 189 | —  |
| `v2.extdeps.formatters.prettier` | `src/v2/extdeps/formatters/prettier.dag` | 191 | —  |
| `v2.extdeps.formatters.rustfmt` | `src/v2/extdeps/formatters/rustfmt.dag` | 286 | —  |
| `v2.extdeps.formatters.swift_format` | `src/v2/extdeps/formatters/swift_format.dag` | 110 | —  |
| `v2.extdeps.typecheckers.mypy` | `src/v2/extdeps/typecheckers/mypy.dag` | 29 | —  |
| `v2.extdeps.typecheckers.pyright` | `src/v2/extdeps/typecheckers/pyright.dag` | 47 | —  |
| `v2.std.generic_instantiation` | `src/v2/std/generic_instantiation.dag` | 36 | —  |
| `v2.std.projection` | `src/v2/std/projection.dag` | 19 | —  |
| `v2.test.algebra_laws.is_prefix_of_prefix_check` | `src/v2/std/algebra_laws/is_prefix_of_prefix_check.dag` | 90 | —  |
| `v2.test.algebra_laws.zip_eq_list_equality` | `src/v2/std/algebra_laws/zip_eq_list_equality.dag` | 93 | —  |
| `v2.test.language_model.go_r1` | `src/v2/extdeps/language_model/go_r1.dag` | 59 | —  |
| `v2.test.language_model.go_r2a` | `src/v2/extdeps/language_model/go_r2a.dag` | 59 | —  |
| `v2.test.language_model.go_r2b` | `src/v2/extdeps/language_model/go_r2b.dag` | 59 | —  |
| `v2.test.language_model.go_r3_external` | `src/v2/extdeps/language_model/go_r3_external.dag` | 59 | —  |
| `v2.test.language_model.python_cross_runtime_drift` | `src/v2/extdeps/language_model/python_cross_runtime_drift.dag` | 49 | —  |
| `v2.test.language_model.python_l2_cross_target_parity` | `src/v2/extdeps/language_model/python_l2_cross_target_parity.dag` | 81 | —  |
| `v2.test.language_model.python_r2a` | `src/v2/extdeps/language_model/python_r2a.dag` | 58 | —  |
| `v2.test.language_model.python_r2b` | `src/v2/extdeps/language_model/python_r2b.dag` | 53 | —  |
| `v2.test.language_model.python_r3_external` | `src/v2/extdeps/language_model/python_r3_external.dag` | 58 | —  |
| `v2.test.language_model.rust` | `src/v2/extdeps/language_model/rust.dag` | 256 | —  |
| `v2.test.language_model.rust_r2a` | `src/v2/extdeps/language_model/rust_r2a.dag` | 57 | —  |
| `v2.test.language_model.rust_r2b` | `src/v2/extdeps/language_model/rust_r2b.dag` | 75 | —  |
| `v2.test.language_model.rust_r3_external` | `src/v2/extdeps/language_model/rust_r3_external.dag` | 57 | —  |
| `v2.test.language_model.typescript_r2a` | `src/v2/extdeps/language_model/typescript_r2a.dag` | 58 | —  |
| `v2.test.language_model.typescript_r2b` | `src/v2/extdeps/language_model/typescript_r2b.dag` | 54 | —  |
| `v2.test.language_model.typescript_r3_external` | `src/v2/extdeps/language_model/typescript_r3_external.dag` | 58 | —  |
| `v2.test.nat_semiring.rung_0_to_2_three_targets` | `src/v2/std/nat_semiring/rung_0_to_2_three_targets.dag` | 95 | —  |
| `v2.test.nat_semiring.rung_l1_go_compiler_slice` | `src/v2/std/nat_semiring/rung_l1_go_compiler_slice.dag` | 58 | —  |
| `v2.test.nat_semiring.rung_l1_python_runtime` | `src/v2/std/nat_semiring/rung_l1_python_runtime.dag` | 96 | —  |
| `v2.test.qualified_name.from_node` | `src/v2/std/qualified_name/from_node.dag` | 222 | —  |
| `v2.workflow.bmc_lifecycle_roundtrip` | `src/v2/workflow/bmc_lifecycle_roundtrip.dag` | 38 | —  |
| `v2.workflow.ci_v1_compiler_test_targets_compile_gate_emit` | `src/v2/workflow/ci_v1_compiler_test_targets_compile_gate_emit.dag` | 110 | —  |
| `v2.workflow.floor2_prepared_subject` | `src/v2/workflow/floor2_prepared_subject.dag` | 182 | —  |
| `v2.workflow.gha_expression_fidelity` | `src/v2/workflow/gha_expression_fidelity.dag` | 22 | —  |
| `v2.workflow.probe_selector_host_health` | `src/v2/workflow/probe_selector_host_health.dag` | 43 | —  |

### RESIDUE-DOC-ONLY — 28 modules

| module | path | lines | live-source mentions |
| --- | --- | --- | --- |
| `examples.cost_estimate` | `dag/examples/cost_estimate/cost_estimate.dag` | 29 | —  |
| `examples.gunbhub_serve_program` | `dag/examples/gunbhub_serve_program/gunbhub_serve_program.dag` | 60 | —  |
| `examples.interp_test` | `dag/examples/interp_test/interp_example.dag` | 40 | —  |
| `gunbc.code_change_workflow` | `dag/gunbc/code_change_workflow.dag` | 371 | —  |
| `gunbc.floor_resolve_realization` | `dag/gunbc/floor_resolve_realization.dag` | 28 | —  |
| `gunbc.hand_lens_host_bridge_scaffold_watchdog` | `dag/gunbc/hand_lens_host_bridge_scaffold_watchdog.dag` | 46 | —  |
| `gunbc.host_runner_memory_cap_plan_emit` | `dag/gunbc/host_runner_memory_cap_plan_emit.dag` | 110 | —  |
| `gunbc.hostname_allocation` | `dag/gunbc/hostname_allocation.dag` | 148 | —  |
| `gunbc.language_subject_scope_scaffold` | `dag/gunbc/language_subject_scope_scaffold.dag` | 10 | —  |
| `gunbc.p3a1_self_fork_homonym_disposition` | `dag/gunbc/p3a1_self_fork_homonym_disposition.dag` | 10 | —  |
| `gunbc.pr_digests` | `dag/gunbc/pr_digests.dag` | 72 | —  |
| `gunbc.site.register_principles` | `dag/gunbc/site/register_principles.dag` | 12 | —  |
| `gunbc.spark.managed_access_apply` | `dag/gunbc/spark/managed_access_apply.dag` | 453 | —  |
| `gunbc.tools.card_intake` | `dag/gunbc/tools/card_intake.dag` | 216 | —  |
| `gunbc.tools.cron_tag` | `dag/gunbc/tools/cron_tag.dag` | 67 | —  |
| `gunbc.tools.grounding_confirm` | `dag/gunbc/tools/grounding_confirm.dag` | 115 | —  |
| `gunbc.witness_family_fanout` | `dag/gunbc/witness_family_fanout.dag` | 65 | —  |
| `shared.dag_util` | `dag/shared/dag_util.dag` | 44 | —  |
| `std.exec_format` | `dag/std/exec_format.dag` | 37 | —  |
| `std.patterns` | `dag/std/patterns.dag` | 24 | —  |
| `v2.extdeps.bmc.lifecycle_fidelity` | `src/v2/extdeps/bmc/lifecycle_fidelity.dag` | 146 | —  |
| `v2.extdeps.formats.json` | `src/v2/extdeps/formats/json.dag` | 52 | —  |
| `v2.extdeps.formats.json_schema` | `src/v2/extdeps/formats/json_schema.dag` | 103 | —  |
| `v2.extdeps.formats.yaml` | `src/v2/extdeps/formats/yaml.dag` | 85 | —  |
| `v2.extdeps.github.expression_fidelity` | `src/v2/extdeps/github/expression_fidelity.dag` | 54 | —  |
| `v2.std.rust_leaf_model_claim` | `src/v2/std/rust_leaf_model_claim.dag` | 61 | —  |
| `v2.test.workflow.host_discovered_owned_data_manifest` | `src/v2/workflow/host_discovered_owned_data_manifest.dag` | 19 | —  |
| `v2.workflow.ci_stage0_partition_compile_gate_emit` | `src/v2/workflow/ci_stage0_partition_compile_gate_emit.dag` | 103 | —  |

### FROZEN-PENDING-RE-ADD — 13 modules

| module | path | lines | re-add anchor |
| --- | --- | --- | --- |
| `tools.auth_declared_but_unwired_witness_transport` | `dag/tools/auth_declared_but_unwired_witness_transport.dag` | 12 | `auth_declared_but_unwired_witness` bin, still declared in `gunbc.ci_release_bins` |
| `tools.bootstrap_witness_transport` | `dag/tools/bootstrap_witness_transport.dag` | 12 | `bootstrap_witness` bin, still declared in `gunbc.ci_release_bins` |
| `tools.dag_collect_fingerprint_witness_transport` | `dag/tools/dag_collect_fingerprint_witness_transport.dag` | 12 | `dag_collect_fingerprint_witness` bin, still declared in `gunbc.ci_release_bins` |
| `tools.dag_compile_clean_seam` | `dag/tools/dag_compile_clean_seam.dag` | 110 | compile-clean entry point — WEAKER ANCHOR: a prose restoration trigger in DESIGN, no bin and no queue line |
| `tools.dag_compile_clean_seam_transport` | `dag/tools/dag_compile_clean_seam_transport.dag` | 124 | compile-clean entry point — WEAKER ANCHOR, as above |
| `tools.dag_compile_clean_shard_transport` | `dag/tools/dag_compile_clean_shard_transport.dag` | 43 | compile-clean entry point — WEAKER ANCHOR, as above |
| `tools.effects_rest_transport_witness_transport` | `dag/tools/effects_rest_transport_witness_transport.dag` | 12 | `effects_rest_transport_witness` bin, still declared in `gunbc.ci_release_bins` |
| `tools.infer_semantics_witness_transport` | `dag/tools/infer_semantics_witness_transport.dag` | 12 | `infer_semantics_witness` bin, still declared in `gunbc.ci_release_bins` |
| `tools.interp_recorded_fixture_witness_transport` | `dag/tools/interp_recorded_fixture_witness_transport.dag` | 13 | `interp_recorded_fixture_witness` bin, still declared in `gunbc.ci_release_bins` |
| `tools.merge_admission_capture_transport` | `dag/tools/merge_admission_capture_transport.dag` | 32 | merge-admission stamping — named on DESIGN's unguarded list |
| `tools.merge_admission_current_context` | `dag/tools/merge_admission_current_context.dag` | 134 | merge-admission stamping — named on DESIGN's unguarded list |
| `tools.parse_witness_transport` | `dag/tools/parse_witness_transport.dag` | 20 | `parse_witness` bin, still declared in `gunbc.ci_release_bins` |
| `tools.v1_dag_parse_transport` | `dag/tools/v1_dag_parse_transport.dag` | 12 | `v1_src_dag_parse` bin, still declared in `gunbc.ci_release_bins` |

### PROSE-NAMED — 79 modules

| module | path | lines | live-source mentions |
| --- | --- | --- | --- |
| `direct_rust_door_ingest_fixture` | `src/v2/compiler/self_host/direct_rust_door_ingest_fixture.dag` | 6 | {'dag': 1} `src/v2/compiler/self_host/direct_rust_door_fixture.dag` |
| `examples.weather` | `dag/examples/weather/weather.dag` | 48 | {'rs': 1} `src/v1/stage0/src/bin/bootstrap_witness.rs` |
| `extdeps.bmc.mock_corpus` | `dag/extdeps/bmc/mock_corpus.dag` | 71 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.cloud.gcp.mock_corpus` | `dag/extdeps/cloud/gcp/mock_corpus.dag` | 110 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.cron.mock_corpus` | `dag/extdeps/cron/mock_corpus.dag` | 22 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.diagnostic.mock_corpus` | `dag/extdeps/diagnostic_mock_corpus.dag` | 39 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.ebay.mock_corpus` | `dag/extdeps/ebay/mock_corpus.dag` | 79 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.filesystem.mock_corpus` | `dag/extdeps/filesystem/mock_corpus.dag` | 22 | {'dag': 2} `dag/gunbc/extdeps_scope_frontier.dag` `dag/gunbc/plans/m4_universal_hermetic_corpus.dag` |
| `extdeps.git.mock_corpus` | `dag/extdeps/git/mock_corpus.dag` | 167 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.github.mock_corpus` | `dag/extdeps/github/mock_corpus.dag` | 71 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.linux.mock_corpus` | `dag/extdeps/linux/mock_corpus.dag` | 15 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.llm.mock_corpus` | `dag/extdeps/llm/mock_corpus.dag` | 55 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.sec.mock_corpus` | `dag/extdeps/sec/mock_corpus.dag` | 23 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.shell.mock_corpus` | `dag/extdeps/shell_mock_corpus.dag` | 31 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.tcgplayer.mock_corpus` | `dag/extdeps/tcgplayer/mock_corpus.dag` | 63 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `gunbc.apply` | `dag/gunbc/apply.dag` | 118 | {'dag': 2} `dag/gunbc/runner_lifecycle.dag` `dag/gunbc/runner_capacity_realize.dag` |
| `gunbc.auth.credentials` | `dag/gunbc/auth/credentials.dag` | 89 | {'dag': 1, 'rs': 1} `dag/gunbc/tailscale_acl_phase2_credential.dag` `src/v1/stage0/src/bin/parse_witness.rs` |
| `gunbc.auth.optional_impersonation` | `dag/gunbc/auth/optional_impersonation.dag` | 20 | {'dag': 2} `dag/test/claim/tailscale_acl_phase2_design_witness_test.dag` `dag/gunbc/tailscale_acl_phase2_credential.dag` |
| `gunbc.auth.patterns` | `dag/gunbc/auth/patterns.dag` | 113 | {'rs': 1} `src/v1/stage0/src/cli_run.rs` |
| `gunbc.bootstrap` | `dag/gunbc/bootstrap.dag` | 126 | {'dag': 2} `dag/gunbc/doc_graph_roots.dag` `src/v2/compiler/self_host/frontier_probe_types.dag` |
| `gunbc.char_at_scaling_probe_support` | `dag/gunbc/char_at_scaling_probe_support.dag` | 58 | {'rs': 1} `src/v1/stage0/src/bin/char_at_scaling_probe.rs` |
| `gunbc.ci_build_job_v1_compiler_unit_receipt` | `dag/gunbc/ci_build_job_v1_compiler_unit_receipt.dag` | 21 | {'dag': 1} `dag/gunbc/ci_spec.dag` |
| `gunbc.ci_input_envelope` | `dag/gunbc/ci_input_envelope.dag` | 86 | {'dag': 3} `dag/gunbc/doc_graph_roots.dag` `dag/gunbc/plans/bounded_input_cost_envelope_scheduling.dag` |
| `gunbc.compile_source_model` | `dag/gunbc/compile_source_model.dag` | 65 | {'dag': 1} `dag/gunbc/plans/seed_debt_bundle_item_2.dag` |
| `gunbc.deployed_intent_v0` | `dag/gunbc/deployed_intent_v0.dag` | 61 | {'dag': 2} `dag/gunbc/host_standup.dag` `dag/gunbc/host_identity_adopt.dag` |
| `gunbc.deployed_intent_v1` | `dag/gunbc/deployed_intent_v1.dag` | 69 | {'dag': 1} `dag/gunbc/host_standup.dag` |
| `gunbc.design_argument` | `dag/gunbc/design_argument.dag` | 93 | {'dag': 1} `dag/gunbc/plans/axiom_syllogism_lens.dag` |
| `gunbc.githooks_pre_push_cli` | `dag/gunbc/githooks_pre_push_cli.dag` | 10 | {'dag': 2, 'rs': 2} `dag/std/emit_on_demand.dag` `dag/gunbc/githooks_pre_push_fmt_transport_scaffold.dag` |
| `gunbc.host_authorized_keys_reconcile` | `dag/gunbc/host_authorized_keys_reconcile.dag` | 104 | {'dag': 1} `dag/gunbc/build_cache_instance.dag` |
| `gunbc.host_build_cache_provision` | `dag/gunbc/host_build_cache_provision.dag` | 335 | {'dag': 4} `dag/gunbc/build_cache_instance.dag` `dag/gunbc/fleet_host_budget.dag` |
| `gunbc.host_identity_assimilation` | `dag/gunbc/host_identity_assimilation.dag` | 266 | {'dag': 3} `dag/gunbc/host_standup.dag` `dag/gunbc/host_identity_adopt.dag` |
| `gunbc.host_identity_converge` | `dag/gunbc/host_identity_converge.dag` | 250 | {'dag': 1} `dag/gunbc/host_standup.dag` |
| `gunbc.host_identity_knob` | `dag/gunbc/host_identity_knob.dag` | 55 | {'dag': 1} `dag/gunbc/host_standup.dag` |
| `gunbc.host_identity_observation` | `dag/gunbc/host_identity_observation.dag` | 89 | {'dag': 1} `dag/gunbc/host_standup.dag` |
| `gunbc.host_network_diagnosis` | `dag/gunbc/host_network_diagnosis.dag` | 213 | {'dag': 1} `dag/gunbc/prose_row_frontier.dag` |
| `gunbc.host_toolchain_components` | `dag/gunbc/host_toolchain_components.dag` | 195 | {'dag': 1} `dag/gunbc/prose_row_frontier.dag` |
| `gunbc.interpreter_kernel_model` | `dag/gunbc/interpreter_kernel_model.dag` | 82 | {'dag': 1} `dag/gunbc/plans/interpreter_kernel_d.dag` |
| `gunbc.namespace_census_receipt` | `dag/gunbc/namespace_census_receipt.dag` | 74 | {'dag': 1} `dag/gunbc/doc_graph_roots.dag` |
| `gunbc.network_identity_subsumption` | `dag/gunbc/network_identity_subsumption.dag` | 134 | {'dag': 5} `dag/test/claim/dgx_spark_witness_test.dag` `dag/test/claim/host_phase_status_witness_test.dag` |
| `gunbc.p1_retention_cohort_receipt` | `dag/gunbc/p1_retention_cohort_receipt.dag` | 8 | {'dag': 1} `dag/gunbc/doc_graph_roots.dag` |
| `gunbc.plans.affected_set_self_confirmation` | `dag/gunbc/plans/affected_set_self_confirmation.dag` | 29 | {'rs': 1} `src/v1/stage0/src/cli_run.rs` |
| `gunbc.plans.branch_merge_admission_model` | `dag/gunbc/plans/branch_merge_admission_model.dag` | 172 | {'dag': 2} `dag/test/claim/merge_lifecycle_interleaving_witness_test.dag` `dag/gunbc/merge_lifecycle.dag` |
| `gunbc.plans.fleet_subsumption_manual_gaps` | `dag/gunbc/plans/fleet_subsumption_manual_gaps.dag` | 196 | {'dag': 12} `dag/test/claim/retained_shell_script_witness_test.dag` `dag/gunbc/build_cache_instance.dag` |
| `gunbc.plans.host_convergence_circuit_residue` | `dag/gunbc/plans/host_convergence_circuit_residue.dag` | 75 | {'dag': 1} `dag/gunbc/host_converge.dag` |
| `gunbc.plans.merge_admission_gate_shape_proposal` | `dag/gunbc/plans/merge_admission_gate_shape_proposal.dag` | 78 | {'dag': 1} `dag/gunbc/merge_admission.dag` |
| `gunbc.plans.transport_argv_anemia_dissolution` | `dag/gunbc/plans/transport_argv_anemia_dissolution.dag` | 89 | {'dag': 2} `dag/extdeps/git/git.dag` `dag/extdeps/exec/command.dag` |
| `gunbc.process_algebra` | `dag/gunbc/process_algebra.dag` | 147 | {'dag': 3} `dag/gunbc/doc_graph_roots.dag` `dag/gunbc/plans/invert_hand_maintained.dag` |
| `gunbc.runner_slot_enforcement` | `dag/gunbc/runner_slot_enforcement.dag` | 129 | {'dag': 3} `dag/gunbc/host_standup.dag` `dag/gunbc/runner_slot_allocation.dag` |
| `gunbc.seed_closed_vocabulary_wildcard_census` | `dag/gunbc/seed_closed_vocabulary_wildcard_census.dag` | 175 | {'rs': 1} `src/v1/stage0/src/cli_run.rs` |
| `gunbc.site.interaction` | `dag/gunbc/site/interaction.dag` | 14 | {'dag': 1} `dag/gunbc/design/interaction.dag` |
| `gunbc.spark.provisioning` | `dag/gunbc/spark/provisioning.dag` | 543 | {'dag': 2} `dag/extdeps/systems/nvidia_dgx_spark_setup.dag` `dag/test/claim/spark_provisioning_witness_test.dag` |
| `gunbc.srv3_os_install_diagnostic` | `dag/gunbc/srv3_os_install_diagnostic.dag` | 1410 | {'dag': 1} `dag/gunbc/non_fold_residue.dag` |
| `gunbc.tailscale_acl_emit` | `dag/gunbc/tailscale_acl_emit.dag` | 52 | {'dag': 1} `dag/gunbc/host_standup.dag` |
| `gunbc.test_node_wall_clock_ratchet` | `dag/gunbc/test_node_wall_clock_ratchet.dag` | 99 | {'dag': 1} `dag/gunbc/plans/structural_quadratic_wall_coverage_audit.dag` |
| `gunbc.tools.review` | `dag/gunbc/tools/review.dag` | 187 | {'dag': 4} `dag/test/claim/workflow_default_field_projection_fold_witness_test.dag` `src/v2/lens/meta_exec_confinement.dag` |
| `gunbc.tools.review_codex` | `dag/gunbc/tools/review_codex.dag` | 205 | {'dag': 2, 'rs': 1} `dag/test/claim/workflow_default_field_projection_fold_witness_test.dag` `dag/gunbc/roadmap_belt_actuate.dag` |
| `gunbc.v1_maintenance_standing` | `dag/gunbc/v1_maintenance_standing.dag` | 83 | {'dag': 6, 'rs': 1} `dag/test/claim/match_arm_pattern_identity_emission_witness_test.dag` `dag/test/claim/documentary_refs_witness_test.dag` |
| `gunbc.workflow.types` | `dag/gunbc/workflow/types.dag` | 311 | {'dag': 1} `dag/gunbc/plans/host_effect_orchestration.dag` |
| `std.behavioral` | `dag/std/behavioral.dag` | 51 | {'rs': 1} `src/v1/stage0/src/bin/parse_witness.rs` |
| `std.durable_compare_and_set` | `dag/std/durable_compare_and_set.dag` | 292 | {'dag': 1} `dag/test/claim/durable_compare_and_set_witness_test.dag` |
| `std.methods` | `dag/std/methods.dag` | 67 | {'dag': 1, 'rs': 2} `src/v1/compiler_tests_rust.dag` `src/v1/stage0/src/v1_compiler_compiler_tests_rust.rs` |
| `std.stack` | `dag/std/stack.dag` | 56 | {'dag': 1, 'rs': 1} `dag/gunbc/witness_floor_workflow.dag` `src/v1/stage0/src/bin/parse_witness.rs` |
| `std.verification` | `dag/std/verification.dag` | 34 | {'dag': 3} `dag/gunbc/plans/resolver_type_name_collision_wall.dag` `dag/gunbc/plans/realization_measurement_loop.dag` |
| `tools.gunbc_ci` | `dag/tools/gunbc_ci.dag` | 25 | {'dag': 2, 'rs': 1} `dag/std/emit_on_demand.dag` `src/v2/test/claim/host_language_transport_script/corpus/wall_residue_live_test.dag` |
| `v2.extdeps.languages.ecmascript` | `src/v2/extdeps/languages/ecmascript.dag` | 1340 | {'dag': 1} `dag/gunbc/language_target_registry.dag` |
| `v2.extdeps.languages.machine_code` | `src/v2/extdeps/languages/machine_code.dag` | 559 | {'dag': 4} `dag/extdeps/languages/riscv/subject.dag` `dag/test/claim/language_target_registry_totality_test.dag` |
| `v2.extdeps.languages.ptx` | `src/v2/extdeps/languages/ptx.dag` | 223 | {'dag': 1} `dag/gunbc/language_target_registry.dag` |
| `v2.extdeps.languages.swift` | `src/v2/extdeps/languages/swift.dag` | 2366 | {'dag': 2} `dag/gunbc/language_target_registry.dag` `src/v2/test/claim/complexity/accumulator_copy_roster_gate_swift_test.dag` |
| `v2.extdeps.languages.wasm` | `src/v2/extdeps/languages/wasm.dag` | 2019 | {'dag': 2} `dag/gunbc/language_target_registry.dag` `dag/gunbc/plans/language_target_self_host_frontier.dag` |
| `v2.std.datetime` | `src/v2/std/datetime.dag` | 658 | {'dag': 2, 'rs': 1} `dag/extdeps/pin.dag` `src/v2/test/manual/parse_forensics_scaling_witness.dag` |
| `v2.std.float` | `src/v2/std/float.dag` | 174 | {'dag': 1} `dag/gunbc/non_fold_residue.dag` |
| `v2.std.probe_selector` | `src/v2/std/probe_selector.dag` | 674 | {'dag': 2} `dag/gunbc/non_fold_residue.dag` `dag/gunbc/plans/dag_v2_defork_audit.dag` |
| `v2.test.workflow.glob_discovery_law` | `src/v2/workflow/glob_discovery_law.dag` | 113 | {'dag': 1} `src/v2/test/claim/complexity/accumulator_copy_roster_gate_test.dag` |
| `v2.workflow.class_b_import_closure_transport` | `src/v2/workflow/class_b_import_closure_transport.dag` | 118 | {'dag': 2, 'rs': 1} `dag/test/claim/long/rust_test_fixtures_import_closure_witness_test.dag` `src/v2/workflow/class_b_import_closure_probe.dag` |
| `v2.workflow.compile_door_ledger` | `src/v2/workflow/compile_door_ledger.dag` | 341 | {'dag': 1} `src/v2/test/claim/long/door_real_module_probe_test.dag` |
| `v2.workflow.compiler_closure_ingest_transport` | `src/v2/workflow/compiler_closure_ingest_transport.dag` | 150 | {'dag': 4} `dag/tools/ci_gates.dag` `dag/gunbc/ci_layer_roots.dag` |
| `v2.workflow.phase_profile_proof_plan` | `src/v2/workflow/phase_profile_proof_plan.dag` | 22 | {'rs': 1} `src/v1/stage0/tests/phase_profile_claim_executor.rs` |
| `v2.workflow.source_root_ingest_gate` | `src/v2/workflow/source_root_ingest_gate.dag` | 18 | {'dag': 3} `dag/test/claim/guarantee_rung_drop_witness_test.dag` `dag/tools/ci_gates.dag` |
| `v2.workflow.source_root_ingest_transport` | `src/v2/workflow/source_root_ingest_transport.dag` | 90 | {'dag': 2} `dag/tools/ci_gates.dag` `src/v2/test/claim/host_language_transport_script/corpus/migrated_transports_clean_test.dag` |

### CITED-AUTHORITY — 103 modules

| module | path | lines | live-source mentions |
| --- | --- | --- | --- |
| `extdeps.access.aws_iam` | `dag/extdeps/access/aws_iam.dag` | 45 | {'dag': 1} `dag/gunbc/principal_projection.dag` |
| `extdeps.access.zanzibar` | `dag/extdeps/access/zanzibar.dag` | 62 | {'dag': 1} `dag/gunbc/principal_projection.dag` |
| `extdeps.audit.cloudevents` | `dag/extdeps/audit/cloudevents.dag` | 58 | —  |
| `extdeps.bmc.ipmi` | `dag/extdeps/bmc/ipmi.dag` | 48 | —  |
| `extdeps.boot.emit` | `dag/extdeps/boot/emit.dag` | 127 | —  |
| `extdeps.boot.framebuffer` | `dag/extdeps/boot/framebuffer.dag` | 15 | —  |
| `extdeps.boot.freestanding_payload` | `dag/extdeps/boot/freestanding_payload.dag` | 19 | —  |
| `extdeps.boot.freestanding_witness` | `dag/extdeps/boot/freestanding_witness.dag` | 198 | —  |
| `extdeps.boot.linux_x86_boot` | `dag/extdeps/boot/linux_x86_boot.dag` | 37 | —  |
| `extdeps.cloud.gcp.iam_admin` | `dag/extdeps/cloud/gcp/iam_admin.dag` | 114 | —  |
| `extdeps.cloud.gcp.serviceusage` | `dag/extdeps/cloud/gcp/serviceusage.dag` | 51 | —  |
| `extdeps.cloud.gcp.sts` | `dag/extdeps/cloud/gcp/sts.dag` | 111 | {'rs': 2} `src/v1/stage0/src/cli_run.rs` `src/v1/stage0/src/bin/effects_rest_transport_witness.rs` |
| `extdeps.cloud_init.cloud_init` | `dag/extdeps/cloud_init/cloud_init.dag` | 58 | {'dag': 1} `dag/gunbc/prose_row_frontier.dag` |
| `extdeps.colo.centersquare` | `dag/extdeps/colo/centersquare.dag` | 47 | —  |
| `extdeps.colo.colocation_america` | `dag/extdeps/colo/colocation_america.dag` | 45 | —  |
| `extdeps.colo.coresite` | `dag/extdeps/colo/coresite.dag` | 58 | —  |
| `extdeps.colo.dataverge` | `dag/extdeps/colo/dataverge.dag` | 47 | —  |
| `extdeps.colo.digital_realty` | `dag/extdeps/colo/digital_realty.dag` | 59 | —  |
| `extdeps.colo.equinix` | `dag/extdeps/colo/equinix.dag` | 71 | —  |
| `extdeps.colo.evocative` | `dag/extdeps/colo/evocative.dag` | 49 | —  |
| `extdeps.colo.h5` | `dag/extdeps/colo/h5.dag` | 59 | —  |
| `extdeps.colo.halsey_165` | `dag/extdeps/colo/halsey_165.dag` | 73 | —  |
| `extdeps.colo.hivelocity` | `dag/extdeps/colo/hivelocity.dag` | 45 | —  |
| `extdeps.colo.interserver` | `dag/extdeps/colo/interserver.dag` | 100 | —  |
| `extdeps.colo.iron_mountain` | `dag/extdeps/colo/iron_mountain.dag` | 44 | —  |
| `extdeps.colo.natcoweb` | `dag/extdeps/colo/natcoweb.dag` | 123 | —  |
| `extdeps.colo.netrality` | `dag/extdeps/colo/netrality.dag` | 45 | —  |
| `extdeps.colo.qts` | `dag/extdeps/colo/qts.dag` | 45 | —  |
| `extdeps.colo.summit` | `dag/extdeps/colo/summit.dag` | 49 | —  |
| `extdeps.colo.three_sixty_five` | `dag/extdeps/colo/three_sixty_five.dag` | 89 | —  |
| `extdeps.colo.tierpoint` | `dag/extdeps/colo/tierpoint.dag` | 86 | —  |
| `extdeps.colo.types` | `dag/extdeps/colo/types.dag` | 110 | —  |
| `extdeps.container.oci.ctrl_session_witness` | `dag/extdeps/container/oci/ctrl_session_witness.dag` | 361 | —  |
| `extdeps.container.oci.image_config` | `dag/extdeps/container/oci/image_config.dag` | 100 | —  |
| `extdeps.container.oci.linux` | `dag/extdeps/container/oci/linux.dag` | 156 | —  |
| `extdeps.container.oci.manifest` | `dag/extdeps/container/oci/manifest.dag` | 98 | —  |
| `extdeps.container.oci.runtime_config` | `dag/extdeps/container/oci/runtime_config.dag` | 97 | —  |
| `extdeps.currency.currency` | `dag/extdeps/currency/currency.dag` | 24 | {'dag': 1} `dag/gunbc/prose_row_frontier.dag` |
| `extdeps.darwin.rusage` | `dag/extdeps/darwin/rusage.dag` | 31 | {'dag': 2, 'rs': 1} `dag/test/claim/peak_resident_measured_witness_test.dag` `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.darwin.sysctl` | `dag/extdeps/darwin/sysctl.dag` | 64 | {'dag': 3, 'rs': 1} `dag/gunbc/extdeps_scope_frontier.dag` `dag/gunbc/prose_row_frontier.dag` |
| `extdeps.ebay.ebay` | `dag/extdeps/ebay/ebay.dag` | 94 | —  |
| `extdeps.ebay.ebay_contracts` | `dag/extdeps/ebay/ebay_contracts.dag` | 34 | —  |
| `extdeps.ebay.errors` | `dag/extdeps/ebay/errors.dag` | 61 | —  |
| `extdeps.ebay.inventory` | `dag/extdeps/ebay/inventory.dag` | 468 | —  |
| `extdeps.ebay.oauth` | `dag/extdeps/ebay/oauth.dag` | 151 | —  |
| `extdeps.energy.nj_electricity` | `dag/extdeps/energy/nj_electricity.dag` | 61 | —  |
| `extdeps.exec.xargs` | `dag/extdeps/exec/xargs.dag` | 31 | {'dag': 1} `dag/gunbc/prose_row_frontier.dag` |
| `extdeps.filesystem.ntfs` | `dag/extdeps/filesystem/ntfs.dag` | 28 | —  |
| `extdeps.filesystem.posix` | `dag/extdeps/filesystem/posix.dag` | 34 | —  |
| `extdeps.formats.elf.encode` | `dag/extdeps/formats/elf/encode.dag` | 201 | —  |
| `extdeps.formats.elf.hello_static_witness` | `dag/extdeps/formats/elf/hello_static_witness.dag` | 259 | —  |
| `extdeps.formats.elf.primitives` | `dag/extdeps/formats/elf/primitives.dag` | 61 | —  |
| `extdeps.formats.elf.relocation` | `dag/extdeps/formats/elf/relocation.dag` | 26 | —  |
| `extdeps.formats.elf.sections` | `dag/extdeps/formats/elf/sections.dag` | 31 | —  |
| `extdeps.formats.elf.segments` | `dag/extdeps/formats/elf/segments.dag` | 79 | —  |
| `extdeps.formats.elf.types` | `dag/extdeps/formats/elf/types.dag` | 190 | —  |
| `extdeps.git.versioning` | `dag/extdeps/git/versioning.dag` | 73 | —  |
| `extdeps.github.auth` | `dag/extdeps/github/auth.dag` | 61 | {'rs': 1} `src/v1/stage0/src/bin/parse_witness.rs` |
| `extdeps.github.ci` | `dag/extdeps/github/ci.dag` | 30 | {'dag': 1} `dag/gunbc/plans/realization_measurement_loop.dag` |
| `extdeps.github.gists` | `dag/extdeps/github/gists.dag` | 76 | {'rs': 2} `src/v1/stage0/src/bin/parse_witness.rs` `src/v1/stage0/src/bin/effects_rest_transport_witness.rs` |
| `extdeps.github.github_contracts` | `dag/extdeps/github/github_contracts.dag` | 15 | —  |
| `extdeps.github.issues` | `dag/extdeps/github/issues.dag` | 202 | {'dag': 1, 'rs': 1} `dag/gunbc/extdeps_scope_frontier.dag` `src/v1/stage0/src/bin/effects_rest_transport_witness.rs` |
| `extdeps.github.mergeable_state` | `dag/extdeps/github/mergeable_state.dag` | 39 | —  |
| `extdeps.github.mergeable_state_contracts` | `dag/extdeps/github/mergeable_state_contracts.dag` | 18 | —  |
| `extdeps.gitignore` | `dag/extdeps/git/gitignore.dag` | 36 | —  |
| `extdeps.languages.go.module` | `dag/extdeps/languages/go/module.dag` | 144 | {'dag': 1} `dag/gunbc/commit_workflow.dag` |
| `extdeps.languages.go.primitives` | `dag/extdeps/languages/go/primitives.dag` | 211 | {'dag': 1, 'rs': 1} `dag/std/checked_arithmetic.dag` `src/v1/stage0/src/std_checked_arithmetic.rs` |
| `extdeps.languages.python.primitives` | `dag/extdeps/languages/python/primitives.dag` | 86 | —  |
| `extdeps.languages.rust.primitives` | `dag/extdeps/languages/rust/primitives.dag` | 112 | {'dag': 2, 'rs': 2} `dag/extdeps/languages/rust/types.dag` `dag/std/checked_arithmetic.dag` |
| `extdeps.languages.typescript.primitives` | `dag/extdeps/languages/typescript/primitives.dag` | 103 | —  |
| `extdeps.linux.edac` | `dag/extdeps/linux/edac.dag` | 26 | —  |
| `extdeps.linux.proc_meminfo` | `dag/extdeps/linux/proc_meminfo.dag` | 41 | {'dag': 1} `dag/extdeps/linux/procfs.dag` |
| `extdeps.linux.rusage` | `dag/extdeps/linux/rusage.dag` | 31 | {'dag': 2, 'rs': 1} `dag/test/claim/peak_resident_measured_witness_test.dag` `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.llm.anthropic_errors` | `dag/extdeps/llm/anthropic_errors.dag` | 80 | —  |
| `extdeps.llm.anthropic_rest` | `dag/extdeps/llm/anthropic_rest.dag` | 96 | {'rs': 1} `src/v1/stage0/src/bin/effects_rest_transport_witness.rs` |
| `extdeps.llm.llm` | `dag/extdeps/llm/llm.dag` | 12 | —  |
| `extdeps.llm.openai_contracts` | `dag/extdeps/llm/openai_contracts.dag` | 32 | —  |
| `extdeps.llm.openai_errors` | `dag/extdeps/llm/openai_errors.dag` | 66 | —  |
| `extdeps.llm.openai_rest` | `dag/extdeps/llm/openai_rest.dag` | 92 | {'rs': 1} `src/v1/stage0/src/bin/effects_rest_transport_witness.rs` |
| `extdeps.netplan.netplan` | `dag/extdeps/netplan/netplan.dag` | 69 | {'dag': 1} `dag/gunbc/prose_row_frontier.dag` |
| `extdeps.observability` | `dag/extdeps/observability.dag` | 49 | —  |
| `extdeps.posix.rusage` | `dag/extdeps/posix/rusage.dag` | 66 | {'dag': 3, 'rs': 2} `dag/test/claim/peak_resident_measured_witness_test.dag` `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.provisioning.ubuntu_seeded_install_media_toolchain` | `dag/extdeps/provisioning/ubuntu_seeded_install_media_toolchain.dag` | 39 | —  |
| `extdeps.realestate.nj_industrial` | `dag/extdeps/realestate/nj_industrial.dag` | 70 | —  |
| `extdeps.render.surface` | `dag/extdeps/render/surface.dag` | 79 | {'rs': 2} `src/v1/stage0/src/cli_run.rs` `src/v1/stage0/src/v1_interpreter.rs` |
| `extdeps.render.terminal_capability` | `dag/extdeps/render/terminal_capability.dag` | 60 | {'rs': 1} `src/v1/stage0/src/v1_interpreter.rs` |
| `extdeps.runtime.api.darwin` | `dag/extdeps/runtime/api/darwin.dag` | 59 | —  |
| `extdeps.runtime.api.windows` | `dag/extdeps/runtime/api/windows.dag` | 114 | —  |
| `extdeps.runtime.local` | `dag/extdeps/runtime/local.dag` | 22 | —  |
| `extdeps.sec.edgar_rest` | `dag/extdeps/sec/edgar_rest.dag` | 71 | {'dag': 2} `dag/extdeps/sec/edgar.dag` `dag/gunbc/prose_row_frontier.dag` |
| `extdeps.shell.credentials` | `dag/extdeps/shell/credentials.dag` | 19 | —  |
| `extdeps.tailscale.acl` | `dag/extdeps/tailscale/acl.dag` | 312 | {'dag': 2} `dag/test/claim/host_standup_spine_witness_test.dag` `dag/gunbc/host_standup.dag` |
| `extdeps.tailscale.acl_api` | `dag/extdeps/tailscale/acl_api.dag` | 70 | —  |
| `extdeps.tcgplayer.catalog` | `dag/extdeps/tcgplayer/catalog.dag` | 180 | —  |
| `extdeps.tcgplayer.pricing` | `dag/extdeps/tcgplayer/pricing.dag` | 78 | —  |
| `extdeps.tcgplayer.store` | `dag/extdeps/tcgplayer/store.dag` | 97 | —  |
| `extdeps.tcgplayer.tcgplayer` | `dag/extdeps/tcgplayer/tcgplayer.dag` | 82 | —  |
| `extdeps.tools.diffutils` | `dag/extdeps/tools/diffutils.dag` | 29 | {'dag': 1} `dag/extdeps/tools/gnu_coreutils.dag` |
| `extdeps.transports.sql` | `dag/extdeps/transports/sql.dag` | 41 | {'dag': 1} `src/v2/test/fixture/frontier_probe_elision_boundary_overlay.dag` |
| `extdeps.vendor.arm` | `dag/extdeps/vendor/arm.dag` | 31 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.vendor.qualcomm` | `dag/extdeps/vendor/qualcomm.dag` | 31 | {'dag': 1} `dag/gunbc/extdeps_scope_frontier.dag` |
| `extdeps.version.pep440` | `dag/extdeps/version/pep440.dag` | 310 | —  |
| `gunbc.hand_lens_host_bridge_scaffold_index` | `dag/gunbc/hand_lens_host_bridge_scaffold_index.dag` | 23 | —  |