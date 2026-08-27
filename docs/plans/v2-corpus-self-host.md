# Plan — v2 self-hosts the entire v2 corpus

> **Status: ACTIVE anchor**, opened 2026-08-26 by operator ruling — v2 self-hosts the ENTIRE v2 corpus, started from scratch. Supersedes the 2026-08-16 root-partition document, whose evidence base was bankrupted (section 7). Every claim names how and when it was measured, or says it is unmeasured. **No transcribed current-state board is treated as authority**, and where this plan does carry transcription it says so: the seed-sizing table in section 1 is disclosed historical transcription and names the missing producer required to replace it. The rule is that a measurement is cited by naming the producer that re-derives it; a figure with no producer is carried only with that absence stated.

## 1. The goal, and what it buys

**v2 compiles the entire v2 corpus, and the emitted Rust builds.** That is the enabling event for the deletion below; it is not one milestone among several. The operator's framing: when v2 self-compiles, we bankrupt v1 and a lot of the hand Rust.

Measured on `baa4a8586f` (2026-08-26) by partitioning every `.rs` under the stage0 seed on the `// Source module:` emit header:

| class | files | lines |
| --- | --- | --- |
| emitted from `.dag` | 133 | 166,727 |
| hand-written Rust | 65 | 105,604 |
| total seed | 198 | 272,331 |

**These figures are a TRANSCRIPTION and there is no entry point that re-derives them.** Independently re-run on the same pinned commit by a second session (2026-08-26), every figure reproduced exactly — 133/65/198 files, 105,604 hand-written lines, `cli_run.rs` 48,794, `v1_interpreter.rs` 17,258, `src/v2` at 1,281 files and 71 compiler modules — EXCEPT the emitted line count, which came back 166,834 against the 166,727 here, with the total carrying the same delta so it is one discrepancy and not two. No conclusion in this plan turns on 107 lines. The finding is the shape rather than the digits: two people applying the same DESCRIBED PROCEDURE to the same commit got different answers, which is what a described procedure does and an instrument does not. The seed partition has no entry point the way emission now has `measure_entry_emission`, and building one is the item this row exists to name.

The hand-written half is what self-host retires, and it is CONCENTRATED: `cli_run.rs` (48,794) and `v1_interpreter.rs` (17,258) are 62.5% of it between them, the rest a long tail of 1-5k-line files. So this is not 65 problems; it is two and a tail.

Subject scale: `src/v2` is 1,281 `.dag` files / 282,387 lines, of which `src/v2/compiler` is 71 modules.

## 2. Where the wall is — and why no green check has ever shown it

**Emission is not the wall. cargo is.** Modules already emit Rust; they then fail `cargo build`. That correction is the one load-bearing inheritance from the superseded document.

**No required check measures rustc.** The required-CI v2-emission phase classifies `CompileDisposition` — Completed / Refused / NotExecuted — and NEVER INVOKES CARGO. `gunbc.ci_layer_roots` `required_v2_emission_dissolution` says so in its own words: the receipt carries this emission boundary, *same producer, stopping before cargo*.

So a green v2-emission phase means the v2 root EMITS. It carries no information about whether the emitted Rust compiles. A reader who takes it as a self-host signal is wrong, and that misreading is why the rustc population has been invisible to every green run. Verified 2026-08-26 by reading the phase's call site and the dissolution row — not inferred from a run.

## 3. Hosting — settled, and previously mis-stated

The whole-corpus compile requires ~7 GiB, from `gunbc.whole_corpus_compile_admission` — a threshold derived from two dated CI receipts and deliberately not scaled for corpus growth.

| host | admits? | basis |
| --- | --- | --- |
| CI self-hosted runner | YES | a witness named for exactly this: refuses the budget that was SIGKILLed and ADMITS the CI runner. Corroborated by the floor's ~9.4 GiB peak on that runner. |
| srv1 / srv2 | likely, UNMEASURED | operator granted access 2026-08-26; capacity not yet measured |
| BuildBuddy remote | no | MemAvailable under 5 GiB; two SIGKILL exit-137 receipts |
| in-session | forbidden | shared slice, swap off; the kernel kills the LARGEST task, possibly another session's floor run |

**A correction this plan exists partly to record.** This was carried to the operator as *no host can run it*. That was false. The true statement is *neither of the two routes we habitually reach for qualifies* — and CI, the third host, was already admitted by a witness whose name says so. It is a denominator error: HOSTS WE USE mistaken for HOSTS THAT EXIST, and it cost a plan built around a constraint that did not hold. The corpus-wide number is obtainable.

## 4. The measurement

The instrument is `tools.emission_entry_instrument` `measure_entry_emission`: emit, assemble, cargo under JSON message format, returning a typed measurement carrying per-member identity and location — so two runs JOIN rather than merely differ, and an unreached stage renders as its own variant instead of as zero.

