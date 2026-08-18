# The floor's shared-fill ledger — measuring what a slow witness actually costs

gunbc#8455 established that the floor's per-row wall time is not always the row's cost, and
named the repair it did not perform: *attribute shared cost to the entry rather than its first
row*. This is that repair, plus the mechanism behind the defect, root-caused to source.

## The mechanism, named

The floor builds a **fresh evaluation frame per claim**, so the sharing #8455 measured is not an
eval memo — a per-claim memo cannot survive between claims. It is a population of
**process-global corpus-scan caches in the seed**, each filled at most once per process and read
free thereafter:

| cache | what the fill is |
|---|---|
| `reference_edges` | one O(corpus) tokenize+parse pass over the whole pool |
| `module_graph_facts` | the live module graph over the pool roots |
| `module_path_index` | module → path over the source roots |
| `inert_carrier_data`, `complexity_linearity_audit`, `complexity_linearity_wildcard`, `fallback_arm_census`, `non_fold_residue`, `doc_graph_report`, `test_migration_behavior_discovery`, `test_migration_debt` | whole-corpus census reports |

The #8455 specimen resolves exactly onto the first row. `g2_data_reference_under_selection`
calls `specimen_projection` → `v2.lens.live_read_classification`
`live_read_selection_manifest_live` → `dependency_resolution_facts_live`, whose seed realization
`cli_run.rs` `reference_resolution_facts` performs that whole-pool parse behind
`REFERENCE_EDGE_CACHE`. The first claim pays the pass (12197ms); the second reads the map
(31ms). Same call, 393x, 35ms apart.

## Why the existing number cannot answer the paring question

Nothing is computed twice — the caches are correct, and the sharing is the saving. What is wrong
is the **attribution**, in the direction that matters, because the ceiling refuses per row:

- A refusal names the **first toucher**, whose own cost may be milliseconds.
- **Splitting** such a row moves the charge to whichever fragment runs first
  (`gunbc.witness_row_cost` `witness_decomposition_does_not_reduce_entry_cost_note`, observed
  rather than predicted).
- **Removing** such a row does not remove its cost either. The next claim to touch the same fill
  pays the same seconds — so a quarantine decided on the per-row number can trade a named
  refusal for an unnamed one in a module that was fast before.

## What the ledger records, and the quantity it yields

`cli_run/shared_fill.rs` records, per fill: the cache, the key it is declared on, what the fill
cost, which claim paid it, and every later claim and module that read it. `[floor-shared-fill]`
lines are emitted once at the end of the fold. The disposition on each line is derived by
`gunbc.witness_row_cost` `shared_fill_disposition` from the payer and the consumer breadth —
three states because three remedies:

- `outside-fold` — preparation's cost. No witness can be pared to recover it.
- `exclusive` — one module consumed it. It is removable with that module.
- `shared` — more than one module consumed it. It survives any single removal.

From that, joined against the `[floor-witness-slow]` rows the floor already prints:

```
what removing module M saves  =  M's rows' wall time
                              -  fills M paid that are `shared`
                              (+ nothing for `outside-fold` fills at all)
```

## What this is not

It refuses nothing, skips nothing, and changes no verdict; it prints a table (DESIGN §5's
sanctioned stopped-line audit — it reports, it does not green). It does not make any witness
faster, and it takes no position on which rows should be pared. It measures the quantity that
decision needs and that nobody currently has.

`unattributed_hits` on the total line is the honest bound: reads served by a fill this ledger did
not observe. A nonzero count means the shared figure is a **lower** bound on the sharing, not the
whole of it.

## Two bounds on reading the output

**Run-to-run variance is real and is not this ledger's subject.** `where_refinement_cast_literal_oci_other_digest_algorithm_accepts` was measured at 1505ms, 1274ms and 1364ms — the first two on *byte-identical* input (a rerun of one frozen merge ref) — an ~18% spread straddling a 1500ms ceiling (sharp-raven-273, 2026-08-18, runs 32155779798 / 32153078487). So for a row within ~20% of its ceiling, which side it lands on is partly a fact about the runner. A fill difference smaller than that is not readable from one run.

**A right-censored row cannot be decomposed.** Exclusive-vs-shared is a subtraction from a
*completion* cost, so a row whose figure is its interrupt point carries no cost to decompose. On
main run 32177951514, 84 of the 97 budget-refused rows sit pinned at 1552–1560ms and must be
excluded from any ranking; the deadline is cooperative (polled every 4096 evals), so the other 13
ran past it and carry real overshoots — up to `grounding_lens_whole_tree` at 21868ms. The
`[floor-witness-slow]` channel is the better census (720 rows, all carrying measured elapsed), on
the same condition: its ~85 rows in the pinned band are censored, the rest are measurement.

