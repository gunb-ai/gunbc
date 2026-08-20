# Work-elimination alpha: the measurement cannot be taken, and here is the proof

Measured 2026-08-20 by `silent-bear-842`. **This is a report that a measurement is not
available, established three ways.** It is the honest state of the lane, not a failure to close.

`strategy.ci_alpha` `work_elimination_alpha` (private repo) carries the obligation: *"Instrument
one real repository through both paths and record baseline unit-minutes against admitted
unit-minutes with a receipt. Until that exists this is the most valuable claim in the model and
the least evidenced."* Its durability line: *"Strongest of the seven if it holds, because it is
the only one a competitor cannot buy."*

> **We could not have priced the one advantage a competitor cannot buy even if we still had it,
> because the path we run does not measure itself — and the mechanism that would select has no
> caller.**

## What is firmly established

### Baseline, entries — an identity, not an observation

**9,776 witnesses execute on every run.** This is not a measurement of a policy. The fold has no
selection concept to configure — DESIGN states it outright: *"no plan entry, no plan function, no
batch id, no worker role, no selection flag."* The whole discovered roster runs unshrunk **by
construction**, per the 2026-08-13 operator directive.

### Baseline, wall — our floor, n=20

Last 20 **successful** main runs of `witnesses.yml`:

| n | min | p50 | p90 | max | mean |
| --- | --- | --- | --- | --- | --- |
| 20 | 26.5 | **28.3** | 31.3 | 31.6 | 28.9 min |

**Three limits, attached so this cannot be promoted past what it is:**

1. **One job on one repository — ours.** Self-hosted arm64, warm toolchain, a 9,776-witness
   whole-corpus fold. This is *our floor*, almost certainly at the heavy end of any distribution
   of "a CI job". It is **not** a measurement of a customer's job, and it must not be used to
   fill `typical_job_minutes`, which is a planning figure about customer jobs and stays
   unmeasured with its obligation intact. An honest `-1` beats a confident wrong subject.
2. **Wall-clock of the whole job**, including checkout and `cargo build` — not unit-minutes of
   billed compute. Those differ, and unit-minutes needs a unit definition the model owns.
3. **Successful runs only.** A cancelled run's duration measures when someone pushed, not what
   the job costs.

A scenario that follows and is explicitly **not** a measurement of customer jobs: if a customer
job resembled ours rather than a six-minute figure, **two** fit in a billed Hetzner hour rather
than ten, and multiplexing headroom is half an hour per paid hour rather than fifty-four minutes.

### Admitted — not computable without a code change

Three routes, all closed, each verified rather than assumed:

| route | why it is closed |
| --- | --- |
| the `.dag` authority | `entry_affected_by_touched_paths` is **per entry**, and each call runs `dependency_closure_live_excluding` — the ~100s, multi-GB, OOM-class walk its own note warns about. ~1,300 entries × N ranges is days, not a dispatch. |
| the Rust twin | `entry_file_touched_via_import_closure` and `compile_clean_scope_plan_for_ci` are **private `fn`s with zero references anywhere under `src/v1/stage0/src/bin`**. No CLI flag reaches them. |
| the public surface | The only `pub` entries into that chain — `witness_layer_roots_compile_clean_check` / `_emit_check` — return **`Bool`**. They answer *is it clean*, never *which entries would be selected*. |

## Why unit-minutes are unavailable for **both** arms

Not just the admitted one — that correction matters, because it was previously held as an
admitted-arm problem.

- `write_witness_row_cost_receipt` is called **once**, at `claim_executor.rs:8690`, inside the
  batch walk and fed by batch records.
- `run()` **returns early** at `claim_executor.rs:10513` into `run_required_floor` when
  `--required-floor` is set, so the batch walk is never entered on the path CI runs.
- `run_required_floor` contains only **phase-level** timing — prepare, index warm, shared index
  warm, published, projection — and **no per-claim duration and no TSV write anywhere**.
- `RequiredFloorOutcome` carries counts and name lists. **No duration fields.**
- The floor log times only failures and slow rows: **1,022 lines for 9,776 witnesses.**

**The log-parse route was deliberately not computed.** Those 1,022 rows are selected *for being
slow*; summing them and calling it corpus cost is survivorship bias pointing the expensive way —
it would inflate baseline unit-minutes, which inflates the alpha, which is the direction we would
want the answer to go. That is the most dangerous property a wrong number can have here, and a
figure that exists gets quoted.

## The finding: one cut, three consequences

| | |
| --- | --- |
| **unexercised** | affected-set selection deleted at the root; the whole roster runs by directive |
| **unproduced** | no per-claim duration on the required-floor path |
| **unreachable** | the selection authority is live library code with no caller, no flag, and a `Bool`-only public surface |

The alpha went dark and the light that would have shown it went dark in the same motion — which
is why no gap was visible for anyone to notice.

**A note on the `Bool` surface**, because it is a known defect class in that exact function
rather than an artifact of this investigation: compile-clean check returning `Bool` collapses
*refused*, *file-not-found* and *content-mismatch* into one value. Asking it for a count is
asking it to stop discarding a discriminator it already computed — the stronger form of the
request.

## Carrier status

`selected_software_execution_alpha` **stays `WorkAvoidedUnmeasured`**. A filled carrier with a
soft receipt is worse than an honest `-1`, and `work_avoided_permille` divides admitted by
baseline — so feeding it entry counts would produce a confident per-mille from the wrong
quantity.

**Sharpened obligation**, which is strictly better than the one it carries because it is
actionable: the floor path carries no per-claim duration and the selection authority has no
counting surface, so **neither** arm is measurable in unit-minutes today.

## What would unblock it — one item, two additions

Both in `claim_executor` / `cli_run`, both additive. Neither restores CI behaviour, neither
enrolls anything, neither touches the emitted workflow.

1. **Instrument the required-floor path with per-claim duration**, carried on
   `RequiredFloorOutcome` and written to the receipt the batch path already knows how to write.
2. **Give the surviving selection authority a counting entry point** that reports which entries a
   touched-path set reaches, instead of collapsing to `Bool`.
