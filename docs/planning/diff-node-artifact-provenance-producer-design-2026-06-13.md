# diff→NodeArtifactProvenance producer — design (Part 2) + frontier-selection fusion ladder sketch (Part 3)

Work item: `adhoc-0fa0067b-131` (sunny-ant-116) · CI-investigation tree (swift-stag-552).
Companion to Part 1: `affected-set-ci-gating-recommendation-2026-06-12.md`.

**Status: DESIGN / SCOPING ONLY.** No substrate lands from this doc. Part 3 stays a sketch until
the corpus gate (#4785) lands **and** the Part-1 `affects_v4` coverage gap is closed (CI-investigation
manager constraint, 2026-06-13). Any `ci_select_*` / gating change coordinates through the dep-graph
coordinator (snappy-crab-849).

## Why this exists — the through-line from Part 1

Part 1 found the structural false-negative class: the host detector's `affects_v4` path **allowlist**
(`tools/ci_affected_components/src/lib.rs`) is *narrower* than the floor's real compile closure
(`v2-compiler compile --source-root src/v4`). A hand-maintained path→bucket table **drifts** from the
closure it is supposed to mirror, because nothing structurally ties the two together.

The principled fix is not "add more prefixes to the allowlist" — that just defers the next drift. It
is to **derive** the path→node mapping from the *same source ingest the compiler actually compiles*,
so the detector's closure is a superset of the compile closure **by construction**. That derived
mapping is exactly `NodeArtifactProvenance`, and the thing that produces it from a real repo ingest
(not a fixture, not a parallel scanner) is the **diff→NodeArtifactProvenance producer** — the lane
RR-K §7 named as the blocker for real-input affected-set coverage:

> "real-input coverage is blocked on a separate design-first source-provenance producer lane that
> derives `NodeArtifactProvenance` from the compiler/source-authority ingest, not from snapshots or a
> parallel scanner." — RR-K worksheet §7

This is also the gap that stalled the tidy-wolf tree (work items `adhoc-bc4d39de-88f`,
`adhoc-9bb1e6fb-9ba`): the **source_authority API exposes only a per-source-read producer**, with no
whole-ingest enumerator to fold into the `FreeMonoid<NodeArtifactProvenance>` that consumers need.

## Substrate as it stands (verified against the tree, 2026-06-13)

| Element | Location | Shape |
|---|---|---|
| `NodeArtifactProvenance { node, artifact }` | `src/v4/std/artifact.dag:97` | file-grain (path→node), the shape affected-set already consumes |
| `SourceIrArtifactProvenance { node, artifact, span_index }` | `src/v4/compiler/source_authority.dag:65` | span-grain; **produced internally, span_index stripped by the public API** |
| `source_ir_artifact_provenance(source_read)` | `source_authority.dag:236` | per-**single**-`DagSourceReadWitness` → `Outcome<SourceIrArtifactProvenance>` |
| `source_ir_node_artifact_provenance(source_read)` | `source_authority.dag:263` | per-**single**-read → `Outcome<NodeArtifactProvenance>` (strips span) |
| `affected_set_reading_from_git_diff_provenance(graph, git_diff, provenance)` | `src/v4/lens/edit_locus.dag:273` | consumes `FreeMonoid<NodeArtifactProvenance>` — **the seam to fill** |
| `affected_testgen_reading_from_fixture_diff(git_diff)` | `src/v4/test/claim/workflow/affected_testgen_ci_runner.dag:150` | today feeds **hand-built** 2-row provenance fixture |

**The gap, precisely:** the per-read producer exists; the *closure* producer does not. There is no
function that enumerates the source-root ingest set and folds the per-read provenance into one
`FreeMonoid<NodeArtifactProvenance>`. So every consumer that needs real provenance hand-builds it
(fixtures), and the host transport reinvents it as a path allowlist that drifts (Part 1).

**Crucially, the per-read → affected-set wire is already PROVEN** (verified 2026-06-13 in
`src/v4/test/claim/lens_affected_set/edit_locus_resolver.dag`):

- `edit_locus_source_provenance_affected_set_wire_holds` (`:192`) takes a *single*
  `source_ir_node_artifact_provenance(source_read)`, builds the graph, and drives
  `affected_set_reading_from_git_diff_provenance(graph, git_diff, provenance: [provenance])` to the
  expected frontier. End-to-end, on one read.
- `edit_locus_source_provenance_producer_rejects_malformed_source_holds` (`:259`) proves a malformed
  source → `Rejected` (the fail-closed reject the coverage law lifts).

