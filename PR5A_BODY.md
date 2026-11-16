# PR 5(a): remote-build serving-setup converger — model, value-comparing observe, fleet-converge observe mode

## What this is
The model + read-only **observe** arm of the remote-build serving-setup converger. It replaces
the hand steps performed on `srv1` on 2026-11-15: it derives, per roster host, the *desired*
state of the remote-build serving setup and reads the *observed* state over fleet SSH, then
reports the per-item drift as a **plan receipt**. It is **model-only + read-only observe — no
host is mutated in this PR.** The converge (apply) arm is **PR 5(b)**, which goes through the
fleet-converge plan→apply spine only after the manager's GO and the per-host plan receipts.

Stacked on **PR 4** (`session/proud-moth-338`, host-scoped ledger home model + roster **srv1-only**
per review 95791 — PR 5(b) admits the rest as each host's home is created and read back);
read its model surface (roster, `host_capacity_*`, the per-host build-pool carve) — this PR
consumes it.

## Desired state (derived, not literal) — per host
Authorities in brackets; no magic numbers restated:
- **group** `gunbc-compute` gid 1006, members = the host ledger's service user + the serving
  account. [gunbc.fleet.fleet_posix_accounts `fleet_posix_compute_ledger_group_name`/gid;
  service user from `host_capacity` area ownership]
- **account** `gunbc-build` uid 1005, gid 1005, supplementary `gunbc-compute`, login shell, not
  root, not in a root-equivalent group (the serve entry refuses otherwise). [fleet_posix_remote_build_user;
  the serve-time principal check in gunbc.compute.remote_build_serve]
- **ledger** `/opt/gunbc/compute/capacity` root + the two areas at `03770`,
  `<service_user>:gunbc-compute`. [gunbc.compute.host_capacity `host_capacity_home_directories`]
- **toolchain / cargo homes**, **checkout** `~gunbc-build/gunbc`, **serve binary**
  `<instance_root>/bin/gunbc`, **materialization store** `…/materialization-store` + `-staging`
  0755 `gunbc-build:gunbc-build`. [gunbc.compute.remote_build `remote_build_serving_instance`]
- **authorized_keys** `~gunbc-build/.ssh/authorized_keys` = exactly the rendered forced-command
  line for that host. [gunbc.compute.remote_build_serve `remote_build_authorized_keys_lines`]
- a host whose forced-command line is **unrenderable refuses** (never silently Met).

## Seed-binary freshness rule (declared)
The installed seed binary at `<instance_root>/bin/gunbc` is built from the admitted checkout for
**the host's own architecture** (srv1 aarch64; check srv2..srv4 per host). It is kept at a
**declared revision and must move with it**: `srv1`'s is still the `532fbb2` build and must not
be left behind when the admitted revision moves. The observe arm reads the installed binary's
presence; the freshness comparison against the declared revision belongs to the apply arm (5(b)).

## srv2 ledger switch
`srv2`'s move from its old per-instance ledger to `/opt/gunbc/compute/capacity` is an **apply**
concern → PR 5(b). It must occur when `srv2` has **no builds in flight** (its old in-flight
reservations do not carry over — PR 4 body concurs). Why the plan can act without a separate
in-flight probe: the ledger is switched by (re)creating the host-scoped `/opt/gunbc/compute/capacity`
areas; reservations are *content* under the ledger, not a precondition of the directory layout, and
the serve entry re-reads capacity at serve time from the current home — so the switch is to a
fresh, empty host-scoped ledger and an in-flight build on the OLD home would simply complete there.
The apply will therefore be dispatched for `srv2` only after its runner reports no running slot
occupancy (checked at apply dispatch time, not a model precondition). [Detail finalised in 5(b).]

## Readback: value-compared as a `std.goal_assessment::GoalAssessment` (unread ≠ absent)
`serving_observed_from_reads` builds a per-leg `ObservationAttempt` — each fact is **established**
(a value was read; for `test -d/-x` the exit-0 boolean, where `false` is a *confirmed* absence, and
for `authorized_keys` a `test -f` exit-1 confirms the file is genuinely absent) or **refused** (SSH
refused, `sudo -n` failed, an unexpected exit). `serving_assess` folds the established facts into
**deviations** and the refused legs into **unknowns** and calls
`std.goal_assessment::assess_by_deviations_and_unknowns`, so **unknowns win**: a host with an
unreadable leg is `GoalIndeterminate` and the observe **refuses** — it never reports drift an apply
would "write" against a fact nobody read (`gunbc.design_argument::fail_closed`; `std.goal_assessment`
paragraph ONE). The values are parsed from the raw reads — `/etc/passwd` (`':'`-split + `parse_int`),
`/etc/group` (`etc_group_entries`/`etc_group_entry_named`), `authorized_keys` (trimmed, exact) — so a
host whose account exists at the **wrong uid**, whose ledger group **omits the serving account**, or
whose `authorized_keys` carries a **stray key** is `GoalDiverged` (runnable drift the apply may act
on), while a **confirmed-absent** dir/key is `GoalDiverged` (drift), and an **unread** leg is
`GoalIndeterminate` (refused). This is the I-missed-it a presence-only reading would hide: it would
read Met / report drift for an unread host, and the serve entry would then refuse it at runtime — the
receipt-that-lies the design forbids. There is no second verdict vocabulary: the four arms are
`GoalAssessment`'s (`GoalSatisfied`/`GoalDiverged`/`GoalIndeterminate`/`GoalAssessmentRefused`).

## Observe mode
`RemoteBuildServingObserve` is a `FleetConvergeWorkflowMode` arm beside `RunnerHostFileObserve`
(its facts mirror it: `FleetSshKeyConsumed`/`ExecutorDomain`/shared-job-id, so the key-agent/WIF
gating derived from `ssh_key_demand` covers it). The run exits **success** on a settled (adopted) or
diverged (reportable drift) host and **failure** on an indeterminate or refused host — so a receipt
that could mislead an apply never exits green. **Main-only** — not added to the branch-ref
(run-scoped) route, so no IAM/WIF change. The per-host plan receipts are produced by a MAIN
dispatch of this mode after merge (manager sequencing).

## Generated workflow — regenerated in this PR
`.github/workflows/fleet-converge.yml` is **generated** by
gunbc.fleet.fleet_converge_workflow `expected_fleet_converge_yml`; `heal` does not regenerate
workflows and `drift_watch` only reports drift after merge. It is therefore **regenerated in this
PR** via `gunbc.instruments.generated_artifact_gate` `main_wet_one(path=".github/workflows/fleet-converge.yml")`
(the single-artifact projection of `main_wet`'s roster — chosen over `main_wet` so no unrelated
artifact is rewritten/resurrected), reflecting the new mode in the dispatch `options:`, the step,
and its gating. The receipt path is **repo-relative** (`target/remote-build-serving-plan-receipt.txt`),
matching the sibling `gunbc_ci_runner_host_file_converge_receipt_path` — the generated step cats
`"$ROOT/<path>"`, so an absolute path would double-prefix and miss after a successful observe.

### Pre-existing drift published by the whole-artifact projection (NOT this PR's change)

The artifact is projected **whole** (no partial projection exists), so this regen also publishes one
mode — its dispatch **step and its artifact-upload step** — present on **main's model** but **absent from
main's yml**: **`managed_host_network_install`** ("Managed host network install", `network_install_ci.dag`). That is **pre-existing drift** introduced on
main by commit **`041051dd7d31` / PR #16334** — *"PXE 2c: the one-shot network install workflow (first
unit: bobo / srv15)"* — which added the mode to the model but did not regenerate the yml (`heal` does not
regen workflows; `drift_watch` only reports). I am **keeping it** (per review: do not hand-delete a
generated artifact); it makes that lane's mode dispatchable on merge and is otherwise unchanged by this PR —
**the #16334 / network-install owner should be aware**.

