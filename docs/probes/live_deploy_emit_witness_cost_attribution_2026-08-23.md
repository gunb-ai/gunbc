# Where the live_deploy.emit witnesses spend their 5000ms

**Subject:** the five `test.claim.live_deploy.emit` witnesses reported
INTERRUPTED-BEFORE-VERDICT against `required_floor_claim_cpu_safety_limit_ms`
(`v2.workflow.required_floor`). Their verdicts are unknown — not passing, not failing.

**Instrument:** `claim_batch` with `GUNBC_INTERP_PROFILE=1`, which prints per-witness
thread-CPU and total interpreter node-evals. Local arm64 session container, release build
of `v1-compiler`. Every number below is executed, not argued. Standalone claim-runs are
2–3× off the floor's own figures (memo disabled, no shared fill), so the RATIOS below are
the finding and the absolute milliseconds are indicative — the CI figures in §1 are the
enforcement-basis ones.

## 1. What CI measured

Floor run `32621117917` (main, `13db52a25d`, 2026-08-23T05:44Z) reported
`interrupted_before_verdict=1` and printed `[over-cost]` lines for **sixteen**
`test.claim.live_deploy.emit` rows between **3512ms and 4826ms CPU** against a 5000ms
limit — the whole module sitting at 70–96% of the cap, with the tail crossing it:

```
witness_apply_script_contains_systemd_and_tailscale        cpu_ms=3538  pass
witness_scripts_stay_within_ci_runner_grants               cpu_ms=4472  pass
emitted_twin_scripts_touch_none_of_productions_effects     cpu_ms=4684  pass
twin_belt_timer_actuates_its_own_tree_not_productions      cpu_ms=4826  pass
twin_and_production_configure_disjoint_tailscale_endpoints  BUDGET-REFUSED, Cpu >= 5001ms
```

WHICH rows cross is therefore scheduling noise on a population that is uniformly near the
line. Reporting "five" or "one" is a property of the run, not of the module.

## 2. The cost is the emission, and the emission is the pipeline BUILD

Splitting `live_deploy_apply_script_for` by stage (one probe module, one `claim_batch`
resolve, four `test fn`s):

```
stage                                          node-evals     CPU
deployment_apply_plan (membership reconcile)        4,651     39ms
membership_effects                                  4,209      6ms
apply_intent_from_effects (BUILD the Pipeline)  1,940,780   3205ms
orch_emit_pipeline (JOIN the built steps)         108,000    ~230ms
full live_deploy_apply_script_for               2,049,505   3341ms
```

A control witness in the same module that touches only the spec
(`witness_spec_listen_port_is_8080`) is **1,088 node-evals / 33ms**. So the witness
assertions cost nothing; 95% of every one of these sixteen rows is
`apply_intent_from_effects`, and the orchestration join — the part that looks like an
accumulator — is 5%.

Within the build, two roughly equal centres:

```
deploy_memory_cap_apply_steps        891,313 evals   1502ms
per-member emit_deploy_member_effect 1,145,290 evals 1843ms
deploy_apply_preamble_steps              650 evals      2ms
```

## 3. The unit cost: ~6.7k node-evals per emitted shell WORD

Serializing statements through `gunbc.shell_command_text` `shell_command_text_of_stmts`
(→ `v2.workflow.bash_command_fold_serialize` `bash_fold_serialize_node`), holding
everything but one variable fixed:

```
DISTINCT statements (5 words each)     1 -> 98,907   2 -> 119,952   8 -> 246,270   16 -> 414,694
  => ~78k fixed for the first serialization in a claim, then ~21,053 evals per statement

words per statement (8 statements)     1 -> 180,537   5 -> 391,969   10 -> 663,929
  => ~6,714 evals per WORD, linear in word count

characters per word (8 stmts x 5 words) 4 -> 391,969   40 -> 391,969
  => flat. Word LENGTH is free; word COUNT is the whole cost.

IDENTICAL statements                   1 -> 98,902   16 -> 99,446
  => ~36 evals marginal. The eval memo collapses a repeated node completely.
```

So the emitted apply script's ~80 distinct shell commands, at ~21k evals each, IS the
2.05M. There is no defect in `gunbc.live_deploy.emit` producing this; the module emits
about the number of commands a deploy of this shape needs, and each one costs what the
v2 grammar-directed backward emit costs.

## 4. Why it crossed the line THIS WEEK

gunbc#8976 (2026-08-23, the CPU admission axis) took srv1's committed runner-slot width
from **5 to 21**. `deploy_memory_cap_apply_steps` emits one `systemctl revert` per runner
slot unit, so the emitted block went from 5 revert commands to 21 — sixteen new commands
at ~21k evals each.

Measured across two CI runs of the same witness rather than inferred:

```
run 32612922091 (2026-08-23T02:27Z, before #8976)  witness_apply_script_contains_systemd_and_tailscale  cpu_ms=3127
run 32621117917 (2026-08-23T05:44Z, after  #8976)  same witness                                         cpu_ms=3538
```

+411ms on one witness, on a population whose top rows had ~300ms of headroom. That is the
whole event: nothing regressed in the emitter, the fleet got wider and the script got
longer.

## 5. One hypothesis raised here and FALSIFIED, recorded so it is not re-raised

`deploy_memory_cap_apply_steps` computes `script = deploy_memory_cap_apply_script_for_host(host)`
and then gates on `deploy_memory_cap_script_author_precedes_shadow_revert(script: script,
units: units)`, whose body re-derives the same ordered script and compares. That reads as a
second construction of a 26-command block — DESIGN §5's validation-standing-where-construction-
was-available, at full price.

It is not, and the reason matters for anyone else reading a cost profile here: the
re-derivation is **memo-collapsed and free**. Its inputs are `deploy_runner_memory_cap_author_stmts()`
(nullary) and `deploy_memory_cap_shadow_revert_stmts(units)` (same units), so the second
`shell_command_text_of_stmts` call hits the memo. Deleting the conjunct was implemented and
measured: `deploy_memory_cap_apply_steps` stayed at **891,313** evals and the witness stayed
at **3184ms**, against 891,313 / 3181ms before. Zero change, so the edit was reverted.

The general form: on this substrate a duplicated pure derivation is not evidence of
duplicated cost, and a cost claim about one must be measured before it is asserted.

## 6. What would actually move the number

The root is the per-word cost of the grammar-directed backward emit
(`v2.std.grammar` `formal_production_unique_lhs_exact_match` and the
`v2.extdeps.languages.bash_command_fold` leaf path around it), which is
`src/v2` compiler surface — a load-bearing file under DESIGN §4/§7, not this module's to
improvise against under a witness-cost brief. Nothing inside
`gunbc.live_deploy.emit` or `test.claim.live_deploy.emit` reduces the command count
without deleting coverage, and the two most expensive rows are inherently comparative
(a twin-vs-production disjointness claim needs both scripts by construction, so it cannot
be split into cheaper claims).

**Next-rung trigger for this class:** per-word emit cost in the bash fold serializer. Until
it moves, the live_deploy.emit population's headroom is a function of fleet runner-slot
width — at ~21k evals (~30ms CI) per emitted command, another width increase re-crosses the
line, and the interrupts return.
