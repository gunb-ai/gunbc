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
| #15020 | 0198ec26 | `extdeps.github.pulls`: the list state filter is its own type | clean merge |
| #14793 | 92d35c4f | `ObservationUnrepresentable.reason` is the probe cause (`NonEmptyStr`) | clean merge |
| #14056 | d5657221 | Applied head identity in `node_type_equals` | mirrors provisional |
| #15276 | dfc7e243 | C4-H2: the remaining `<scratch>/target` sites onto `extdeps.rust.cargo` `cargo_default_target_dir` | clean merge, plus the review 87029 fix (two duplicate imports deleted) |
| #14411 | 79be16cb | Failure-mode row: `unify_generics` fixes a fold accumulator to the `[]` placeholder | applied as a diff (see below) |
| #14369 | 393a472c | C4-B: parse gate measured; scope the PASS=0 FAIL=0 reading | applied as a diff |
| #14437 | e9b7a15f | C4: registry-vs-rows identity check (frozen seed residue, run by hand) | applied as a diff |

**Mirrors.** Several of these PRs regenerate the same `src/v1/stage0/src/*.rs` mirrors, and the merges first kept the base side of those generated files. Commit 20d024d replaced them with the seed's own fixed point: `claim_executor --regen-round-cost --base-mirror` against the merge base 94870562 converged in 2 stages, then `heal_stage0_overlap_refusal_wet` and `seed_build_artifacts_regen` `main_wet` ran clean. It ran on srv1 because the CI heal job reruns those rounds and exceeds its 90-minute limit on this branch (runs 40623770492 and 40723909435), so CI heal has not independently confirmed the bytes.

**Taken back out: #15015 and #15277.** Both change a generated function's surface AND the hand-written seed code that calls it (`cli_run.rs`, `data_initializer_identity.rs`): #15015 adds `undeclared_transport_node`, and #15277 adds a parameter to `resolve_local_coproduct_wire_policy`. Heal builds its regenerator from the checkout before it regenerates. So a branch whose hand-written seed code needs regenerated mirrors, without those mirrors committed, can never pass heal: building from the old mirrors fails, and the rebuild after regenerating fails against the old hand-written code. That happened on this branch (heal run 40616715648, E0061 and E0425). Each PR's own branch carries mirrors consistent with its hand-written code, so each lands on its own after this branch, by merging main and letting heal re-converge. They are listed as follow-ons below.

**Applied as a diff.** #14411, #14369 and #14437 were cut from a repository history that has no common ancestor with today's `main`. Their PR diffs were applied with `git apply -3`, and all three applied cleanly. Their original branches cannot be merged.

**Composition risk.** Keyed rosters can merge cleanly into duplicates (`docs/plans/generated-file-conflict-policy.md`, case 3). The composed run below is what rules that out. A clean merge does not.

## Evidence at 20d024d (base: merge base 94870562)

Run on srv1 with `~/n7c/run2.sh`, `~/n7c/privchk.sh` and `~/n7local.sh`, head and base built and run side by side.

- **Required check:** `v2-bootstrap` passed at 20d024d, over the converged mirrors.
- **Whole-corpus compile, both root orderings:** 340 distinct refusals at head, 341 at base. Compared by file and message, no refusal is new. Two head rows differ from base only by line (`dag/test/claim/diagnostic_span_location_rendering_witness.dag`, 88→89 and 139→140), because #15276 added one import line above them; both are inherited. One inherited refusal is gone: `dag/test/claim/parse_test_fn_decl_return_clause.dag` (`expected 'Product(DeclaredName)', got 'Primitive(String)'`).
- **gunbc-private roster** (`private_roster_main`, private f8c9b2ac): exit 0 at head and at base.
- **N7** (`//gunbc/instruments:v2-native-census-scoped`, covering #14933 and #14575): exit 0 at both, identical terminal rows (164 modules, 35 infer refusals, 2067 residual rows, diff none). The members claim `//v2/test/claim/parse/expression_bodied_fn_decl_parse:all` exits 2 at both head and base, so it is inherited, not caused here.

Still owed: side-chat approval at the exact final head, then the merge. CI heal confirming the mirror bytes needs its time limit raised or its duplicated bootstrap round removed first.

## Follow-on PRs (not in this branch)

### Approved, but land on their own after this branch

- **#15015** (an operation with no declared transport refuses by name): after this branch lands, merge main into its branch, let heal re-converge, then a composed run. It couples hand-written `cli_run.rs` to a new generated function.
- **#15277** (`CoproductWireContract` rows keyed by `(module_path, decl_name)`): same as #15015. It changes `resolve_local_coproduct_wire_policy`'s signature and its `cli_run.rs` caller.

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
- The 14 source PRs carried here are already closed as superseded by #15290.
