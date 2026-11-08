# Keen-otter program handoff — 2026-11-08

The user stopped this program because its coordination and approval process was obstructing delivery. This handoff replaces the old queue. No further experiment, review cycle or implementation is assigned to the retiring tree. Old per-step holds are not an approval queue that a successor must continue servicing.

## Recovery and scope

The integration branch is `integration/keen-otter-handoff-2026-11-08`, based on main at the start of wind-down. It contains the catalog and preserved uncommitted work, not a claim that all unfinished compiler changes work together. Each of the 73 public PR heads is preserved remotely as `archive/keen-otter-2026-11-08/pr-N`. Original branches and PR discussion/evidence are retained. Closing means stopped, not merged or accepted.

See [PRS.md](PRS.md) for every PR, owner, full commit and recovery ref. [public-pr-inventory.json](public-pr-inventory.json) preserves the pre-closure PR descriptions. Private administration work is catalogued only in the private repository's identically named integration branch, at `handoff/2026-11-08/private-pr-inventory.json`.

Fetch the integration branch and the desired archive branch, then merge or cherry-pick only the work you intend to resume. A successor should use an ordinary development loop and assess actual behavior, rather than resume the retired packet bureaucracy.

## Equality and broker/printer: #15992

The public PR head is 5a56f80166d92326509921eb3033c0343f7f4f4e. Its generated Rust build remains broken by E0369 on equality involving DemandEngine/NativeDemandValue; earlier revisions exchanged this for a missing equality helper. It also conflicts with main. No root fix has been established.

A separate three-site workaround exists at be3a9b8612162cea0a822df64720a7efd19986bf (tree d40daa3a67518e0e43c66e34d93e93a9f68d50ac). Broker install and private publish were reported to get past the three resolve refusals. **The printer command was not tested.** This is neither deployment evidence nor a completed acceptance witness.

The witness import repair is a separate chain: be3a9b86 -> 669a4e51841774aaf59a6323b3561419af53727b -> 1d67569427c50b7937c27735f86cf0a0e9493f6a (tree bb893749f78cf37afb47c5236a2192fbec78b38c). The imports were source-reviewed; the repaired witness was not run. Recoverable bundle and raw commit carriers are in #15992 comments 6723347505, 6723357392, 6722893253 and 6722936989. The bundle contains stopgap-3site and witness-repair-v2; its decoded length is 14679 bytes and hash prefix 082e4e92. Owner quiet-pike-120 is preserving fetchable refs as part of shutdown.

The retained-seed equality experiment admitted and evaluated equal holders through a composite generic/function-member path (T2/T0/T1). It did not identify the admitting arm, prove generated-Rust behavior or reproduce CI identity. The T5/T3 discriminator remains unrun: eqr4 carriers 6724000239, 6724000530, 6724000820, 6724001071. It is unfinished research, not a prerequisite imposed here on taking over the broker/printer workaround.

## Controller and memory: #16038

Head bf5bb5632fea0d8769200ba797053e88b7df9c7c has green CI but its 18 wet claims remain unobserved. The wet attempt was OOM-killed at 8 GiB before entry resolve, with zero answers. Historical 17-function summaries at 670264ab are not validation of this head; only the last full output survives and the historical claim_batch binary hash was not recorded.

The diagnostic patch in comment 6721933706 was built separately on instrumented tree 96b0642950fd3e43e13602f83d46eb2586e49502. Its three renderer/helper tests passed. The shared index completed over 10,149 files in 135.5 s at point VmRSS 7,112,978,432 bytes (6.62 GiB). The process then hit the 8 GiB memcg limit at 149 s, between the boundary line and warm completion line. This localizes an interval, not a particular warm substep, and does not prove a larger bound sufficient. No claim answered; the diagnostic route was Hermetic.

END: https://github.com/gunb-ai/gunbc/pull/16038#issuecomment-6723422964

Unfinished choices include diagnosing closure-load versus whole-root census and changing whole-root indexing. Closure scoping is a semantic change: lookup authority, duplicate refusal and bare-name uniqueness differ between routes. It must not be presented as a proven safe memory optimization. No further run is in progress.

## Other principal unfinished lanes