So the producer is **mostly assembly of proven parts, not new minting** — which keeps the minting
authority single (MODELING M9). The only genuinely new design is the *closure* layer: a
source-root ingest carrier and the monoid fold over it.

## Part 2 — producer design

### Contract

A producer (working name `node_artifact_provenance_from_source_root`) with the shape:

```
fn node_artifact_provenance_from_source_root(
  ingest: SourceRootIngest          // the same closure the floor compiles: --source-root src/v4
) -> Outcome<FreeMonoid<NodeArtifactProvenance>>
```

- **Input is the compiler's own ingest set**, enumerated through source_authority — *not* a git
  diff, *not* a filesystem scan, *not* a snapshot. The diff is matched against the produced map
  downstream (in `affected_set_reading_from_git_diff_provenance`); it is **not** an input to the
  producer. This preserves RR-K §2.1 single "what changed" authority: the producer supplies the
  *map*, `edit_locus` does the *matching*.
- **Real source, real nodes — no synthetic snapshot, no second parser** (tidy-wolf constraints,
  `adhoc-bc4d39de-88f` / `adhoc-9bb1e6fb-9ba`, relayed 2026-06-13): the host side converts the
  *actual* PR changed paths into a modeled `GitDiffNameOnly`, and the producer emits
  `NodeArtifactProvenance` from **parsed/resolved real source nodes** via source_authority — never
  from a hand-built fixture snapshot. A Rust-side source-tree scanner is a **forbidden direction**:
  it would be a second parser authority and violate substrate-is-authority. The whole-ingest fold
  over `source_ir_node_artifact_provenance` is the only sanctioned shape, which is exactly this
  design.
- **Output is the existing `FreeMonoid<NodeArtifactProvenance>`** — the exact shape
  `affected_set_reading_from_git_diff_provenance` already consumes. No new consumer type; the
  fixture rows in `affected_testgen_ci_runner.dag:150` are replaced by this producer's output.
- **Implementation is a fold of the existing per-read producer** over the ingest set:
  `source_ir_node_artifact_provenance` per `DagSourceReadWitness`, monoid-concatenated. The only
  *new* API is the whole-ingest enumerator + fold; the per-read minting stays where it is
  (no second authority). The new pieces are exactly two:
  1. a **`SourceRootIngest` carrier** = `FreeMonoid<DagSourceReadWitness>` (the source-root closure
     as a set of reads — the same set `--source-root src/v4` walks);
  2. the **fold** `node_artifact_provenance_from_source_root` that maps each read through the
     already-proven `source_ir_node_artifact_provenance` and concatenates, short-circuiting to
     `reject` on the first `Rejected` read (lifting the proven per-read fail-closed to the closure).

### M9 — attach the ingest carrier to the discovery concept family, don't fork it

DFS the concept DAG before minting `SourceRootIngest` (MODELING M9). Whole-tree enumeration prior
art already exists in the **discovery lane** (`src/v4/test/claim/workflow/discovery_types.dag` +
`glob_discovery_law.dag`):

| discovery lane (decl enumeration) | producer (source-read enumeration) |
|---|---|
| glob-discovery walks the tree → `List<OwnedDataDeclRecord>` | source-root walk → `FreeMonoid<DagSourceReadWitness>` |
| `OwnedDataDeclRecord { entry, module, decl_name, initializer }` — per-decl, carries source `entry` (file path) | `DagSourceReadWitness { source, artifact, compilation_unit }` — per-read, carries `artifact.file_path` |
| `ResolvedDeclRef { module, name }` — resolved ref | `NodeArtifactProvenance { node, artifact }` — resolved node↔artifact |
| `OwnedDataDiscoveryReceipt { ..._count, transport_projection_complete }` — scalar OOM-guard summary | a coverage-receipt scalar (covered-read count, `coverage_complete`) — same OOM-guard shape |

These are the **same** "whole-tree enumeration → per-item resolved record → scalar receipt" shape.
So the producer should **not** mint a parallel enumerator: it should reuse / mirror the
glob-discovery enumeration and emit a discovery-receipt-shaped scalar coverage summary (the
fail-closed/OOM guard below reads that scalar, not the full provenance list).

**Home question — RULED (snappy-crab-849, 2026-06-13): promotion approved, one enumeration
authority, attach-don't-mint.** The whole-tree enumeration + discovery-receipt shape promotes from
`test/claim/workflow` to the **compiler/source-authority layer** so the discovery lane **and** the
producer share one enumeration authority (one walk, not two). Approved with **four binding
conditions** (implementation gated; design refinement in-doc allowed now):

