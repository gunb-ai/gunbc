# remote-build-client-closure-cost — design (2026-11-26)

Plan item assigned by clever-raven-401: the remote-build **client** entry resolves ~906 modules
(68 s / 3.66 GiB on a quiet-ish box; 832,637 ms / 905 modules on a contended host) before it does
anything. Same class as the serving-side per-leg resolve that gunb-ai/gunbc#17114 cut. Design only;
no code until the go.

## 1. Where the cost is, and with what instrument

Engine figures (`[pre-entry]` lines, the engine's own reporting — never my estimate):

| entry | closure_files | resolve ms | max_rss_kib | where read |
|---|---|---|---|---|
| `remote_build_serve` request arm | 898 | 143,992 | 3,435,568 | my box, 2026-11-26 |
| `remote_build_serve_front_door` (probe) | 150 | 9,751 | 374,128 | my box, 2026-11-26 |
| `remote_build_client` `remote_test_cli` | 906 | 68,193 | 3,663,224 | shadow-gate run 45774944663 step 7 |
| `remote_build_client` (claim lane) | 905 | 832,637 | — | calm-wren-314, contended host |
| `generated_artifact_gate` `main_wet_one` | 3,706 | 1,391,228 | 13,336,092 | my box, #17129 evidence |
| `hostname converge dispatch admission` | 1,585 | 301,548 | 6,084,572 | runs 45770097040, 45773631715 |

Static instrument: `/tmp/closure_bfs.py` — BFS over `module`/`import` lines of `.dag` files.
Calibrated against the engine on two entries: door BFS 147 vs engine 150 (−2%), client BFS 897 vs
engine 906 (−1%). So **percentages** from the instrument are trustworthy; wall and RSS figures are
quoted only from engine runs. `/tmp/import_mass.py` gives each direct import's closure and its
exclusive mass; `/tmp/prune_sim.py` re-runs the BFS with named modules treated as leaves (what the
closure becomes once that edge carries a leaf instead of a hub).

**Fact that shapes the whole design:** resolve is per **module**, not per function — both client
entries (`remote_test_cli`, `remote_gate_cli`) live in `remote_build_client.dag` and both pay ~900
files. The front door got to 150 not by being a different function in the same module but by living
in its **own** module. So the client's fix is its import list, not its entry split.

## 2. What the client demands vs what it drags in

| direct import | closure | excl. | what the client actually names |
|---|---|---|---|
| `gunbc.compute.test_run` | 894 | 1 | `snapshot_worktree`, `list_tree`, `show_file`, `test_exec` + 4 result types — 5 call sites, all local git (`remote_build_client.dag:107,637,751,789,831,834`) |
| `gunbc.compute.work_provider_local` | 883 | 0 | `ComputeExec`, `ComputeExecOk`, `ComputeExecFailed` — **3 names**, declared at `work_provider_local.dag` |
| `gunbc.compute.test_selection` | 106 | 0 | `select_witness_files`, `classify_witness_module` + 3 types |
| `gunbc.compute.remote_build` | 860 | 0 (146 once its peers go) | ~55 names: the whole request/answer/pin/candidate/placement vocabulary — the client is `remote_build`'s other half |
| `gunbc.fabric.fabric_executor_class` | 760 | 0 (35) | `ExecutorExecutionClass` + `executor_execution_class_roster`, which is a `data` literal at `fabric_executor_class.dag:210` |
| `gunbc.fleet.fleet_known_hosts_anchor` | 576 | 0 | real demand: leg kinds, `fleet_ssh_exec_key_leg`, `anchored_ssh_target`, `known_hosts_file_rows`, `create_or_adopt_owner_only_directory`. Its `fabric_reach` edge is not: it names **one** function, `fabric_rail_known_host_rows` (`fleet_known_hosts_anchor.dag:74`) |
| `gunbc.fleet.fleet_ssh_locus` / `fleet_ssh_credential_verify` | 578 / 577 | 1 each | 4 and 6 names — real demand |
| `extdeps.ssh.session`, `extdeps.git.object_store`, `extdeps.*`, `std.*` | ≤107 | — | the client's true floor |

The reach/enrollment chain behind the anchor:
`fleet_known_hosts_anchor` → `gunbc.spark.fabric_reach` (570) → `gunbc.fleet.fleet_host_key_enrollment`
(562) → `gunbc.auth.fleet_convergence_federation` (518, 153 exclusive).

And the host-side compute machine behind the "3 names" import:
`test_run`/`work_provider_local` (883) → `remote_build` (860) → `gunbc.typed_argv_exec` (587),
`fabric_executor_class` (760) → **`gunbc.runner.runner_slot_allocation` (759, 525 exclusive)**,
`gunbc.emitted_subject_build_gate` (703), plus `attempt_lifecycle` (314), `work_request` (313),
`roadmap_dashboard_instance` (292), `compute_attempt_slots_root`, `compute_work_records`,
`compute_producer_lease`, `materialization_store_budgets`. The requesting side of the route imports
the host's seat-census machine — which is why a client process can sit in the same code that refuses
srv3 requests.

**The mass is shared, so cuts pay only as a SET** (this is DESIGN.md:523 — "a number underdetermines
the chain. Targeting one axis yields a cache, batch, pool or knob at the measured link that moves
cost rather than removing it"):

| cut (simulated) | closure | saving |
|---|---|---|
| none | 897 | — |
| wrong-side trio `{test_run, work_provider_local, test_selection}` | 866 | −3% |
| `{remote_build, fabric_executor_class}` | 863 | −3% |
| **the five together** | **599** | **−33%** |
| five + `{spark.fabric_reach, fleet_host_key_enrollment}` | **425** | **−52%** |

Stated beforehand, as the rule asks: the multiplicity that collapses is the client process's
~900-file resolve; the route that disappears is the host-side compute/runner/roadmap machine inside
the **client**; what must stay equal is every client answer — probe endings, leg words, receipt JSON,
push/delete argv, and the witness lanes' tallies.

## 3. The proposal — same move as #17114, on the client's side of the wall

Four leaf extractions (each: the declaration **moves**, it is not copied; every importer re-points in
the same motion — *Single authority*, *Replacement migrations*):

- **L1** `gunbc.compute.compute_exec` ← `ComputeExec/Ok/Failed` from `work_provider_local`
  (27 importers re-point). The seat-census / attempt / slots machine then cannot be reached from the
  client at all.
- **L2** `gunbc.compute.worktree_snapshot` ← `snapshot_worktree`, `list_tree`, `show_file`,
  `test_exec` + their result types from `test_run` (4 importers).
- **L3** extend `gunbc.compute.remote_build_wire` (the 39-file leaf #17114 created) with the
  client↔host vocabulary the client already names (~55 names); `remote_build.dag` keeps the host-side
  admission/placement machine. The wire leaf is already the single home of the answer contract, so
  this grows an existing authority instead of opening a new one.
- **L4** `gunbc.fabric.executor_class_roster` ← `data executor_execution_class_roster` +
  `ExecutorExecutionClass` (19 importers); and `fabric_rail_known_host_rows` moves out of
  `gunbc.spark.fabric_reach` into a leaf the anchor can import (the other 9 importers of
  `fabric_reach` are untouched).

**Tier C, deferred with a stated reason:** the `fleet_known_hosts_anchor` / `fleet_ssh_locus` /
`fleet_ssh_credential_verify` trio (576–578 each) is the client's deepest remaining hub. Before I
promise a number for it I must check whether `anchored_ssh_target` and `fleet_ssh_exec_key_leg`
*call into* enrollment/federation at runtime or merely name their types: if it is only names, a leg
vocabulary + deadline leaf takes the client toward the door's 150; if it is calls, the client really
does need that machine and the honest answer is "not droppable here". I will not guess this one.

Target: 897 → ~425 static files (−52%). Against the measured pair (150 files → 9.75 s / 374 MiB;
898 files → 144 s / 3.44 GiB) that lands in the region where resolve wall and RSS fall several-fold;
**to be re-measured on the real entry, not interpolated**.

## 4. Witnesses, red controls, and the falsification I accept

- Reuse the door's instrument (`w_import_lines`, `w_source` in
  `dag/test/claim/compute/remote_build_serve_front_door_witness.dag:353-369`): one claim on the
  client module's import list — it must name the leaves and none of the five hubs, and an empty
  reading fails (`if count(door) == 0 || count(request_entry) == 0 { return false }`). Structural,
  not a second measurement, so it cannot drift into the CI-resolve node's lane (§5a).
- The client witness lanes must hold at the same tally (shadow-gate lane is 11/11 at
  `ff6176950fa`; the `remote_build_client` witness lane too), and the receipts must be
  byte-comparable before/after.
- **Red controls:** (i) re-add any one hub import → the claim goes red; (ii) restore `ComputeExec`
  in `work_provider_local` with the client importing it again → red; (iii) the resolve figure
  re-taken on the real entry must actually drop — a claim that still passes with the hubs reachable
  is not evidence, so the measurement is part of the control, per DESIGN.md:523 ("use measurement to
  falsify or confirm the derivation, never to choose the repair").
- Falsification I accept: if L1–L4 land and the re-taken `closure_files` does not move materially,
  my demand-graph reading is wrong and the item goes back, rather than the import claim being
  quietly weakened.

## 5. Overlap with `gunbc-entries-resolve-checked-in-ci`

Not in the tree yet (`git grep gunbc-entries-resolve-checked-in-ci` = empty), so I cannot read its
boundary — reading from its name and parent's framing:

- (a) **Same class, different layer.** That node makes every entry's resolve cost a *checked fact*
  (a required check that resolves entries and publishes `closure_files`/`max_rss`); mine removes work
  from one entry's demand graph. My claim must stay STRUCTURAL (which modules the client imports) so
  it does not become a second measurement authority next to its check — if it lands first, my row
  should become one of its budget rows rather than a parallel instrument.
- (b) **My figures are its seed data**: client 906 files / 68,193 ms / 3,663,224 KiB;
  generated_artifact_gate 3,706 / 1,391,228 ms / 13.3 GiB; `hostname converge dispatch admission`
  1,585 / 301,548 ms / 6,084,572 KiB (twice); serve request arm 898 / 143,992 ms / 3.44 GiB.
- (c) **Shared defect class, other owners.** The `hostname converge dispatch admission` step and
  `generated_artifact_gate` are the same shape (a control-plane entry paying a hub's whole closure to
  do one narrow job). If the plan wants one node rather than four, the general node is
  "an entry resolves only the closure its demands reach", with these as members.

## 6. Proposed roadmap node (text for parent to carry; operator holds #16859)

```
id: remote-build-client-closure-cost
intention: remote-build bandwidth   # sibling of remote-build-probe-front-door (#17114)
boundary: "The remote-build client entries resolve only what the requesting side demands. Pre: the
  client entry resolves 906 modules in 68,193 ms at 3,663,224 KiB max_rss (shadow-gate run
  45774944663 step 7), and 905 modules in 832,637 ms on a contended host; the client module's
  import list names gunbc.compute.work_provider_local (for three result types),
  gunbc.compute.test_run, gunbc.compute.test_selection, gunbc.compute.remote_build and
  gunbc.fabric.fabric_executor_class, which is how a process that runs three git commands and an
  ssh leg arrives at the host-side seat census, runner slot allocation, the materialization budget
  machine and the roadmap dashboard instance. Post: those five names are absent from the client's
  imports, the same declarations are consumed from leaves that hold them
  (compute_exec, worktree_snapshot, remote_build_wire, executor_class_roster, and a leaf for
  fabric_rail_known_host_rows), and the re-taken [pre-entry] figure shows the drop. Tier C (the
  fleet ssh hub) is admitted only after a call-graph read says whether the anchored target needs
  the enrollment machine at runtime or names its types."
approach: "Leaf extraction, not a cache: DESIGN.md, Minimize the demand graph before materializing
  its answers (gunbc.design_argument::demand_before_materialization) and Measurement is the
  adversary, not the selector -- the cut set is derived from the import graph (the mass is shared,
  so no single edge pays: 3% each alone, 33% as a set, 52% with the reach/enrollment pair) and each
  moved declaration keeps ONE home with every importer re-pointed in the same motion
  (gunbc.design_argument::single_authority, replacement migrations). Witnessed on the client
  module's import list with the front door's own instrument, and on the client witness lanes'
  unchanged tallies."
red_control: "Two arms, both required. (i) Re-adding any one of the five hub imports, or restoring
  ComputeExec in work_provider_local, turns the import claim red; a claim that passes with the hubs
  reachable is not evidence. (ii) The [pre-entry] resolve figure re-taken at head must move: the
  measurement falsifies the derivation if closure_files is materially unchanged, and then the
  derivation -- not the claim -- is withdrawn."
done-when: "Figures re-taken at head posted on the item, client receipts byte-comparable before and
  after, import claim green with its two red arms, and the overlap with
  gunbc-entries-resolve-checked-in-ci recorded so one fact does not get two instruments."
```

## Addendum, 2026-11-26 — the derivation above is wrong about the size of the prize, and it is withdrawn

Everything measured below is the static import BFS (`/tmp/closure_bfs.py`), calibrated against the
engine on this entry: engine 909 files at head `4ddcdf0b30a`, BFS 897 — so read BFS numbers as
engine + 12. Same tree, same binary, minutes apart. Two instruments were built for this:
`/tmp/cut_impact.py` (breadth that arrives ONLY through one edge) and `/tmp/entry_reads.py`
(which of an entry's imported names the entry's body actually reads, versus names it merely carries).

### 1. The client's closure is a dense core, not a stack of removable wings

`/tmp/cut_impact.py` says the eight heavy edges carry everything: delete all of them from the client's
import list and the closure is **1 file** (the client itself). That sounds like the opposite of dense,
but the per-edge cut says otherwise — an edge only pays when nothing else re-reaches what it reaches:

| cut | files saved | remaining |
| --- | --- | --- |
| client → `test_run` | 24 | 874 |
| client → `fabric_executor_class` | 0 | 898 |
| client → `fleet_known_hosts_anchor` | 0 | 898 |
| client → `remote_build` | 0 | 898 |
| client → `fleet_ssh_locus` | 1 | 897 |
| BOTH edges into `fabric_executor_class` (client's and `remote_build`'s) | **33** | 864 |

Zero, because the module is re-reached inside the closure: `fabric_executor_class` has two importers in
the closure (`remote_build`, the client), `fleet_known_hosts_anchor` has five (`typed_argv_exec`,
`fleet_ssh_credential_verify`, `fleet_bootstrap_principal_session`, `fleet_ssh_locus`, the client). So
the 3% / 33% / 52% figures in the derivation above were produced by `prune_sim.py`, which removed those
modules *and their whole subtree at once* — that is the cost of the client **stopping importing the
compile path**, which is not a leaf move at all. The largest leaf move on the table, splitting the
executor-class hub so both its importers stop reaching it, is **33 files of 897 — 3.7%**.

### 2. What the client actually reads confirms it: the heavy edges are its own demands

`/tmp/entry_reads.py gunbc.compute.remote_build_client`: of 27 imports, only four names are carried and
never read (`ExecutorExecutionClass` from the executor hub; `RemoteBuildWork`,
`remote_build_candidates`, `remote_build_admissible_hosts`, `RemoteBuildOperationDecode` from
`remote_build`; `shape_fleet_ssh_exec` from the anchor). Every other heavy import is read — the client
names `snapshot_worktree`, `list_tree`, `show_file`, `test_exec` (test_run), 60 names from
`remote_build`, `executor_execution_class_roster`, and the whole ssh-leg vocabulary
(`anchored_ssh_target`, `fleet_ssh_exec_key_leg`, `ServingProbeLeg`, `HostBuildLeg`,
`portable_remote_words`, `ssh_target_wire`). Deleting carried-but-unread names is a hygiene claim, not a
cost cut: the module is still imported for the names that are read, so it still pays its closure.

Consequence: **L1 as delivered cannot cut the client's closure, and did not** (+1 file: the leaf is one
more file to parse). Its real justification is the one `#17114` established — the requesting side names
the answer contract and the exec producer from a leaf, so the front door stays a door — not bandwidth.

### 3. The shape that does cut is the one `#17114` proved: a second, thin entry

Measured closures of candidate entries:

| entry shape | BFS files |
| --- | --- |
| `remote_build_wire` alone (the door's whole contract) | 39 |
| wire + `fleet_ssh_locus` + `extdeps.ssh.session` | **589** |
| the same + `fleet_known_hosts_anchor` explicitly | 589 (already in the chain) |
| `remote_build` + `test_run` + `test_selection` (the compile path) | 894 |

So a **client-side probe entry** — a leg that asks a host whether it serves, and does nothing else — answers
over **589 files instead of 897 (−34%)**, because it stops importing the compile path (`remote_build`,
`test_run`, `test_selection`, `work_provider_local`, the executor hub). That is the same motion as the
serve front door, on the other end of the wire, and it is the only cut of size in this entry.

The floor of that cut is one chain: `fleet_ssh_locus` → `fleet_ssh_credential_verify` (577) →
`fleet_known_hosts_anchor` (576) → `typed_argv_exec` (703), entered because the probe leg must shape and
run an ssh command against an anchored target. Going below ~589 means splitting that chain — its
five importers read 12–20 names each from the anchor (measured), so it is a real split and it is the
sibling node's territory (`fleet-converge-dispatch-admission-step-slim-entry`, crisp-pike-295), not
something to do silently from this row.

### 4. What this row should therefore chase, in order

1. **Client probe entry** (mirror of the front door): probe legs resolve ~589 files, not ~909. Witnessed
   the same way the door is — import-list rows plus the probe answer's behaviour, with the two red arms.
2. **Anchor-chain split**, jointly with the sibling row: the leg-words/wire half (what a leg must name to
   be shaped) separate from the credential/known-hosts materialization half. Unlocks the last ~440 files
   for both routes; needs a call-graph read, and it is the same read `Tier C` deferred.
3. Leaf extraction of the kind L1 is (roster leaf, snapshot leaf): **keep only where it buys a boundary,
   not bandwidth** — 33 files and 24 files measured. The row's red control said "the measurement
   falsifies the derivation"; here it falsified the derivation, so the derivation is withdrawn rather
   than the claim weakened. That is the whole reason the control has two arms.

### 5. Corrected node text (supersedes §6's boundary/approach; red_control and done-when stand)

```yaml
boundary: "The remote-build client pays for a closure it does not read on every leg. Pre: the client
  entry resolves 906 modules (68,193 ms, 3,663,224 KiB; 832,637 ms on a contended host). The closure is
  dense, not winged: deleting all eight heavy edges from the client's import list leaves 1 file, and a
  single edge pays 0–24 files because its target is re-reached inside the closure (the executor hub has
  2 importers here, the ssh anchor 5), so the largest available leaf move — splitting the executor hub
  for both its importers — is 33 of 897 files. Only four imported names are carried and never read.
  What pays is an entry that stops importing the compile path: the probe shape measures 589 files
  against the compile path's 894."
approach: "Second thin entry, not a cache and not a leaf move: a client-side probe entry in the shape
  gunb-ai/gunbc#17114 proved on the serving side — it answers over the shared wire contract
  (`remote_build_wire`, 39 files) plus the ssh-leg half it must actually run, and never imports
  `remote_build`, `test_run`, `test_selection`, `work_provider_local` or the executor hub. Derived with
  the import graph as the adversary (`cut_impact`, `entry_reads`), so the cut set is measured before it
  is attempted; each moved declaration keeps ONE home with every importer re-pointed in the same motion
  (gunbc.design_argument::single_authority, replacement migrations). The anchor chain below the 589 floor
  is handed to the sibling row `fleet-converge-dispatch-admission-step-slim-entry` with the measured
  importer counts, not cut from here."
```