| Lane | Remaining work and limits |
| --- | --- |
| #16053 | Real attempt produced no verdict: typed-lambda parse error plus driver ERR-trap capture defect. Source fix e15ae6eaa1d5968b9606cba65e75950561c6b3d5 (tree ad47162c13e234d7f83ec7aab867e9f94c1b9f08, parent 7277bc328ab816aa1380680297811b835c439054) and driver v7 were reviewed; 17 stub fixtures passed. No real rerun. Carriers 6722337417, 6723218654, 6722975610, 6723000709, 6723254456 preserve source, stubs and raw commit. |
| #15868 | Head ffd4e4ecdf3054d5423bd836c858a46318938fab has green CI but review 92072 requested changes. Malformed optional-layer wrapper gets the wrong refusal class. Reason/class hand-pairing and silently absorbed position fields remain separate design debts. Local follow-up work needs the owner inventory; six claim cases were not executed. |
| #15695 | Native-test verdict work remains incomplete. A proposed frozen subject omitted the PR itself, so cannot validate it. Re-freezing on main plus PR was being prepared when stopped. Do not treat fixture results as subject results. |
| #15965 / native-seven | Frozen R4/R5 packages were not run. Native-seven acceptance remains incomplete. |
| declaration-provider-outcomes | Historical controls were observed, but acceptance record and landing-head native-seven handback are absent. The N7 packet has no executed new result. |
| #15931 | Shared-check lets/matches still needs behavioral observation. |
| #15917 -> #15560 | Generated-host roster repair blocks census retirement: heal reports behavioral_receipt_host.rs absent. #15560's bootstrap pass is not a clean heal. |
| #16041 / #16046 / #16056 | Main-rs emission, direct Rust door and admission-roster work remain incomplete. Native crate partition is not delivered. |
| #15710 | REST redaction source is preserved; host-effect witnesses remain blocked by equality work. |
| #15626 | Typed document citation source consolidation was rebased and approved (92077); preserved with the other stopped PRs. No reason to restart its old review loop. |
| Other PRs | Full titles, owners, descriptions and exact heads are in PRS.md and the JSON catalog. Their closure is not a claim of correctness or obsolescence. |

## Evidence and local work

Draft release assets remain the durable evidence source. Do not rely exclusively on srv1 paths: older keeper receipt directories disappeared on the same machine, cause unknown. Historical srv2 N7 timing/memory figures lack their expired source log and must not be treated as sizing measurements.

| Release tag | Archive SHA256 |
| --- | --- |
| evidence-16038-c16038d-6c16c24a | 566d7d8dd02f9fc818b4b754c9705b54195fad2891e36892f67a3d91dbe28829 |
| evidence-16038-prior-receipts-20261107 | 2f0f1c44871fa789798418550b340b4a0512d253cb8d9723c94e96f5495da185 |
| evidence-16038-3f909f13 | 3d429d1b13d301efcbba52aa7526a237bb3eb5702ffd492410e2b8ffaa43d258 |
| evidence-eqv4-r2-974db36a | 573a542f08b67fb4ba3bbe255bb1ed54f7c29db2796c4ded3d21283fe7ad5ac8 |

Additional releases: evidence-16053-claimrun-2fdd35ad, evidence-eqv4-v5-fdc5a6b1, evidence-e0369-v8-5bd47bd5. Source comment packets remain attached to their closed PRs.

`keen-koi-376-tracked-wip.patch` preserves seven tracked dirty files against the head in `local-work.json`; it is unvalidated unfinished work. Accessible clean session heads also have remote `archive/keen-otter-2026-11-08/session-ID` refs. Owner handoffs supplement this catalog as received. Clever-ibex's local doctrine commit 7e32d39a383f0221117044934710c2d0791b2418 is superseded by main, not a suggested integration.

Keeper calm-crab retains a 66 MB scratch evidence archive locally (`memory/pending/calm-crab-645-handoff/keeper-scratch-evidence.tgz`, hash prefix f4b75278); it is not yet an off-host public asset. Its unfinished queue includes restoration r10, #16013 v14, hashread v6, key-kind v19, #15917 REV-3l and #16021 review. These are recorded dependencies, not additional ownership claims over other trees' PRs.

Quick-newt reports no active units, bookings or background jobs. Failed units and consumed-attempt markers are inert and retained. Calm-crab and clever-ibex report no schedules or background tasks. The tree is being archived after preservation; closure results are recorded separately.

## Shutdown updates

All 73 public and eight private catalogued PRs are closed. Handoff PR #16088 remains open. The daily audit schedule fba2cf87 is cancelled. No active remote executor job or booking remains.

The stopgap and repaired witness are now published as normal recovery branches `archive/keen-otter-2026-11-08/stopgap-3site` and `archive/keen-otter-2026-11-08/witness-repair-v2`, reconstructed from the hash-verified bundle. This removes the need for a successor to request a local checkout or decode comment packets.

Additional owner disclosures: #15551 has 35/36 historical population claims passing; the one failure was called pre-existing but that was not proven. Its parent control was not run. #15932's final head is 33700c4b88348116736d0d821003057c24753c36, captured after the first snapshot. Vivid-crane's uncompiled event route is published as `session/vivid-crane-742-event-route` at a73ac24fb0779b185472561326ed7752af2a317f; PR3 scratch designs are in #15932 comment 6724242844.

The keeper archive is now durably retained in a private draft release, rather than only a session path. It is 66,316,178 bytes, SHA256 f4b75278cbfa289d51b5562f3c517505b953800951d7660b3ccdbaef8582299f. Access remains restricted because it includes infrastructure details.

Owner messages in owner-handoffs.json are historical reports, not current instructions. In particular, references to keen-otter-only grants, old queue ownership, and already-closed PRs do not reinstate the retired process. Later corrections supersede earlier messages.

Keen-heron's untracked files and packet archive, plus keen-moth's unrun 9,015-byte carrier_closure_probe.sh (SHA256 9c9c624d93bea3b7a1176a737626018287b16e84134a3ef9235e3e1a4e293b72), are preserved in the private draft release `handoff-keen-heron-809-2026-11-08`. Proud-badger's scratch artifact branch final head is 0a18dbc60154914cda45a9b5a938566376f0a465.