## CI (v2-bootstrap)

`v2-bootstrap` (the required merge context) is a job in `witnesses.yml`, which triggers on
`pull_request` only for `base: main`. This PR is intentionally **stacked on** `session/proud-moth-338`
(PR 4, unmerged) per the manager, so GitHub's required-context `v2-bootstrap` does **not** fire on the
stacked head yet — it runs when the PR is **retargeted to main after PR 4 merges**. As an early,
**non-required** compile signal in the meantime I dispatched `witnesses.yml` on the head ref:
**run [`43804460269`](https://github.com/gunb-ai/gunbc/actions/runs/43804460269)** (workflow_dispatch,
`v2-bootstrap`). The whole-closure reconcile is additionally proven by the regen above (`main_wet_one`
loaded + reconciled the full graph, exit 0).

## Evidence (run locally with the existing seed binary; NOT a local release build)
Both claim witnesses green locally (`--claim-run`, exit 0).

- **setup witness** `dag/test/claim/compute/remote_build_serving_setup_witness.dag` — **18/18 PASS**. Desired-state
  derivation + the `GoalAssessment` judgement; ADOPT (a matching host settles as a no-op); the
  value drift controls (wrong uid / group missing the account / stray authorized-keys line → `GoalDiverged`);
  the **central red control** (an unread SSH/sudo leg → `GoalIndeterminate`, the run refuses — *not* drift an
  apply would write) and its **paired control** (a confirmed-absent dir/key is still `GoalDiverged` drift, so
  real drift is not collapsed into indeterminate); the unrenderable-host `GoalAssessmentRefused`; and the
  roster-decoupling control (a not-yet-rostered candidate still renders distinct per-host lines).
