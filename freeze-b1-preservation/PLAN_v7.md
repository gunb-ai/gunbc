# v7 working plan (sleek-seal-226, #15999 freeze-2) — POSTED 2026-11-08T13:50Z

STATUS: **v7 posted and live-verified; report sent to tidy-ant-251 (msg_c99814dc).** Waiting on his byte
check + his own hermetic re-run, then sleek-owl's focused review (grace/signal/decoder paths), then the
keeper. No grant, no host contact, no attempt spent. FLEET STOP still binds.

Posted comments (payloads gzip+base64, 76-col wrapped): B 6722265512 (58,336 B) C 6722269400 (61,773)
D 6722269634 (61,449) E 6722269910 (62,669) F 6722270193 (47,764) G 6722270398 (58,134) A 6722277395
(41,977, the packet text, posted last). Packet text 94,455 B sha256
`4814584f6c19a6928f0197e173daebc18d21c7fbc6879f4810255b956fea224c` (quoted in A's header; not a payload).
Frame: /tmp/packet/packet-evidence-freeze2.sh sha256 `4321316ccc8cf08148cf4d97a8161449512c9e8789fae25e8e97b9c9fecc21c6`,
92,520 B, mtime 11:31:17, 946 lines / 44 functions / 501 top-level statements.

Live verification: `build/verify_comments.py <7 ids>` = 59 payloads 0 bad; a separate cross-check proved
live == manifest (59, no missing/extra/group-mismatch, no duplicate basenames); `--corrupt` on the frame in
B returns exactly one BAD. The attached `build/check_accounting.py` reproduces the published
`receipts/ACCOUNTING_v7.txt` byte for byte (both `de4fe67faacc0311ed5e144da0c25586aa0750cb45de0fdc3e37bdcd58b66056`),
which closes the fact that the receipt (12:15:22) predates the checker's last edit (12:40:35).

Controls of record (one fixed-order round, `receipts/FULLSWEEP_v7.log` 12:15:22 -> 13:10:48, log names the
frame sha): hermetic 15/15 cases OK=103 BAD=0; cap-kill 32/0; gates 35/0; argv 36/0; outcome 8/0;
recon 10/0; audit rows=34 AUDIT CLEAN + auditor controls 15/15; seed negctl 20/0; seed pre-run PASS;
chain 12/0; static 121/0 + 6 static inversions refused; accounting inversions 7 refused + 1 stated GAP.

Accounting (UNCHANGED bounds): caps 3763 + ladder/slack 253 + allowance 120 + checkpoints 20 + reap 5 =
**4161 of RT 4200, remainder 39**; 11 bounded sites; GRACE_LOCAL=3 at exactly 7 named local git sites.

## What was fixed while producing v7 (defects 5, 6, 10 in §0; plus tooling)
* Stage 7 (`build/patch_v7_stage7.py`): `PHASE_RC_ORIGIN` cleared by the kill ladders before the caller read
  it -> every real cap expiry blamed the command. Now reset only at each launch site.
* Stage 8 (`build/patch_v7_stage8.py`): a cap kill at a reconstruction site refused with a CONTENT-shaped
  reason; added `rc_cause()` (phase + rc + producer) at the five `caprun` recon sites, `PHASE_RC` plumbing,
  and removed my patch's doubled `RECON_REFUSE:` prefix (stutter rule). Mutants in `build/mk_static_mutants.py`.
* Fixtures made content-addressed (`gates_fixture_v2.sh`, `outcome_fixture_v4.sh`, `recon_fixture_v4.sh`).
* Poster (`/tmp/fix/poster_packet_freeze2_v7.py`): manifest-driven, refuses over-limit groups (payload bodies
  are gzipped; largest posted body 62,669 B), unique paths AND basenames, no unrendered markers, `--only=`
  for staged posting, and comment A's header carries the measured packet-text sha. Bug found in the field: it
  could not parse `#issuecomment-<id>` from gh's URL, so B was posted before the run refused; I seeded
  `/tmp/packet/posted_ids_v7.json` from the API and continued -- nothing was posted twice.
* Generator (`build/make_packet_v7.py`): fact wiring repaired (receipt wordings, `in_receipt` value/match
  semantics, `PLANNED_TOTAL\s+`, `CASES: suite of N, dispatched N`, ledger end marker `^# -{10,}$`), v7
  preamble span added, stale v6 launcher argv / SH row / staged-name rule replaced with the frame's sha,
  self-reference removed (the packet text is not a payload), §8 lead explains the size-vs-theme grouping and
  that v5's bytes are not re-attached. It refuses on stale/missing/BAD receipts and on any superseded value.
* Verifier (`build/verify_comments.py`): accepts both payload-header shapes, flags a name used twice.

## Queued, not in v7
The i-gate `OUTCOME` naming the refused field instead of a kill status (rc=137) -- tidy-ant-251 said queue it.

## Still owed / the §6b diagnosis RECORD (corrected 14:03Z after tidy-ant-251's check)
Evidence: `/tmp/v17/claims-rtp.log` (17 witnesses: 6 PASS all refusal-shaped; 11 FAIL = 8 `…is_accepted` +
3 `…is_read_and_refuses_unbound_at_its_atom`; all FAIL bare), one `[floor-reach-probe]` line per entry with
`reach=surface_unarmed dispatches=2548897 last_builtin=rtp_refuses_as`, `eval_self=0.000ms` on every witness.
`rtp_refuses_as` exists only in `src/v2/test/claim/namespace_xl0/return_tail_position.dag`.
**Positive control: absent.** The simplest acceptance claim `a_tail_return_of_a_fn_body_is_accepted` FAILED, so
v17 never shows the acceptance path accepting anything; the 15/15 cpr suite is the G1 completion-partition
log-grammar claims and does not touch this route. v18's probes are the built control
(`rtp_ctl_spelling_is_resolved` passes iff the acceptance claim passes).
**Standing hypotheses (instrument-side):** the surface was unarmed on the only probed fill; 16 of 17 witnesses
reuse the warm artifact and print NO probe, so they carry no reachability evidence; hence a merit red at the
intended boundary and one at an earlier boundary are indistinguishable in that log.
**Withdrawn (tidy-ant caught it, I verified at `claim_execution.rs` @a4a21da4 L241-250):** the claim that a
bare `Bool(false)` witness is an observation failure by corpus law. That arm is the optional `_failure_receipt`
companion runner and says a missing companion is "not declared" and that the auto-derived name **"must not
invent a required loudness hook for every Bool(false) witness"** — the corpus declines to require the hook.
So the 11 rtp + 2 ce FAILs MAY be merit reds; the attribution must not rest on any loudness law. The packet's
§5 "all four probes FAIL ⇒ NO-OBSERVATION" is my declared contract, set before any run, not a corpus rule.
**Corrected count:** the merge `9c15c98c -> 7d062bda` dag hunk is 1 insertion / 1 deletion (`git diff --numstat`
prints `1  1`): `node_atom_identity_optional(node: a.at)` -> `a.at.node`, the refusal-atom plumbing the three
unbound-read claims assert. The merge touches neither `claim_execution.rs` nor `return_tail_position.rs`
(#15695's own file), so the parse fix is unchanged across the comparison.
Nothing published carries the wrong premise (checked the posted packet text and the attached
`FREEZE2_REASON_WITNESS_SPEC.md`), so no posted comment needs an erratum. Correction messages:
msg_21744e60 (original), tidy-ant's objection, msg_6c436ef1 (correction).
**Next step:** v18's four reason-witness probes + the loudness-free expectations decide it in one observation.
No rerun under the fleet stop.

## Rules I follow
Shas only from measured output; never publish a receipt older than what it covers (or prove reproducibility,
as above); never edit or delete a posted comment; ONE live verifier; count-asserted anchors and read back;
one cut-repo user at a time; never touch `PRE_REG_ROW_15695_freeze2_rev5.md` or the seed/base/patch pins;
never edit a running script; fixtures locate frame definitions by content with a unique-count assertion.

## BLOCKING SUBJECT-MISMATCH (raised 14:1xZ, msg_7c849519) — awaiting an A/B ruling
PR #15695 is OPEN: branch `session/jolly-carp-900-parser`, head **495da688** (merge of origin/main into
zb/p15695), base f08a6ae4, 4 files +592/-58: `src/v2/extdeps/languages/dag.dag` (335 rewritten),
`src/v2/std/grammar.dag` (-17), `src/v2/test/claim/parse/condition_position_record_parse.dag` (NEW 17,151 B),
`src/v2/workflow/floor_pure_producer_share.dag` (-22).
The frozen freeze-2 base **57ce2e78 = merge(7d062bda main, a4a21da4)** where a4a21da4 is the head of
**branch `repair/fatal-cause-authority`** (my worktree's branch), not the PR. Verified: 495da688 is not an
ancestor of 7d062bda nor of a4a21da4; `git diff 495da688 7d062bda` on those 4 files = 58 ins / 592 del; the new
claim file does not exist in main; `BASE_PATCH_7d062bda_to_57ce2e78.diff` touches 18 `src/v2/test/claim/**`
dag files and none of the PR's 4; `SEED_FREEZE2_4a9efe08.patch` touches exactly one file
(`namespace_xl0/return_tail_position.dag`).
=> **v18 as frozen observes a subject that does NOT contain the PR's parse fix.** v17 DID run at the PR head
(495da688, tree 29ca30b0). Third hypothesis for the 11 acceptance FAILs: the PR's own grammar rewrite is the
only claim-relevant difference between v17's subject and v18's, so they may be a PR-side regression at the
parse/atom boundary; v18 as frozen cannot distinguish that.
Candidate corrected subject, measured: `git merge-tree --write-tree 7d062bda 495da688` -> rc 0, no conflicts,
tree **6f1a51ac24f948a16a7f775670a50037c9c586cc** (all four PR files present).
**Ask (A/B):** (A) keep the seed and add a plainly-worded limitation to the packet (this run does not test the
PR's 4-file change), or (B) re-freeze on main+PR with the claim edits + probes re-based = new seed, new packet,
new review chain. I change no pin, expectation or packet byte until the ruling; v7 posted stays as is.
