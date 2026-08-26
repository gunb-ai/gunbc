# Plan — v2 self-hosts the entire v2 corpus

**Status:** ACTIVE anchor, opened 2026-08-26 by operator ruling ("we're doing v2 self hosting the
entire v2 corpus… I would just start from scratch"). Supersedes
`docs/plans/self-host-cargo-refusal-root-partition.md` (2026-08-16), whose disposition is §7.

**Rules this document holds itself to.** Every claim names how it was measured and when, or says it
is unmeasured. **No transcribed instrument output** — a measurement is cited by naming the producer
that re-derives it (DESIGN, operator ruling 2026-08-24), because a copied number rots without
anyone touching either end. Citations name a module and symbol, never a file:line (DESIGN §3).

---

## 1. The goal, and what it buys

**v2 compiles the entire v2 corpus, and the emitted Rust builds.** That is the enabling event for
the deletion below; it is not a milestone on a list.

**What bankruptcy deletes, measured on `baa4a8586f` 2026-08-26** (`find src/v1/stage0/src -name
'*.rs'`, partitioned on the `// Source module:` emit header):

| | files | lines |
|---|---:|---:|
| emitted from `.dag` | 133 | 166,727 |
| hand-written Rust | 65 | 105,604 |
| total seed | 198 | 272,331 |

The hand-written half is what self-host retires, and it is concentrated: `cli_run.rs` (48,794) and
`v1_interpreter.rs` (17,258) are **62.5% of it between them**. The rest is a long tail of 1–5k-line
files. So this is not 65 problems; it is two and a tail.

**Subject scale:** `src/v2` is 1,281 `.dag` files / 282,387 lines, of which `src/v2/compiler` is 71
modules.

---

## 2. Where the wall actually is

**Emission is not the wall. cargo is.** The superseded document established this and it survives as
the one load-bearing inheritance: modules already emit Rust, then fail `cargo build`.

**No required check measures rustc, and this is the finding that opened this plan.** The required
CI v2-emission phase (`v1_compiler.cli_run` `run_required_v2_emission`, selected by
`RequiredCiPhase::V2Emission`) classifies `CompileDisposition` — `Completed` / `Refused` /
`NotExecuted` — and **never invokes cargo**. `gunbc.ci_layer_roots`
`required_v2_emission_dissolution` says so in its own words: *"same producer, stopping before
cargo."*

So a green v2-emission phase means **the v2 root emits**. It carries no information about whether
the emitted Rust compiles. A reader who takes it as a self-host signal will be wrong, and that
misreading is the reason the rustc population has been invisible to every green run.

**Verified 2026-08-26** by reading `run_required_v2_emission`'s call site in `claim_executor` and
the dissolution row. Not inferred from a run.

---

## 3. Hosting — settled, and previously mis-stated

The whole-corpus compile requires ~7 GiB (`gunbc.whole_corpus_compile_admission`, threshold derived
from two dated CI receipts, deliberately not scaled for corpus growth).

| host | admits? | basis |
|---|---|---|
| **CI self-hosted runner** | **YES** | `v1_compiler.memory_governor.tests` `whole_corpus_compile_refuses_the_budget_that_was_sigkilled_and_admits_the_ci_runner` — a witness asserting exactly this. Corroborated: the floor peaks ~9.4 GiB on a CI runner. |
| **srv1 / srv2** | likely, unverified | operator granted access 2026-08-26; capacity **not yet measured**. |
| BuildBuddy remote | no | MemAvailable under 5 GiB; two SIGKILL exit-137 receipts. |
| in-session | forbidden | shared slice, swap off; the kernel kills the largest task, possibly another session's floor run. |

**A correction this plan exists partly to record.** This was carried to the operator as *"no host
can run it."* That was false. The true statement is *"neither of the two routes we habitually reach
for qualifies"* — and CI, the third host, was already admitted by a witness whose name says so.
The error is a denominator error: *hosts we use* mistaken for *hosts that exist*. It cost a
plan built around a constraint that did not hold.

**Consequence: the whole-corpus compile is hostable, so the corpus-wide number is obtainable.**

---

## 4. The measurement

**Instrument:** `tools.emission_entry_instrument` `measure_entry_emission` — emit, `cssl_assemble`,
cargo under JSON message format; returns a typed `EmissionMeasurement` carrying per-member identity
and location, so two runs **join** rather than merely differ, and an unreached stage renders as its
own variant instead of as zero.

**Two subjects, and they are different quantities.** Conflating them is the trap this section
exists to prevent:

- **per-entry** — errors in the closure of entry E. Fits any host. **Not a lower bound on the
  corpus figure in any statable way**: the corpus records a narrow `--entry` run that reported clean
  while twelve real sites sat outside its closure.
- **whole-corpus** — the number this plan drives to zero. Hostable on CI per §3.

**The union of per-entry closures is not the corpus.** This must stay a first-class caveat wherever
a board is published, or a green board will eventually be read as "v2 self-hosts," which it does not
mean.

**Never run the whole-corpus route on a host that does not admit it.** Its failure mode is SIGKILL
with no diagnostic; a harness grepping that output reads **zero**, indistinguishable from a real
zero. Record exit status explicitly, always.

**Instrument vintage is part of the measurement, and the default route gets it wrong.** An
ordinary `ctrl-build --remote` dispatch runs with cwd `/root/workspace/repo-root`, which is itself
a gunbc checkout — so the shim's gunbc-checkout guard **fires on the runner** and execs the
*runner's own installed* gunbc, of unknown vintage. Measured stale by `clever-ibex-598`
(2026-08-26). This is the DEFAULT path, not an edge case: a remote dispatch, in a checkout, doing
the obvious thing.

**So S2 must build gunbc from source and run it in one dispatch.** The binary's vintage is then the
tree's by construction and cannot drift. (The same rule already existed for an unrelated reason —
a remotely built binary is amd64 and will not execute in an arm64 session container — and it turns
out to carry a freshness guarantee too.) Matching the working directory is **necessary and not
sufficient**: harnesses such as `witness_bin_ready` build their own binary, so match the *binary*,
not the cwd. The vintage of the actually-routed BuildBuddy compiler is **unmeasured** and must not
be inferred from the runner-installed figure.

