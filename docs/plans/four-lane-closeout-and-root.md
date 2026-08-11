# Four-lane closeout — what was delivered, and the root

**Closed 2026-08-10.** Lanes: CONVERGE (`wise-moth-430`), CI RESET (`warm-heron-497`), CI2-0 (`valiant-boar-65`), DEVBOOT-0 (`keen-moth-104`). Falsifier (`stern-ant-249`) archived mid-flight; its PR was adopted by the parent session.

This note exists because four sessions were archived holding measurements that lived only in message threads. Every number below is measured, attributed to the lane that measured it, and marked where it stops.

---

## 1. The honest board

Displacement means *what a person or machine can now do that it could not before*. Not PRs.

| lane | asked | displacement delivered |
|---|---|---|
| CONVERGE | a merged main revision becomes observed fleet state | **none.** srv1 still cannot observe `refs/fleet/desired`. #8108 merged, never deployed. |
| CI RESET | concurrency key + red control + fast gate | **none merged.** main still groups by `run_id`. Fix exists on a branch. |
| CI2-0 | tests execute on a v2-owned native route | **none.** Board 0/8876. Cohort empty at the current frontier. |
| DEVBOOT-0 | live modeled fleet proof receipt | **none live.** Fixture path works; no wet capture exists. |

Four lanes, several days, **zero displacement**. Everything landed is *capability* — real, witnessed, and unexercised against reality.

What capability did land, and it is not nothing:

