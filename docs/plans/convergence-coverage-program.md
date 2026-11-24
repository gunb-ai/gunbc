# Convergence Coverage Program — production convergence-path roster

Roadmap node: `cc-path-census` (lane: v2 self-host and native emission; planning
owner still-gull-398; manager warm-fox-517). This file is the program carrier the
ticket names; the roster itself is `dag/gunbc/convergence_coverage_roster.dag`
(module `gunbc.convergence_coverage_roster`) and it is the denominator the eight
`cc-*` leaves consume.

## What this node delivers (handback)

1. **The roster** — `gunbc.convergence_coverage_roster.roster_rows()`: one row per
   production convergence path over the corpus population that references the
   substrate `gunbc.ensure` or `std.realization_reconcile`.
2. **The identity-join later leaves read** —
   `census_identity_join(required, rows)`, keyed on the route
   `DeclarationRef { module_path, decl_name }`, mirroring
   `host_convergence_census.census_identity_join`. It enforces presence of the 20
   derived fold-site routes.
3. **A planted omitted path that reds the census** —
   `convergence_coverage_census_red_control()` and `planted_omission_is_caught()`:
   removing `gunbc.oomd_install.reconcile_oomd` from the roster reddens the census
   with the identical `IdentityMissingFromCensus` signal. A census that cannot be
   reddened by a planted omission is not a census.

## Row unit (guard one)

A row is a PATH, identified by `(module_path, function, operation)`, not a file.
The identity-join key is the route `DeclarationRef`, not the file path. Row
count: **75 rows** — **20 enforced** substrate fold-site paths (over **13** files),
**3 pre-substrate** operator-review first-slice folds, and **52 named exclusions**
(2 substrate + 5 plan/census carriers + 45 consumers). The 20 enforced paths sit
in 13 files, so the file count and the path count differ and both are reported.

## The four honesty conditions

1. **The denominator is re-derivable, not a trusted number.** The enforced
   population is the production paths that *invoke* a substrate fold-site
   function (`ensure_decide`, `ensure_reconcile`, `ensure_after_grounding`,
   `ensure_observation_as_attempt`, `reconcile_grounded`, `reconcile_effect`,
   `reconcile_effect_without_read`, `reconcile_application_then_ground`). The
   derivation command is embedded in the roster module header. The importer grep
   over `--include=*.dag` for `gunbc.ensure` / `std.realization_reconcile` with
   the `test/claim`, `recurring_failure_mode` and `integration_concern`
   exclusions yields 65 files; the fold-site scan yields the 20 enforced paths.
2. **Raw derivation output with its real exit status** is pasted in the PR body
   as the denominator evidence (not the number alone).
3. **All 65 files stay visible.** The 45 consumers, 5 carriers and 2 substrate
   modules each carry a named-exclusion row with its own reason and the wrapper
   its fold composes through (or the substrate symbols it references). Enforced
   vs excluded is a difference in what the completeness check *refuses on*, not a
   licence to drop a row. Row count in the handback reads enforced 20, named
   exclusions 45 + 5 + 2.
4. **The join under-claims honestly.** It enforces presence of the 20 fold-site
   routes and names what it cannot reach.

## What the join cannot reach (guard three)

This join cannot enforce a completeness discovery join over independently
enumerated host-effect declarations so that a missing *wrapper-mediated* fold
refuses: that join is `SubstrateInputsOnly`, the same limitation
`host_convergence_census` records in its NEXT-RUNG TRIGGERS ("a discovery join
over independently enumerated host-effect declarations so a missing mutation
identity refuses" is not authorable on the current path). A fold that composes
entirely through a cross-module wrapper without importing a substrate fold-site
is therefore an acknowledged blind spot of the enforced join, named here rather
than hidden. The 45 consumer rows name the wrapper (typically
`gunbc.host.host_effect_realize.host_effect_apply` / `host_effect_apply_gated`,
`converge_apply_for_host`, `deployment_effect_plan`, `membership_reconcile`)
rather than asserting the roster is complete over folds the join cannot see.

## D14 section 10 (classified against; never pasted into a row)

The D14 ensure-authority plan file `dag/gunbc/plans/ensure_closure_design.dag`
was deleted in commit `970754b2577`. Its section 10 "Census: what this replaces
on main" is the *On main / Becomes* target list the red control names as NOT a
census row ("A row that restates D14's target list instead of naming a corpus
route is not a census row."). No row restates it. Where a row traces to D14 it
cites the **live ruling text** at `dag/gunbc/plans/demand_engine_program.dag`
line 264, not the deleted table.

## Pre-substrate finding (first-slice)

The operator-review first-slice folds the ticket names — `srv3_ensure_apt`
(`gunbc.host.host_effect_realize`), `ensure_host_tool`
(`gunbc.runner.runner_microvm_network_apply`), `converge_typed_remote_file`
(`gunbc.typed_remote_file_write`) — live in modules that **do not import the
substrate today**. They are pre-substrate folds D14 names for migration, so they
are classified supplied-versus-established anyway and marked `PreSubstrate`, but
they are **not** in the enforced fold-site population. That the first-slice named
folds are not yet in the enforced population is a finding, not an embarrassment.
The fourth first-slice fold, the `RunArgv` / `RunArgvStdin` / `RunArgvOutcome`
family, is Rust substrate reached from the corpus route
`gunbc.host.host_effect_realize.reconcile_shell_exec_observed`, which is an
enforced row.

## Supplied vs established

`EstablishedOperand`: removing the prerequisite-producing ensure starves a
downstream consumer of a required operand (e.g. an `EnsurePlan` the apply leg
consumes, a `HostToolPrerequisite`, a written remote-file operand).
`SuppliedReport`: the fold only reports a status a reader displays or logs and
does not consume as a structural operand (e.g. liveness/reap reconciles,
capability/pending verdicts). Each enforced row carries its rationale; the
covering `cc-*` leaf confirms it once bound.

## Execution contract

Until the roster is bound by a claim process, `ExecutionContractUnspecified` is
the honest state (ticket centering point 5). The census-completeness claim
`convergence_coverage_census_is_complete()` and the fail-closed control
`convergence_coverage_census_red_control()` are executed over the corpus with
`claim_batch` on srv1 under the shared heavy-slot limiter; the raw PASS/FAIL
lines, the real exit status, the binary `sha256` from disk, and the srv1
`HOST`/`BOOT` compare are recorded in the PR body. The `node-digest` criteria
digest is reported UNKNOWN (the lab revision it computes at, c48e06d8, has a
failing self-test) unless a block-gate IDENTICAL result against main is shown.