### Two subjects, and they are different quantities

- **per-entry** — errors in the closure of entry E. Fits any host. NOT a lower bound on the corpus figure in any statable way: the corpus records a narrow `--entry` run that reported clean while twelve real sites sat outside its closure.
- **whole-corpus** — the number this plan drives to zero. Hostable on CI per section 3.

**The union of per-entry closures is not the corpus.** This stays a first-class caveat wherever a board is published, or a green board is eventually read as *v2 self-hosts*, which it does not mean.

### Instrument vintage is part of the measurement, and the DEFAULT route gets it wrong

An ordinary remote dispatch runs with a working directory that is ITSELF a gunbc checkout, so the gunbc shim's checkout guard fires ON THE RUNNER and execs the runner's own installed binary, of unknown vintage — measured stale 2026-08-26. This is the default path, not an edge case: a remote dispatch, in a checkout, doing the obvious thing.

So the census must BUILD gunbc FROM SOURCE AND RUN IT IN ONE DISPATCH. The binary's vintage is then the tree's by construction and cannot drift. Matching the working directory is necessary and NOT sufficient — some harnesses build their own binary, so match the BINARY. The vintage of the actually-routed remote compiler is unmeasured and must not be inferred from the runner-installed figure.

**Probe the property, not a proxy.** `--version` is rejected by both installed binaries, so it never discriminates; ask the binary whether it can read a `//` annotation.

**Never run the whole-corpus route on a host that does not admit it.** Its failure mode is SIGKILL with no diagnostic, and a harness grepping that output reads ZERO — indistinguishable from a real zero. Record exit status explicitly, always.

**Current state: there is no board.** The 2026-08-24 measurement bankruptcy deleted the probe corpus whole; one file survives. The instrument was restored, no board was. Any figure quoted for the current rustc population today is unsourced.

## 5. The ratchet — and why a count must not be the gate

The mechanism EXISTS and is correctly designed: `gunbc.emit_subject_clean_frontier` with its runner `tools.emit_subject_clean_ratchet`. Its own header already argues against the naive form, and that argument is preserved here because it is the thing most likely to be re-proposed: **no cardinality is a gate oracle**, but emptiness is not nothing — it decides whether a population is inhabited or evaluated, including the essential distinction between an empty roster and an identified roster with no failures. Roster membership is admitted PER IDENTITY, and the roster DIGEST, not its count, is its identity. An earlier revision of this clause said counts are display-only and decide nothing; the ratchet owner measured that as literally false, and the corrected form is theirs.

**Why a count must not gate.** DESIGN section 5 forbids a merge-blocking literal grounded in a measurement copied from the current tree. Worse here, the incentive INVERTS: the cheapest way to make a count fall is to LOSE SUBJECTS — narrow the entry set, drop a module — and a count-gate rewards exactly that. A discovery that loses subjects makes it fall furthest of all. Identity-grain rows cannot be gamed that way.

**The working template**, landed 2026-08-26 in the declaration index: an enumerated debt roster whose rows REFUSE once their subject stops failing, plus the inverse arm for planted controls that stop discriminating. The roster can only shrink, and both directions are wired.

**Enrolment status: nothing runs it.** The ratchet runner's cadence is `NoConsumer`; no workflow, phase or required lane reaches it. That was stated as independent of the enrolment question and of the operator agreement gating it. **The operator gave that agreement 2026-08-26.**

## 6. Program

Each step names its verification. A step is not done because it landed; it is done when something EXECUTES and a discriminating input goes red.

