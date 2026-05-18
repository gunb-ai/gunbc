# R4 — Program Dispatch Plan to v4-done (T-1 … T-32)

> **Purpose.** A single reviewable artifact: the full remaining-work dependency
> chart + the proposed lane/manager dispatch mapping for maximum
> parallelization. **This is a discussion PR** — review/correct your lane's rows
> before anything fans out. Derived from `src/v4/TASKS.md` (1265 lines, verified
> 2026-05-18); statuses marked *derived* are first-articulation and explicitly
> open to lane-manager correction in review.
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
                 ┌─ P1-KEYSTONE (Practice-10 / #3240)  ─┐
                 ├─ T-25-core (refinement substrate)    ─┤
   keystone ─────┼─ T-30 (hollow-alias gate)            ─┼──▶ T-4 ──▶ T-9 ──▶ T-10 ──▶ T-11 ──▶ T-16 ──▶ T-15
   cluster       └─ T-29 (C++ ABI; cpp slice only)      ─┘  (5 langs)  infer    emit    per-tgt  omni    self-host
                                                                                                          fixed-point
                                                                                                          (anti-regress)
   T-1..T-8  =  LANDED  (front-end in CP-1b reconciliation tail; not the bottleneck)
```

Everything else parallel-fills around that spine.

> **Review status (2026-05-18):** **4/4 lanes ratified** — Lane A
> (`fierce-cat-31`), Lane B (`swift-ram-178`), Dissolution (`jolly-ibex-599`),
> T-4-mgr (`vivid-carp-207`) — all corrections folded below. T-4-mgr CONFIRMED
> the T-4 keystone HOLD and supplied **material staleness corrections** (T-29 /
> T-4.10 / T-4.12 already **LANDED**, not Wave-0; see Folded clarifications) plus
> a new keystone-scope finding (early-canonical files' rework obligation).
>
> **Lane-A clarification (folded):** CP-1b close sits on the **T-8→T-9** leg, a
> T-9 prerequisite running **parallel wall-clock to the T-4 keystone wait** —
> *not* on the T-4→T-9 edge and *not* keystone-gated. CP-1b therefore progresses
> while T-4 waits on ratification (strengthens, not weakens, the parallel story).

---

## 2. Per-task dependency + dispatch table

Legend — **Blocked-on:** `LANDED` · `READY` (dispatch now) · `IMPL` (interface
exists, parallelizable) · `IFACE` (needs a contract frozen first) · `DESIGN` ·
`OP` (operator ruling) · `CP1` (needs v4 front-end output).

| Task | Scope | Status (derived) | Depends on | Blocked-on | Proposed lane |
|---|---|---|---|---|---|
| T-1 | std/node.dag root | LANDED | — | LANDED | — |
| T-2 | std/algebra.dag | LANDED | T-1 | LANDED | — |
| T-3 | std/* scalar+support stack | LANDED (datetime Wave-A2 tail) | T-1,T-2 | IMPL (tail) | Lane A / std |
| T-6 | compiler/01_tokenize | LANDED | T-3 | LANDED (CP-1b tail) | Lane A |
| T-7 | compiler/02_parse | LANDED | T-6 | LANDED (CP-1b tail) | Lane A |
| T-8 | 03_normalize + 03_resolve (+T-28 bundled) | LANDED (seam scaffold until CP-1b closes) | T-7 | IMPL (CP-1b close) | Lane A |
| **T-4** | extdeps/languages ×5 fact-bundles | WIP, **HELD** | T-3, **P1-KEYSTONE (= Practice-10/#3240 *ratification* — NOT the already-merged numeric #3226)**, T-25-core, T-30 | **OP + DESIGN** | T-4 mgr |
| T-4.5 | extdeps/process + file_system | SCAFFOLD | T-3, T-25-core | IMPL | T-4 mgr |
| T-4.6 | extdeps/formats ×7 | SCAFFOLD | T-25-core, T-26 | IMPL | T-4 mgr |
| T-4.7 | frameworks/react | SCAFFOLD | T-4 (ts) | CP1 (LanguageModel) | T-4 mgr |
| T-4.8 | coordination | SCAFFOLD | T-4, T-4.7 | IFACE | T-4 mgr |
| T-4.9 | languages/verilog | NOT STARTED | T-1,T-2 | OP (IN-B probe) | T-4 mgr |
| T-4.10 | formats/spice | **LANDED** (#3168 merged) — *pre-D2-reversal canonical path; see rework-obligation note* | T-1 | LANDED | T-4 mgr |
| T-4.11 | claim/english_ingest | SCAFFOLD | T-3/verification.dag | IFACE (AssertKind) | Lane B |
| T-4.12 | languages/llvm_ir | **LANDED** (#3171 merged; +#3229 de-prose +#3300 cost-move on main) — *pre-D2-reversal canonical path; see rework-obligation note* | T-1,T-2 | LANDED | T-4 mgr |
| T-4.13 | languages/machine_code | NOT STARTED | T-3, T-4 LanguageModel | CP1 | T-4 mgr |
| T-4.14 | languages/ptx | NOT STARTED | T-1,T-2 | OP (IN-B probe) | T-4 mgr |
| **T-9** | compiler/04_infer | SCAFFOLD | T-8/CP-1b-close (∥) + T-4 | IMPL (T-4 keystone; CP-1b-close parallel, not keystone-gated) | Lane A |
| **T-10** | 05_emit + 00_compile | SCAFFOLD | T-9, T-4 | IMPL+IFACE | Lane A |
| **T-11** | emit per-target ×5 | NOT STARTED | T-10 | IMPL | Lane A |
| T-12 | lens/complexity + cost | SCAFFOLD (fan-out **in flight**) | T-9 (refine) | **READY** (witness-first) | Lane A (lens) |
| T-13 | lens/parallelism,effect,ownership,idempotency | SCAFFOLD | T-9 (refine) | **READY** | Lane A (lens) |
| T-14 | TestClaim corpus + fixtures | SCAFFOLD | T-19 | IMPL | Lane B |
| **T-15** | bin/main + self-host fixed-point gate | NOT STARTED | ~ALL | IMPL (terminal) | Lane B |
| T-16 | full-stack omni-emission demo | NOT STARTED | T-4,T-10,T-11,T-4.5–4.8 | IMPL | Lane A |
| T-17 | lens/synthesis + std/report | SCAFFOLD | T-12, T-9 | DESIGN+IMPL | Lane A (lens) |
| T-18 | lens/coverage meta-lens | SCAFFOLD | T-12,T-13 | IMPL | Lane A (lens) |
| T-19 | lens/testgen | SCAFFOLD | T-1,T-2,T-3 | **READY** | Lane B |
| T-20 | workflow/bootstrap AS DATA | Materially advanced — #3213 landed bootstrap+CI-as-data on main; remaining = fill tail | T-1 | **READY** | Lane B |
| T-21 | lens/affected_set (IRT-1..4 held) | SCAFFOLD | T-1,T-2,T-3 | **READY** (honor IRT) | Lane B |
| T-22 | compiler/05_eval interpreter (PRIMARY exec) | SCAFFOLD | T-9 (refine) | **READY** | Lane B |
| T-23 | lens/application | **IFACE FROZEN** | lens fwk | IMPL (in flight) | Lane A (lens) |
| T-24 | workflow/ci AS DATA | SCAFFOLD (T-20/T-24-adjacent slice landed via #3213) | T-20, T-21 | IMPL | Lane B |
| T-25-core | refinement base + fail-closed validate | SCHEDULED | — | **DESIGN (OP)** | std / OP |
| T-25-tail | refinement prover (erase) | SCHEDULED | T-9 | IMPL (optim) | Lane A |
| T-26 | std/ boundary carriers (URL/HttpMethod) | SCHEDULED | T-3 | **READY** | std/ = canonical home; Lane A may run 1st PR as conduit |
| T-28 | std/ module-graph | SCHEDULED (bundled→T-8) | T-3 | IMPL (in T-8) | Lane A |
| T-29 | extdeps C++ ABI feeder | **LANDED** — core #3267 merged (cpp_abi.dag on main); residual #3277 (cpp GAP) OPEN/in-flight | T-3/machine | LANDED (residual #3277 in-flight) | #3277 worker — **NOT in T-4-mgr subtree; attribution open (reparent or correct)** |
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
- **T-4-mgr staleness correction (verified vs main):** T-29 core (#3267),
  T-4.10 (#3168), T-4.12 (#3171) are **already LANDED** — the original
  "Wave-0 dispatch NOW = T-29/T-4.10/T-4.12" column for T-4-mgr is **empty in
  reality**. Wave-0 parallel count drops **≈14 → ≈11**.
- **NEW keystone-scope finding (decision-relevant):** the early-canonical-path
  language/format files modelled **pre-D2-reversal** (spice #3168, llvm_ir
  #3171, and any sibling pre-#3240 models) likely carry a **fact-bundle REWORK
  obligation that is itself Practice-10/#3240-keystone-gated** — the *same*
  class as T-4 ×5 and #3280 A-vs-B. "LANDED" ≠ "done": ratifying #3240 also
  scopes their rework. This **widens the keystone's blast radius** (see §3).
- **Coord flag:** T-29 residual #3277's worker (quick-bat-761) is **not in the
  T-4-mgr subtree** (absent from the graph). §2 attribution corrected to
  "open"; reparent/attribution routes via the owning manager, not assumed.

---

## 3. Keystone cluster — ranked by fan-out (the long-pole collapsers)

| Keystone | Kind | Unblocks | Owner |
|---|---|---|---|
| **P1-KEYSTONE / Practice-10 / #3240** | **OPERATOR ratification** | T-4 → T-9 → T-10 → T-11 → T-16; **also #3313 Wave-2 lenses; also the principled basis for #3280 A-vs-B** | **Operator** |
| **T-25-core** | **DESIGN direction (operator review)** | T-4 refinement-bearers, T-4.5, T-4.6 | **Operator** + std |
| T-30 hollow-alias gate | IMPL+OP — interim P5(b) mirror on main; generated checker + operator closure remain | T-4 fact-bundle integrity | Dissolution |
| ~~T-29 C++ ABI~~ | **LANDED (#3267)** — no longer a keystone; residual #3277 in-flight | (was: T-4 cpp slice) | #3277 worker |

**Keystone blast radius (widened — T-4-mgr finding):** Practice-10/#3240 does
not only gate T-4 ×5 forward — it also scopes the **rework obligation on the
already-LANDED pre-D2-reversal files** (spice #3168, llvm_ir #3171, siblings).
Ratifying #3240 is therefore *higher-leverage than first stated*: it unblocks
the forward spine **and** defines the backward-rework set in one ruling. "Landed
pre-#3240" is not "done."

**One decision, three threads:** ratifying Practice-10/#3240 simultaneously
unblocks the critical-path spine (via T-4), the #3313 dissolution-lens Wave 2,
and gives `vivid-carp-207` the principled basis to resolve **#3280 A-vs-B**
(fact-bundle-vs-bare-alias *is* the D2/Practice-10 question). These are not four
problems; they are one keystone.

---

## 4. Wavefront — maximal parallel shape

- **Wave 0 (dispatch NOW — no keystone needed):** T-19, T-20, T-21, T-22
  (Lane B Priority-1) · T-12, T-13, T-23 lens fan-out (Lane A — *already in
  flight*) · T-26, T-25-tail (std) · T-31(b) mop-up (Dissolution). **≈11
  parallel.** *(T-29/T-4.10/T-4.12 removed — already LANDED, not fresh
  dispatch; their pre-#3240 rework is keystone-gated, not Wave-0.)*
- **Wave 1 (P1-KEYSTONE + T-25-core + T-30 land):** T-4 ×5 languages, T-4.5,
  T-4.6 unblock (T-4 mgr).
- **Wave 2 (T-4 lands):** T-9, T-4.7, T-4.8, T-4.13, T-18.
- **Wave 3 (T-9):** T-10, T-12/T-13 refine to real, T-17, T-22 refine.
- **Wave 4 (T-10):** T-11, T-16, T-14, T-24.
- **Wave 5 (T-16):** T-15 (self-host fixed-point — the terminal anti-regression
  gate) → **v4-done.**

The wall-clock long pole = the keystone cluster latency + the serial
`T-4→T-9→T-10→T-11→T-16→T-15` spine. Everything else is parallel-fillable and
should not wait.

---

## 5. Proposed lane/manager dispatch (the thing to discuss)

| Lane / manager | Owns | Wave-0 dispatch NOW | Gated (later waves) |
|---|---|---|---|
| **Lane A** `fierce-cat-31` (compiler pipeline + lens program) | T-3 tail, T-6–T-8 CP-1b close, T-9, T-10, T-11, T-16, lens T-12/13/17/18/23, T-25-tail, T-26, T-28 | lens fan-out (running), CP-1b close, T-26 | T-9 (post-T-4), T-10, T-11, T-16 |
| **Lane B** `swift-ram-178` (test/bootstrap infra) | T-19, T-20, T-21, T-22, T-24, T-14, T-15, T-4.11, T-32 | **T-19, T-20, T-21, T-22** (Priority-1) | T-24 (post-20/21), T-14, T-15 (terminal) |
| **T-4 mgr** `vivid-carp-207` | T-4 ×5, T-4.5–T-4.14, T-29 | T-29, T-4.10, T-4.12 | T-4 (post-keystone), T-4.5–4.8 |
| **Dissolution** `jolly-ibex-599` | T-30, T-31, Wave-2 lenses, 🟡 burn-down | T-31, T-30 bootstrap mirror | Wave-2 lenses (post #3240) |
| **Operator** | Keystone rulings | **Practice-10/#3240 ratification; T-25-core direction; IN-B probes (T-4.9/T-4.14); #3280 A-vs-B** | — |

---

## 6. Review asks (per reviewer)

1. **`fierce-cat-31`** — confirm/correct Lane A rows; is CP-1b close on the
   T-9 critical edge, or independent? Is T-26 better under std or Lane A?
2. **`swift-ram-178`** — confirm Lane B owns T-19/20/21/22 and can take all
   four Wave-0 in parallel; IRT-1..4 honored in T-21/T-22 scope; T-15 terminal
   ownership.
3. **`vivid-carp-207`** — confirm T-4 stays HELD on the keystone cluster (not
   dispatchable pre-ratification); T-29/T-4.10/T-4.12 are genuinely READY now.
4. **`jolly-ibex-599`** — confirm T-30 mirror status + T-31 independence; Wave-2
   lens coordination.
5. **Operator** — the §3 keystone package: ratify Practice-10/#3240 + give a
   T-25-core direction. That one pass collapses the long pole *and* unjams
   #3280 + #3313.

> Nothing in Wave 0 waits on this PR's merge — it documents what's already
> dispatchable and the gated remainder. Merge = lane managers have ratified
> their rows and the operator has the keystone package.
