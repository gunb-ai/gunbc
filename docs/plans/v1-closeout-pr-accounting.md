# v1 closeout: open PR accounting

As of 2026-10-09; it includes the operator's side-chat dispositions. There are 162 open PRs on gunb-ai/gunbc. Apart from #13203 (blackjack, kept up by request), every one of them ends either in the mega branch #13641 (`integration/v1-closeout`) or closed with a stated reason. The owners come from the dashboard's PR-to-session records and the side chats. The dispositions come from each manager's closeout report. Merge state is GitHub's `mergeStateStatus` at the time of writing.

| Disposition | PRs | What happens |
| --- | --- | --- |
| In the mega branch | 29 | Lands when #13641 lands |
| Pending fold | 66 | On a manager's integration branch, with the conflict worker, or waiting for CI; folded into #13641 next |
| Blocked | 4 | Needs an operator or host action first |
| Left out | 38 | Not in the snapshot. Its remaining work is recorded under Outstanding work before the PR is closed; nothing is dropped |
| Unreported, owned by a live lane | 10 | Waiting on that lane's report |
| Orphans: no live owner | 14 | Triaged by the closeout: fold if done and clean, otherwise close |
| Kept up by request | 1 | #13203 blackjack |

## In the mega branch

These are merged into `integration/v1-closeout`.

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |
| [#13641](https://github.com/gunb-ai/gunbc/pull/13641) | v1 closeout mega branch (wave 1) | ready / BLOCKED | neat-wolf-604 |
| [#13626](https://github.com/gunb-ai/gunbc/pull/13626) | host_reach_identity_probe: the identity probe's argv is spelled by its | ready / CLEAN | stern-boar-596 |
| [#13622](https://github.com/gunb-ai/gunbc/pull/13622) | dispatch-actuator witness: supply the unrooted specimen instead of bor | ready / CLEAN | bold-bee-114 |
| [#13616](https://github.com/gunb-ai/gunbc/pull/13616) | Belt SCM capture: read each re-bound object shard once (recapture 30x  | ready / CLEAN | bold-bee-114 |
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
| [#13513](https://github.com/gunb-ai/gunbc/pull/13513) | mandatory_tag gate: read the lowered data-declaration carrier (clean f | ready / CLEAN | bold-bee-114 |
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
| [#13583](https://github.com/gunb-ai/gunbc/pull/13583) | One directory authority for dashboard instances (D14 OwnedDirectory sl | ready / CLEAN | conflict worker (keep 13583 spec.dag) |
| [#13582](https://github.com/gunb-ai/gunbc/pull/13582) | Delete the v2 packrat parse memo carrier (R2, stacked on #13577) | draft / CLEAN | conflict worker (stack on 13577) |
| [#13615](https://github.com/gunb-ai/gunbc/pull/13615) | Delete unused memo and preparation carriers (native-memory-inert-carri | draft / CLEAN | conflict worker (stack; keep floor_preparation_witness_test.dag deleted) |
| [#13608](https://github.com/gunb-ai/gunbc/pull/13608) | RFM: fixture-closure-union-emit suspected superlinear cost | ready / CLEAN | via eager-gull-22 |
| [#13596](https://github.com/gunb-ai/gunbc/pull/13596) | target_invocation witness: repair two false claims; file the unplanned | ready / CLEAN | via eager-gull-22 |
| [#13595](https://github.com/gunb-ai/gunbc/pull/13595) | Reclassify the match-arm proven-disjoint InternalError as a located Ty | draft / CLEAN | via eager-gull-22 |
| [#13590](https://github.com/gunb-ai/gunbc/pull/13590) | Native ingest: refuse two files claiming one module path | ready / CLEAN | via eager-gull-22 |
| [#13588](https://github.com/gunb-ai/gunbc/pull/13588) | std.change: import any and Present so its own names resolve natively | ready / CLEAN | via eager-gull-22 |
| [#13586](https://github.com/gunb-ai/gunbc/pull/13586) | guarantee_stall: correct accumulator-copy native-resolve residue recei | ready / CLEAN | via eager-gull-22 |
| [#13585](https://github.com/gunb-ai/gunbc/pull/13585) | Enroll native-resolve coverage for fold handler binders | ready / CLEAN | via eager-gull-22 |
| [#13580](https://github.com/gunb-ai/gunbc/pull/13580) | regen-round-cost: re-exec once when the seed build replaces the runnin | ready / CLEAN | via eager-gull-22 |
| [#13579](https://github.com/gunb-ai/gunbc/pull/13579) | Climb import-less UniqueBare from proximity to lexical on-chain bindin | ready / CLEAN | via eager-gull-22 |
| [#13575](https://github.com/gunb-ai/gunbc/pull/13575) | Isolate fixture census on a scratch MultiEntryIndex | ready / CLEAN | via eager-gull-22 |
| [#13571](https://github.com/gunb-ai/gunbc/pull/13571) | Fallout from #13179: latent T? == T sites and stale grounded_principal | ready / CLEAN | via eager-gull-22 |
| [#13520](https://github.com/gunb-ai/gunbc/pull/13520) | Delete dead emit_resolve_transitively_fn stamp from generated main | ready / CLEAN | via eager-gull-22 |
| [#13519](https://github.com/gunb-ai/gunbc/pull/13519) | stage0 witness bins: close compile subjects through the closure author | ready / CLEAN | via eager-gull-22 |
| [#13517](https://github.com/gunb-ai/gunbc/pull/13517) | Managed-host cut O1c-3b: BmcSecure gated through the convergence fold | ready / DIRTY | via eager-gull-22 |
| [#13507](https://github.com/gunb-ai/gunbc/pull/13507) | Board identity: bound host's FRU board reaches its per-board authority | ready / CLEAN | via eager-gull-22 |
| [#13503](https://github.com/gunb-ai/gunbc/pull/13503) | Move AMI-bundle-derived MegaRAC content out of public gunbc | ready / BLOCKED | via eager-gull-22 |
| [#13497](https://github.com/gunb-ai/gunbc/pull/13497) | Emit arrival_converge; IAM pair and pinned accessor (msg_f03558d1) | ready / CLEAN | via eager-gull-22 |
| [#13341](https://github.com/gunb-ai/gunbc/pull/13341) | v2 parse: occurrence ids minted once per parse across a rejected attem | ready / CLEAN | via eager-gull-22 |
| [#13333](https://github.com/gunb-ai/gunbc/pull/13333) | v2 resolve: the single-tree namespace binds its own variants and recor | ready / CLEAN | via eager-gull-22 |
| [#13247](https://github.com/gunb-ai/gunbc/pull/13247) | mtcollins1 runner: boot leg, runner-host medium + runner-host-up termi | ready / BLOCKED | via eager-gull-22 |
| [#13225](https://github.com/gunb-ai/gunbc/pull/13225) | mtcollins1 runner: extract qualification instruments from the run (leg | ready / DIRTY | via eager-gull-22 |
| [#13598](https://github.com/gunb-ai/gunbc/pull/13598) | Retire native_serve_request_budget_unrealized: native-serve holds the  | ready / CLEAN | via gentle-dove-36 |
| [#13593](https://github.com/gunb-ai/gunbc/pull/13593) | File emitter defect: variant pattern nested inside Present drops the R | ready / CLEAN | via gentle-dove-36 |
| [#13584](https://github.com/gunb-ai/gunbc/pull/13584) | Seed eval: empty list is not Unit | ready / CLEAN | via gentle-dove-36 |
| [#13574](https://github.com/gunb-ai/gunbc/pull/13574) | Interim check-time refusal of list-read == Present{..} | ready / CLEAN | via gentle-dove-36 |
| [#13567](https://github.com/gunb-ai/gunbc/pull/13567) | Emit algebra length on host String as string_length | ready / CLEAN | via gentle-dove-36 |
| [#13549](https://github.com/gunb-ai/gunbc/pull/13549) | Fix every T? == T site the whole-population census found | ready / CLEAN | via gentle-dove-36 |
| [#13454](https://github.com/gunb-ai/gunbc/pull/13454) | Native broker 2K-b: identity-keyed type_summaries; Host-Option decided | ready / CLEAN | via gentle-dove-36 |
| [#13260](https://github.com/gunb-ai/gunbc/pull/13260) | emit_rust: a type parameter carries its own cardinality (K? emits Opti | ready / CLEAN | via gentle-dove-36 |
| [#13255](https://github.com/gunb-ai/gunbc/pull/13255) | Emitter: a cloned Rc match scrutinee owes Clone on its generics wherev | ready / CLEAN | via gentle-dove-36 |
| [#13243](https://github.com/gunb-ai/gunbc/pull/13243) | Inference: a variant literal of a generic coproduct takes the expected | ready / DIRTY | via gentle-dove-36 |
| [#13224](https://github.com/gunb-ai/gunbc/pull/13224) | Native effect realization admission; native mains bind admitted handle | ready / CLEAN | via gentle-dove-36 |
| [#13202](https://github.com/gunb-ai/gunbc/pull/13202) | Native broker 2C/G: optional data rows keep their ? (signature and JSO | ready / CLEAN | via gentle-dove-36 |
| [#13460](https://github.com/gunb-ai/gunbc/pull/13460) | map_get fork: the declared projection renames to map_get_checked | ready / CLEAN | via lively-ram-153 |
| [#13108](https://github.com/gunb-ai/gunbc/pull/13108) | admit_callers: restore the native-route caller-admission wall (DP-M6 i | ready / CLEAN | via lively-ram-153 |
| [#13621](https://github.com/gunb-ai/gunbc/pull/13621) | A data reference uses its value: ConstantMemberEdge marks the lowered  | draft / BLOCKED | via sharp-raven-357 |
| [#13618](https://github.com/gunb-ai/gunbc/pull/13618) | workflow scripts: a refused bash emission refuses, never emits an empt | ready / CLEAN | via sharp-raven-357 |
| [#13545](https://github.com/gunb-ai/gunbc/pull/13545) | Compare an Optional with an Optional at 46 latent sites no gated closu | ready / CLEAN | via sharp-raven-357 |
| [#13511](https://github.com/gunb-ai/gunbc/pull/13511) | Join corpus Int and kernel Int at inhabitance (N7 g_tokenize_parse) | ready / CLEAN | via sharp-raven-357 |
| [#13502](https://github.com/gunb-ai/gunbc/pull/13502) | Pair identity-cast native with the real emit artifact | ready / CLEAN | via sharp-raven-357 |
| [#13501](https://github.com/gunb-ai/gunbc/pull/13501) | v2 tokenize: bind EmptyPattern on 01_tokenize's import chain (N7 resol | ready / CLEAN | via sharp-raven-357 |
| [#13496](https://github.com/gunb-ai/gunbc/pull/13496) | Bool-native rust_logic/variant emit after BindingRef octet de-fork | ready / CLEAN | via sharp-raven-357 |
| [#13469](https://github.com/gunb-ai/gunbc/pull/13469) | Out-of-gate emit census: binder types, A4 order on rust fixtures, refi | ready / CLEAN | via sharp-raven-357 |
| [#13453](https://github.com/gunb-ai/gunbc/pull/13453) | Floor judges base-revision facts with the base revision's own compiler | ready / CLEAN | via sharp-raven-357 |
| [#13438](https://github.com/gunb-ai/gunbc/pull/13438) | Precedence-climbing tree contract for dag binary expressions | ready / DIRTY | via sharp-raven-357 |
| [#13436](https://github.com/gunb-ai/gunbc/pull/13436) | XL-2: named conversions for every non-String interpolation hole (std.n | ready / BLOCKED | via sharp-raven-357 |
| [#13425](https://github.com/gunb-ai/gunbc/pull/13425) | Gate the foreign-language grammar claims (census phase 4): runnable te | ready / CLEAN | via sharp-raven-357 |
| [#13406](https://github.com/gunb-ai/gunbc/pull/13406) | Claims tokenize .dag text through tokenize_prepared(dag_prepared_lex() | ready / CLEAN | via sharp-raven-357 |
| [#13380](https://github.com/gunb-ai/gunbc/pull/13380) | RFM: 'first element of a list' nickname fork (roster first / list_head | ready / CLEAN | via sharp-raven-357 |
| [#13379](https://github.com/gunb-ai/gunbc/pull/13379) | v2.std.grammar: qualify the GrammarExpr Optional arm (N7 resolve link) | ready / CLEAN | via sharp-raven-357 |
| [#13325](https://github.com/gunb-ai/gunbc/pull/13325) | v2 infer: derive a call's result type from a declared return (instanti | ready / CLEAN | via sharp-raven-357 |
| [#13320](https://github.com/gunb-ai/gunbc/pull/13320) | v2 infer: an integer literal elaborates at its expected type through s | ready / CLEAN | via sharp-raven-357 |
| [#13308](https://github.com/gunb-ai/gunbc/pull/13308) | Census part 2: the native infer census on v2-native-census (census-inf | ready / CLEAN | via sharp-raven-357 |
| [#13307](https://github.com/gunb-ai/gunbc/pull/13307) | v2: bare count over lists binds the std.algebra collection size row, a | ready / CLEAN | via sharp-raven-357 |
| [#13126](https://github.com/gunb-ai/gunbc/pull/13126) | Grammar overlap: validation refuses every overlap row; required zero-c | ready / CLEAN | via sharp-raven-357 |
| [#13123](https://github.com/gunb-ai/gunbc/pull/13123) | C4: native route enforces constructor confinement (sole_constructor +  | ready / CLEAN | via sharp-raven-357 |
| [#13625](https://github.com/gunb-ai/gunbc/pull/13625) | CAX41 guest onboarding; fleet-converge from a branch after a per-run n | draft / BLOCKED | side chat, draft; folds when CI green (touches fleet-converge.yml) |
| [#13614](https://github.com/gunb-ai/gunbc/pull/13614) | seed interpreter: withdraw the spelling-admitted, label-blind PureCall | ready / CLEAN | side chat, folds when CI green |
| [#13611](https://github.com/gunb-ai/gunbc/pull/13611) | demand engine: a request under a different nature is a durable key con | ready / CLEAN | side chat, folds when CI green |
| [#13597](https://github.com/gunb-ai/gunbc/pull/13597) | Make the crate partitioner the only emitted-Rust layout; delete single | draft / DIRTY | side chat, not yet done: needs a main merge + regen, the remaining acceptance runs, and a PR body update; folds once marked ready |
| [#13610](https://github.com/gunb-ai/gunbc/pull/13610) | walk_cgroup, entry_presence: tail-position self-recursion so both modu | ready / CLEAN | via silent-lark-156 |
| [#13472](https://github.com/gunb-ai/gunbc/pull/13472) | Realize rust shell stderr-capture channels (unblocks #13217 floor) | ready / DIRTY | via silent-lark-156 |

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
| [#13430](https://github.com/gunb-ai/gunbc/pull/13430) | XL-2 PR2b: string templates — disjoint lexer, one ^dag_string_template | ready / BLOCKED | XL-2 cancelled |
| [#13097](https://github.com/gunb-ai/gunbc/pull/13097) | G1: belt verify through the materialization provider; compute outcomes | ready / BLOCKED | bold-bee: red on T?==T sites until #13549; draft |
| [#13604](https://github.com/gunb-ai/gunbc/pull/13604) | Prepare the required-floor gate once; project policy from that subject | draft / BLOCKED | eager-gull: left out |
| [#13591](https://github.com/gunb-ai/gunbc/pull/13591) | Resolve free kernel calls to std.primitives identity; infer refuses th | draft / CLEAN | eager-gull: left out |
| [#13576](https://github.com/gunb-ai/gunbc/pull/13576) | DRAFT: floor-control for #13575 (plan forged-probe census) | draft / UNSTABLE | eager-gull: left out |
| [#13432](https://github.com/gunb-ai/gunbc/pull/13432) | [DO NOT MERGE] baseline control: fixture-closure union with only a KVM | draft / BLOCKED | eager-gull: left out |
| [#13420](https://github.com/gunb-ai/gunbc/pull/13420) | Managed-host cut 5: fan, served-UI and KVM observations over ManagedHo | draft / BLOCKED | eager-gull: left out |
| [#13391](https://github.com/gunb-ai/gunbc/pull/13391) | Refuse a product value at a kernel-scalar (or refined) declared type | ready / DIRTY | eager-gull: left out |
| [#13288](https://github.com/gunb-ai/gunbc/pull/13288) | mtcollins1 runner: the dedicated runner-qualification group, ensured o | ready / CLEAN | eager-gull: left out |
| [#13252](https://github.com/gunb-ai/gunbc/pull/13252) | mtcollins1 runner offer at measured one-socket shape + census meminfo  | ready / CLEAN | eager-gull: left out |
| [#13211](https://github.com/gunb-ai/gunbc/pull/13211) | mtcollins1 runner: dispatch the floor to the attempt's slot and read t | ready / BLOCKED | eager-gull: left out |
| [#13633](https://github.com/gunb-ai/gunbc/pull/13633) | Re-land #13255 (3rd conflict) + propose de-hotspotting native_emission | draft / DIRTY | gentle-dove: left out |
| [#13488](https://github.com/gunb-ai/gunbc/pull/13488) | Design: nested optionality census + layer-count carrier (no code) | ready / BLOCKED | gentle-dove: left out |
| [#13475](https://github.com/gunb-ai/gunbc/pull/13475) | fold_list empty: [] no longer locks the accumulator as List<Unit> | draft / BLOCKED | gentle-dove: left out |
| [#13265](https://github.com/gunb-ai/gunbc/pull/13265) | Type parameters bind only inside their own declaration (rule 2 / A'; s | draft / DIRTY | gentle-dove: left out |
| [#13632](https://github.com/gunb-ai/gunbc/pull/13632) | dusk qwen 3 | ready / BLOCKED | qwen: jq length(null)->0 fabricated default (DESIGN §5) |
| [#13634](https://github.com/gunb-ai/gunbc/pull/13634) | C2 #13482: the seven review fixes (own-interface entries, lookup-first | draft / BLOCKED | royal-moth: left out |
| [#13482](https://github.com/gunb-ai/gunbc/pull/13482) | C2: typecheck materialization through the local store | draft / CLEAN | royal-moth: left out |
| [#13637](https://github.com/gunb-ai/gunbc/pull/13637) | XL-2 3c: enumerate ServiceSetAside consumers before delete-first | ready / BLOCKED | sharp-raven: left out |
| [#13635](https://github.com/gunb-ai/gunbc/pull/13635) | XL-2 cleanup mgr | draft / BLOCKED | sharp-raven: left out |
| [#13630](https://github.com/gunb-ai/gunbc/pull/13630) | Recover callee Arrow through Instantiation-grounding refusal | ready / CLEAN | sharp-raven: left out |
| [#13623](https://github.com/gunb-ai/gunbc/pull/13623) | Type undeclared lambda actuals from preceding application formals (N7  | ready / CLEAN | sharp-raven: left out |
| [#13558](https://github.com/gunb-ai/gunbc/pull/13558) | N7: derive payload-binder facts for construct-field consumption | ready / CLEAN | sharp-raven: left out |
| [#13548](https://github.com/gunb-ai/gunbc/pull/13548) | DRAFT: unimported-type import migration (source_reference_repoint) | ready / CLEAN | sharp-raven: left out |
| [#13541](https://github.com/gunb-ai/gunbc/pull/13541) | Retire v2.std.algebra filter/any onto the collection roster's callback | ready / DIRTY | sharp-raven: left out |
| [#13426](https://github.com/gunb-ai/gunbc/pull/13426) | Foreign lexer reds: rust ingest bare +, TS single-quoted string body;  | draft / CLEAN | sharp-raven: left out |
| [#13399](https://github.com/gunb-ai/gunbc/pull/13399) | Move the bottom seam to the leaf std.error_primitives; trim diverges t | draft / DIRTY | sharp-raven: left out |
| [#13392](https://github.com/gunb-ai/gunbc/pull/13392) | Manual git R0 fixture: request the default -z records its decoders mod | draft / CLEAN | sharp-raven: left out |
| [#13390](https://github.com/gunb-ai/gunbc/pull/13390) | Split RFC 3986 §2.1 percent-coding into extdeps.uri.percent_encoding;  | draft / CLEAN | sharp-raven: left out |
| [#13378](https://github.com/gunb-ai/gunbc/pull/13378) | std.unicode.scalar: partial from_code_point (typed refusal) + char_tex | ready / DIRTY | sharp-raven: left out |
| [#13377](https://github.com/gunb-ai/gunbc/pull/13377) | node_query: qualify the Cardinality/Optional references (import-level  | ready / BLOCKED | sharp-raven: left out |
| [#13284](https://github.com/gunb-ai/gunbc/pull/13284) | v2: kernel-String concat through a free-monoid structure bound on the  | ready / DIRTY | sharp-raven: left out |
| [#13557](https://github.com/gunb-ai/gunbc/pull/13557) | plans: tie native memory rulings to resumable roadmap milestones | draft / CLEAN | side chat: plans/roadmap rows only; optional |
| [#13613](https://github.com/gunb-ai/gunbc/pull/13613) | Tailscale ACL: admit tag:dashboard to the dusk-1 serving API (tcp:8080 | ready / BLOCKED | side chat: red (AmbiguousBareNameRead in tailscale_acl_witness); unrelated to cutover |
| [#13339](https://github.com/gunb-ai/gunbc/pull/13339) | D2 follow-up: role-singleton observation by host self-report (ssh prob | draft / CLEAN | silent-lark: left out |
| [#13217](https://github.com/gunb-ai/gunbc/pull/13217) | Deployment risk D2: role-following singletons resolve the prod-role ho | ready / DIRTY | silent-lark: left out |
| [#13429](https://github.com/gunb-ai/gunbc/pull/13429) | Workspace pack: same-size synthetic transcript placeholders; withhold  | ready / DIRTY | superseded by 13442 |
| [#13578](https://github.com/gunb-ai/gunbc/pull/13578) | Generalize hosted OpenAI-compat failures (one classifier, no retry) | ready / CLEAN | swift-ibex: left out (RC) |

## Unreported, owned by a live lane

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |
| [#13640](https://github.com/gunb-ai/gunbc/pull/13640) | mtcollins1 runner: dispatch floor to ephemeral slot + collect instrume | draft / DIRTY | eager-gull-22 (session quiet-stag-623) |
| [#13516](https://github.com/gunb-ai/gunbc/pull/13516) | Managed-host cut O1c-3a: ManagedHostAdmission population with drop-arm | ready / CLEAN | eager-gull-22 (session eager-gull-22) |
| [#13330](https://github.com/gunb-ai/gunbc/pull/13330) | Derived-node identity step 2, shape 2: unify_generics reads a containe | ready / BLOCKED | gentle-dove-36 (session gentle-dove-36) |
| [#13643](https://github.com/gunb-ai/gunbc/pull/13643) | Census: cross-module record field values are judged (rcf pin rewritten | draft / BLOCKED | sharp-raven-357 (session sharp-raven-357) |
| [#13642](https://github.com/gunb-ai/gunbc/pull/13642) | Census: quoted-key brace is not a record field at resolve, not lowerin | ready / BLOCKED | sharp-raven-357 (session sharp-raven-357) |
| [#13428](https://github.com/gunb-ai/gunbc/pull/13428) | Carry the alias declaration: its name resolves to its target (ruling B | ready / CLEAN | sharp-raven-357 (session sharp-raven-357) |
| [#13639](https://github.com/gunb-ai/gunbc/pull/13639) | Deployment risk conformance: one environment model for the repo | draft / BLOCKED | silent-lark-156 (session silent-lark-156) |
| [#13629](https://github.com/gunb-ai/gunbc/pull/13629) | Floor: reach differential reuses the prepared required_floor authority | draft / BLOCKED | swift-bat-828 (session swift-bat-828) |
| [#13617](https://github.com/gunb-ai/gunbc/pull/13617) | Key native producer_compiler on the running seed executable | draft / BLOCKED | swift-bat-828 (session swift-bat-828) |
| [#13212](https://github.com/gunb-ai/gunbc/pull/13212) | Starter eval set: .dag modeling and DESIGN.md adherence, graded mechan | ready / CLEAN | valiant-crab-775 (session valiant-crab-775) |

## Orphans

No live session owns these. The proposed disposition: a CLEAN, non-draft PR from the last week gets reviewed for folding, and anything DIRTY, drafted, or older than a week is closed. Each one is checked before it's closed.

| PR | Title | Draft / merge state | Note |
| --- | --- | --- | --- |
| [#13607](https://github.com/gunb-ai/gunbc/pull/13607) | Connect srv1 workspace VM lifecycle and kernel prerequisites | draft / BLOCKED | opened 2026-10-09; proposed: close (record remaining work first) |
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
| #13626 follow-up: FQDN vs short slot label in the identity probe | stern-boar-596 | Decide whether to accept an FQDN; today it is a fail-closed refusal |
| #13578 dusk-1 Qwen replica + failure classifier | swift-ibex-601 | Fix review 78337: a third placement arm, refusing an absent clock, the stale capacity claim |
| #13430 XL-2 string templates (PR2b) | XL-2 lane (closed) | Unowned. Its REQUEST_CHANGES is stale; it needs #13436's named conversions |
| #13613 Tailscale ACL grant for dusk-1 | side chat | Find the ambiguous name in tailscale_acl_witness |
| #13597 crate partitioner | side chat | Main merge + regen, the remaining acceptance runs, and a PR body update |
| Unit-modeling fix (review 78356): RetryAfterSeconds and QuotaTermSeconds as Second | closeout worker adhoc-1bfd89a3-3eb | In progress; its PR targets #13641 |
| Wave-2 conflicts (#13583, #13582, #13615, #13609) + full regen | closeout worker adhoc-ac536796-f2c | In progress; its PR targets #13641 |
| session/lively-wren-411 @6194f56012 | bold-bee-114 | Duplicate member_not_a_binder fix, superseded by #13560. No PR; delete the branch |
| wise-koi-396 (shell-dag live-deploy restart-tailscale) | stern-boar-596 | Nothing was ever pushed; the work is unstarted |

### Operator steps

| Step | Source |
| --- | --- |
| OpenRouter accessor grant + a live harness_hosted_probe_cli run | swift-ibex-601 |
| claude-code-oauth-harness: the secret, the grant, srv1 custody, a live turn | swift-ibex-601 |
| cursor-api-key-harness and codex-auth-harness: secrets, grants, srv2 custody, a live turn each | swift-ibex-601 |
| Consume or delete the unused secrets: openrouter -2/-3, groq, opencode | swift-ibex-601 |
| OCI srv1 accessor grant (phone approval), key custody, converge | nimble-heron-805 |
| The browser toolchain on the srv1/srv3/srv4 floor runners (unblocks #13382/#13383/#13411) | lively-ram-153 |
| Fabric storage measurement: transcript_rate_observe, source_pack, checkpoint_measure, restore | warm-badger-442 |
| srv1-09 floor runner: no cgroup memory limit (HostBudgetUnreadable); floor_class mislabels it 'structural' | stern-boar-596 |

### Open decisions

| Decision | Source |
| --- | --- |
| Remove Group B (srv9-12) from config/convergence | valiant-crab-775 |
| Control-plane platform (#13084/#13161) | warm-badger-442 |
| r2_bucket_ensure skips lifecycle convergence when a purpose's rules are [] | warm-badger-442 |
| external_model_scope 2 red on main; quarantine_probe_disposition 2/9 red | warm-badger-442 |
| ObservedReceipt next rungs: in-substrate sha256; an executing EEPROM read | warm-badger-442 |
| Close or keep #13382/#13383/#13411 | lively-ram-153 |
| #13544 Group A decommission is DIRTY: rebase it or fold it | valiant-crab-775 |

### Lanes not yet reported

eager-gull-22, gentle-dove-36, sharp-raven-357 and silent-lark-156 are finishing their integration branches. swift-bat-828 (state of CI) is in an error state and has 2 PRs. neat-boar-16's #12942 is in the mega branch, and its unowned follow-ups are in its memory note `pending/neat-boar-16-capacity-freeze.txt`, which still needs reading.

## Notes

- The plans bankruptcy (#13550) deleted `buganizer-visual-contract.md` and `printed-chassis-program.md`, and that deletion won over edits on our side. Some comments and `sites/README.md` still mention them.
- #13602's green checks are on its current head `f867e9d670`, which is later than the srv1 authorization push.

## Kept up by request

- [#13203](https://github.com/gunb-ai/gunbc/pull/13203) Complete Blackjack simulation milestone (branch `blackjack-milestone`) stays open and outside the closeout.
