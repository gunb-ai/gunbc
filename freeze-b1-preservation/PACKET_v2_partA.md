# FREEZE-2 evidence packet v2 — the 34-row claims run at the frozen seed
**Author:** sleek-seal-226 (owner of #15695 since its previous owner ceased to exist).
**Supersedes:** packet v1 (`829ca20e…`, comment 6705371718) and the earlier draft (`a36241a7…`). v1's seed is
superseded by the B2 ruling (see §3); its run side was incomplete (no launch contract), which this revision
supplies. Nothing in v1 that was verified is contradicted here except where §3 and §6 say so explicitly.

## 1. What this run is for, and what it is not

v17 (`cc1-15695-495d-v17`) ran once and returned a **REAL verdict**, not a setup refusal: EXIT 1, with
**rtp 6 PASS / 11 FAIL** and **ce 11 PASS / 2 FAIL**, every FAIL bare (`Bool false`, no error line). That
verdict stands as recorded and is not re-opened here.

This packet asks for **one** attempt at a different question: at a **clean base** (the landed #15999 repair
merged with green main) plus a **frozen probe block**, do the 34 pre-registered rows observe what the
pre-registered ledger says — and specifically, does the tail-return/guard acceptance path accept **anything**
at all. The positive control is `a_tail_return_of_a_fn_body_is_accepted`, a CONTROL row that is expected to
PASS on both live hypotheses: if it comes back FAIL, the six refusal rows that passed at v17 passed for the
wrong reason, and that is the finding this run exists to settle.

**It is not a re-litigation of v17, and it is not a land path.** #15695 stays parked. The carried obligation
(§7) binds: expectations stay as frozen, actual verdicts are reported UNCHANGED, and FAIL→PASS remains a
hypothesis until observed.

## 2. The base — named as shas, built by an explicit two-step

Base commit **`57ce2e787aa5227fc1840961c607981f4abdde86`**, tree **`5a14ac7ca50f43312760f092c86ebee7c679ce37`**.
It is the merge of exactly two things, both named by sha:

| component | sha | provenance |
|---|---|---|
| the landed #15999 repair head | `a4a21da46284e64a31efff08585c42aaf589089a` | the reviewed repair, parent of the base |
| green main | `7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9` | the head sha of witnesses **run 42342945138**, whose `v2-bootstrap` job succeeded (completed 07:07Z) |

The base is never named as a branch, and it was never pushed. It exists only in the private cut repository
`/var/tmp/freeze2-cut-15695/repo` (mode 0700). **Reproduce it** (this is the check a reviewer runs; sleek-owl-397
already did, and got the tree):

```
git checkout a4a21da46284e64a31efff08585c42aaf589089a
git merge --no-ff --no-edit 7d062bda1b6dab67bc72e0c49d0eb635c45ca7f9
git rev-parse HEAD^{tree}      # must equal 5a14ac7ca50f43312760f092c86ebee7c679ce37
```

The checkout comes FIRST and then the merge. The one-line form `git merge --no-ff A B` that an earlier revision
of this packet showed is an **octopus merge** from any other HEAD and produces a different commit; writing it
as two steps removes the ambiguity (reviewer point M3).

## 3. The seed — re-frozen under the B2 ruling

Parent ruling msg_8dd63ef4 item (2): the newer instruction wins, so this seed carries an accurate, non-DRAFT
commit message; the DRAFT message of the superseded seed stays where it is, unpushed, and is not re-posted.

| field | value |
|---|---|
| seed commit | **`4a9efe081d8046969eb4e9ef06d219e27ce31113`** |
| seed tree | **`a0de248f1918aff315145392e2057db6ecc66c1a`** |
| parent | exactly one: `57ce2e78…` (the base above) |
| seed patch against the base | `SEED_FREEZE2_4a9efe08.patch`, 4,025 B, sha256 `5ad5a0ee2c404fc23bd6e258ee5922891d1eeed2792a2736ba85e20f02a623cc` |
| probe diff (its source artifact) | `SEED_PROBE_SET_rebase_candidate_v2.diff`, 4,025 B, same sha256 as above |
| probe block | 45 lines, sha256 **`e07e18f2c6f6feec061c3b2d55255d05832c070801e414a425734dedeb0aebd0`** |

**Why the seed changed (B2).** v17's packet's rtp lexeme witnesses matched on
`optional_present(value: s) =>` / `optional_absent() =>`. That arm shape does not exist anywhere else in the
corpus — every other match arm at this base is `Present { value: x }` / `Absent`, and a grep of the corpus found
no other arm in the lexeme shape. All 21 rtp names share **one** resolve, so a refusal at that one arm would
have spent the single attempt with no rtp verdicts at all. The canonical form is now used. A grep is evidence of
risk, not proof of refusal — which is exactly why the form was changed rather than argued about.

**The probe-block digest is a superseding re-anchor, not a silent edit.** `e07e18f2…` replaces
`af6f17477ac0f39591abb5345840e0eac827e388072afddb39f002b7eb17bd22`. The reason is the B2 canonical-arm repair:
the block's *content* changed, so its digest had to change. The 34 ledger rows are unaffected (§5); the ledger
registers the digest as a citation and rev5 updates exactly those citations.

The four probes are `rtp_ctl_spelling_is_resolved`, `rtp_ctl_spelling_is_absent`,
`rtp_ctl_spelling_is_file_refused`, `rtp_ctl_spelling_is_resolve_refused` — single-line witnesses at seed file
lines 362, 364, 366, 368, over a `fn rtp_ctl_spelling() -> String` helper. Their literals were frozen **before**
any run; at most one can pass (the spellings are pairwise distinct); all four FAIL means NO-OBSERVATION, and
more than one PASS voids the row set.

Round-trip proof of the seed (re-run by the parent, then by sleek-owl-397, both matching): apply the patch to
the base with `git apply --check` → **NO fuzz**, `git apply` → rc 0, `git add -A && git write-tree` →
`a0de248f1918aff315145392e2057db6ecc66c1a`. The minimality check requires exactly 2 lines removed and 2 added
in the constructor expressions, with the canonical arms present once each and the lexeme patterns absent.

## 4. Pre-run checks — the checker, and what it had to be fixed to notice

`prerun_checks_freeze2_seed_v2.sh`, sha256 **`CHECKER_SHA`**, with its registered expectations artifact
`prerun_expectations_freeze2_v2.env`, sha256 `0d491688082c3232ad3f3b574a669142190f93e6a55778a83454a044434d6bdc`
(§8 attaches both), and the digester `probe_block_digest.py` `c0975d0d…`.

M2 from the review is applied: a `trap` covers error/exit so an unexpected death still prints a receipt; the
last line is always `OUTCOME: PASS|FAIL|ABORT rc=<n>`; **missing or unreadable inputs exit 2** (an ABORT — an
input failure is not a verdict about the seed); there is **no `exec > >(tee …)`** anywhere, because it can
truncate the receipt before the last line; the script is **hermetic** (every input arrives through
`EXPECT_FILE`/`DIGEST_TOOL`/`REPO_DIR`, with no `/tmp/packet` dependency and nothing attached implicitly); and
it writes only inside a private 0700 work root it creates.

**The defect this revision caught in itself, and its fix.** The first version had **no candidate channel**:
`<WORK_ROOT>` was its only argument and every identity value came from the expectations file. It therefore
proved *"the expectations file still describes a commit that exists"* — not *"the artifact about to be run is
that commit"*. Handing it the **base instead of the seed passed every check**, which is how the negative
controls found it (control M1, `NEGCTL … BAD`). It is now
`prerun_checks_freeze2_seed_v2.sh <WORK_ROOT> <CANDIDATE_COMMIT>`: the candidate is a **required argument with
no default**, because defaulting it to the frozen seed would rebuild the very circularity the controls exposed.
An empty artifact likewise became exit 2 rather than a content FAIL: a zero-byte registered artifact cannot
express the 45-line block, so calling that a mismatch would let a truncated write read as a verdict.

Controls: `negctl_seed_checker_v2.sh` `NEGCTL_SHA`, receipt `NEGCTL_RECEIPT` —
**NEGCTL RESULT: OK=`NEGCTL_OK` BAD=`NEGCTL_BAD`**, 14 cases: the genuine seed passes (rc 0, `OUTCOME: PASS`,
and no FAIL line printed at all), ten refusals each naming the broken identity, three input-class aborts. Every
mutant is built by `commit-tree` in the cut repository **and kept reachable**, and after each mutation the
harness **requires the file to actually differ** from its parent: a mutation that changes nothing is reported
`SETUP` and counted BAD, never a pass. That guard earned its keep twice — it caught a mutant built from a base
that never contained the probe block, and a mutant whose target literal does not exist at the main component.

Receipts at the exact commits (both by `CHECKER_SHA`, `STATUS: PASS`):

| where | candidate | outcome |
|---|---|---|
| at the **base** `57ce2e78` | the base | `RECEIPT_prerun_57ce2e78_PASS.txt` sha256 `bd31f058…` (comment 6703855069) — 10 OK / 0 FAIL |
| at the **SEED** `4a9efe08` | the seed | `seed-prerun-receipt.txt` sha256 **`SEED_RECEIPT_SHA`** — `SEED_RECEIPT_SUMMARY` |

The seed receipt records the 21 `test fn`, the probe block digesting to `e07e18f2…`, and
`APPLIED TREE == CANDIDATE TREE == a0de248f…`.

## 5. Expectations — frozen, and read back before any run

`PRE_REG_ROW_15695_freeze2_rev5.md`, sha256
**`6830d48781d65e7f6bb4218e2ff3a5221c9aa8203ac59c82fb0adab519bb8304`**, **34 unique rows** = 21 rtp + 13 ce.
Rev5 supersedes rev4 `b77f206d2bc05cec2ae04b962d0ce4ef176ddb7f697361e80ddb363c9a8e06e2` (comment 6703139348,
left untouched). The chain is `a31b8bb6 → 3a5974e2 → c89ad90e → b77f206d → 6830d487`.

Rev5 differs from rev4 in **nothing but the three probe-digest citations** (to `e07e18f2…`), the probe-diff
file name and sha, and one word on line 104 ("without content change" → "canonical re-form"), which the digest
change made untrue. The build script asserts that all 38 pipe-rows are byte-identical and aborts if a stale
digest survives anywhere. Expectations were written from **intended semantics** before any run; no row has been
moved after seeing output, and none will be.

The auditor and its output, both generations, with which one governs:

| generation | auditor | its output | status |
|---|---|---|---|
| 1 | `96605ab8…` | `858aa419…` | superseded by generation 2 (it predates the row-set correction); posted at comment 6703139348 |
| 2 | **`99b0056a…`** | **`a791bd25…`** | **governing**; posted at comment 6703191405 — `AUDIT CLEAN` |

For rev5 the auditor is the same program with its one hard-coded constant removed: it took the probe-block
digest as a literal, which is the identical circularity it was written to remove for names and literals (an
auditor that carries the value it is auditing cannot notice it drifting). It now **derives** the digest from the
attached artifact. Its controls: `negctl_auditor_rev5.py` `850e47e7…`-lineage, **15/15 OK**, adding
block-by-stale-citation RED, block-by-updated-citation GREEN, and probe-artifact missing/emptied → rc 2.

## 6. The frame, its pins, entries and rosters

Frame: `packet-evidence-freeze2.sh`, sha256 **`FRAME_SHA`** — derivation, change list and the launch contract
are in §6c. Pin `4a9efe08…`, tree `a0de248f…`. Two entries, each with ONE resolve:

| entry | path | git blob | roster |
|---|---|---|---|
| rtp | `src/v2/test/claim/namespace_xl0/return_tail_position.dag` | `f200902855004b7385897cae205f046636c56d25` | **21** names = v17's 17 **in the same declared order** + the 4 probes at their declared positions 8–11 |
| ce | `src/v2/test/claim/self_host/closure_emission.dag` | `ee1c679e7ef1e2923edadfe0a6f0c63cf881b019` | **13** names, unchanged from v17 |

The blob pins were **measured at the seed** (`git rev-parse <seed>:<path>`), not copied. The ce blob is
byte-identical to v17's pin, which is the check that the 1,112-file merge did not touch that lane.

**Why there is no cpr entry.** v17 ran three entries; its cpr lane is
`src/v2/test/claim/parse/condition_position_record_parse.dag` with 15 witnesses, and it was the one lane at
15/15 PASS. That file **does not exist at this base**: it is #15695's *own branch content* (it exists at v17's
pin `495da688…`, which was that PR's head), and none of its 15 witness names appears anywhere in the seed tree
— verified by `git grep` for several of them and by a tree listing. Running it here would spend the single
attempt on a `REFUSED_SETUP`. What remains is exactly the 34 rows of the frozen ledger, which is a useful
consistency signal: the run shape and the ledger shape now coincide. The rtp roster relationship is proved, not
asserted: removing the 4 probe names from the seed's declared order reproduces v17's 17-name roster exactly, in
the same order (`PROVED` in the frame derivation record).

The build target was verified at the seed rather than assumed: `src/v1/stage0/Cargo.toml` is still
`name = "v1-compiler"` with `[[bin]] name = "claim_batch"`, and `src/v1/stage0` is still a workspace member. If
the merge had renamed that package, the one attempt would have died in the build.