**Probe the property, not a proxy.** `--version` is rejected by both installed binaries, so it
never discriminates. Ask the binary whether it can read a `//` annotation.

**Current state: there is no board.** The 2026-08-24 measurement bankruptcy deleted `docs/probes/`
whole; one file survives. The instrument was restored, no board was. **Any figure quoted for the
current rustc population today is unsourced.**

---

## 5. The ratchet

**The mechanism exists and is correctly designed.** `gunbc.emit_subject_clean_frontier` +
`tools.emit_subject_clean_ratchet`. Its own header already argues against the naive form, and the
argument is worth preserving here because it is the thing most likely to be re-proposed:

> counts appear in this module as DISPLAY ONLY and decide nothing. Admission is per-identity: the
> debt is carried as ROWS.

**Why a count must not be the gate.** DESIGN §5 forbids a merge-blocking literal grounded in a
measurement copied from the current tree. Worse here, the incentive inverts: the cheapest way to
make a count fall is to **lose subjects** — narrow the entry set, drop a module — and a
count-gate rewards exactly that. A discovery that loses subjects makes it fall furthest of all.
Identity-grain rows cannot be gamed that way.

**The working template**, landed 2026-08-26 in `v1_compiler.declaration_index`: an enumerated debt
roster (`PRE_EXISTING_CITATION_DEBT`) whose rows **refuse once their subject stops failing**
(`CitationDebtRowStale`), plus the inverse arm for planted controls that stop discriminating
(`PlantedControlNoLongerRefuses`). The roster can only shrink. Both directions are wired.

**Enrolment status: nothing runs it.** `emit_ratchet_runner_cadence` is `NoConsumer`; no workflow,
phase or required lane reaches the ratchet. #9346 states this is "independent of the enrolment
question and of the operator agreement that gates it." **The operator gave that agreement
2026-08-26.**