- **S1 — land the ratchet's correctness fixes.** State deliberately not asserted here: a plan carrier must not record whether a PR is open or merged. That fact rots within hours, it is free to re-derive at read time with `gh pr view`, and nothing in the corpus refuses when it goes stale — so a carrier that asserts it will eventually instruct a reader to distrust an accurate source. This clause previously did exactly that about #9346, which was merged while the sentence said otherwise. **The receipt for DELETING rather than CORRECTING it** came the same night from a different PR: three readings of #9349 inside minutes — a dashboard reporting failing from a superseded run, a peer re-deriving green at the check-run level, and a third check finding the head had moved again and the PR was mid-run — each correct when taken, none current when quoted. A correction would have reproduced exactly that: a fact true when written, passed onward, with nothing in the sentence able to say which head it described. A PR-state sentence has no spelling for AS OF WHICH HEAD, so a true reading and a stale one become indistinguishable the moment either is quoted.
- **S2 — re-derive the corpus census on a CI runner.** Whole-corpus, via section 4's instrument, binary built from source in one dispatch, exit status recorded. Output is a population at IDENTITY GRAIN — code, module, site — never a count. *Verification:* the run's own receipt, naming commit and instrument.
- **S3 — enrol a required CI phase that runs cargo.** The gap section 2 names. *Verification:* a discriminating red — a deliberately broken emission MUST fail the phase. Without that the phase is a decoration, permanently green by construction, and worse than absent because it will be cited as coverage.
- **S4 — enrol the ratchet against S2's population**, today's blockers as the admitted baseline roster, identity grain, rows-refuse-when-fixed. *Verification:* a planted still-failing row must refuse when its subject is repaired, and an added blocker must refuse.
- **S5 — drive the population down BY ROOT, not by site.** Sequencing follows section 8.
- **How S3 and S4 establish their reds: MUTATION, not inspection.** Measured receipt, gunbc#9348 (2026-08-26): eleven acceptance controls all passed; the author refused to ship on that green and mutated their own suite instead. Three arms left FOUR claims failing under nothing, so three more were authored at the survivors. Two of the eleven were PERMANENTLY GREEN BY CONSTRUCTION — decoration, the state DESIGN section 4b calls worse than absent because it is cited as coverage. Neither was caught by review, by reading the claims, or by the suite going green. Cost: six arms, two rounds, about an hour. Two arms failing ALONE is what a discriminating control looks like, and it is only demonstrable by attack.
- **S6 — retire the seed.** Only after the emitted Rust compiles AND behavioral equivalence is re-established (section 9). Concentrated per section 1.

## 7. Disposition of the superseded document

The 2026-08-16 root-partition document (2,773 lines, operator-directed, last touched 2026-08-24) is SUPERSEDED on the operator's ruling that it is out of date, and DELETED on their authorisation 2026-08-26. Git history retains it.

**What is inherited:** section 2's emission-is-not-the-wall correction. That is the durable finding.

**Why it could not be the anchor, measured 2026-08-26:** all TEN of its probe-corpus evidence links dangled — the directory retains one unrelated file. Its census was dated 2026-07-26; its own text called it three weeks stale, treat as shape not current counts; the underlying data file was deleted 2026-08-16 and the shell probe that produced it went in the bankruptcy. It named a session that no longer exists.

**Its headline sizing finding — that a large majority of distinct sites appear on two or more modules — is CURRENTLY UNSUPPORTED**, because the board it cites is deleted. That finding is worth re-deriving, not inheriting: if it holds, the ratchet's early motion comes from root fixes rather than site-by-site grinding, which changes the whole shape of S5. S2 must re-establish it or refute it.

## 8. Sequencing input — historical, explicitly NOT current

The superseded census (2026-07-26, instrument since deleted — DO NOT PLAN AGAINST THESE AS CURRENT) recorded roughly nine and a half thousand error instances over 20 modules and 24 rustc codes.

**That census also recorded a CONCENTRATION — three codes at roughly three quarters of instances — and this plan states it as a HYPOTHESIS WITH A TEST, not as a finding.** The two halves of a dead census do not decay at the same rate, and the caveat is easy to attach to the wrong one. The magnitude is inert: nobody can act on nine and a half thousand without a board, so its staleness costs nothing. The concentration is what a reader would actually act on — it is a claim about the emitter's failure DISTRIBUTION, and the emitter has been under continuous change for a month by lanes whose express purpose is moving that distribution. A magnitude drifts; a concentration can INVERT, and a plan that sequenced work by the stale one would put its effort exactly where the wins have already been taken. **The test:** re-derive the code partition with `measure_entry_emission` over the ratchet roster and compare the top three against that list. Until that runs, no wave in this plan may be ordered by which codes are believed to dominate.

It is repeated here ONLY as the shape S2 should expect to confirm or refute, and because a plan that hides its prior is harder to falsify. If S2 returns a materially different distribution, that is a finding about the intervening month, not an error in this row.

## 9. What self-host is NOT

**Errors-to-zero is necessary, not sufficient.** Two independent gaps: the emitter cannot currently produce `main.rs`; and behavioral equivalence is UNMEASURED. DESIGN section 7 requires the emitted module be behaviorally equivalent to the seed on a discriminating corpus, proven by execution — explicitly NOT a byte-identical fixed point, which would force v2 to reproduce the seed's warts. The behavioral-receipt phases were among the five deleted from the required run on 2026-08-21, so that measurement is not currently taken.

So a rustc-clean corpus permits S6 to be PLANNED. It does not authorize it.

## Dissolution trigger (DESIGN §6)

Delete this plan when the whole v2 corpus emits Rust that cargo builds with zero blocking diagnostics, a required CI phase enforces that by execution with a discriminating red, and behavioral equivalence to the seed is re-established — at which point S6 retires the hand-written seed and the plan's subject no longer exists.
