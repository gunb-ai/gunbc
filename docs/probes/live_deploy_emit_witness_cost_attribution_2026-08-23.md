# Where the live_deploy.emit witnesses spend their 5000ms

**Subject:** the `test.claim.live_deploy.emit` witnesses reported
INTERRUPTED-BEFORE-VERDICT against `required_floor_claim_cpu_safety_limit_ms`
(`v2.workflow.required_floor`). Their verdicts are unknown — not passing, not failing.

**THE POPULATION IS SIXTEEN, NOT FIVE**, and the dispatch title that said five (and this
document's own first draft, which repeated it) was a mismeasurement — §1 is the correction.
A single floor run is a SAMPLE of a population sitting uniformly under the line; the union
across runs is the population. Three unrelated trees reported 5, then 1, then 3.

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
`gunbc.live_deploy.emit` reduces the command count without deleting coverage.

**AN EARLIER REVISION OF THIS SECTION ALSO SAID THE TWO DEAREST ROWS ARE "INHERENTLY
COMPARATIVE — a twin-vs-production disjointness claim needs both scripts by construction,
so it cannot be split into cheaper claims". THAT IS FALSE**, and it was caught by two
readers independently reading the conjuncts rather than the claim's name. §8 is the
correction and the repair; the sentence is called out rather than quietly edited because
its failure mode is specific — a "by construction" that is really a goal will stop the next
person from looking.

**Next-rung trigger for this class:** per-word emit cost in the bash fold serializer. Until
it moves, the live_deploy.emit population's headroom is a function of fleet runner-slot
width — at ~21k evals (~30ms CI) per emitted command, another width increase re-crosses the
line, and the interrupts return.

## 7. The shape question, asked and answered: FLAT in production-table size

Raised on review of this receipt: `v2.std.grammar` `formal_production_unique_lhs_exact_match`
calls `formal_productions_for_lhs(productions: productions, lhs: lhs)` — a full filter of the
production table — from inside the per-child match. If that filter re-ran per emitted word,
the real shape would be **words × |productions|**, which is a cost-shape defect that DESIGN
§6's standing rule fixes without needing anyone's ranking, since `n` is not time-stable.

Measured directly, holding the child count fixed at eight and varying the table by padding it
with rows whose `lhs` matches nothing (so the answer is unchanged):

```
table                     node-evals   CPU
base (bash_fold)               5,037   10ms
base + 100 inert rows          6,450   12ms
base + 400 inert rows         10,650   21ms
```

**~14 evals per padding row, ONCE — not once per child.** Were the filter re-running per
child, +400 rows would have cost 400 × 14 × 8; it cost 400 × 14. So
`formal_productions_for_lhs` is memo-collapsed across the words that share a table and an
`lhs`, which is exactly the production case: `bash_fold_formal_productions()` is nullary, so
every word in a claim is handed the same table value.

Two consequences, and the second is the one that matters for how the lane is classified:

- **The per-word cost is flat in production-table size.** The hypothesis is falsified.
- **Production selection is not where the per-word cost is.** Eight matches over the real
  table cost 5,037 evals *including* building the eight child nodes — an upper bound of
  ~630 per match against ~6,714 per word. So the whole lhs-exact selection is under **10%**
  of a word's cost, and removing it entirely would buy under 10%. The other ~90% is the rest
  of the leaf path — `bash_fold_target_model_for_emitted_witness` building the
  serialize-source and rules nodes, and `target_serialize_source_from_model` doing the
  token-directed spelling.

SCOPE, STATED SO IT IS NOT OVERREAD: this varies the table at
`formal_production_unique_lhs_exact_match`'s own parameter, which is the function the
hypothesis named and the one that calls the filter. It does not vary the table seen by the
whole word path, because nothing outside `bash_command_fold` can inject one.

**So the classification is: a linear-in-input path with a large constant, not a shape
defect.** DESIGN §6's always-fix rule does not reach it, and pricing it in elegance is the
purity trap that clause names. It is an honest ranking question for the operator, and the
number to rank it against is the ~6,714 evals per emitted word that every emit consumer in
the corpus pays — not this module's sixteen rows.

**If that lane opens, its evidence must be behavioural, not fast.** Changing how emit selects
or spells a production is a correctness change; the discriminating receipt is BYTE-IDENTICAL
emitted output over a discriminating corpus, with the speedup as a secondary measurement. A
cost repair that changes output is a correctness change wearing a performance PR's clothes.

## 8. The split: no conjunct ever compared two emitted scripts

`twin_and_production_configure_disjoint_tailscale_endpoints` built FOUR scripts in one
claim — production apply, twin apply, production retract, twin retract — and crossed at
5009–5015ms. It was the most reliable crosser in this population, appearing in every run of
this family, which is what one expects of whichever row sits highest.

Read the seven conjuncts rather than the name and each is over **exactly one** script.
Disjointness is asserted by **pattern negation against a command spelling derived from the
other SPEC** — `tailscale_enable_command(endpoint: …)`, an argv join costing nothing — never
against the other spec's emitted script. So regrouping by script is lossless against the
witness as written.

Four rows, one script each, all seven conjuncts preserved verbatim. Measured as an A/B in a
single `claim_batch` run, so the before and after share a resolve and a memo state:

```
BEFORE  tmp control, the original four-script claim   PASS   5653ms
AFTER   production_apply_serves_its_own_endpoint_…    PASS   3231ms
        twin_apply_serves_its_own_endpoint_…          PASS   3344ms
        production_retract_offs_its_own_endpoint      PASS   1190ms
        twin_retract_offs_its_own_endpoint_…          PASS   1204ms
```

**Worst row 5653ms → 3344ms, −41%**, and the before-control PASSES — which is the half that
matters: the split preserves the verdict, it does not manufacture one. Every new row lands
where this module's ordinary rows already sit, and a red now locates which script and which
direction instead of which of seven conjuncts.

**THIS IS A LOCAL UNBLOCK, NOT THE REPAIR.** Fifteen rows remain at 70–96% of the same
limit. The regrouping moves the row that crossed and leaves the population where it was; the
cost is systemic and §6's trigger is unchanged. The next fleet runner-slot width increase
lengthens the emitted script again and the next-highest row crosses. Anyone reading this as
"the live_deploy.emit cost problem is fixed" has read it wrong.

**AND IT MAKES ONE STRENGTHENING HARDER, DECLARED HERE RATHER THAN DISCOVERED LATER.** The
claim is LITERAL — it negates a spelling. The strictly stronger form is RELATIONAL: assert
the two emitted endpoint values differ *from each other*, which catches a drift that keeps
both spellings absent from each other's script while still colliding. That form genuinely
needs both scripts in one claim, so adopting it re-fuses two of these rows and re-crosses the
line at today's per-word cost. It is therefore gated on the emit-cost lane rather than on
anyone's appetite. The "by construction" in the retracted sentence was this goal, mistaken
for a description of the row in tree.