---

## 6. Program

Each step names its verification. A step is not done because it landed; it is done when something
executes and a discriminating input goes red.

**S1 — land the ratchet's correctness fixes.** #9315 merged (`baa4a8586f`). **#9346 is still OPEN**
— verified 2026-08-26, contrary to a report that both had merged.

**S2 — re-derive the corpus census on a CI runner.** Whole-corpus, via §4's instrument, exit status
recorded. Output: population at identity grain (code, module, site), not a count.
*Verification:* the run's own receipt; the board names the commit and the instrument.

**S3 — enrol a required CI phase that runs cargo.** The gap §2 names. Subject: the corpus, hosted
per §3. *Verification:* a discriminating red — a deliberately broken emission must fail the phase.
Without that, the phase is a decoration (DESIGN §4b).

**S4 — enrol the ratchet against S2's population**, today's blockers as the admitted baseline
roster, identity grain, rows-refuse-when-fixed. *Verification:* a planted still-failing row must
refuse when its subject is repaired; an added blocker must refuse.

**S5 — drive the population down by root, not by site.** Sequencing follows §8.

**S6 — retire the seed.** Only after emitted Rust compiles **and** behavioral equivalence is
re-established (§9). Concentrated per §1.

---

## 7. Disposition of the superseded document

`docs/plans/self-host-cargo-refusal-root-partition.md` (2,773 lines, operator-directed 2026-08-16,
last touched 2026-08-24) is **superseded**, on the operator's ruling that it is out of date.

**What is inherited:** §2's emission-is-not-the-wall correction. That is the durable finding.

**Why it cannot be the anchor, measured 2026-08-26:** all **ten** of its `../probes/` evidence links
dangle — `docs/probes/` retains one unrelated file. Its census is dated 2026-07-26, its own text
calls it "three weeks stale… treat as shape, not current counts," the TSV was deleted 2026-08-16,
and the shell probe that produced it went in the bankruptcy. It names a session that no longer
exists. Its headline sizing finding — that a large majority of distinct sites appear on two or more
modules — is **currently unsupported**: the board it cites is deleted.

**That finding is worth re-deriving, not inheriting.** If it holds, the ratchet's early motion comes
from root fixes rather than site-by-site grinding, which changes the whole shape of S5. S2 must
re-establish it or refute it.

**Open question for the operator:** delete it, or leave it marked superseded? DESIGN's
replacement-migration doctrine says a surviving X is an attractor — while it stands, nearby
questions keep being answered in its vocabulary. It is also a shared coordination surface
(`smart-ram-730`), so deletion is not unilaterally mine.

---

## 8. Sequencing input — historical, explicitly not current

The superseded census (2026-07-26, instrument deleted, **do not plan against these as current**)
recorded 9,444 error instances over 20 modules and 24 rustc codes, with three codes — E0308
mismatched types, E0277 trait bound, E0599 no method — at roughly three-quarters of instances.

It is repeated here **only** as the shape S2 should expect to confirm or refute, and because a
plan that hides its prior is harder to falsify. If S2 returns a materially different distribution,
that is a finding about the last month, not an error in this row.

---

## 9. What self-host is not

**Errors-to-zero is necessary, not sufficient.** Two independent gaps:

- The emitter cannot currently produce `main.rs`.
- **Behavioral equivalence is unmeasured.** DESIGN §7 requires the emitted module be *behaviorally
  equivalent to the seed on a discriminating corpus*, proven by execution — explicitly **not** a
  byte-identical fixed point. The behavioral-receipt phases were among the five deleted from the
  required run on 2026-08-21; that measurement is not currently taken.

So a rustc-clean corpus permits S6 to be planned. It does not authorize it.

---

## 10. Ownership

| area | owner |
|---|---|
| root partition / frontier | `smart-ram-730` |
| ratchet design + enrolment | `loyal-lark-254` |
| emission leaf-spelling root | `smart-wolf-868` |
| alignment, sequencing, this document | `warm-hawk-909` |

Operator agrees each step as we move; no step is assumed granted by the presence of a later one.
