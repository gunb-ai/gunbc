# Row-29 srv1 user-work slice admission — preservation archive

Dashboard node adhoc-8735da47-596. Repo gunb-ai/gunbc. Implementation PR #16013 -> main.

## Why this archive exists
The keen-otter program was wound down by the operator (#16088): its PR queue was closed
and the per-step approval / grant queues were RETIRED. The named push of the fix head,
the keeper runs, CI on a pushed head, exact-head re-reviews, and the operator merge
authorization that this lane was waiting on WILL NOT come. This branch preserves the
local-only work (code + merge candidate + evidence) so it is not lost when the lane is
closed. NO PR was created or merged from this push; this is preservation only.

## The defect and the fix (DESIGN 6b)
Defect (srv1 lab, #15822 comment 6663129463 row 29): validation + reviewers run in
gunbc-user-work.slice (MemoryMax 44 GiB) alongside six standing obligation services
(~41 GiB together at serve start). With standing services filling the slice, a per-attempt
review could not be admitted and the goal-audit reviewer was memcg-OOM-killed mid-run.

Fix (earliest unjustified boundary = admission): refuse, typed, BEFORE launch when the
slice cannot hold the work; never OOM it. The fix does NOT raise the cap.
- std.measure measure_fits<Q,S> is the single fit rule, now consumed by BOTH
  gunbc.compute.host_capacity compute_reserve_against AND a new pure slice fold.
- Pure fold user_work_slice_admit_against(ceiling, charged, requested) -> MeasureFit<Memory,One>
  at dag/gunbc/live_deploy/slice_bounds.dag:388. measure_sub(ceiling, charged): if charge
  exceeds the ceiling -> Below{available:0}; else measure_fits(requested, available).
  A zero ceiling is a FINITE bound nothing fits under (Below), NOT unboundedness; an
  unbounded/absent MemoryMax is handled by the CALLER (parse_systemd_memory_limit_show ->
  MemoryLimitUnbounded) BEFORE the fold, so the fold never sees a stand-in zero for "no limit."
- The roadmap belt gates the review spawn (roadmap_belt_actuate.dag:12595) and audit spawn
  (:12286) via belt_user_work_slice_fit BEFORE metering_roster_admit: Fits -> launch;
  Below -> BeltSliceFull -> the spawn defers the review (typed deferral, refused before
  launch, so it is never launched and never OOMed).
- Two integration_concern rows record the residual concerns:
  user_work_slice_admission_deferral_has_no_visible_count and
  user_work_slice_admission_check_races_within_a_tick.

## Commits preserved
A) Implementation fix head (branch archive/silent-bee-845-2026-11-08/eager-gull-535)
   commit 483fa8db3d5e4bc7d560072046dcfda53b469f89   tree 4cc7c265197285218c3ef61e7344e56fd11acb8d
   parent f5755e601582df0803ef39105ca8b8409a08e06e (the PR #16013 head, which IS on origin)
   Reproduce: `git apply fix-483fa8db.patch` on f5755e6 yields this commit's tree; the patch
   is 1590 B, sha256 f819afb6731c... (matches the reviewed patch comment 6702077436).
   `git hash-object -t commit raw-commit-483fa8db` (833 B) = 483fa8db3d5e4bc7d560072046dcfda53b469f89.

B) Local merge candidate (branch archive/silent-bee-845-2026-11-08/eager-gull-535-merge-candidate)
   commit f448d87eb060ce3ffbfc438c5eb07b5c61d148d7   tree 21711613fb67c42f9fefc089b852ee15ec363eb5
   parents [483fa8db3d5e4bc7d560072046dcfda53b469f89, ed007978dc2a67096df619b1b91566458dfb2ffd (main)]
   A LOCAL merge commit of the fix with main (merge-base 50652e0777329a585efa6919ef5ab9a0c81222ae),
   prepared under LOCAL-merge-candidate authorization only (never pushed). It resolves the single
   textual conflict dag/gunbc/integration_concern.dag as a UNION: keeps both authored blocks plus
   main's base_mirror_round concern + its sighting_revision helper; the registry has 22 entries,
   each exactly once. host_capacity.dag auto-merged disjoint (the fold's inputs are parameters and
   it calls an unchanged compute_backing_ceiling). Independent source review confirmed the rebuilt
   tree equals 21711613 and the commit identity via the published raw bytes.
   Reproduce scoped diff: `git -c core.abbrev=11 diff 483fa8db f448d87e` = 871319 B
   (sha256 40bc325d960075513f3e3f5667b0c7d31d32a0d8a9cd13968ecff578237c2fc1).

## Review / verification status (no overclaim)
- Independent byte review (sleek-owl-397): READY on the v14 packet and on the merge candidate
  (source content + commit identity, independently re-derived).
- Keeper review (calm-crab-645): CLEAN on the v14 packet.
- CI at PR head f5755e6: heal check 42328355922 FAILURE (the measure_fits callee defect that the
  local 483fa8db head resolves); witnesses 42328355710 outcome UNKNOWN. No rerun was performed.
- NOT compiled and NOT executed as a witness: BUILD POLICY forbids local compiler/test/witness/
  build/regen work, and the keeper-packet run was never granted. So compilation of the merged tree
  and the witness are UNVERIFIED. Reviewer approvals are NOT a merge authority.

## Reviewer flag resolved (not a defect)
belt_user_work_slice_fit reads instance.dispatch_worker_memory_max. It has exactly two callers
(roadmap_belt_actuate.dag:12286 audit, :12595 review); both size the MemoryMax their own transient
unit enforces via dispatch_review_unit_properties (roadmap_dispatch_actuator.dag:1427; reached by
dispatch_review_unit_command:1497 and dispatch_audit_unit_command:1545). A third consumer of that
properties fn is dispatch_supervisor_unit_command:1604 (NOT a slice-fit caller). compute_build_memory_ceiling
is read only in host_capacity.dag (build-pool appropriation). So neither slice-fit check has
build-sizing intent; changing them to compute_build_memory_ceiling would be wrong. No code change.

## Coordination with proud-tern-546 (#16068)
#16068 (attempt-worker byte admission + user-work 44 GiB) overlaps #16013 only in slice_bounds.dag.
Agreed order: #16013 lands first, then #16068 rebases attempt_byte_admission_decision onto the shared
fold (charged = declared in-flight sum). The fold is generic in LOGIC (pure over three ByteSize
readings, no slice internals) but user-work-NAMED; for one shape AND one name, the generic rename
(e.g. slice_admit_against) belongs in the change that ADDS the second consumer, repointing all call
sites. Waits-vs-Refused (esp. unbounded) must live in each caller, never as a stand-in into the fold.

## Keeper witness packet (retained, never run)
keeper-v14-packet.sh — sha256 05c04682105a08fefb25e1617da2df2d03050987dd919d0937137beb8c459c6e, 61183 B.
Retained unchanged (v14; no v15 by admin disposition). Its launch (calm-crab-645 keeper packet) was
never granted and remains suspended. Do NOT claim it was executed. Do NOT claim the launcher runs
under env -i (inherited build/claim env UNKNOWN). It must be run only as --run inside the booked
scope, never --self-test.

## Gates that were NEVER granted (and now will not come)
named push of 483fa8db [WITHHELD]; contingent publisher-write [WITHHELD]; CI on a pushed head;
exact-head re-reviews; operator merge authorization. Program wind-down (#16088) closed these queues.
PR #16013 is left open and unmerged; the fix head 483fa8db is preserved here and (as of this push)
is on origin only as an archive branch, never on main.

## File inventory (this archive/)
MANIFEST.md                         this file
fix-483fa8db.patch                  1590 B, sha256 f819afb6731c... (the fix diff vs f5755e6)
merge-candidate-vs-p1.patch         871319 B, sha256 40bc325d... (483fa8db..f448d87e, core.abbrev=11)
raw-commit-483fa8db                 833 B raw commit object (hash-object -t commit = 483fa8db...)
raw-commit-f448d87e                 1088 B raw commit object (hash-object -t commit = f448d87e...)
keeper-v14-packet.sh                61183 B, sha256 05c04682... (calm-crab v14 witness driver)
comments/comment-<id>.md            the 9 durable PR comment bodies:
  6701704265 heal analysis; 6702029102 import note; 6702077436 the fix patch payload;
  6702240796 correction; 6702291816 commit-identity (raw 483fa8db); 6704379214 keeper v14 packet;
  6706244976 merge-candidate record; 6708376429 raw merge-commit + reproduce note + flag verdict;
  6708405926 append-only correction (three consumers of dispatch_review_unit_properties).
