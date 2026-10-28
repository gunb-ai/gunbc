# Debt-paydown integration: state and follow-ons

Status: OPEN. This is the hand-off record for the debt-paydown tree (manager `stern-bat-401`), which was paused on 2026-10-28. It explains what branch `integration/debt-paydown` carries, what it still owes before it lands, and which PRs remain open as follow-ons. Delete this file when the last follow-on closes. Its rows are hand-off state, not a ledger.

## What the branch carries

These are the PRs that were approved, or approved apart from a small fix. Each was merged onto `main` in the order below. Once this branch lands, close each source PR as superseded and link it here.

| PR | Head merged | What it does | How it was integrated |
|---|---|---|---|
| #14952 | ccbb62ed | Convergence witness on typed effects. The dropped coverage is declared as `gunbc.rung_drop.convergence_fetch_window_fixture_interposition`. | clean merge |
| #14933 | a61ef827 | `claim_batch --outcomes-out`: terminal ledger rendered by the `.dag` wire | clean merge |
| #14575 | 05a28f7d | Outside-discovery-roots census of the claim files no acquirer can see | clean merge |
| #14082 | 3f0190c2 | v1: type `LitSymbol` as `Symbol` (producer only) | mirrors provisional (see below) |
| #15228 | 213a3e12 | v1 builtins: the digest row declares its own digest carrier | `src/v1/00_core.dag` resolved by hand: kept #14082's `symbol_type` and deleted `hash_type` as #15228 intends; mirrors provisional |
| #15026 | 53220486 | A unit variant's phantom marker is decided by scope lookup, not name match | mirrors provisional |
| #15008 | 45773a78 | Equality bounds forward through one call-graph fixpoint at any arity | mirrors provisional |
| #15015 | 8738e878 | An operation with no declared transport refuses by name | mirrors provisional |
| #15020 | 0198ec26 | `extdeps.github.pulls`: the list state filter is its own type | clean merge |
| #14793 | 92d35c4f | `ObservationUnrepresentable.reason` is the probe cause (`NonEmptyStr`) | clean merge |
| #14056 | d5657221 | Applied head identity in `node_type_equals` | mirrors provisional |
| #15277 | 5b4edb8c | `CoproductWireContract` rows keyed by `(module_path, decl_name)` | mirrors provisional |
| #15276 | dfc7e243 | C4-H2: the remaining `<scratch>/target` sites onto `extdeps.rust.cargo` `cargo_default_target_dir` | clean merge, plus the review 87029 fix (two duplicate imports deleted) |
| #14411 | 79be16cb | Failure-mode row: `unify_generics` fixes a fold accumulator to the `[]` placeholder | applied as a diff (see below) |
| #14369 | 393a472c | C4-B: parse gate measured; scope the PASS=0 FAIL=0 reading | applied as a diff |
| #14437 | e9b7a15f | C4: registry-vs-rows identity check (frozen seed residue, run by hand) | applied as a diff |

**Provisional mirrors.** Several of these PRs regenerate the same `src/v1/stage0/src/*.rs` mirrors. Where the only conflicts were in those generated files, the integration kept the base side provisionally, as `docs/plans/generated-file-conflict-policy.md` allows for a deterministic projection. The branch does not land on that. The heal job (`gunbc.heal_workflow`) must converge the mirrors to the seed's own fixed point over the merged `.dag`, with a receipt, before the branch is reviewed. A v2-bootstrap green over stale mirrors judges main's behaviour, not this branch's.

**Applied as a diff.** #14411, #14369 and #14437 were cut from a repository history that has no common ancestor with today's `main`. Their PR diffs were applied with `git apply -3`, and all three applied cleanly. Their original branches cannot be merged.

**Composition risk.** Keyed rosters can merge cleanly into duplicates (`docs/plans/generated-file-conflict-policy.md`, case 3). The composed run below is what rules that out. A clean merge does not.

## What the branch owes before it lands