```
PASS srv1_desired_account_is_the_declared_principal
PASS the_serving_account_is_not_root
PASS srv1_ledger_is_shared_with_its_service_user
PASS the_two_ledger_areas_are_service_user_shared_with_the_compute_group
PASS srv1_serving_paths_match_the_serving_instance
PASS every_pr5_candidate_host_renders_a_forced_command
PASS a_host_outside_the_candidate_row_has_no_forced_command
PASS a_matching_host_is_adopted_as_a_settled_assessment
PASS wrong_uid_diverges_the_account
PASS group_without_the_account_diverges
PASS authorized_keys_with_a_stray_key_diverges
PASS confirmed_absent_authorized_keys_is_drift_not_indeterminate
PASS confirmed_absent_store_dir_is_drift_not_indeterminate
PASS an_unreadable_passwd_is_indeterminate_not_drift
PASS an_unreadable_store_leg_is_indeterminate
PASS the_receipt_names_indeterminate_never_drift_for_an_unread_leg
PASS a_host_outside_the_candidate_row_refuses_the_assessment
PASS a_not_yet_rostered_candidate_still_renders_a_forced_command
```
- **converge witness** `dag/test/claim/compute/remote_build_serving_converge_witness.dag` — **3/3 PASS** (forces
  the observe entry to compile; proves the receipt carries the host/probe header + terminal state and
  **leaks no authorized_keys forced-command material**).
```
PASS the_receipt_body_names_the_host_and_the_drift
PASS the_receipt_body_leaks_no_forced_command_material
PASS a_converged_host_receipt_reports_settled
```
- **per-host plan receipts** — from the main dispatch of `RemoteBuildServingObserve` (post-merge).
- **probe** per host (after 5(b) apply + GO): `ssh -i ~/.ssh/gunbc-remote-build gunbc-build@<anchored addr> -- remote-build-v1 probe` → `remote-build-v1 ending=ready`, exit 6.

## Roster admission (review 95791)
PR 4 keeps the **live** roster `remote_build_serving_hosts` at **srv1 only** (admitting a host before
its `/opt/gunbc` home/store/account exist is fail-open). This PR therefore does **not** roster
srv2/srv3/srv4. The OBSERVE arm covers not-yet-rostered hosts because it takes its candidate set
from a **PR 5-owned** row `remote_build_serving_candidate_hosts` = [srv1..srv4], not the live roster.
**PR 5(b)** grows the live roster per host only after that host's apply created + read back its
home, store and account (srv2 via an atomic `replacement_migration` for the `/home`→`/opt` switch,
dispatched when srv2 has no running slot occupancy).

## Not in this PR
Converge/apply (PR 5(b)), any host mutation, butch/bobo/new-Supermicro (follow-ups). Not merged
by me; awaiting review + `v2-bootstrap` green.
