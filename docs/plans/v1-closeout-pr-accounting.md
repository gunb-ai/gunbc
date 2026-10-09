# v1 closeout: open PR accounting

As of 2026-10-09; it includes the operator's side-chat dispositions. There are 162 open PRs on gunb-ai/gunbc. Apart from #13203 (blackjack, kept up by request), every one of them ends either in the mega branch #13641 (`integration/v1-closeout`) or closed with a stated reason. The owners come from the dashboard's PR-to-session records and the side chats. The dispositions come from each manager's closeout report. Merge state is GitHub's `mergeStateStatus` at the time of writing.

| Disposition | PRs | What happens |
| --- | --- | --- |
| In the mega branch | 33 | Lands when #13641 lands |
| Pending fold | 64 | On a manager's integration branch, with the conflict worker, or waiting for CI; folded into #13641 next |
| Blocked | 4 | Needs an operator or host action first |
| Left out | 47 | Not in the snapshot. Its remaining work is recorded under Outstanding work before the PR is closed; nothing is dropped |
| Unreported, owned by a live lane | 0 | Waiting on that lane's report |
| Orphans: no live owner | 13 | Triaged by the closeout: fold if done and clean, otherwise close |
| Kept up by request | 1 | #13203 blackjack |

## In the mega branch

These are merged into `integration/v1-closeout`.

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |
| [#13641](https://github.com/gunb-ai/gunbc/pull/13641) | v1 closeout mega branch (wave 1) | ready / BLOCKED | neat-wolf-604 |
| [#13626](https://github.com/gunb-ai/gunbc/pull/13626) | host_reach_identity_probe: the identity probe's argv is spelled by its | ready / CLEAN | stern-boar-596 |
| [#13622](https://github.com/gunb-ai/gunbc/pull/13622) | dispatch-actuator witness: supply the unrooted specimen instead of bor | ready / CLEAN | bold-bee-114 |
| [#13616](https://github.com/gunb-ai/gunbc/pull/13616) | Belt SCM capture: read each re-bound object shard once (recapture 30x  | ready / CLEAN | bold-bee-114 |
| [#13610](https://github.com/gunb-ai/gunbc/pull/13610) | walk_cgroup, entry_presence: tail-position self-recursion so both modu | ready / CLEAN | silent-lark-156 |
| [#13602](https://github.com/gunb-ai/gunbc/pull/13602) | Runner obligation and census read committed identities; Spark training | ready / CLEAN | side chat |
| [#13601](https://github.com/gunb-ai/gunbc/pull/13601) | Fabric cell witnesses: least-cause refusal join; carry #11625/#11716 i | ready / CLEAN | side chat |
| [#13600](https://github.com/gunb-ai/gunbc/pull/13600) | witnesses: match optional results instead of comparing T? == T (3 file | ready / CLEAN | side chat |
| [#13599](https://github.com/gunb-ai/gunbc/pull/13599) | srv1 lab follows main instead of a July session branch | ready / CLEAN | side chat |
| [#13587](https://github.com/gunb-ai/gunbc/pull/13587) | Bind gunbc run --arg against declared parameter types | ready / DIRTY | bold-bee-114 |
| [#13577](https://github.com/gunb-ai/gunbc/pull/13577) | Price the parse memo below the cost floor: the production door admits  | ready / CLEAN | side chat |
| [#13570](https://github.com/gunb-ai/gunbc/pull/13570) | BMC onboarding: Redfish PasswordChangeRequired is the factory phase; s | ready / CLEAN | side chat |
| [#13569](https://github.com/gunb-ai/gunbc/pull/13569) | store: held-session op + O(1) named hold-slot reads | ready / CLEAN | royal-moth-86 |
| [#13568](https://github.com/gunb-ai/gunbc/pull/13568) | Delete the pasted-operator-token refusal (operator ruling 2026-10-08) | ready / CLEAN | side chat |
| [#13566](https://github.com/gunb-ai/gunbc/pull/13566) | Skip module-surface members in reference-derived residency reading | ready / CLEAN | bold-bee-114 |
| [#13563](https://github.com/gunb-ai/gunbc/pull/13563) | Altra firmware library: beta BIOS 4.01 + SCP archives, flashable membe | ready / CLEAN | side chat |
| [#13560](https://github.com/gunb-ai/gunbc/pull/13560) | Identity-only skip for module-surface members in record-field lenses | ready / CLEAN | bold-bee-114 |
| [#13554](https://github.com/gunb-ai/gunbc/pull/13554) | srv13 first contact: factory BIOS defaults (PXE disabled by default) | ready / CLEAN | side chat |
| [#13550](https://github.com/gunb-ai/gunbc/pull/13550) | Bankrupt docs/plans: delete hand-written plans, keep DESIGN-linked and | ready / DIRTY | side chat |
| [#13516](https://github.com/gunb-ai/gunbc/pull/13516) | Managed-host cut O1c-3a: ManagedHostAdmission population with drop-arm | ready / CLEAN | eager-gull-22 |
| [#13513](https://github.com/gunb-ai/gunbc/pull/13513) | mandatory_tag gate: read the lowered data-declaration carrier (clean f | ready / CLEAN | bold-bee-114 |
| [#13472](https://github.com/gunb-ai/gunbc/pull/13472) | Realize rust shell stderr-capture channels (unblocks #13217 floor) | ready / DIRTY | silent-lark-156 |
| [#13460](https://github.com/gunb-ai/gunbc/pull/13460) | map_get fork: the declared projection renames to map_get_checked | ready / CLEAN | lively-ram-153 |
| [#13442](https://github.com/gunb-ai/gunbc/pull/13442) | Workspace transcript append rate: size-and-mtime instrument on srv1 (s | ready / CLEAN | neat-wolf-604 |
| [#13412](https://github.com/gunb-ai/gunbc/pull/13412) | semver: route the version scheme's identity compare through the cited  | ready / CLEAN | lively-ram-153 |
| [#13359](https://github.com/gunb-ai/gunbc/pull/13359) | Codex + Cursor worker turns on one gunbai-secrets credential each (cus | ready / CLEAN | neat-wolf-604 |
| [#13357](https://github.com/gunb-ai/gunbc/pull/13357) | Onboard OpenRouter Nemotron 3 Ultra (free) as a quota-leased harness b | ready / CLEAN | neat-wolf-604 |
| [#13351](https://github.com/gunb-ai/gunbc/pull/13351) | Oracle OCI Always Free: network + instance converge (stacked on #13335 | ready / CLEAN | nimble-heron-805 |
| [#13346](https://github.com/gunb-ai/gunbc/pull/13346) | Remove quadratic remainder copying from code-point/octet slicing in bo | ready / CLEAN | neat-wolf-604 |
| [#13210](https://github.com/gunb-ai/gunbc/pull/13210) | v2 native infer accepts variant-scrutinee matches (payload binder typi | ready / CLEAN |  |
| [#13125](https://github.com/gunb-ai/gunbc/pull/13125) | belt: demote the monolithic tick into event-driven attempt obligations | ready / BLOCKED | bold-bee-114 |
| [#13072](https://github.com/gunb-ai/gunbc/pull/13072) | Compute work is a request family of std.materialization_provider; work | ready / CLEAN | bold-bee-114 |
| [#12942](https://github.com/gunb-ai/gunbc/pull/12942) | MQ-5 cut: producers seal what they lower (BodyTerm via sealed #12799 m | ready / CLEAN | neat-boar-16 |

## Pending fold

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |
| [#13609](https://github.com/gunb-ai/gunbc/pull/13609) | Derive ledger roster modules without mutating source roots | ready / CLEAN | conflict worker |
| [#13108](https://github.com/gunb-ai/gunbc/pull/13108) | admit_callers: restore the native-route caller-admission wall (DP-M6 i | ready / CLEAN | conflict worker (04_infer.dag, body_lowering_fold.dag; regen-confirm ROADMAP/rung-drops) |
| [#13583](https://github.com/gunb-ai/gunbc/pull/13583) | One directory authority for dashboard instances (D14 OwnedDirectory sl | ready / CLEAN | conflict worker (keep 13583 spec.dag) |
| [#13582](https://github.com/gunb-ai/gunbc/pull/13582) | Delete the v2 packrat parse memo carrier (R2, stacked on #13577) | draft / CLEAN | conflict worker (stack on 13577) |
| [#13615](https://github.com/gunb-ai/gunbc/pull/13615) | Delete unused memo and preparation carriers (native-memory-inert-carri | draft / CLEAN | conflict worker (stack; keep floor_preparation_witness_test.dag deleted) |
| [#13608](https://github.com/gunb-ai/gunbc/pull/13608) | RFM: fixture-closure-union-emit suspected superlinear cost | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13596](https://github.com/gunb-ai/gunbc/pull/13596) | target_invocation witness: repair two false claims; file the unplanned | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13590](https://github.com/gunb-ai/gunbc/pull/13590) | Native ingest: refuse two files claiming one module path | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13588](https://github.com/gunb-ai/gunbc/pull/13588) | std.change: import any and Present so its own names resolve natively | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13586](https://github.com/gunb-ai/gunbc/pull/13586) | guarantee_stall: correct accumulator-copy native-resolve residue recei | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13585](https://github.com/gunb-ai/gunbc/pull/13585) | Enroll native-resolve coverage for fold handler binders | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13580](https://github.com/gunb-ai/gunbc/pull/13580) | regen-round-cost: re-exec once when the seed build replaces the runnin | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13579](https://github.com/gunb-ai/gunbc/pull/13579) | Climb import-less UniqueBare from proximity to lexical on-chain bindin | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13575](https://github.com/gunb-ai/gunbc/pull/13575) | Isolate fixture census on a scratch MultiEntryIndex | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13571](https://github.com/gunb-ai/gunbc/pull/13571) | Fallout from #13179: latent T? == T sites and stale grounded_principal | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13520](https://github.com/gunb-ai/gunbc/pull/13520) | Delete dead emit_resolve_transitively_fn stamp from generated main | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13519](https://github.com/gunb-ai/gunbc/pull/13519) | stage0 witness bins: close compile subjects through the closure author | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13517](https://github.com/gunb-ai/gunbc/pull/13517) | Managed-host cut O1c-3b: BmcSecure gated through the convergence fold | ready / DIRTY | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13507](https://github.com/gunb-ai/gunbc/pull/13507) | Board identity: bound host's FRU board reaches its per-board authority | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13503](https://github.com/gunb-ai/gunbc/pull/13503) | Move AMI-bundle-derived MegaRAC content out of public gunbc | ready / BLOCKED | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13497](https://github.com/gunb-ai/gunbc/pull/13497) | Emit arrival_converge; IAM pair and pinned accessor (msg_f03558d1) | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13341](https://github.com/gunb-ai/gunbc/pull/13341) | v2 parse: occurrence ids minted once per parse across a rejected attem | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13333](https://github.com/gunb-ai/gunbc/pull/13333) | v2 resolve: the single-tree namespace binds its own variants and recor | ready / CLEAN | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13247](https://github.com/gunb-ai/gunbc/pull/13247) | mtcollins1 runner: boot leg, runner-host medium + runner-host-up termi | ready / BLOCKED | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13225](https://github.com/gunb-ai/gunbc/pull/13225) | mtcollins1 runner: extract qualification instruments from the run (leg | ready / DIRTY | eager-gull-22 @54f257a3f5 (with the sharp-raven/eager-gull fold worker; 19-file conflict) |
| [#13598](https://github.com/gunb-ai/gunbc/pull/13598) | Retire native_serve_request_budget_unrealized: native-serve holds the  | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13593](https://github.com/gunb-ai/gunbc/pull/13593) | File emitter defect: variant pattern nested inside Present drops the R | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13584](https://github.com/gunb-ai/gunbc/pull/13584) | Seed eval: empty list is not Unit | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13567](https://github.com/gunb-ai/gunbc/pull/13567) | Emit algebra length on host String as string_length | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13549](https://github.com/gunb-ai/gunbc/pull/13549) | Fix every T? == T site the whole-population census found | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13454](https://github.com/gunb-ai/gunbc/pull/13454) | Native broker 2K-b: identity-keyed type_summaries; Host-Option decided | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13260](https://github.com/gunb-ai/gunbc/pull/13260) | emit_rust: a type parameter carries its own cardinality (K? emits Opti | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13255](https://github.com/gunb-ai/gunbc/pull/13255) | Emitter: a cloned Rc match scrutinee owes Clone on its generics wherev | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13243](https://github.com/gunb-ai/gunbc/pull/13243) | Inference: a variant literal of a generic coproduct takes the expected | ready / DIRTY | gentle-dove-36 (with the conflict worker) |
| [#13224](https://github.com/gunb-ai/gunbc/pull/13224) | Native effect realization admission; native mains bind admitted handle | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13202](https://github.com/gunb-ai/gunbc/pull/13202) | Native broker 2C/G: optional data rows keep their ? (signature and JSO | ready / CLEAN | gentle-dove-36 (with the conflict worker) |
| [#13621](https://github.com/gunb-ai/gunbc/pull/13621) | A data reference uses its value: ConstantMemberEdge marks the lowered  | draft / BLOCKED | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13618](https://github.com/gunb-ai/gunbc/pull/13618) | workflow scripts: a refused bash emission refuses, never emits an empt | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13545](https://github.com/gunb-ai/gunbc/pull/13545) | Compare an Optional with an Optional at 46 latent sites no gated closu | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13511](https://github.com/gunb-ai/gunbc/pull/13511) | Join corpus Int and kernel Int at inhabitance (N7 g_tokenize_parse) | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13502](https://github.com/gunb-ai/gunbc/pull/13502) | Pair identity-cast native with the real emit artifact | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13501](https://github.com/gunb-ai/gunbc/pull/13501) | v2 tokenize: bind EmptyPattern on 01_tokenize's import chain (N7 resol | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13496](https://github.com/gunb-ai/gunbc/pull/13496) | Bool-native rust_logic/variant emit after BindingRef octet de-fork | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13469](https://github.com/gunb-ai/gunbc/pull/13469) | Out-of-gate emit census: binder types, A4 order on rust fixtures, refi | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13453](https://github.com/gunb-ai/gunbc/pull/13453) | Floor judges base-revision facts with the base revision's own compiler | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13438](https://github.com/gunb-ai/gunbc/pull/13438) | Precedence-climbing tree contract for dag binary expressions | ready / DIRTY | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13436](https://github.com/gunb-ai/gunbc/pull/13436) | XL-2: named conversions for every non-String interpolation hole (std.n | ready / BLOCKED | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13425](https://github.com/gunb-ai/gunbc/pull/13425) | Gate the foreign-language grammar claims (census phase 4): runnable te | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13406](https://github.com/gunb-ai/gunbc/pull/13406) | Claims tokenize .dag text through tokenize_prepared(dag_prepared_lex() | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13380](https://github.com/gunb-ai/gunbc/pull/13380) | RFM: 'first element of a list' nickname fork (roster first / list_head | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13379](https://github.com/gunb-ai/gunbc/pull/13379) | v2.std.grammar: qualify the GrammarExpr Optional arm (N7 resolve link) | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13325](https://github.com/gunb-ai/gunbc/pull/13325) | v2 infer: derive a call's result type from a declared return (instanti | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13320](https://github.com/gunb-ai/gunbc/pull/13320) | v2 infer: an integer literal elaborates at its expected type through s | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13308](https://github.com/gunb-ai/gunbc/pull/13308) | Census part 2: the native infer census on v2-native-census (census-inf | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13307](https://github.com/gunb-ai/gunbc/pull/13307) | v2: bare count over lists binds the std.algebra collection size row, a | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13126](https://github.com/gunb-ai/gunbc/pull/13126) | Grammar overlap: validation refuses every overlap row; required zero-c | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13123](https://github.com/gunb-ai/gunbc/pull/13123) | C4: native route enforces constructor confinement (sole_constructor +  | ready / CLEAN | sharp-raven-357 (~200-file conflict; separate worker after the conflict worker lands) |
| [#13625](https://github.com/gunb-ai/gunbc/pull/13625) | CAX41 guest onboarding; fleet-converge from a branch after a per-run n | draft / BLOCKED | side chat, draft; folds when CI green (touches fleet-converge.yml) |
| [#13614](https://github.com/gunb-ai/gunbc/pull/13614) | seed interpreter: withdraw the spelling-admitted, label-blind PureCall | ready / CLEAN | side chat, folds when CI green |
| [#13611](https://github.com/gunb-ai/gunbc/pull/13611) | demand engine: a request under a different nature is a durable key con | ready / CLEAN | side chat, folds when CI green |
| [#13607](https://github.com/gunb-ai/gunbc/pull/13607) | Connect srv1 workspace VM lifecycle and kernel prerequisites | draft / BLOCKED | side chat, in progress: kernel activation/readback, VM commissioning + SSH, release/reuse/expiry checks, CI |
| [#13597](https://github.com/gunb-ai/gunbc/pull/13597) | Make the crate partitioner the only emitted-Rust layout; delete single | draft / DIRTY | side chat, not yet done: needs a main merge + regen, the remaining acceptance runs, and a PR body update; folds once marked ready |
| [#13428](https://github.com/gunb-ai/gunbc/pull/13428) | Carry the alias declaration: its name resolves to its target (ruling B | ready / CLEAN | with the sharp-raven fold worker (green, approved; conflicts in 03_normalize/body_lowering_fold) |
| [#13643](https://github.com/gunb-ai/gunbc/pull/13643) | Census: cross-module record field values are judged (rcf pin rewritten | draft / BLOCKED | with the sharp-raven fold worker if its last check is green (census, approved) |

## Blocked

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |
| [#13544](https://github.com/gunb-ai/gunbc/pull/13544) | Decommission Group A (srv5-srv8, sold): remove it from fleet config an | ready / DIRTY | DIRTY vs main |
| [#13411](https://github.com/gunb-ai/gunbc/pull/13411) | Accepting wildcards cut 3: name the two wet-step exit gates | ready / CLEAN | stacked 13383 |
| [#13383](https://github.com/gunb-ai/gunbc/pull/13383) | Accepting wildcards cut 2: name the vocabulary walls, the grounding ga | ready / CLEAN | stacked 13382 |
| [#13382](https://github.com/gunb-ai/gunbc/pull/13382) | Accepting wildcards cut 1: name the verdict classifier and the two rou | ready / BLOCKED | browser toolchain |

## Left out

These are drafts, red, superseded or WIP, per the manager's report or a side chat. Closing one never drops its work: what is left gets a row under Outstanding work first.

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |
| [#13097](https://github.com/gunb-ai/gunbc/pull/13097) | G1: belt verify through the materialization provider; compute outcomes | ready / BLOCKED | bold-bee: red on T?==T sites until #13549; draft |
| [#13574](https://github.com/gunb-ai/gunbc/pull/13574) | Interim check-time refusal of list-read == Present{..} | ready / CLEAN | eager-gull: OPERATOR: reverted out (review 78262) |
| [#13604](https://github.com/gunb-ai/gunbc/pull/13604) | Prepare the required-floor gate once; project policy from that subject | draft / BLOCKED | eager-gull: REQUEST_CHANGES review 78326 (hand .dag walker in seed); reverted |
| [#13391](https://github.com/gunb-ai/gunbc/pull/13391) | Refuse a product value at a kernel-scalar (or refined) declared type | ready / DIRTY | eager-gull: REQUEST_CHANGES review 78347 (census missing); conflicts main |
| [#13591](https://github.com/gunb-ai/gunbc/pull/13591) | Resolve free kernel calls to std.primitives identity; infer refuses th | draft / CLEAN | eager-gull: WIP stacked on #13588; native-cli unconfirmed |
| [#13595](https://github.com/gunb-ai/gunbc/pull/13595) | Reclassify the match-arm proven-disjoint InternalError as a located Ty | draft / CLEAN | eager-gull: depends on #13574; reverted out |
| [#13576](https://github.com/gunb-ai/gunbc/pull/13576) | DRAFT: floor-control for #13575 (plan forged-probe census) | draft / UNSTABLE | eager-gull: red draft control for #13575 |
| [#13211](https://github.com/gunb-ai/gunbc/pull/13211) | mtcollins1 runner: dispatch the floor to the attempt's slot and read t | ready / BLOCKED | eager-gull: red: floor 90m cap in fixture-closure-union-emit; needs #13608 newer head 8ff94ca471 |
| [#13420](https://github.com/gunb-ai/gunbc/pull/13420) | Managed-host cut 5: fan, served-UI and KVM observations over ManagedHo | draft / BLOCKED | eager-gull: red: floor stuck-floor pin; waits #13575 |
| [#13432](https://github.com/gunb-ai/gunbc/pull/13432) | [DO NOT MERGE] baseline control: fixture-closure union with only a KVM | draft / BLOCKED | eager-gull: scratch baseline, DO NOT MERGE |
| [#13288](https://github.com/gunb-ai/gunbc/pull/13288) | mtcollins1 runner: the dedicated runner-qualification group, ensured o | ready / CLEAN | eager-gull: stacked on #13211 |
| [#13640](https://github.com/gunb-ai/gunbc/pull/13640) | mtcollins1 runner: dispatch floor to ephemeral slot + collect instrume | draft / DIRTY | eager-gull: superseded quiet-cat-583 WIP flush |
| [#13252](https://github.com/gunb-ai/gunbc/pull/13252) | mtcollins1 runner offer at measured one-socket shape + census meminfo  | ready / CLEAN | eager-gull: waits on census boot measurement |
| [#13475](https://github.com/gunb-ai/gunbc/pull/13475) | fold_list empty: [] no longer locks the accumulator as List<Unit> | draft / BLOCKED | gentle-dove: APPROVED (review 78364), fix judged sound; only CI red (4 failing). Closest to revivable |
| [#13330](https://github.com/gunb-ai/gunbc/pull/13330) | Derived-node identity step 2, shape 2: unify_generics reads a containe | ready / BLOCKED | gentle-dove: REQUEST_CHANGES review 76999 (Bool predicate over DeclField in std/decl_ref.dag; inline it at the caller) |
| [#13633](https://github.com/gunb-ai/gunbc/pull/13633) | Re-land #13255 (3rd conflict) + propose de-hotspotting native_emission | draft / DIRTY | gentle-dove: left out |
| [#13488](https://github.com/gunb-ai/gunbc/pull/13488) | Design: nested optionality census + layer-count carrier (no code) | ready / BLOCKED | gentle-dove: left out |
| [#13265](https://github.com/gunb-ai/gunbc/pull/13265) | Type parameters bind only inside their own declaration (rule 2 / A'; s | draft / DIRTY | gentle-dove: left out |
| [#13632](https://github.com/gunb-ai/gunbc/pull/13632) | dusk qwen 3 | ready / BLOCKED | qwen: jq length(null)->0 fabricated default (DESIGN §5) |
| [#13634](https://github.com/gunb-ai/gunbc/pull/13634) | C2 #13482: the seven review fixes (own-interface entries, lookup-first | draft / BLOCKED | royal-moth: left out |
| [#13482](https://github.com/gunb-ai/gunbc/pull/13482) | C2: typecheck materialization through the local store | draft / CLEAN | royal-moth: left out |
| [#13399](https://github.com/gunb-ai/gunbc/pull/13399) | Move the bottom seam to the leaf std.error_primitives; trim diverges t | draft / DIRTY | sharp-raven: APPROVED (review 76752); needs rebase on main + CI fix |
| [#13426](https://github.com/gunb-ai/gunbc/pull/13426) | Foreign lexer reds: rust ingest bare +, TS single-quoted string body;  | draft / CLEAN | sharp-raven: APPROVED (review 76863); base #13425, never ran CI; easy pickup once sharp-raven is folded |
| [#13284](https://github.com/gunb-ai/gunbc/pull/13284) | v2: kernel-String concat through a free-monoid structure bound on the  | ready / DIRTY | sharp-raven: APPROVED (review 76877); needs rebase + CI fix; unowned since XL-2 was cancelled |
| [#13630](https://github.com/gunb-ai/gunbc/pull/13630) | Recover callee Arrow through Instantiation-grounding refusal | ready / CLEAN | sharp-raven: BLOCKING §5 fail-open: refused callee reaches eval body (reviews 78331/78359) |
| [#13558](https://github.com/gunb-ai/gunbc/pull/13558) | N7: derive payload-binder facts for construct-field consumption | ready / CLEAN | sharp-raven: base n7/payload-binder-combined-base, no CI (approved) |
| [#13642](https://github.com/gunb-ai/gunbc/pull/13642) | Census: quoted-key brace is not a record field at resolve, not lowerin | ready / BLOCKED | sharp-raven: census, approved but CI red (2 failing) |
| [#13541](https://github.com/gunb-ai/gunbc/pull/13541) | Retire v2.std.algebra filter/any onto the collection roster's callback | ready / DIRTY | sharp-raven: conflicts; 15 root scratch scripts (review 78343) |
| [#13637](https://github.com/gunb-ai/gunbc/pull/13637) | XL-2 3c: enumerate ServiceSetAside consumers before delete-first | ready / BLOCKED | sharp-raven: docs-only 3c census (approved); not merged per docs-only preference; input to unowned XL-2 3c |
| [#13635](https://github.com/gunb-ai/gunbc/pull/13635) | XL-2 cleanup mgr | draft / BLOCKED | sharp-raven: duplicate of #13621 |
| [#13548](https://github.com/gunb-ai/gunbc/pull/13548) | DRAFT: unimported-type import migration (source_reference_repoint) | ready / CLEAN | sharp-raven: plan only, empty roster, REQUEST_CHANGES 78338; reverted from the branch |
| [#13377](https://github.com/gunb-ai/gunbc/pull/13377) | node_query: qualify the Cardinality/Optional references (import-level  | ready / BLOCKED | sharp-raven: red: DependencyView import turns off the bare channel (main defect from #13500) |
| [#13430](https://github.com/gunb-ai/gunbc/pull/13430) | XL-2 PR2b: string templates — disjoint lexer, one ^dag_string_template | ready / BLOCKED | sharp-raven: red; tplscan scratch + WIP head (review 78344); waits on #13284 |
| [#13378](https://github.com/gunb-ai/gunbc/pull/13378) | std.unicode.scalar: partial from_code_point (typed refusal) + char_tex | ready / DIRTY | sharp-raven: stack APPROVED (review 76866) but red/stale vs #13453; revive from prep/13378-no-rung-drop |
| [#13623](https://github.com/gunb-ai/gunbc/pull/13623) | Type undeclared lambda actuals from preceding application formals (N7  | ready / CLEAN | sharp-raven: stacked on #13558, red on target (approved) |
| [#13392](https://github.com/gunb-ai/gunbc/pull/13392) | Manual git R0 fixture: request the default -z records its decoders mod | draft / CLEAN | sharp-raven: stacked on red #13378 (approved) |
| [#13390](https://github.com/gunb-ai/gunbc/pull/13390) | Split RFC 3986 §2.1 percent-coding into extdeps.uri.percent_encoding;  | draft / CLEAN | sharp-raven: stacked on red #13378 (approved) |
| [#13557](https://github.com/gunb-ai/gunbc/pull/13557) | plans: tie native memory rulings to resumable roadmap milestones | draft / CLEAN | side chat: plans/roadmap rows only; optional |
| [#13613](https://github.com/gunb-ai/gunbc/pull/13613) | Tailscale ACL: admit tag:dashboard to the dusk-1 serving API (tcp:8080 | ready / BLOCKED | side chat: red (AmbiguousBareNameRead in tailscale_acl_witness); unrelated to cutover |
| [#13639](https://github.com/gunb-ai/gunbc/pull/13639) | Deployment risk conformance: one environment model for the repo | draft / BLOCKED | silent-lark: D2 respawn, REQUEST_CHANGES review 78345 (unstated Absent->srv1 fallbacks) |
| [#13217](https://github.com/gunb-ai/gunbc/pull/13217) | Deployment risk D2: role-following singletons resolve the prod-role ho | ready / DIRTY | silent-lark: D2, red floor + DIRTY |
| [#13339](https://github.com/gunb-ai/gunbc/pull/13339) | D2 follow-up: role-singleton observation by host self-report (ssh prob | draft / CLEAN | silent-lark: stacked on #13217 |
| [#13429](https://github.com/gunb-ai/gunbc/pull/13429) | Workspace pack: same-size synthetic transcript placeholders; withhold  | ready / DIRTY | superseded by 13442 |
| [#13617](https://github.com/gunb-ai/gunbc/pull/13617) | Key native producer_compiler on the running seed executable | draft / BLOCKED | swift-bat-828 (error): seed producer_compiler provenance; author paused it (v1/CI programme); legacy |
| [#13629](https://github.com/gunb-ai/gunbc/pull/13629) | Floor: reach differential reuses the prepared required_floor authority | draft / BLOCKED | swift-bat-828 (error): v1 floor reach-differential reuse; legacy (the floor is retired by the v1 withdrawal) |
| [#13578](https://github.com/gunb-ai/gunbc/pull/13578) | Generalize hosted OpenAI-compat failures (one classifier, no retry) | ready / CLEAN | swift-ibex: left out (RC) |
| [#13212](https://github.com/gunb-ai/gunbc/pull/13212) | Starter eval set: .dag modeling and DESIGN.md adherence, graded mechan | ready / CLEAN | valiant-crab (unreported): eval set, green and CLEAN, but no PR body and no completed review (its one review failed). Write a body, get a review, then it is an easy fold |

## Unreported, owned by a live lane

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |

## Orphans

No live session owns these. The proposed disposition: a CLEAN, non-draft PR from the last week gets reviewed for folding, and anything DIRTY, drafted, or older than a week is closed. Each one is checked before it's closed.

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |
| [#13603](https://github.com/gunb-ai/gunbc/pull/13603) | Fix fleet convergence pool display name length | draft / CLEAN | opened 2026-10-09; proposed: close (record remaining work first) |
| [#13565](https://github.com/gunb-ai/gunbc/pull/13565) | ROADMAP #117: Realize node HTTP serve smoke as typed ops, not heredoc | ready / CLEAN | opened 2026-10-08; proposed: review for fold |
| [#13564](https://github.com/gunb-ai/gunbc/pull/13564) | argv dissolution: replace hand-typed id/hostname argv with typed build | ready / CLEAN | opened 2026-10-08; proposed: review for fold |
| [#13328](https://github.com/gunb-ai/gunbc/pull/13328) | recurring_failure_mode: re-scope the native-route row to the package-p | ready / CLEAN | opened 2026-10-05; proposed: review for fold |
| [#13295](https://github.com/gunb-ai/gunbc/pull/13295) | Consolidate cassette joinery, power and shared platform profiles | ready / DIRTY | opened 2026-10-04; proposed: close (record remaining work first) |
| [#13223](https://github.com/gunb-ai/gunbc/pull/13223) | Separate pinned Orca fit-prototype preparation from printer execution | draft / DIRTY | opened 2026-10-04; proposed: close (record remaining work first) |
| [#13218](https://github.com/gunb-ai/gunbc/pull/13218) | Route approval clients to the active store writer | draft / DIRTY | opened 2026-10-04; proposed: close (record remaining work first) |
| [#13149](https://github.com/gunb-ai/gunbc/pull/13149) | Unify printer starts behind ntfy approval, idle checks and durable cla | draft / DIRTY | opened 2026-10-03; proposed: close (record remaining work first) |
| [#12917](https://github.com/gunb-ai/gunbc/pull/12917) | DS4.1 | ready / DIRTY | opened 2026-10-01; proposed: close (record remaining work first) |
| [#12749](https://github.com/gunb-ai/gunbc/pull/12749) | Observe mode for the approval broker: unit, journal, slice cgroup, /li | ready / DIRTY | opened 2026-09-30; proposed: close (record remaining work first) |
| [#12737](https://github.com/gunb-ai/gunbc/pull/12737) | kimi k3 low | ready / DIRTY | opened 2026-09-30; proposed: close (record remaining work first) |
| [#12707](https://github.com/gunb-ai/gunbc/pull/12707) | luna pro low qual | ready / DIRTY | opened 2026-09-29; proposed: close (record remaining work first) |
| [#12691](https://github.com/gunb-ai/gunbc/pull/12691) | Journal initial commissioning and gate workspace supply on committed r | draft / CLEAN | opened 2026-09-29; proposed: close (record remaining work first) |

## Closed by the closeout

| PR | Why |
| --- | --- |
| #13648 | Auto-opened on neat-boar-16's archive from stale branch pkg11b-fix: no merge base with main, and the only new commit is .probe_tmp scratch. Its rung drops and projection roster are already on main |
| #13649, #13650, #13651, #13652, #13654 | Auto-opened or re-opened when managers were archived; content is in #13641, or a superseded duplicate (#13654 duplicates #13560) |
| #13656 | deep-crab-89 wind-down WIP of the FQDN follow-up; self-contradictory (review 78372) |
| #13636, #13644, #13646, #13647 | Auto-opened integration PRs. Their content is in #13641 (or, for #13647, deliberately left out) |
| #13638 | Duplicate of #13430 |
| #13552, #13555 | They build the BMC-hosted boot route, which was dropped; the srv13-16 naming is carried by #13570 |
| #13631 | Doesn't compile; the credential model is unresolved; to be restarted from scratch |

Every branch was kept.

## Outstanding work that isn't a mergeable PR

This is work that has to go somewhere even though none of it merges now. Each row names its owner today and what unblocks it.

### Follow-up code (no PR yet, or the PR is out of the snapshot)

| Item | Source | What's needed |
| --- | --- | --- |
| #13125 lifecycle fixes: drain disable on retract; drain After/Wants user@<uid>.service | bold-bee-114 (WIP head 9f206434, excluded) | A fresh branch without the scratch_tc test; 2 oracle tests fail today |
| #13125 event-driven units are not deployed on srv2 | bold-bee-114 | Cut over from the legacy user timer gunbc-srv2-deploy-belt.timer to the emitted system units |
| #13097 G1 store/belt cutover | bold-bee-114 | Unblocked when #13549 (the T?==T sweep) lands |
| #13513 door-binding stall | bold-bee-114 | A door-grain claim. #13560's door control refuses an isolated module at TranslateTo |
| #13616 real-path receipt | bold-bee-114 | Capture the first srv2 tick after deploy; per-shard reads should be 1 |
| #13632 jq fan-config migration | stern-boar-596 | Fix length(null)->0 with select(type==...); add a pure request seam and a route test |
| Redfish/http.Client onto transport rest (was #13631) | stern-boar-596 | Restart from scratch; decide the credential model first |
| #13626 follow-up: FQDN vs short slot label in the identity probe | stern-boar-596 / deep-crab-89 | DECISION NEEDED: short (-s) or bare hostname. A WIP attempt (#13656, closed; branch session/deep-crab-89-followup @8255a8f1bd) contradicted itself (review 78372). Today the probe refuses fail-closed |
| #13578 dusk-1 Qwen replica + failure classifier | swift-ibex-601 | Fix review 78337: a third placement arm, refusing an absent clock, the stale capacity claim |
| #13430 XL-2 string templates (PR2b) | XL-2 lane (closed) | Unowned. Its REQUEST_CHANGES is stale; it needs #13436's named conversions |
| #13613 Tailscale ACL grant for dusk-1 | side chat | Find the ambiguous name in tailscale_acl_witness |
| #13597 crate partitioner | side chat | Main merge + regen, the remaining acceptance runs, and a PR body update |
| Unit-modeling fix (review 78356): RetryAfterSeconds and QuotaTermSeconds as Second | closeout worker adhoc-1bfd89a3-3eb | In progress; its PR targets #13641 |
| Wave-2 conflicts (#13583, #13582, #13615, #13609) + full regen | closeout worker adhoc-ac536796-f2c | In progress; its PR targets #13641 |
| #13617 seed binary provenance (paused, legacy) | swift-bat-828 (archived) | If revived, fix review 78362: the prose String row v1_seed_binary_provenance_admission_note becomes a typed admission row; retire/repoint gunbc_cli_build_identity_seed_dissolve_trigger and the ROADMAP boundary text that still describe build.rs rustc-env |
| #13595 String-vs-Int control | gentle-dove-36 | Switch #13574's equality control to a TypeMismatch count, then fold |
| Nested-Present Rc-deref emitter fix; derive the expected identities from the cases; the option-C closure-by-prefix note | gentle-dove-36 | Never started (row and specimen are in #13593) |
| #13482 / #13634 C2 cross-run typecheck store | royal-moth-86 | Review 78363: delete the scratch t_tmp.sh re-added on #13634's tip (19c41daf). Then: Switch to local_store_held_session; a real-route control (store_openings=1, hit/miss/committed>0); unset/cold/warm timings; observed_largest_entry_bytes; the live_pool_thread_tests red; the List-completeness question at ~757 files |
| Fleet-converge mint modes for run_cache_object_read/_write | royal-moth-86 | Never started |
| neat-boar-16 unowned follow-ups (7) | neat-boar-16 | (1) The cost shape of the cost_debt_* / reach_base_standings tests; (2) the typed-store snapshot grain, where symbol_index ~1.71 GB per TypeEnv (calm-pike-525/calm-boar-904); (3) the floor_cost_debt_edit conjunct gap; (4) the reach-differential Err-discard; (5) the CI classifier counting cargo network warnings as structural; (6) the Int/Symbol/Char/Filesystem de-forks; (7) the LoweredShape arm frontier. Detail is in memory pending/neat-boar-16-capacity-freeze.txt |
| OCI UsagePart.micros -> e2_micro_instances rename | nimble-heron-805 | Verified locally, never pushed; land it after #13641 |
| OCI A1 shape | nimble-heron-805 | Stays off until the month-to-date meter is observed |
| #13330 derived-node identity step 2 | gentle-dove-36 (transferred from neat-boar-16) | Fix review 76999: inline declaration_ref_is_type_parameter at its one caller in 04_resolve; otherwise sound |
| #13179 checker message prescribes `== Present{..}`, which executes false at runtime | lively-ram-153 | Change the prescribed remedy to match-on-Present or whole-value equality (#13581 has the receipts) |
| lively-ram recorded next units | lively-ram-153 | Deep substitution in infer_frame_instantiated (derivation on #13210); the sole_constructor native wall; route-gap class (b); DP-M2 residue (4 retained-shell one-offs); the argv census recount (WIP session/witty-tern-54, needs a partitioned emitter); the #13257 follow-up |
| silent-lark D2 stack (#13217 -> #13339; respawn #13639) | silent-lark-156 | #13217 needs a main merge (import union in fleet_converge_workflow.dag, regen fleet-converge.yml keeping #13334 lines), then floor phases_failed=0. #13639 needs the three Absent->srv1 fallbacks fixed (review 78345). After D2: a one-time srv1 ownership-marker write |
| silent-lark gaps | silent-lark-156 | walk_cgroup_pending quadratic concat (§6); fabric_storage_file_store has no head listing; rename srv1_gunbc_approval_broker_root to be host-neutral; python/go still refuse capture channels |
| #13488 nested-optionality carrier | gentle-dove-36 | Review 78334 (REQUEST_CHANGES): the body says 'no code' but it changes 31 compiler files; the plan's own 'go' gate is not in evidence; CardOptional { layers: Int } admits 0/-1 (the plan requires OptionalLayers); it silently saturates at 8 layers; peeling Required succeeds instead of refusing. Restart from the plan's OptionalLayers design after an explicit go |
| #13574 interim T?==T wall | gentle-dove-36 / eager-gull-22 | Review 78262: the approval cites escalation msg_66f62924, which can't be found; it recognizes List reads by leaf name, not declaration (§4). Keep or revert is with the operator |
| #13265 type-param scope | gentle-dove-36 | Review 78367: without module-scope params, Result's `ok` binds to an unrelated fn ok (fabricated type). Don't land until binders carry the instantiated field type |
| sharp-raven integration fold | closeout | integration/sharp-raven-357 @82d7624f5a conflicts with #13641 in ~200 files; a dedicated worker folds it after the conflict worker's PR lands |
| srv2 v2-native census/frontier run on main d0f2067 | sharp-raven-357, owed to the operator | Relaunched detached as srv2 user unit v2native-census.service (invocation 43a0c64d4e45, MemoryMax=60G, ~25 min emit+build, then the corpus walk, 3h cap per instrument). Check: `ssh srv2 'systemctl --user status v2native-census; cat ~/rmain/v2native.exits'`; logs ~/rmain/v2-native-census.log and v2-native-frontier.log. Someone must read the result and report the stage-by-stage counts (swift-bat-828 is archived) |
| LIVE MAIN DEFECT: #13500 made bare DependencyView ambiguous | sharp-raven-357 | Any closure with gunbc.roadmap.roadmap_hierarchy_view plus src/v2/lens/unused_parameters.dag or structural_resolution.dag refuses. Fix: full explicit import lists in those two lens files (#13377's partial import made it worse) |
| N7 filesystem_io refuses in normalize (service_realization_unreachable) | sharp-raven-357 | Needs XL-2 3c (3b-ii is not sufficient). Census in #13637; forwarded to smart-gull-336. NOTE: the XL-2 lane is cancelled, so 3c is unowned |
| v2-infer PRs validated only by the v1 floor | sharp-raven-357 | Only #13502 and #13308 execute native binaries. Cutting the floor removes the validator unless the claims move to a native job (smart-gull-336 has been told) |
| Logged follow-ups | sharp-raven-357 | #13436 decimal_digit_of_reduced_units accepts out-of-range Int; #13320 ViaHomomorphism arm admits without installing a conversion; bash_orch_if Rejected=>"" fail-open (partly fixed by #13618) |
| #13608 newer head 8ff94ca471 (deep-ram-343's emit fix: is_known_variant scan -> carried variant_to_enum; predicted emit 1500s -> 100-300s) | eager-gull-22 | Unverified, no CI. The snapshot takes #13608 at c7c3ef7744. #13211 needs the newer head |
| eager-gull closed children | eager-gull-22 | bold-moth, warm-crane, zesty-wren, sleek-koi, sharp-ant and quiet-stag were closed before the no-close directive; their PRs are in this accounting via their re-homing |
| #13548 unimported-type import migrate | sharp-raven-357 | Review 78338: a hand-listed roster of native bare spellings duplicates is_kernel_type/is_container_type; a second import-list scanner duplicates source_reference_repoint's. Consume both instead |
| #13630 callee facts on the underived arm | sharp-raven-357 | Review 78359 (§5 fail-open): a refused contract keeps a denotation, so eval dispatches into the body. Gate eval on DerivedGrounding, or carry the recovered Arrow in a separate field |
| #13541 filter/any retirement (337 files) | sharp-raven-357 | Delete the 15 root scratch scripts (review 78343), rebase, then give it a fresh full review |
| #13637 XL-2 3c consumer census (docs-only, approved) | sharp-raven-357 | Not merged (docs-only preference; plans bankruptcy). It is the input to the unowned 3c deletion that clears N7 filesystem_io |
| #13212 starter eval set (49 cases + grader, about 2.1k lines) | valiant-crab-775 / smart-bear-46 | Green and CLEAN. Needs a real PR body and a completed review; its consumer (who runs the eval) should be named |
| session/lively-wren-411 @6194f56012 | bold-bee-114 | Duplicate member_not_a_binder fix, superseded by #13560. No PR; delete the branch |
| wise-koi-396 (shell-dag live-deploy restart-tailscale) | stern-boar-596 | Nothing was ever pushed; the work is unstarted |

### Operator steps

| Step | Source |
| --- | --- |
| OpenRouter accessor grant + a live harness_hosted_probe_cli run | swift-ibex-601 |
| claude-code-oauth-harness: the secret, the grant, srv1 custody, a live turn | swift-ibex-601 |
| cursor-api-key-harness and codex-auth-harness: secrets, grants, srv2 custody, a live turn each | swift-ibex-601 |
| Royal-moth credential checklist: R2 cache tokens, boot SA grant, fleet-desired WIF + census row | royal-moth-86 |
| Consume or delete the unused secrets: openrouter -2/-3, groq, opencode | swift-ibex-601 |
| OCI srv1: accessor grant (phone approval) -> host_credential_custody_converge srv1 oracle_oci_api_signing_key -> as root, `gunbc run ... always_free_ensure.dag --function ensure` until it exits 0. A stale lock in /etc/gunbc/oracle-oci is never broken automatically | nimble-heron-805 |
| The browser toolchain on the srv1/srv3/srv4 floor runners (unblocks #13382/#13383/#13411) | lively-ram-153 |
| Fabric storage measurement: transcript_rate_observe, source_pack, checkpoint_measure, restore | warm-badger-442 |
| Native lane records no verdicts on main; a terminal Failed renders as PlannedWithoutTerminalVerdict; the shared FleetSsh host-effect arm | lively-ram-153 |
| srv1-09 floor runner: no cgroup memory limit (HostBudgetUnreadable); floor_class mislabels it 'structural' | stern-boar-596 |

### Open decisions

| Decision | Source |
| --- | --- |
| Host self-report transport: A (tailnet /instance.json + WhoIs, 2 grants) or B | silent-lark-156 |
| Fabric-store Move mechanism (A: fenced pull via :10000, exact readback, retire) | silent-lark-156 |
| Remove Group B (srv9-12) from config/convergence: the repo models a fleet that was sold and wiped | valiant-crab-775 |
| Control-plane platform (#13084/#13161) | warm-badger-442 |
| r2_bucket_ensure skips lifecycle convergence when a purpose's rules are [] | warm-badger-442 |
| external_model_scope 2 red on main; quarantine_probe_disposition 2/9 red | warm-badger-442 |
| ObservedReceipt next rungs: in-substrate sha256; an executing EEPROM read | warm-badger-442 |
| Close or keep #13382/#13383/#13411 | lively-ram-153 |
| #13544 Group A decommission is DIRTY: rebase it or fold it | valiant-crab-775 |

### Lanes not yet reported

eager-gull-22 is reverting #13574/#13595 and re-gating their integration branches. swift-bat-828 (state of CI) is in an error state and has 2 PRs. neat-boar-16's #12942 is in the mega branch, and its unowned follow-ups are in its memory note `pending/neat-boar-16-capacity-freeze.txt`, which still needs reading.

### swift-bat-828 branches (checked by a side chat)

#13505, #13542, #13551, #13562 and #13619 are merged. #13628 is closed. `swift-bat-828/lens-explicit-imports` is identical to main and has no PR, so the branch can be deleted.

### Falsified defects (do not refile)

The marshal projected-let drop; negative-literal verdict loss (#13289). Both from lively-ram-153.

### Archive flush residue

Archiving a session commits its leftover worktree files as a 'WIP' commit and pushes them. That happened on #13648 (.probe_tmp scratch) and #13634 (t_tmp.sh). Any archived manager's branch may carry such a tip commit. Check the tip before reviving or merging one of these branches.

## Notes

- The plans bankruptcy (#13550) deleted `buganizer-visual-contract.md` and `printed-chassis-program.md`, and that deletion won over edits on our side. Some comments and `sites/README.md` still mention them.
- #13602's green checks are on its current head `f867e9d670`, which is later than the srv1 authorization push.

## Kept up by request

- [#13203](https://github.com/gunb-ai/gunbc/pull/13203) Complete Blackjack simulation milestone (branch `blackjack-milestone`) stays open and outside the closeout.