1. **Heal convergence** of the stage0 mirrors on the branch head, with the HealPublished receipt.
2. **Required check:** `witnesses` green at the converged head.
3. **Composed whole-corpus run** at that head, in both orderings, with no newly refused rows. Also run:
   - the gunbc-private roster check, because #14082, #15228 and #14056 change acceptance;
   - the N7 census, for #14933 and #14575.
   - Remote runs on BuildBuddy were refusing ("permission denied") at hand-off, so these may need the fleet runners.
4. **Evidence still owed by individual PRs:**
   - #14082's mirror fixed-point check never completed. Step 1 discharges it.
   - #15228 was last tested before #15189 landed over two of its files. Step 3 discharges it.
   - #15026, #15008 and #15015 were granted to land one per batch. In one branch they need the step 3 run, with no newly refused row attributable to any of them.
   - #14056, #15020, #14793, #14369, #14437, #14411, #15276 and #15277 had a dashboard approval but no side-chat review at hand-off. Review them as part of this branch.
5. **Side-chat approval** at the exact final head, then the merge queue.

## Follow-on PRs (not in this branch)

### Approved, but a known fix comes first

- **#15110** (machine width, one derivation): dropped from a batch on an N7 residual, a machine_width_range cascade. Fix the root of the cascade, then re-test.
- **#15022** (open fold carrier closes at the step's produced type): dropped for an empty-list fold seed. Re-verify at head 4959accc.
- **#14868** (ambient-name-census: refuse only newly refused modules): needs the base_stdout caller fix. Two earlier RED attempts did not discriminate.
- **#14776** (wet fold refuses an undeclared function): side-chat REQUEST_CHANGES says the join must require `FnItem`. First probe which route reads a zero-param test fn as `DataItem`.
- **#14699** (decl_facts refuses an unparsable pool file): side-chat REQUEST_CHANGES says the refusal must exit 2 through the typed observation boundary, with rc 2/0/1 evidence.

### Open REQUEST_CHANGES

- **#15255:** the `SubjectOutsidePopulation` arm is never produced or consumed. Wire it to exit 2 with a RED, or declare the frontier.
- **#15245:** move the receipts and counts out of `//` prose into the RFM row, and run REDs A/B plus the control in one dispatch.
- **#14986:** drop the unmodeled sudoers grant, split out the mtcollins1 rework, then a fresh composed run.
- **#14969:** depends on #14952, which is in this branch. Re-verify once the branch lands.
- **#14940:** set the rung to Mitigatable and make the annotation agree with the typed row.
- **#14693:** implement the three join seats, or mark them as expected-red probes, then merge main.
- **#14657:** drop `ev/`, derive cited/uncited from the projection, and retitle.
- **#14484:** route the partition-mirror derivation through a census row, or declare it a frontier with a trigger.
- **#15229:** split out the Nat-to-kernel-Int rework and drop `fixtures/zz_*.sh` and the probes.

### Approved but stale: merge main and regenerate

- **#14632** fixes the `kc_local_alias` main break from #14330. Do this one first.
- #14913, #14750 (after #15228 lands), #14430, #14419 and #14407.
- **#14701:** merge main (not a rebase), then review.
- **#14443:** merge main, then dispatch a fix leaf for its three `located_refusal` claims that fail on main.

### Blocked outside the PR

- **#14705:** the pinned srv3 script (PR comment issuecomment-6354008506) must be run as root on srv3.
- **#14545:** needs an operator decision on gunbc-private's Nat-as-String sites.
- **#14432:** the argv corpus is red on clean main, cause unknown. Bisect from a515cc81.
- **#14884:** frozen until main's import-census regression is fixed. Since window ae563700cbe..98eb656bfcd, `compile_dag_diagnostic_census` returns CensusNotRunnable for any source with an import; the suspects are #15123 and #15209.

### Close

- #15280 duplicates #14705.
- #15282 duplicates #14484's derivation and adds scratch shell.
- #15278 is a C1 cast-wall WIP snapshot; #15033 already landed the cast wall.
- #15279: re-extract the `.dag` change; this branch carries about 30k lines of mirror churn.
- #15283 is a heal WIP with a fabricated "consumed" verdict.
- #14009 is a stale draft.
- #14426 and #14389 are pre-ruling C4-D work.
- Close the 16 source PRs listed above once this branch lands.