- CONVERGE: a stale receipt can no longer fabricate `converged` (#8108 `ReceiptConvergedUnbacked`); the CLI can no longer render a false converged (#8101); the collector compiles on the CI path at all (#8103).
- CI2-0: three compiler repairs — arrow-lambda grammar, the sugar capture-slot repair, the graft dissolution guard.
- DEVBOOT-0: capture → harvest → receipt with typed refusal when `compile_count=` is absent from SERVED prose.

**One of those prevented silent wrongness before it reached main.** A multi-line sum declaration was lowering a *phantom variant named by the leading pipe token* — an `Accepted` program containing a coproduct arm that does not exist in the source. Caught, fixed, red control enrolled.

---

## 2. What the four lanes named as root

Each lane was asked for the root with evidence, and explicitly invited to reject the manager's hypothesis. **All four declined to confirm it.**

**`valiant-boar-65` — the sharpest, and it re-cut the question.** Of 11 rejecting members in one real 15-file closure, 10 attributed:

| class | count | modules |
|---|---|---|
| retention-then-rejection (normalize contradicting its own contract) | **5** | `dag/std/algebra`, `src/v2/std/logic`, `src/v2/std/optional`, `src/v2/std/diagnostic`, `dag/std/occurrence_identity` |
| total graft dissolution (guard working correctly) | 2 | `src/v2/std/live_tree`, `dag/std/error_primitives` |
| genuine frontend gaps | **3** | `dag/std/types` at **LEX**, `src/v2/std/node` at **PARSE**, `dag/std/content_hash` at **PARSE** (where-refinement form) |
| reason uncaptured | 1 | `src/v2/std/algebra` |

> *"For half the population the capability is not absent, the contract is broken — and a broken contract is a bounded repair with a known intended behaviour, while a missing capability is a program."*

normalize retains wrapper declarations for un-lowerable bodies under its declared `lowered | wrapper-retained{cause}` / "corpus stays green" frontier, then its own module-grain `well_formed` gate rejects the tree those retentions live in. **Three genuine gaps is a very different roadmap from eleven.**

**`warm-heron-497`** — CI floor wall-clock is the binding feedback loop; a one-row workflow fix cannot land without spending a floor, and the broken key on main stacks another floor fleet-wide on every main move. Measured: 43m39s, 40m3s, 1h0m58s across three runs, ~2.5h wall, nothing merged. Explicitly refused to fuse this with instrument fragmentation, which they name as a *separate* measured cause of false confidence.

**`wise-moth-430`** — the floor is the single serialized gate for **both merge and deploy**. Measured the piece touching the manager's hypothesis: the interpreted seed pays **~70s per invocation** before any work. *"I measured the SEED's slowness, not the v2-ingest blocker that keeps us on it."*

**`keen-moth-104`** — hours went not into any compile fix but into *"re-proving what proof meant each time acceptance moved."* Proof machinery was validated on fixtures while acceptance required fleet execution; admission divergence a second silent gap.

---

## 3. Synthesis

Two lanes named **acceptance-contract shape** as their most expensive cost. Two named **CI floor latency**. All four named **instrument/admission divergence** as a distinct contributor. None supported *v2-cannot-ingest-std* as the root — it survives only as an unmeasured explanation for why the floor is slow.

These are one thing at three scales: **nothing could be verified incrementally.**

- The **contract** demanded the whole corpus before the first identity counted (CI2), and moved between measurements (DEVBOOT) — so work was repeatedly re-derived.
- The **mechanism** demanded 60–75 minutes before anything was known, and gated deploys as well as merges.
- The **instruments** built to escape those two costs were themselves unverified, so the day was spent debugging them.

Independent corroboration: **#8083 "CONVERGE-0 acceptance" merged while its own contract was unmet and its displacement read 0/1.** Once the outcome PR merges, the gate is gone, and every later defect becomes an orphan support PR while the denominator never moves. A terminal outcome needs a durable owner that survives support merges — a work item, never a PR.

---

## 4. Findings that outlive their lanes

1. **`rejected_with_pending` prepends accepted-carrier diagnostics to a later rejection**, so the *first* reason is systematically wrong. Bit three times: corrupted a sweep census, redded a working guard, and forces every reason assertion to be a containment read. A measurement tax paid by everyone downstream. *(valiant-boar)*
2. **Entry-vs-member position divergence** — the same retained content passes in entry position and fails in member position, same instruments, same route. **Measured, unattributed, open.** A compiler behaving differently by position with no named reason silently decides later questions.
3. **Admission diverges across three entrypoints.** `claim_batch` accepts §4c-invalid source that `gunbc` refuses; `claim_executor` is a third answer. Local green and CI green are different facts.
4. **A full-corpus `gunbc compile` exits 1 on clean main** — 25 standing `source annotation sits inside a declaration body` rows in `dag/test/claim/host_phase_status_witness_test.dag`. Exit status alone cannot discriminate a new defect from the standing population; the signal is diagnostic *text*.
5. **The deploy is gated on the floor.** `fleet` needs `ci.result == 'success'`; Deploy and Advance are push-only with no dispatch arm. A floor red or cancel blocks displacement entirely.
6. **The concurrency group key falls back to `run_id`**, unique per run, so `cancel-in-progress` — already `true` — is unreachable on push. Every main move stacks another floor.

---

## 5. Where the work lives

- `docs/plans/v2-frontend-std-ingestion-frontier.md` — per-module stage-and-reason table, written for a reader with no CI2 context (`22857aa365`).
- Compiler repairs on `session/valiant-boar-65`: `ccb8e8c99d` arrow-lambda grammar (15 controls), `c84842c98e` sugar capture-slot repair (9 controls), `e588031201` graft dissolution guard (4 controls). Witnesses enrolled in `dag/gunbc/ci_layer_roots.dag`.
- CONVERGE merged: #8100, #8101, #8103, #8108. Handoff: run `31408794329`, two-read close, watch member 1.
- CI RESET: commit `607e3a5825`, authority `dag/gunbc/ci_workflow_expressions.dag`, witness `ci_concurrency_group_uses_ref_not_run_id`.
- DEVBOOT-0: #8078, pushed `54b1e7703e1`.
- Falsifier: #8107 — mode fix + journal artifact. Cost half not delivered; title corrected to say so.

---

## 6. Next steps, in dependency order

1. **CI capacity is down.** 10 runs queued, 0 in progress, nothing started since 16:56Z. Every item below is blocked on it.
2. **Land the concurrency key alone**, off main, not inside a paused draft.
3. **Land the three compiler repairs**, especially the sugar fix — a phantom coproduct variant is found six weeks later by something unrelated.
4. **Close CONVERGE**: get one green floor on `a9d0c324`, then the two-read close.
5. **Repair normalize's contract violation** — 5 of 11, bounded, known intended behaviour. This is the cheapest large win on the board.
6. **Then** the three genuine frontend gaps (LEX/PARSE), which are a program rather than a repair.
