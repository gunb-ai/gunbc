# R4 — Program Dispatch Plan to v4-done (T-1 … T-32)

> **Purpose.** A single reviewable artifact: the full remaining-work dependency
> chart + the proposed lane/manager dispatch mapping for maximum
> parallelization. **This is a discussion PR** — review/correct your lane's rows
> before anything fans out. Derived from `src/v4/TASKS.md` (1265 lines, verified
> 2026-05-18); statuses marked *derived* are first-articulation and explicitly
> open to lane-manager correction in review.
> **Anchor convention:** every `TASKS.md:NNN` line anchor below refers to
> this file — `src/v4/TASKS.md`. There is no root-level `TASKS.md`; do not
> hunt for one.
>
> Authoring session: `witty-cat-59` (root). Lanes referenced: `fierce-cat-31`
> (Lane A — compiler pipeline), `swift-ram-178` (Lane B — test/bootstrap infra),
> `vivid-carp-207` (T-4 manager), `jolly-ibex-599` (Dissolution burn-down).

---

## 1. The honest answer: most remaining work is *not* design- or interface-blocked

Of ~31 live tasks, **~14 are pure-IMPL-ready now** (interface/design exists,
just needs bodies). The genuine long pole is a **4-item keystone cluster that
funnels through T-4**, and 2–3 of those are *operator rulings*, not unwritten
engineering.

**Irreducible serial spine (remaining):**

```
                 ┌─ P1-KEYSTONE (Practice-10 A1 invariant) ─┐
   keystone ─────┼─ T-25-core (refinement substrate)       ─┤
   cluster (×4)  ├─ T-30 (hollow-alias gate)               ─┼──▶ T-4 ──▶ T-9 ──▶ T-10 ──▶ T-11 ──▶ T-16 ──▶ T-15
   = TASKS.md:64 └─ T-29 (C++ ABI; cpp-slice feeder)        ─┘  (5 langs)  infer    emit    per-tgt  omni    self-host
   exact set                                                                                              fixed-point
                                                                                                          (anti-regress)
   T-1..T-8  =  LANDED  (front-end in CP-1b reconciliation tail; not the bottleneck)
   T-29  =  core #3267 MERGED, **residual #3277 OPEN**; TASKS.md:64/:286/:1035
            STILL declares it a hard T-4-cpp-slice feeder → REMAINS in the
            cluster (NOT "LANDED/removed"); de-classify only when #3277 lands
            AND TASKS.md is updated (authority = TASKS.md, not this plan)
```

Everything else parallel-fills around that spine.