1. **Sequencing.** `affects_v4` tripwire first (already GO) → **#4785 corpus gate MERGES** (in final
   sign verification; re-raise if it stalls past **2026-06-14 EOD**) → *then* the relocation model
   PR. Producer implementation lands strictly **after** the relocation. No implementation here until
   tripwire **and** #4785 both clear.
2. **Shape.** A **standalone model PR** that declares the promoted types at the
   compiler/source-authority layer **and migrates the discovery-lane consumers in the same PR** —
   never a window with two live enumerators. M9 obligation: the PR description must DFS from `std/`
   and justify the chosen home file (P2-style, as in #4792's `program_partition.dag`).
3. **Load-bearing.** This rewires what the `v4_lens_gate` equivalence law witnesses, so:
   **design-sign before the ready-flip**, authority through snappy-crab-849, with by-execution
   receipts — **including a perturb receipt** (the equivalence law must go RED when the walk is
   deliberately broken).
4. **Frozen surface.** If the home or the consumer migration would touch any of the **#4741-frozen
   five** (`target_model.dag`, `06_value_expression.dag`, `05_eval.dag`, v4 `04_infer.dag`,
   `dag.dag`) — **STOP**; that slice holds until #4741 merges.

### Fail-closed law (load-bearing — this is the kill criterion)

The producer is the point where Part 1's silent false-negative must become a fail-closed widen:

1. **Parse/resolve failure** — if any ingested source's `source_ir_node_artifact_provenance`
   returns `Outcome::reject`, the producer returns `reject` (or a `FailClosed` carrier), which the
   affected-set maps `AffectedSetFailClosed → RerunNodeSetFailClosed → full roster` (RR-K §2.2). A
   source the compiler can't ingest must **widen** execution, never drop silently.
2. **Coverage law** — `coverage(produced provenance) ⊇ ingest closure`. Every artifact the floor
   would compile must appear in the produced map. A changed path that matches **no**
   `NodeArtifactProvenance` row must route to the fail-closed superset, **not** to "skip" — this is
   exactly the `src/v4/program.dag` / `workflow/runtime_run.dag` class Part 1 found. The producer
   makes the allowlist's under-selection structurally impossible: the map is the ingest, so an
   in-closure path cannot be absent unless ingest itself failed (case 1).

This is the contract that closes Part 1's gap *by construction* rather than by patching prefixes:
`detector closure ≡ compile closure` because both are `source_ir_*` over the same ingest.

**One arc with the `affects_v4` tripwire.** The separately-dispatched `affects_v4` widening-only
patch (snappy-crab-849's tree) and this producer are not two fixes — they are the **interim guard
and the durable replacement of the same coverage law.** The tripwire patch makes the allowlist
fail-closed *now* (any unrecognized `src/v4/*.dag` → widen) so the gap Part 1 found cannot bite
before the producer lands. The producer's coverage law (`coverage ⊇ ingest closure`, derived from
the real ingest) then **subsumes** the tripwire entirely: once the path→node map *is* the ingest,
there is no allowlist left to under-select, and the tripwire becomes dead code to retire. Sequencing
is tripwire → producer → retire tripwire; the two changes should be read and reviewed as that single
arc, not as competing detectors (M9: still one "what changed" authority throughout).

### Grain decision — file-level for v1, defer span-level

`SourceIrArtifactProvenance` carries `span_index` (byte-range → occurrence), and
`design-provenance-span.md §4.4` designs the span→node query. **The producer should stay file-grain
(`NodeArtifactProvenance`) for v1** and *not* surface `span_index`, because:

- The floor compiles **file-granular** (`--source-root src/v4` is a file-set closure); a file→node
  map is exactly sufficient to make the detector closure ⊇ the compile closure. Span grain buys
  nothing for *floor gating* — the unit of skip is the file.
- Span grain (which node *within* a file changed) is what testgen / edit-narrowing want (Part 3),
  and pulling it in now would also pull in the unimplemented span→node query (a separate WRITE/SYN
  lane). Keeping v1 file-grain avoids smuggling that.

So Part 2 closes the source_authority API gap **minimally**: add the whole-ingest enumerator + fold
returning `FreeMonoid<NodeArtifactProvenance>`; leave `SourceIrArtifactProvenance`/`span_index`
sealed until a span-consuming lane needs it.

### What Part 2 does NOT do (non-goals, per RR-K §5 + MODELING M9)

- No second "what changed" detector. The producer feeds the existing `edit_locus`/`affected_set`
  authority; it does not classify or select.
- No host Rust re-implementation. The host `ci_affected_components` bin stays *transport* that
  projects the modeled provenance (RR-K §2.4); it does not recompute the map.
- No `affects_v4` allowlist edit in this lane. Closing that gap fail-closed is the Part-1 follow-up
  (a separate coordinated change through snappy-crab-849); the producer is the *durable* replacement
  that makes the allowlist unnecessary, landed behind it.
- No gating promotion. The producer feeds the **shadow** path; promotion is Part 3 + dep-graph
  coordination.

## Part 3 — frontier-selection + affected_testgen_ci_runner fusion (LADDER SKETCH, design-only)

Goal: `affected_testgen_ci_runner` selects the witness set to run from a **real** node frontier
(diff → producer map → `affected_set_rerun_nodes` frontier), and feeds that narrowed set to the
multi-entry `claim_batch` runner (#4783, jolly-fox-125) — the natural executor for a
frontier-selected witness set (its CLI is additive multi-entry; contract in #4783's body).

Ladder (each rung gated on the one below):

1. **[blocked]** Part 2 producer lands — `affected_testgen_ci_runner` consumes real
   `FreeMonoid<NodeArtifactProvenance>` instead of the 2-row fixture; affected-set reading runs on
   real ingest. Fail-closed law (above) is the acceptance receipt.
2. **[blocked]** Part-1 `affects_v4` gap closed fail-closed **and** #4785 corpus gate landed (it
   enrolls semantic witnesses into `v4_lens_ci`). Selection must **narrow** that enrolled set, not
   race it.
3. **Fuse:** node-frontier selection (`affected_set_rerun_nodes` over the real graph) projects the
   subset of #4785's witnesses reachable from the diff frontier. Selection lives only in the
   `Produced` arm; `FailClosed`/cyclic/unresolved → full witness set (RR-K §2.2). Per-claim pins
   (`test_claim_ci_selection_fail_closed`) and job needs-closure survive every narrowing (RR-K §2.3).
4. **Run:** the selected witness set → `claim_batch` multi-entry CLI (#4783). One frontier-selected
   batch instead of the full roster.
5. **Roll out:** shadow → canary (compare frontier-selected vs full witness outcome on a cohort) →
   enforce, coordinated through snappy-crab-849. Never before rungs 1–2.

**Why selection is worth more than it looks (cost note):** `v4_lens_ci` cost is dominated by
per-entry closure re-resolve and is DRAM-bandwidth-sensitive (memory
`ci-lens-timeout-dram-bandwidth`). Shrinking the witness set via frontier selection saves
super-linearly on the bandwidth-bound re-resolve, not just the linear floor-skip minutes Part 1
modeled. That is the real prize of consolidation — but only once the producer makes the frontier
trustworthy on real input.

## Open questions for the manager / dep-graph coordinator

1. Producer home: `src/v4/compiler/source_authority.dag` (where the per-read minter lives) vs a new
   `src/v4/lens/` module that imports it. Leaning source_authority (keeps minting authority single).
2. `SourceRootIngest` enumeration: **answered 2026-06-13 by tree inspection** — no source-root
   ingest set-carrier exists yet; only the per-read `DagSourceReadWitness` and its proven
   per-read producer. So the enumerator + `FreeMonoid<DagSourceReadWitness>` carrier **is** new
   design (above). The residual tidy-wolf detail (`adhoc-bc4d39de-88f` / `adhoc-9bb1e6fb-9ba`) is
   only needed to confirm whether the *host* side already enumerates the source-root file set for
   the floor compile (so the carrier can be produced by transport, not recomputed).
3. Sequencing of the `affects_v4` fail-closed patch vs the producer: **resolved by the dep-graph
   coordinator (snappy-crab-849, 2026-06-13)** — the one-predicate widening-only patch is dispatched
   as a *separate* leaf (immediate safety, by-execution receipt + tripwire); the producer here is the
   *durable* follow-up that makes the allowlist unnecessary. Job-timings aggregator (measurement-only)
   lands after the gap patch. Safe-slice gating canary/enforce is HELD at operator level — this lane
   stays design-only; only zero-behavior-change shadow prep that falls out of Part 3 naturally is in
   scope.
