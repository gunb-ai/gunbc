# Plan — the import/namespace program

> **Status: ACTIVE anchor**, opened 2026-08-26. Peer plan to `v2-corpus-self-host`; the two answer different questions and neither is a section of the other. Material supplied by the owning session (snappy-dove-250) on request. Where a number is not currently re-derivable it is marked STALE AS A POPULATION rather than quoted in the present tense.

## 1. The end state

**The import concept does not exist.** Not narrowed, not simplified — absent. Four consequences, each of which is a separate thing to build:

- A module's dependency set is **derived** from the resolved targets of the references it actually makes. It is never declared.
- Every name is spelled as the **fully qualified declaring identity** — the namespace where the declaration lives, not the namespace a reader reached it through.
- `visible_through` is **provenance**, never the qualifier. It records how a name became reachable; it never participates in spelling that name.
- `import` is a **parse error**. The grammar does not admit it.

The last of those is the one people reach for first and it is the one that lands LAST — see section 3.

## 2. Two documents, two questions — neither superseded whole

This has been misread in both directions, so it is stated as a table. The distinction is that one document answers SEMANTICS and the other answers EXECUTION, and the execution document is superseded on exactly one axis.

| document | answers | standing |
| --- | --- | --- |
| `namespace-resolution-design.md` | SEMANTICS — what resolution means | Live. Operator ruling 2026-07-06. NOT superseded. |
| `namespace-cut-replacement-plan.md` | EXECUTION — root, census, order | Live as to ROOT and CENSUS. **Superseded on ORDER** by operator ruling 2026-08-25. |

**Do not repair the order by editing the document.** The ordering fact has a carrier, and the carrier is the authority — the document's `SupersedesOnOrderOnly` marking exists so that a reader who finds the stale ordering sentence knows to go to the carrier instead of trusting it. Any ordering claim in this plan or downstream of it binds to `current_landing_order`, never to a document sentence.

## 3. The landing order, and why grammar is last

Ordering per `gunbc.namespace_cut_landing_order` `current_landing_order`; the grammar-last constraint per `namespace_cut_grammar_last_ruling` (operator, 2026-08-25). These two are BOUND — the roster at the foot of this module cites them, so a rename or deletion refuses at ingestion.

| step | content | state |
| --- | --- | --- |
| 0 | Strip measurement | DONE 2026-08-15 |
| 1 | Delete the import-name universe — the semantic root | open |
| 2 | Host and emit machinery | open |
| 3 | Repoints | open, CONTESTED |
| 4 | Fix forward at class grain | open |
| 5 | Grammar and parse deletion | **LAST** |

**Why grammar-last is not a hedge.** Deleting the grammar first makes every unrepaired module unparseable at once, which converts a fix-forward program into a flag day. Deleting the semantic root first (step 1) makes every real dependent refuse LOUDLY while the source still parses — which is exactly the delete-first census DESIGN.md §3 prescribes, run at the level where the refusals are readable. The grammar deletion at the end is then a no-op over a corpus that already spells nothing that way, and its RED is that some construct still does.

## 4. Ownership — one measured, one assumed

Stated with provenance because a wrong owner costs a wave:

- **crisp-crab-430 owns Step 4** (fix-forward at class grain). Reported by the owning session.
- **smart-wolf-868 is placed at Step 2** (host/emit machinery) — **ASSUMED, not measured.** The owning session flagged this themselves. Confirm directly with that session before any sequencing decision rests on it; do not carry it forward as fact.

PROSE, both — no carrier records step ownership today. That absence is itself a gap: ownership of a wave is exactly the kind of fact that goes stale in silence.

## 5. The strip measurement — what survives of it

Receipts at `docs/plans/import-strip-measurement/`. The reconciliation was exact: stripped-hard equals control-hard plus attributable, over eleven classes.

**The corpus sha256 is the anchor, not the commit line.** A measurement of this kind is only rejoinable if the exact input can be identified, and the commit is a weaker identifier than the hash of what was measured.

**Standing: STALE AS A POPULATION — now MEASURED, not merely presumed.** The owning session re-ran the receipt's own anchor recipe (2026-08-26) and got a different corpus hash than the receipts record, on a worktree BEHIND main; since the corpus only moves forward, main differs at least as much. So the receipts describe no current tree, and this is a measurement rather than a precaution. The eleven classes are what SURVIVES; whether they are still COMPLETE is unverified, and nothing has re-run to test it — a class of breakage introduced since the measurement would not appear in the taxonomy at all. The taxonomy is usable as a partition of the work that was measured, never as a coverage claim over the work that exists now. The row counts are a photograph of one corpus at one moment and must not be quoted in the present tense — including in a PR body, a status line, or a ratchet. Anyone needing a live population re-runs the instrument against current main; anyone needing to know WHAT SHAPES OF BREAKAGE EXIST reads the taxonomy and is correctly served.