> **Review status (2026-05-18):** **4/4 lanes ratified** — Lane A
> (`fierce-cat-31`), Lane B (`swift-ram-178`), Dissolution (`jolly-ibex-599`),
> T-4-mgr (`vivid-carp-207`) — all corrections folded below. T-4-mgr CONFIRMED
> the T-4 keystone HOLD and supplied corrections (**T-4.10/T-4.12 LANDED**;
> **T-29 STAYS a keystone-cluster feeder** per TASKS.md:64/:286/:1035 +
> #3277-OPEN — codex REQUEST_CHANGES corrected an earlier wrong "T-29 landed"
> reclassification; see Folded clarifications) plus
> a new keystone-scope finding (early-canonical files' rework obligation).
>
> **Lane-A clarification (folded):** CP-1b close sits on the **T-8→T-9** leg, a
> T-9 prerequisite running **parallel wall-clock to the T-4 keystone wait** —
> *not* on the T-4→T-9 edge and *not* keystone-gated. CP-1b therefore progresses
> while T-4 waits on ratification (strengthens, not weakens, the parallel story).

---

## 2. Per-task dependency + dispatch table

Legend — **Blocked-on:** `LANDED` · `READY` (no unmet dep — full scope
dispatchable now) · **`READY*`** (only a **bounded pre-T-9 scope** is
dispatchable now — witness/scaffold/contract authoring; the
dependency-satisfied implementation is **T-9-gated**, see *T-9 trigger*) ·
`IMPL` (interface exists, parallelizable) · `IFACE` (needs a contract frozen
first) · `DESIGN` · `OP` (operator ruling) · `CP1` (needs v4 front-end output).

> **Anti-conflation rule (codex BLOCKING `a67577c3`, folded):** Wave-0 ≠
> dependency-satisfied implementation. A `READY*` row's Wave-0 work is **only**
> its named bounded pre-T-9 scope; the *refine-to-real* phase dissolves at the
> stated **T-9 trigger** and is **Wave-3**, not Wave-0. Lanes dispatch the
> bounded scope only; they do **not** read `READY*` as "build the whole thing
> now."

| Task | Scope | Status (derived) | Depends on | Blocked-on | Proposed lane |
|---|---|---|---|---|---|
| T-1 | std/node.dag root | LANDED | — | LANDED | — |
| T-2 | std/algebra.dag | LANDED | T-1 | LANDED | — |
| T-3 | std/* scalar+support stack | LANDED (datetime Wave-A2 tail) | T-1,T-2 | IMPL (tail) | Lane A / std |
| T-6 | compiler/01_tokenize | LANDED | T-3 | LANDED (CP-1b tail) | Lane A |
| T-7 | compiler/02_parse | LANDED | T-6 | LANDED (CP-1b tail) | Lane A |
| T-8 | 03_normalize + 03_resolve (+T-28 bundled) | LANDED (seam scaffold until CP-1b closes) | T-7 | IMPL (CP-1b close) | Lane A |
| **T-4** | extdeps/languages ×5 fact-bundles | WIP, **HELD** | T-3 (landed) + the **TASKS.md:286/:64 exact feeder set `{P1-KEYSTONE, T-30, T-29, T-25-core}`** — **P1-KEYSTONE** (= Practice-10 **A1-invariant** ratification — NOT the closed #3240 tracker, NOT the merged numeric #3226), **T-30**, **T-29** (cpp-slice; #3277 OPEN), **T-25-core**. Never a partial subset — all four gate T-4. | **OP + DESIGN** | T-4 mgr |
| T-4.5 | extdeps/posix + file_system | SCAFFOLD | T-3, T-25-core | IMPL | T-4 mgr |
| T-4.6 | extdeps/formats — T-4.6 owns **7** (TASKS.md:121): csv/json/json_schema/openapi/toml/yaml + `sql.dag` (v3 SQL port, single authority; v2-compile clean). Live `formats/` dir lists **8** files; the 8th, `spice.dag`, is **T-4.10's** (row below, LANDED #3168) — *not* a T-4.6 member, none displaced/retired | SCAFFOLD (T-4.6 7/7 present) | T-25-core, T-26 | IMPL | T-4 mgr |
| T-4.7 | frameworks/react | **SATISFIED** — `react.dag` header is `Status: T-4.7`; Rust smoke ratchet covers the pinned framework substrate | T-4 (ts) | LANDED | T-4 mgr |
| T-4.8 | coordination | MODELED — decomposed PR #3207 `WireContractFacts` + `CoordinationBind` shape; `WIRECONTRACT-OBLIGATION-TABLE-T4.8` per-effect rows present | T-4, T-4.7 | IMPL | T-4 mgr |
| T-4.9 | languages/verilog | **PASS (IN-B)** per file header | T-1,T-2 | LANDED | T-4 mgr |
| T-4.10 | formats/spice | **LANDED** (#3168 merged; receipt: `src/v4/extdeps/formats/spice.dag`, plus this row as the r4-program-dispatch-plan source-of-truth alignment) — *pre-D2-reversal canonical path; see rework-obligation note* | T-1 | LANDED | T-4 mgr |
| T-4.11 | claim/english_ingest | SCAFFOLD | T-3/verification.dag | IFACE (AssertKind) | Lane B |
| T-4.12 | languages/llvm_ir | **LANDED** (#3171 merged; +#3229 de-prose +#3300 cost-move on main) — *pre-D2-reversal canonical path; see rework-obligation note* | T-1,T-2 | LANDED | T-4 mgr |
| T-4.13 | languages/machine_code | **IMPL (modeled slice)** — D2-REV per file header; Isa-parameterized slice + zero-diagnostic smoke landed | T-3, T-4 LanguageModel | CP1 (T-4 LanguageModel; modeled slice landed) | T-4 mgr |
| T-4.14 | languages/ptx | **PASS (IN-B)** — **DECISIONS.md L-3**; `ptx.dag` header intentionally domain-neutral | T-1,T-2 | LANDED | T-4 mgr |
| **T-9** | compiler/04_infer | SCAFFOLD | T-8/CP-1b-close (∥) + T-4 | IMPL (T-4 keystone; CP-1b-close parallel, not keystone-gated) | Lane A |
| **T-10** | 05_emit + 00_compile | SCAFFOLD | T-9, T-4 | IMPL+IFACE | Lane A |
| **T-11** | emit per-target ×5 | NOT STARTED | T-10 | IMPL | Lane A |
| T-12 | lens/complexity + cost | SCAFFOLD (fan-out **in flight**) | T-9 (refine) | **`READY*`** — bounded scope: witness-first Acceptance authoring. *T-9 trigger:* real lens fold over inferred model → Wave-3 | Lane A (lens) |
| T-13 | lens/parallelism,effect,ownership,idempotency | SCAFFOLD | T-9 (refine) | **`READY*`** — bounded scope: witness/Acceptance authoring. *T-9 trigger:* real fold over inferred model → Wave-3 | Lane A (lens) |
| T-14 | TestClaim corpus + fixtures | SCAFFOLD | T-19 | IMPL | Lane B |
| **T-15** | bin/main + self-host fixed-point gate | **SCAFFOLD (bin/main receipt landed)** — `src/v4/bin/main.dag` is the trampoline authority, with parse smoke + CI receipt on main; full self-host fixed-point remains terminal Wave-6 work | ~ALL | IMPL (terminal) | Lane B |
| T-16 | full-stack omni-emission demo | NOT STARTED | T-4,T-10,T-11,T-4.5–4.8 | IMPL | Lane A |
| T-17 | lens/synthesis + std/report | SCAFFOLD | T-12, T-9 | DESIGN+IMPL | Lane A (lens) |
| T-18 | lens/coverage meta-lens | SCAFFOLD | T-12,T-13 | IMPL | Lane A (lens) |
| T-19 | lens/testgen | SCAFFOLD | T-1,T-2,T-3 | **READY** | Lane B |
| T-20 | workflow/bootstrap AS DATA | Materially advanced — #3213 landed bootstrap+CI-as-data on main; remaining = fill tail | T-1 | **READY** | Lane B |
| T-21 | lens/affected_set (IRT-1..4 held) | SCAFFOLD | T-1,T-2,T-3 | **READY** (honor IRT) | Lane B |
| T-22 | compiler/05_eval interpreter (PRIMARY exec) | SCAFFOLD | T-9 (refine) | **`READY*`** — bounded scope: interpreter scaffold + IRT-3 eval-shape. *T-9 trigger:* eval over inferred types → Wave-3 | Lane B |
| T-23 | lens/application | **IFACE FROZEN** | lens fwk | IMPL (in flight) | Lane A (lens) |
| T-24 | workflow/ci AS DATA | SCAFFOLD (T-20/T-24-adjacent slice landed via #3213) | T-20, T-21 | IMPL | Lane B |
| T-25-core | refinement base + fail-closed validate | SCHEDULED | — | **AUTHORIZE — shape operator-ratified (TASKS.md §962+ / coercion-design.md Cat-6); NOT a design fork** | std (build) + OP (authorize stamp) |
| T-25-tail | refinement prover (erase) | SCHEDULED | T-9 | IMPL (optim) | Lane A |
| T-26 | std/ boundary carriers (URL/HttpMethod) | SCHEDULED | T-3 | **READY** | std/ = canonical home; Lane A may run 1st PR as conduit |
| T-28 | std/ module-graph | SCHEDULED (bundled→T-8) | T-3 | IMPL (in T-8) | Lane A |
| T-29 | extdeps C++ ABI — **T-4 cpp-slice side-branch feeder** (TASKS.md:64/:286/:1035) | core #3267 MERGED; **residual #3277 OPEN** | T-3/machine | **KEYSTONE-CLUSTER FEEDER (still declared by TASKS.md)** — not "LANDED"; de-classify only when #3277 lands AND TASKS.md updates | #3277 worker — attribution open (reparent/correct) |
| T-30 | hollow-alias / fact-density gate | SCHEDULED — interim P5(b) mirror already on main (Rust gate + fact_density.dag witness + smoke) | — (none) | IMPL+OP (generated checker + operator closure remain) | Dissolution |
| T-31 | de-prose/de-template backward sweep | SCHEDULED (indep. of T-4 gates, TASKS.md:1197-1199) | — | **READY** | Dissolution |
| T-32 | minimum never-hand-edited bootstrap seed | SCHEDULED | — | DESIGN (doc) | Lane B / OP |

*(T-5 REMOVED; T-27 DROPPED.)*

**Folded clarifications (lane-manager review):**
- **T-31** decomposes into **(a) rework-rider** — corrections that ride other
  in-flight PRs (not independently dispatchable), and **(b) mop-up** — the true
  parallel-fill backward sweep (Wave-0-dispatchable, independent of T-4 gates).
- **Lens footnote:** the PREFIX driver/corpus gate is **T-23 + driver**, not
  T-12 alone; T-12/T-13/T-17/T-18/T-23 rows match the running lens fan-out.
- **T-4-mgr staleness correction (verified vs main):** T-4.10 (#3168),
  T-4.12 (#3171), T-4.14 (#3170) are **already LANDED**. T-4.9 (`verilog.dag`)
  carries **PASS (IN-B)** in its file header (probe receipt; §2 Status column echoes that for T-4.9). **T-4.14** (`ptx.dag`) records the same **PASS (IN-B)** dispatch posture under **DECISIONS.md L-3** with a **domain-neutral** file header (no process-axis status tag in the carrier). T-4.13
  (`machine_code.dag`) carries a **D2-REV Isa-parameterized modeled slice**
  with a zero-diagnostic `compile_to_dag` smoke; §2 keeps T-4.13 **CP1-blocked on T-4
  LanguageModel** per TASKS.md while recording the landed smoke/model
  evidence. These are table-accuracy only updates, without changing
  TASKS.md's remaining LanguageModel/refinement authority.
  **T-29 is NOT landed-and-removed**
  (codex REQUEST_CHANGES, TASKS.md authority): core #3267 merged but #3277
  OPEN and TASKS.md:64/:286/:1035 still declares T-29 a hard T-4-cpp-slice
  feeder — it **stays in the keystone cluster**, never was Wave-0. Wave-0
  parallel count ≈14 → ≈11 (driven by T-4.10/T-4.12 + the READY* re-tier,
  not by mis-dropping T-29).
- **T-4.6 / SQL table-accuracy note:** TASKS.md schedules SQL DDL by extending
  T-4.6 with `src/v4/extdeps/formats/sql.dag`, but the current
  `src/v4/extdeps/formats/` census is seven files **including `spice.dag` and
  excluding `sql.dag`** (`csv`, `json`, `json_schema`, `openapi`, `spice`,
  `toml`, `yaml`). The §2 row preserves the TASKS decision while making the
  live-tree absence explicit.
- **NEW keystone-scope finding (decision-relevant):** the early-canonical-path
  language/format files modelled **pre-D2-reversal** (spice #3168, llvm_ir
  #3171, verilog T-4.9, ptx T-4.14, and any sibling pre-#3240 models) likely
  carry a **fact-bundle REWORK obligation that is itself
  Practice-10/#3240-keystone-gated** — the *same* class as T-4 ×5 and #3280
  A-vs-B. "LANDED" ≠ "done": ratifying the
  **verbatim invariant** (`modeling-discipline.md` ~§594–600 — the fold is
  already on main; *not* the closed #3240 tracker) also scopes their
  rework. This **widens the keystone's blast radius** (see §3).
- **Coord flag:** T-29 residual #3277's worker (quick-bat-761) is **not in the
  T-4-mgr subtree** (absent from the graph). §2 attribution corrected to
  "open"; reparent/attribution routes via the owning manager, not assumed.
- **T-4.7/T-4.8 coordination update:** T-4.7 is satisfied on this tree
  (`src/v4/extdeps/frameworks/react.dag` plus its v3 smoke ratchet). T-4.8 is
  modeled against the decomposed PR #3207 interface: `WireContractFacts`
  carries exchange / settlement / consistency; `CoordinationBind` carries
  `CoordinationEffectKind`; `WIRECONTRACT-OBLIGATION-TABLE-T4.8` is present as
  per-effect obligation rows. T-4.6 does not gate T-4.8, but T-16 still
  consumes both `WireContract`/`DeploymentUnit` and format-backed artifacts.

---

## 3. Keystone cluster — ranked by fan-out (the long-pole collapsers)

| Keystone | Kind | Unblocks | Owner |
|---|---|---|---|
| **P1-KEYSTONE / Practice-10 A1 invariant** (the live gate; #3240 = closed tracker) | **OPERATOR ratification** | T-4 → T-9 → T-10 → T-11 → T-16; **also #3313 Wave-2 lenses; also the principled basis for #3280 A-vs-B** | **Operator** |
| **T-25-core** | **AUTHORIZE-to-build — shape operator-ratified (TASKS.md §962+ / coercion-design.md Category 6: base type + fail-closed validation at a named constructor boundary). NOT a design fork — earlier "design direction" framing overstated; corrected per §7 #3b** | T-4 refinement-bearers, T-4.5, T-4.6 | std (build) + **Operator** (authorize stamp) |
| T-30 hollow-alias gate | IMPL+OP — interim P5(b) mirror on main; generated checker + operator closure remain | T-4 fact-bundle integrity | Dissolution |
| **T-29 C++ ABI** | **STILL a keystone-cluster feeder** (TASKS.md:64/:286/:1035 declares it a hard T-4-cpp-slice prereq) — core #3267 merged, residual #3277 OPEN; NOT "landed/removed" | T-4 cpp slice | #3277 worker (attribution open) |

**Keystone blast radius (widened — T-4-mgr finding):** Practice-10/#3240 does
not only gate T-4 ×5 forward — it also scopes the **rework obligation on the
already-LANDED pre-D2-reversal files** (spice #3168, llvm_ir #3171, verilog
T-4.9, ptx T-4.14, siblings).
Ratifying the **Practice-10 A1 invariant** is therefore *higher-leverage than
first stated*: it unblocks the forward spine **and** defines the backward-rework
set in one ruling. "Landed pre-A1" is not "done."

> **Naming convention (cursor review, folded):** herein **"#3240"** refers
> ONLY to the **CLOSED/SUPERSEDED rework tracker** (and its historical task
> IDs A1/B1/B2/C1) — it is *not* a live PR to act on. The live operator gate
> is the **Practice-10 A1-invariant ratification** (the verbatim blockquote,
> §7 #3a). Any "#3240" below is a historical/tracker reference.

**Related cluster — 2–3 distinct operator items, NOT "one decision"**
(still-hawk-102 review, folded — the earlier "one keystone" headline
overstated collapsibility):

- **(a) Ratify a verbatim invariant that ALREADY EXISTS — small, today.**
  (Corrected twice, now verified vs `origin/main`.) The keystone *fold* is
  **already on main** — `docs/modeling-discipline.md` §581–763 (7-row
  derived-operations registry, all findings, dispositions, §800
  always-BLOCKING line, §854 checklist). #3240 is a CLOSED tracker; the
  earlier "ratify #3240" and "not-yet-drafted PR / authorize a drafting
  effort" framings were **both wrong**. What the operator ratifies is the
  **verbatim invariant blockquote at `modeling-discipline.md` ~§594–600**
  — *"Do not hand-roll a derived operation. …"* — currently flagged
  *(proposed — pending operator ratification, #3240 A1)*. On ratification,
  **`still-hawk-102`** (ready now, no blockers) lands that block into
  `INVARIANTS.md` + `MODELING.md` and de-hedges `modeling-discipline.md`
  at exactly L61 / L591–592 / L718,746,750. **Size: SMALL** — 3 files,
  ~30–60 lines, mostly deleting hedge-parentheticals + mirroring one
  invariant block; additive, low-risk (INVARIANTS.md is load-bearing →
  mechanical/exact). It is **ratify-exact-text-today**, not a drafting
  effort. **It kicks off NOTHING larger** — verified end-to-end vs
  origin/main: #3241 (fold, 01:04Z) + #3242 (🟡-legend, 01:36Z) +
  **#3243 (the retroactive v4 dissolution audit sweep, C1) MERGED
  03:18Z** are ALL on main. C1 is **DONE**, was never A1-gated, and has
  **no live downstream dependency** to track (its C1 manager delivered
  #3243 and archived). Authorizing A1 ⇒ only the small placement PR. The
  one ongoing thing is the standing autonomous 🟡 burn-down lane (seeded
  by C1's inventory; also not A1-gated). Maximally low-stakes.
- **(b) A-vs-B = B is *implied by* (a), not identical to it.** Ratifying
  the keystone-fold ⇒ A-vs-B = B *because* D2-REV / machine-readable
  inhabitance is the primitive-case application of Practice-10. State and
  confirm the implication explicitly — do **not** let the narrower A-vs-B
  ruling ride (a) unscrutinized.
- **(c) T-25-core is a separate operator item — but an *authorize-to-build
  stamp*, NOT a design fork.** The shape is **operator-ratified**
  (TASKS.md §962+ / coercion-design.md Category 6: base type + fail-closed
  validation at a named constructor boundary); the earlier "genuine design
  fork / needs you" framing was **overstated** and is corrected in §7 #3b.
  It is a distinct one-line authorize (+ blessing the 2 Wave-2-prereq T#s),
  not collapsed by (a), and not a design decision.

So the long pole is **2–3 operator items**, not one line: ratify the
verbatim invariant text (a — exists today, small), explicitly confirm
A-vs-B=B (b), authorize T-25-core (c — ratified shape, a stamp not a
design call).

---

## 4. Wavefront — maximal parallel shape

> **Wave-ordering invariant (TOPOLOGICAL — dispatch-safe on its face):** a
> task sits in a wave **strictly later than every task it depends on** (per
> §2 Depends-on / TASKS.md). Within a wave, all tasks are **mutually
> independent** — true parallel, *no* intra-wave dependency, no "ordered
> within the wave" indirection. Any consumer in the same-or-earlier wave as
> a *fresh* input is a Facts-Flow-Forward violation and is re-sorted out
> (not papered over). Re-sorted per operator BLOCKING review: T-18, T-16,
> T-4.8, T-17, T-15 all moved to satisfy this invariant (see waves below).

- **Wave 0 (dispatch NOW — no keystone needed):**
  - *Full-scope `READY` (no T-9 dep):* T-19, T-20, T-21 (Lane B Priority-1) ·
    T-23 IFACE-frozen lens contract (Lane A) · T-26 (std-authoritative) ·
    T-31(b) mop-up (Dissolution).
  - *`READY*` — bounded pre-T-9 scope only (refine-to-real is Wave-3 at the
    named T-9 trigger):* T-22 (interpreter scaffold + IRT-3 shape) · T-12/T-13
    lens **witness/Acceptance authoring** (Lane A fan-out — *already in
    flight*).
  - **≈9 parallel work-fronts.** *(Not Wave-0: **T-29** — a TASKS.md-declared
    keystone-cluster T-4-cpp feeder (core #3267 merged, residual #3277 OPEN);
    T-4.10/T-4.12 already LANDED; pre-#3240 rework keystone-gated; **T-25-tail**
    — depends T-9, Wave-3 IMPL-optim, never Wave-0.)*
- **Wave 1 (the keystone-cluster ×4 land — TASKS.md:64/:286 exact set
  `{P1-KEYSTONE + T-30 + T-29 + T-25-core}`; T-29 is NOT dropped — core
  #3267 merged but residual #3277 still gates the T-4 cpp slice):**
  **T-4 ×5 languages** unblock on the **full ×4 set**. **T-4.5** (per the
  §2 T-4.5 row, deps `T-3, T-25-core`) and **T-4.6** (per the §2 T-4.6
  row, deps `T-25-core, T-26`)
  unblock on **T-25-core only** — their sole keystone-cluster feeder; NOT
  gated on P1-KEYSTONE / T-30 / T-29 — once their own §2 table deps are
  met, i.e. potentially **earlier** than the full ×4 (T-4 mgr). The §2
  per-task table is the dependency authority; this wave heading does not
  widen T-4.5/T-4.6's gate. **+ lens-pipeline-derivations T-#** (`match_arm_shape`
  / `closed_vocab_scan` / `concept_home`) **+ Layer-0 lens stage plug-in**
  (Fresh CP+Lens lane) → **Layer-0 hygiene CI HARD-GATE bites here** —
  table-stakes (L0.1–L0.15 read parse+resolve only, both LANDED, **no T-9
  dep**; #3313 §4/§9 "Layer 0 first, build-first, on in every profile").
  **Maximum-utility-early (operator directive): the CI hygiene gate is live
  by end-Wave-1 / early-Wave-2, well before the full L1.x suite.**
- **Wave 2 (T-4 lands):** T-9, T-4.7, T-4.13. *(T-4.8 removed → Wave 3:
  deps T-4.7 which is fresh in Wave 2. T-18 removed → Wave 4: meta-lens
  over T-12/T-13 which only become real in Wave 3.)*

> **L0/L1 ASYMMETRY (sunny-wolf-435, verified vs #3313 §4/§9/§10 — serves
> the operator "max utility early" directive):** Layer-0 hygiene
> (L0.1–L0.15) reads **parse+resolve only (both LANDED), NO T-9** → its CI
> hard-gate goes live **end-Wave-1** once just (2) lens-pipeline-derivations
> + the Layer-0 lens stage land. Layer-1 (L1.1–L1.12) needs **post-T-9
> closure + (1a) concept registries** → Wave-3. So the dispatch plan
> delivers the table-stakes CI hygiene gate *early* without compromising
> standards; the L1.x fan-out (batch-d, still held) is the later Wave-3 set.
>
> **Wave-2 dissolution-lens PREREQUISITES — cross-lane (sunny-wolf-435,
> #3313 author, folded; reinforces the batch-(d) hold — L1.x cannot fan
> out until these land + #3313 stabilizes + keystone ratifies):**
> **Classification (sunny-wolf-435, #3313 author — verified: NONE fold under
> T-25-core; T-25-core is type-shape/refinement substrate, these are lens-read
> metadata registries — different ratification surface):**
> - **(1a) Cross-cutting concept registries → NEW own T-#** (proposed
>   *"T-26-adjacent: lens-supporting concept registries"*, sibling to T-26,
>   std-authoritative, Lane A conduit-OK): `canonical_observations` (L1.8;
>   shared "domain-concept vs observation" fact), `CanonicalConcept`
>   (L1.12; cross-file concept identity). Land **before** Wave-2 consumers.
> - **(1b) Per-lens exemption registries → fold into each lens's own task,
>   NO separate T-#:** `WrongHomeExemption` (L1.8) / `VacuousArmExemption`
>   (L1.9) / `CanonicalCarrier.Exemption` (L1.10.b) — each consumed by
>   exactly one lens; carrier lands in that lens's `.dag`.
> - **(2) Three derived lens-stages → NEW shared T-#** (proposed
>   *"Wave-2-prereq lens-pipeline derivations"*): `match_arm_shape` (reused
>   L1.1/L1.9/L1.11/L0.7/L0.13), `closed_vocab_scan` (L1.7), `concept_home`
>   (L1.8); home `src/v4/lens/` beside complexity/cost/affected_set =
>   **Fresh Compiler-Pipeline+Lens lane**, NOT Dissolution-lane.
>
> **Two new owning-T#s to assign** (operator/substrate, P10-shape gap): (1a)
> concept registries + (2) lens-pipeline derivations. (1b) needs none.
> **Cross-lane edge (was missing in §5):** Compiler-Pipeline+Lens builds
> (1a)+(2) → Dissolution-lane Wave-2 *consumes*; (1b) is in-lens. Batch (d)
> held until (1a)+(2) land + #3313 stabilizes + A1 ratified.
- **Wave 3 (T-9):** T-10, T-12/T-13 refine to real, T-22 refine, **T-4.8**
  (deps T-4.7 from Wave 2). **+ Layer-1 (L1.1–L1.12) lens fan-out**
  (Dissolution lane; needs post-T-9 closure + (1a) concept registries;
  batch-d hold until #3313 stabilizes + A1).
- **Wave 4 (T-10):** T-11, T-14, T-24, **T-17** (deps T-12 *real*, Wave 3),
  **T-18** (meta-lens over T-12/T-13 *real*, Wave 3).
- **Wave 5 (T-11):** **T-16** (full-stack omni demo — deps T-11 Wave 4,
  T-4.8 Wave 3, T-4.5–4.7).
- **Wave 6 (T-16):** T-15 (self-host fixed-point — terminal anti-regression
  gate; deps ~ALL incl. T-16) → **v4-done.**

The wall-clock long pole = the keystone cluster latency + the serial
`T-4→T-9→T-10→T-11→T-16→T-15` spine. Everything else is parallel-fillable and
should not wait.

---

## 5. Proposed lane/manager dispatch — **FRESH lanes under `witty-cat-59`**

Operator directive 2026-05-18: **do not reuse the existing managers.** Each lane
below is a **new composite manager session** spun under `witty-cat-59`
(`dashboard-ops work-items create --shape composite "<title>"`), with **no
inherited subtree or held-state**. Spawned on the Wave-0 go. Two existing
sessions are *not* folded in — see exceptions.

| Fresh lane (new composite) | Owns | Wave-0 dispatch NOW | Gated (later waves) |
|---|---|---|---|
| **Fresh: Compiler-Pipeline (+Lens, gated)** | **Pipeline scope (active on Wave-0 go):** T-3 tail, T-6–T-8 CP-1b close, T-9, T-10, T-11, T-16, T-25-tail, T-28; **T-26 = std-authoritative, conduit-only**. **Lens scope (T-12/13/17/18/23) — GATED on the `fierce-cat-31` lens fan-out CLOSEOUT** (one lens owner at a time — see exception 2; no P2 parallel-authority drift) | Wave-0: T-3 tail / CP-1b / T-26 only | **Mirrors §4 topology (strict, codex BLOCKING):** W2 T-9 → W3 T-10 (+T-4.8, +T-12/13 refine) → W4 T-11 (+T-17, +T-18 — lens-scope, also post-closeout) → W5 **T-16** → W6 T-15 handoff. Lens scope (T-12/13/17/18/23) additionally post-`fierce-cat-31`-closeout. |
| **Fresh: Test/Bootstrap-Infra** | T-19, T-20, T-21, T-22, T-24, T-14, T-15, T-4.11, T-32 | **T-19, T-20, T-21** (full) · **T-22** (`READY*` scaffold scope) | §4-mirrored: T-24 W4 (post T-20/T-21), T-14 W4 (post T-19), **T-15 W6** (terminal, post T-16) |
| **Fresh: extdeps/T-4** | T-4 ×5, T-4.5–T-4.14 | **— (T-4.9/T-4.10/T-4.12/T-4.14 LANDED; T-4.13 modeled slice landed but LanguageModel-blocked; T-29 core #3267 merged; #3338 merged (language/extdeps smoke + doc hygiene); **residual #3277 OPEN** & still a TASKS.md-declared T-4-cpp feeder**; T-4 ×5 HELD on the **full TASKS.md:286 feeder set `{P1-KEYSTONE, T-30, T-29, T-25-core}`** — all four, never keystone-only (same gate as the §2 T-4 row and this row's gated cell))** | T-4 (post the **full TASKS.md:286 feeder set `{P1-KEYSTONE + T-30 + T-29 #3277 + T-25-core}`** — not "keystone + T-29" alone; T-25-core and T-30 are hard T-4 prerequisites too). **T-4.5–4.8 follow their own §2 table deps, NOT the full ×4** (T-4.5: `T-3,T-25-core`; T-4.6: `T-25-core,T-26`; T-4.7/T-4.8: post-T-4) — §2 is the sole dependency authority; this lane cell does not widen their gate. |
| **Fresh: Dissolution** | T-30, T-31, Wave-2 lenses, 🟡 burn-down | **T-31(b) mop-up; T-30 generated-checker** | Wave-2 lenses — *consumes* the §4 cross-lane prereqs (new std/ carriers + 3 derived lens-stages built by Compiler-Pipeline+Lens) — post A1-ratification |
| **Operator** | Keystone rulings | **see §7 decision sheet** | — |

**Exceptions (NOT migrated — improvising around these is forbidden):**
1. **`vivid-carp-207`** stays *solely* as the **#3280 CORE-freeze custodian**
   (#3308/#3309 + audit-trail pins frozen, no new work) until the operator
   A-vs-B ruling resolves and archives it. The Fresh extdeps/T-4 lane owns all
   *forward* T-4 work; it does **not** inherit the freeze.
2. **`fierce-cat-31`** keeps the **in-flight lens fan-out** (~6 active
   Acceptance-PR children) to **closeout only** — it archives when that fan-out
   completes; the Fresh Compiler-Pipeline+Lens lane owns all *new* work.
   Mid-flight migration is rejected as pure churn.

---

## 6. Review status (per reviewer)

1. **`fierce-cat-31`** — ✅ RATIFIED. CP-1b is on the **T-8→T-9** leg
   (parallel to T-4, off the keystone edge); T-26 = std-authoritative /
   Lane A execution-OK; lens rows match the running fan-out. Folded.
2. **`swift-ram-178`** — ✅ RATIFIED. Owns + can parallel T-19/20/21/22;
   IRT-1..4 binding in T-21/T-22; T-15 terminal. Correction folded: T-20
   materially advanced via merged #3213.
3. **`vivid-carp-207`** — ✅ RATIFIED. T-4 ×5 stays HELD on the **full
   keystone cluster** — the TASKS.md:286 feeder set
   `{P1-KEYSTONE, T-30, T-29, T-25-core}`, all four (never keystone-only).
   Folded: **T-4.10/T-4.12 already LANDED** (not Wave-0).
   **CORRECTION (codex REQUEST_CHANGES, TASKS.md authority):** T-29 was
   wrongly reclassified "LANDED/removed" — TASKS.md:64/:286/:1035 still
   declares it a hard T-4-cpp-slice feeder + #3277 OPEN; **T-29 restored to
   the keystone cluster.** New keystone-scope finding (pre-#3240
   backward-rework) folded into §3.
4. **`jolly-ibex-599`** — ✅ RATIFIED. T-30 interim P5(b) mirror on main;
   T-31 (a)rider/(b)mop-up split; Wave-2 post-#3240 coordination. Folded.
5. **Operator** — ⏳ OPEN: the §3 cluster — **ratify the verbatim invariant
   blockquote at `modeling-discipline.md` ~§594–600** (exists on main; #3240
   is a closed tracker, not it; `still-hawk-102` then lands+de-hedges —
   small), confirm **A-vs-B=B**, and **authorize T-25-core** (shape already
   ratified — TASKS.md §962+/Cat-6; a stamp, not a direction). That
   sequence collapses the long pole *and*
   unjams #3280 + #3313, **and** scopes the pre-#3240 backward-rework set.

> Nothing in Wave 0 waits on this PR's merge — it documents what's already
> dispatchable and the gated remainder. Merge = lane managers have ratified
> their rows and the operator has the keystone package.

---

## 7. Operator decision sheet (every blocking question)

**Root decision — the A-vs-B modeling ruling.** Under **D2-REV**
(operator-ratified 2026-05-17: *fact-bundle modeling supersedes alias-identity*),
how is each language's per-primitive scalar (e.g. Python `bool`/`int`/`float`)
modeled?

- **Option A — keep the cheap form:** `type PyBool = Bool` (bare alias) +
  `data py_bool_grounding: GroundingMap { spelling: "bool" }`. Fast, ~no rework.
  *But this is the shape your own #3280 merge message called "D2-reversal-wrong
  … expected it fixed," and it re-introduces the bare alias D2-REV exists to
  kill.*
- **Option B — fact-bundle:** eliminate the bare alias / require each primitive
  to carry **real proven-coincidence facts** grounded from its own spec;
  deduplicate to a `std/` carrier only on machine-readable proven coincidence.
  More work; it *is* the D2-REV / Practice-10 principle already ratified.
- **Recommendation: B.** A contradicts the reversal you already ratified and is
  the hollow-alias T-30 exists to gate.

Everything else **follows from or is independent of** that root call:

| # | Blocking question | Choice / tradeoff | Rec |
|---|---|---|---|
| 1 | **A-vs-B** (above) | A = cheap, contradicts D2-REV · B = principled, more rework | **B** |
| 2 | **#3280 disposition** (follows #1) | If **B**: #3308/#3309 stay held; rework obligation on `862bbde6e`, owner = **Fresh extdeps/T-4 lane**; archive `vivid-carp-207` freeze after. If **A**: unblock #3308/#3309, apply landed shape, archive freeze. **NO REVERT of `862bbde6e`** — the operator's manual merge stands; rework **rides forward** only; a downstream worker must not improvise a destructive revert (standing #3280 do-not-revert). | **B-path, forward-only** |
| 3a | **Ratify the verbatim invariant blockquote at `docs/modeling-discipline.md` ~§594–600** (*"Do not hand-roll a derived operation. …"* — already on main, flagged proposed/#3240-A1). The fold §581–763 is ALREADY on main; #3240 is a closed tracker (not it). On ratify → `still-hawk-102` (ready, no blockers) lands the block into INVARIANTS.md+MODELING.md + de-hedges L61/L591–592/L718,746,750 — **SMALL** (3 files ~30–60 ln, mechanical, low-risk). Ratify-exact-text-today, NOT a drafting effort. Kicks off NOTHING larger — verified end-to-end: #3241 (fold 01:04Z) + #3242 (legend 01:36Z) + **#3243 (C1 v4-audit sweep) MERGED 03:18Z** all on main; C1 is DONE, never A1-gated; no live downstream dependency. A1 ⇒ only the small placement PR. Maximally low-stakes. | **Ratify the text** |
| 3b | **T-25-core authorize-to-build** | NOT an open design fork (overstated earlier) — shape is **operator-ratified** (TASKS.md §962+: base type + fail-closed validation obligation, per coercion-design.md Category 6). Decision = authorize T-25-core to build now as a keystone-cluster T-4 feeder (status sibling of T-30/T-29). + bless the 2 new Wave-2-prereq T#s (names/owners). | **Authorize** |
| 4 | **Wave-0 go** | Go = ~6 full + ~3 bounded work-fronts start now (keystone-independent). Wait = nothing moves. | **Go** |
| 5 | **Fresh-lane exceptions** | Default: `vivid-carp-207` = sole #3280 freeze custodian; `fierce-cat-31` = lens-fan-out closeout only. Alt: hard-cut (churn/risk). | **Defaults** |
| 6 | **De-prose Python removal** | You directionally said "kill the de-prose py." Now = delete `strict_deprose_dag.py` + test + 2 CI steps (brief gap until lens enforcement lands) · Sequence = no gap, ratchet lingers. | **Delete now** |
| 7 | **#3313** (dissolution-lens design — **in active rework**: taxonomy shifted, L1.6 retired→L1.10 family {a TemplateHole, b CanonicalCarrier}, new A0/A1 umbrella, pre-Practice-10-ratification) | Post my review of the design-in-progress; advancement waits on **#3313 stabilizing AND the Practice-10 A1-invariant ratification** (the live gate — *not* the closed #3240 tracker). Wave-2 batch (d) (warm-koi-304/#3318) **held at track-not-finalize** so witnesses aren't authored against the moving taxonomy; batches a/b/c unaffected. | **Post + sequence; batch (d) hold** |
| 8 | **#3321 driver shape** — ✅ **RESOLVED (operator 2026-05-18): B2 substrate-native.** | `registry.dag` split out → own clean `.dag` PR (lands now, no gate). `tools/gunbc_prefix_lens_driver/` Rust crate (~467 LOC) **dropped entirely**; B1 also rejected — **no interim out-of-substrate enforcement shell, Python OR Rust** (same thesis ruling as the de-prose-py kill). Whole-corpus gate re-scoped to the **v2 filesystem-walk / corpus-enumeration substrate primitive** = the **T-21/T-24 corpus-drive capability** (built once, PREFIX gate is first consumer). **Operator-accepted consequence:** the CI lens gate lands when that primitive lands, not before. | **Done — B2** |

**Not "one line."** #3a **implies** #1=B (state+confirm the implication; not
an equivalence — #1 must not ride #3a unscrutinized). #2 falls out of #1
(forward-only, no revert). #3a is **ratify-exact-text-today** (the invariant
blockquote exists on main; still-hawk then does a small mechanical land +
de-hedge), NOT a drafting effort. #3b (T-25-core) is a separate
**authorize-to-build stamp** (shape operator-ratified — TASKS.md §962+ /
coercion-design.md Category 6; NOT a design fork — earlier framing
overstated). #4/#5/#6 are independent and low-risk; **#7 still needs you**;
**#8 is resolved (B2)**. Honest count: the long pole is **2–3 operator
items** — (a) ratify the verbatim invariant text, (b) confirm A-vs-B=B,
(c) authorize T-25-core (ratified shape — a stamp, not a design call) —
plus the low-risk independents.