**The signature to look for, and the one that refutes the model.** A shared fill divided among
consumers looks like *one large row and cheap siblings* — the g2 pair's 12197ms then 31ms. Several
siblings all landing at roughly the *same* cost is the opposite evidence: nothing was amortized,
so each is paying its own way. The eight `effect_reach_test` rows clustered at 1788–2134ms are
therefore a prior *against* shared fill in that module, and the ledger will say so directly rather
than by inference.

## Receipt — run 32192150969, 9425 claims, one fold

19 fills. The run is main's inherited red (`failed=0 stale_quarantine=5 budget_refused=102`),
not this diff's; the ledger reported from a complete fold.

**Read `fill_ms` as SELF time.** The first receipt exposed a defect in this instrument and it is
fixed in the same PR: these caches compose, so an outer fill's wall contains its inner fills'.
A 17890ms `module_graph_facts` fill contained a 12054ms `module_path_index` fill and a 5141ms
`reference_edges` fill — ~695ms of its own. The first receipt's `TOTAL fill_ms=302362` was an
inclusive sum and over-counts; the corrected total over self time is **~180s**, and the
per-fill payer/consumer/disposition columns were never affected.

### The finding: one row's entire cost is a fill 29 other modules read

```
cache=module_path_index  key=dag+src/v2   fill_ms=51508
  paid_by  test.claim.dissolution_census_witness_test.unbound_dissolution_empty_literal_refuses
  read by  139 claims across 29 modules            disposition=shared
```

That payer is one of the five rows in the corpus with an *exact* cost: **51620ms**. The fill is
51508ms of it. So **99.8% of that witness's measured cost is a computation 29 other modules
consume**, and it is the only `shared` fill in the run. Removing the witness recovers ~112ms and
hands 51.5s to whichever of the other 29 modules runs first. This is the paring question answered
at identity grain, on the row where it matters most.

### And the opposite finding, which matters just as much

The 86741ms row — `extdeps_scope_placement_gate_loudness_witness`
`red_seed_runner_failure_detail_projects_located_receipt`, the most expensive exact row in the
corpus — paid a 29017ms `module_graph_facts` fill and a 24965ms `reference_edges` fill, **both
`exclusive`**: no other module reads either. Its cost is genuinely its own, and it is a real
paring target. Two rows adjacent on a slow list, opposite answers; nothing in the per-row wall
time distinguishes them.

### Preparation's share, which no paring can recover

Three fills totalling ~108s inclusive are `outside-fold` — a 55392ms `module_graph_facts`, a
35060ms `module_path_index`, a 17646ms `reference_edges`, all paid before any claim ran. That
cost belongs to preparation. A quarantine of every witness on the floor would not move it.

### The prediction, confirmed on an experiment nobody had to build

When #8457 quarantined 102 over-budget witnesses, this model predicted the cost would not leave
with them — the next module to touch the same fill would pay it. Main before (run 32177951514)
against main after (run 32193032348), **one module, one run, one tree**:

```
single_refinement_carrier_emits_no_unsupported_cast    746ms -> 45941ms   61.6x
nested_refinement_carrier_deficit_remains_observable   728ms ->   763ms   flat
```

Nothing about either row changed; 38 modules were removed from the fold ahead of them. If the
45.9s were the row's own work it would have been 45.9s before. If it were a general slowdown the
sibling would have moved too. It is the first-touch signature exactly: the fill that row now pays
was previously paid by one of the quarantined 102, and the sibling still rides it free in both
eras. **Quarantining a first toucher relocates the bill; it does not retire it.** (Measured by
tidy-lark-471. It landed in the failures channel rather than the refused one, which is why a scan
for new refusals read the run as refuting the prediction.)

### What `exclusive` does and does not establish

`exclusive` means **no other module in the executed population read this fill on this run**. It
does not establish that no other module ever would. If the payer were removed, a would-be
consumer could pay the same fill next run — the first-toucher trap one level up, and the ledger
cannot see it from inside one run.

Two things bound how far that doubt reaches. The `key` is the fill's *declared inputs*, so an
exclusive fill on a key no other consumer requests is exclusive by construction rather than by
luck — the 29017ms `module_graph_facts` above is keyed on `dag+src/v2` roots that the shared
51508ms `module_path_index` fill also serves, while the 17890ms one is keyed on a four-root pool
only its own module asks for. And the answer is decidable by execution rather than argument: run
the fold with the payer absent and read the ledger again. Until that run exists, an `exclusive`
row is one run's observation, and a recovery estimated from it is an upper bound.

The distinction has a second edge worth stating, because it corrects a reading of the same rows
from isolation. A witness whose SUBJECT is the corpus is not made cheap by shrinking the corpus:
the 71s row measures 70–85ms in a 67-module closure and 71060ms against the floor's 2376-module
subject, with nothing about the witness changed. Isolation is structurally unable to price that
class, which is why the ledger measures on the floor path (quiet-ibex-39, 2026-08-18).

`unattributed_hits=1`: one read was served by a fill this ledger did not observe, so the shared
figure is a lower bound. One is small; it is reported rather than rounded away.