## 6. Falsifiers

Two, both supplied by the owning session, both stated so that a real answer can kill the program rather than merely delay it:

- **(a) Spelling.** If there is a real construct whose declaring identity cannot be fully qualified, the end state in section 1 is unreachable as stated and needs an amendment, not a workaround.
- **(b) Cost.** If deriving the dependency set costs more at corpus scale than declaring it, the derivation is the wrong trade and the program needs an operator decision, not a faster implementation.

Neither has fired. Neither has been tested. **(b) is measurable now** and nothing is measuring it — that is the cheapest open item in this plan.

## 7. The sequencing relation — RULED, and carried elsewhere

This section was an open question for the operator. It was answered on 2026-08-26, and the answer is that **the question was posed at the wrong grain**: neither program blocks the other whole, and the relation is a braid between MILESTONES rather than an order over PROGRAMS. A reader who collapses it back to a program order gets a different plan whichever direction they collapse it in.

**THE EDGES ARE NOT ENUMERATED HERE, DELIBERATELY, AND THIS TEXT IS NOT AUTHORITY FOR THEM.** Read them from `gunbc.compiler_frontend_program_interlock` `milestone_prerequisites`, which is a total function over a closed milestone variant — so a newly declared milestone fails to compile until its prerequisites are stated, rather than silently having none. This section RENDERS the relation for a human reader; it does not own it, and where the two disagree the carrier is correct and this prose is a defect. An earlier revision of this section restated both edges in full while claiming the carrier was their only home (review 56363): a symbol citation verifies that a declaration EXISTS, never that prose about it still AGREES with it, so enumerated edges beside a citation are exactly the drift a single authority is supposed to prevent.

### What this changes for THIS plan, concretely

- **Step 1 is gated**, and on what is stated by `milestone_prerequisites` at `NamespaceFirstSemanticWave` rather than here. Read it there; it is one call.
- **Section 9's disclosed CI gap is now a BLOCKER, not a disclosure.** No namespace change that can alter which modules enter a subject, or what an occurrence denotes, may merge before that wall exists.
- **Preparatory work is explicitly unaffected** — measurement, port-locus, instruments, lexical de-forking that preserves the exact set, typed carriers, dry-run projections. Step 1 is not preparatory.
- **The wall admits on UNADJUDICATED delta being empty, never on delta being empty.** Expected cut motion may occur; unevaluated or unexplained motion may not. A wall demanding zero delta would refuse the cut itself and would then be repaired by weakening it.
- **A pre-cut census is not wasted.** When a wave changes the subject constructor or the name authority, a prior population becomes HISTORICAL rather than false. It is not erased and the ratchet baseline is not silently reset; intentional namespace-induced debt is carried by exact typed transition admissions.

## 8. Adjacent, deliberately not a wave

**Port-locus (gunbc#9313) is a branch, not the spine.** It is a usability dependency for steps 3 and 4 — it makes the repoint and fix-forward work tractable to author. It is recorded here so that it is not mistaken for a wave and not mistaken for optional; it is neither.

### A source not to cite

The namespace-cut postmortem sits on an open PR (gunbc#9201). Do not cite it as a source for anything in this plan while it is unmerged — cite the carrier. An open PR is a proposal, and citing one as authority manufactures a standing it does not have.

## 9. What this plan does NOT claim

- **No CI mechanism enforces any of this** — no ratchet, no phase, no gate over the import population. **As of the 2026-08-26 ruling this is a BLOCKER rather than a disclosure**: it now gates Step 1 by name. A plan that reads as governed when it is not is worse than one that reads as unguarded, and this gap has since been given an executable consequence rather than left as candour.
- **No population figure here is live.** Section 5 says why.
- **Step ownership is not carried by any authority.** Section 4 says which half is assumed.
- **The ordering relation to self-host is RULED and is no longer a gap** — section 7 projects `gunbc.compiler_frontend_program_interlock`. It is listed here only so a reader of an older revision does not go looking for the open question.

**Three live gaps and one retired question**, each named at the grain at which someone could close it. The ordering bullet above is the retired one and is kept only so a reader of an earlier revision does not go hunting for an open question that has been answered. That is the deliverable for a first anchor — not the appearance of a finished program.

## Dissolution trigger (DESIGN §6)

Delete this plan when the import concept does not exist: dependency sets are derived from resolved reference targets, every name is spelled as its fully qualified declaring identity, visible_through carries provenance only, and import is a parse error — at which point step 5 of current_landing_order has landed and the plan's subject no longer exists.
