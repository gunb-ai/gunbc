# proud-badger-77 session artifacts — dashboard node adhoc-afb0f800-78c

**Snapshot branch. NOT a PR candidate, do not merge, do not open a PR from it.**
It exists because the working drafts lived in `/tmp`, which is reclaimed on archive
TTL purge (several earlier probe scripts are already gone). Everything a successor
needs to run, verify and interpret the measurement is here.

Repository: `gunb-ai/gunbc`. Node: `node://adhoc-afb0f800-78c`.
Task: one lowering of nominal type-application child edges, so a declared type node
and a produced type node for the same application carry the SAME child-edge labels.

## The two PRs this work touches

- **#16061** — the measurement instrument, branch `nominal-child-edge-labels`,
  head `5b02c6dd3d963cb9863b163a69cba56e900f8698`, tree
  `a5d5e58f257d507445127048aa855a8062257ab4`. **FROZEN and DO-NOT-MERGE**: every
  packet in it is pinned to its head hash, so any push voids them. It is NOT the
  deliverable; it only answers one empirical question.
- The implementation PR (the deliverable) is opened by this session from branch
  `nominal-child-edge-contract`, never stacked on #15868, against main.

## Measurement packets (verified bytes)

| role | sha256 | size | source |
|---|---|---|---|
| measurement rev7 | `636c6f739597e110a10f485742bd692ca5ce1b9577e054b028bf6cd233fb11cf` | 16269 B | #16061 comment 6714822583, base64 block |
| prep rev5 | `36e09b03e9700fc72b3a58d91c200f651cee09a1c45344c34f315bde387e1ad8` | 1916 B | #16061 comment 6714911966, bash block |

`rev7_meas.sh` / `rev7_prep.sh` here are the decoded forms. The pair is coherent only
if the prep's `SCRIPT_SHA=` line equals the measurement's hash — check it before any
run, that check is what voided an earlier pair.

Budgets in rev7: P1 build 720s, P3 claims 1700s, script deadline 2700s, scope
backstop 2760s, MemoryMax 16G with `MemorySwapMax=0`, `PrivateNetwork`, one attempt,
no retry. Executor is `quick-newt-573`; this session holds **zero** host access.

## Attempt 1 outcome (already spent)

`OUTCOME=NO_OBSERVATION rc=2 reason=closure_failed closure_rc=124` — the
`--print-entry-closure` pre-flight hit its 120s cap (that cap was this session's
trading error), P3 never ran, so **no table exists and nothing may be inferred from
that attempt**. The claim binary it built is
`92afb15ade7bcee735f5ec689d2b79d29d75e086a34207b277b3194841ef7739`, sha-reconciled
before and after. Receipts: #16061 comment 6714793685.

## How to verify a finished run

```
python3 selftest_verify.py                      # must print ALL CHECKS GREEN
python3 verify_nal.py <dir> --roster $(cat roster.csv)
```

`<dir>` is the extracted, base64-decoded receipt bundle. The checker refuses to
accept a missing cell as a red, flags a name matched as a prefix of another,
rejects a changed claim binary, and cross-checks every cell against the machine
ledger (column 0 dotted `module.function`, expected state in `{expect-hold,
expect-red}`, returned verdict in agreement, no duplicate or extra rows). It prints
a CONTROL / ARM / NODE table. Self-test covers nine adversarial bundle shapes.

**Interpretation order matters: read the ARM rows before the CONTROL rows.** The
comparison consumer consults the child-edge label before recursing into the target,
and the two branches are collapsed into one verdict by the reason selector, which
picks by position and not by argument index — so an expected mismatch at index 0
can mask a real one at index 1.

## Artifacts

| file | what it is |
|---|---|
| `verify_nal.py`, `selftest_verify.py` | receipt checker + its self-test |
| `roster.csv` | the 42 probe cells, in order |
| `roster_contract.csv` | the 12 contract cells (the targeted claim run's `--functions` list) |
| `roster_min.csv` | a trimmed 17-cell roster — **declined**, the dominant cost is the whole-pool index + warm-up, not the listed functions; do not re-offer |
| `interpret.md` | the full diagnosis: label producers, canonical presentation, consumer chain, the two candidate repairs and which rows decide between them |
| `pr_body.md` | implementation PR body draft; its slots are filled only from a real table |
| `rfm_skeleton.dag` | the missing-features row, blocked until the table produces the real subject DeclarationRef (a self-referential row about my own claim file is not an RFM row) |
| `discipline_search_evidence.txt` | the command-and-revision record behind every scoped negative in the PR body |

## Standing constraints a successor must keep

No in-container compiler builds; no host runs without a grant routed
keen-heron → sleek-owl → keen-otter; execution through CI only; merge main, never
rebase; never stack on #15868; cite **symbols, never `file:line`** (DESIGN §3); a
scoped negative must name its command and revision or be deleted (DESIGN §6); a red
in a claim module is evidence about the substrate, not a test to adjust; never
normalize labels inside the comparison (symptom patch), never translate between two
presentations (§3 fork) — delete the second presentation.

Blocked as of this snapshot on the fleet stop (srv4 runners refuse their memory
envelope, `cause=MemoryEnvelopeAbsent`) plus the no-re-attempt ruling, so a rerun
needs a fresh named grant.
