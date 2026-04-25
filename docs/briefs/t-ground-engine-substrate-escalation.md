# T-Ground-Engine — Substrate Escalation to Director

**Status**: ROUTED — Director chose **Route 1** on 2026-04-25 (small loader-close, ad-hoc Director dispatch). Manager originally recommended Route 3; Director overruled with substantive reasoning (see "Decision" section below). Engine-Phase-1 implementation re-dispatches in sharpened-(b) form once the loader-close PR lands.

**Source evidence**: [`docs/briefs/t-ground-engine-substrate-audit.md`](t-ground-engine-substrate-audit.md) (PR #768, merged commit `4afc0d794`, 2026-04-25). Citations spot-verified against current main.

---

## Decision (recorded post-routing)

**Route 1 chosen.** Director's reasoning (preserved verbatim from cross-session brief):

1. **Doesn't fit cleanly in Zero-Floor's program.** PB-1 scope = migrating four *existing* bootstrap fixture sets (`std_fixtures` / `STAGED_FILES` / `V3_SPECS` / `COMPILER_FILES`) to generated constructors. Adding a fifth set (`dsl/extdeps/languages/*`) is upstream of PB-1 — it requires deciding the bootstrap shape includes it before PB-1's pattern applies. PB-Bootstrap-Process scope (`bootstrap.dag` declares workflow as data) is a discrete decision that can land before that lane exists. PB-Substrate generates `dag.rs` from `substrate.dag`; loader is adjacent but not the same surface.
2. **The "no target-language realizations in bootstrap" comment at `bootstrap.rs:16-19` is a stale design assertion.** It was set when Engine didn't exist. Engine needs the realizations. Revisiting that decision = small dispatch, not program-scope work.
3. **Faster unblock for Engine.** Route 1 (small loader-close) unblocks sharpened (b) Engine within a few days. Routing through Pure Bootstrap to Zero would have required PB-1 + PB-Substrate + emission-pipeline maturity all to land first.

**Cross-program coordination handled by Director.** Heads-up to Pure Bootstrap to Zero Manager (`stern-swift-335`) flagged: PB-1's eventual constructor-emission pattern will absorb the new fifth fixture set when PB-1-b/c/d/e ship; no coordination conflict on `dag.rs` shape because loader-close doesn't touch substrate types.

**What the manager recommendation got wrong.** Route 3's appeal was avoiding a discrete sub-lane that overlaps with PB-1. Director's correction: the overlap is illusory — PB-1 migrates *existing* fixture sets, doesn't decide *which* sets exist. Loader-close is a shape decision upstream of PB-1's pattern. Manager will internalize this distinction for future cross-manager flagging.

The routes section below is preserved as decision history.

---

## TL;DR

T-Ground-Engine-Phase-1 cannot dispatch as scoped. The Phase 0 substrate audit found that **both** the `.dag`-defined walker (option a) and the pure sibling-crate `.dag`-consumer (option b) block on the same substrate gap: `dsl/extdeps/languages/*` is not loaded into the bootstrap Dag and there is no public accessor for downstream code to read its declarations as parsed values. The pilot's Rust-constant mirror existed precisely because of this gap.

**Live state (post-routing)**: Director chose **Route 1** on 2026-04-25 — small loader-close via ad-hoc Director dispatch. Engine re-dispatches in **sharpened-(b)** form (sibling crate consuming `.dag` declarations via the loader's public accessor) once the loader-close PR merges. Manager's original Route 3 recommendation (route through Pure Bootstrap to Zero) was overruled with substantive reasoning preserved in the "Decision" section above.

---

## What's blocked

| Lane | Block reason |
|---|---|
| T-Ground-Engine-Phase-1 (implementation) | Substrate Gap 1 (extdeps loader). Brief's "no Rust-constant mirror" requirement is unsatisfiable today. |
| T-Ground-Tests | Blocks on Engine + at least one full-reference. Doubly blocked. |
| T-Ground-Dissolve | Blocks on everything. |
| T-Ground-Rust / -Python / -Go | Block on DB-11 + cardinality-substrate (independent of this escalation, but flagged: same extdeps-loader gap will block their `primitives.dag` consumption when those briefs eventually dispatch). |

The Engine-Phase-1 brief itself is not invalidated — its design contracts (no mirroring, state-space discipline, fail-closed by construction, SG-0 untouched) remain load-bearing. Only the **dispatch shape** changes based on the routing decision.

---

## Substrate gaps the audit identified

### Gap 1 — extdeps not loaded into bootstrap Dag

`src/v3/compiler/src/bootstrap.rs:131-151` (`load_runtime_bootstrap_authorities`) chains `STAGED_FILES`, `V3_SPECS`, `COMPILER_FILES` only. `dsl/extdeps/languages/rust/primitives.dag` is in none of these sets. Header at `bootstrap.rs:16-19` is explicit:

> Production bootstrap does not inject target-language realizations. Realization facts for emitted languages live in `dsl/extdeps/languages/*` per the thesis; compiler code does not manufacture those.

A `.dag`-side walker cannot reach `rust_pilot_primitives` via name resolution. A sibling-crate walker that wants to read parsed declarations symbolically faces the same gap: no public `Dag::load_extdeps_language(…)` entry point exists.

### Gap 2 — surface/emission features for a `.dag` walker not yet shipped

Independent of Gap 1, option (a) requires `.dag` capabilities not yet emitted end-to-end:

- `src/v3/std/list.dag:6-15`: *"the current compiler still lacks full structural recursion + list-body emission support."*
- `src/v3/std/list.dag:57-64`: *"generic `data` items are not yet part of the surface grammar."*
- Heterogeneous-variant pattern matching (`IntegerAlgebra | NonIntegerAlgebra` discrimination across a `List<RustPrimitive>` walk) is not demonstrated end-to-end by any existing `.dag` program.

Each gap independently rules out option (a). Gap 1 alone rules out pure (b).

---

## Three routes (audit's two + sharpened middle path)

> **Decision-history section.** The labels below ("recommended", "fallback", route ordering) reflect the manager's pre-routing analysis. **Director chose Route 1** — see the "Decision" section at the top of this doc for the live state. The route descriptions are preserved as decision history; their present-tense framing should be read as past-tense in light of the routing.

### Route 1 (audit's first option) — close Gap 1 only; sibling-crate Engine

Substrate work: bootstrap loads `dsl/extdeps/languages/rust/primitives.dag` plus a stable accessor returning `Declaration`/`Node` for `rust_pilot_primitives`. Engine becomes a sibling crate that walks the parsed `Declaration` structurally.

- **Pros**: smallest unblocking step. Eliminates pilot's mirror. Engine ships in this phase. Doesn't require Gap 2 closure.
- **Cons**: Engine is hand-Rust (sibling crate) interpreting `.dag` AST — transitional, not thesis-terminal. Audit characterizes this as "interpret `.dag` AST in Rust" which is the layer Engine should eventually eliminate. Honest read: this is a **bridge**, not the destination.
- **Dissolution trigger**: when Gap 2 closes naturally (PB-1 / PB-Bootstrap-Process ships list-body emission + heterogeneous-variant pattern matching), sibling-crate Engine collapses into a `.dag` walker (Engine-Phase-2 or T-Ground-Dissolve).

### Route 2 (audit's second option) — pursue option (a) directly

Bundle Gap 1 close with `.dag` walker authoring; schedule Gap 2 closure as Engine-Phase-1 prereqs.

- **Pros**: project-thesis-aligned in one motion. No transitional bridge.
- **Cons**: scope inflates substantially. Engine-Phase-1 becomes an XL+ program by absorbing list-body emission + sum-variant pattern matching scope. Probably parks Engine for quarters.

### Route 3 (manager-sharpened, **originally recommended — overruled**) — route Gap 1 close through Pure Bootstrap to Zero program

Same destination as Route 1, but the substrate work doesn't dispatch as a discrete sub-lane — it gets sequenced inside [Pure Bootstrap to Zero Manager](pure-bootstrap-zero-manager.md)'s scope.

**Why this fits**:
- PB-1 (data-driven bootstrap loader) already owns `bootstrap.rs` evolution. Adding extdeps-load to the data-driven loader is a natural sub-lane (PB-1-a or new sub-lane).
- PB-Bootstrap-Process declares the bootstrap workflow as data; whether to load extdeps is itself a workflow decision that program already manages.
- Avoids creating a discrete "extdeps loader" Director-routed lane that overlaps with PB-1's scope.
- The existing PB-1 sub-lane breakdown can absorb the ask without scope-inflation if sequenced correctly.

**Concrete ask** (for Director coordination with Pure Bootstrap to Zero Manager):
- Add to PB-1's sub-lane breakdown (or call out as PB-Bootstrap-Process completion criterion): "Bootstrap loader can load `dsl/extdeps/languages/*/primitives.dag` (or equivalent target-language realization files) into the production Dag, with a stable public accessor returning parsed `Declaration` for downstream consumers."
- Sequence this sub-lane **early** in PB-1 (before full PB-1 lands), so Engine-Phase-1 unblocks without waiting for the whole XXL program.
- Once it lands, Engine-Phase-1 re-dispatches in (b.i) form: sibling crate walks `Declaration` structurally; no Rust-constant mirror.

**Pros**: leverages an existing parallel-program manager's authority over `bootstrap.rs`. Avoids Director micromanagement of an isolated sub-lane. Routing is a manager-to-manager handoff that Director coordinates rather than schedules.

**Cons**: depends on Pure Bootstrap to Zero Manager accepting the ask within their program's scope. If they reject (e.g., scope creep argument), falls back to Route 1's discrete sub-lane.

### Route 4 (passive) — park Grounding program until substrate matures naturally

No substrate ask. Wait for PB-1 to land naturally, including extdeps-load if it happens. If PB-1's scope doesn't naturally include extdeps-load, Engine remains parked indefinitely.

- **Pros**: zero director cost.
- **Cons**: program drifts. Pilot's empirical findings stale before they inform full-reference scope. Stratum-B finding (Lesson 1 of pilot receipt) starts to lose currency.
- **Recommendation**: **only** if Route 3 is rejected and Route 1 is also rejected. Default-to-passive is the worst outcome for thesis claim Tier 1.

---

## What unblocks on each routing decision

> The labels in the table below ("recommended", "fallback") were the manager's pre-routing analysis. Director chose **Route 1**; ignore the labels for live state.

| Route | Engine re-dispatches | Full-reference re-dispatches | Tests/Dissolve dispatch |
|---|---|---|---|
| Route 3 (originally recommended) | After PB-1 sub-lane closes — likely weeks if sequenced early | After DB-11 + cardinality close (independent) | After Engine + at least one full-reference |
| **Route 1 (Director chose)** | After discrete substrate sub-lane (loader-close PR) closes — Director-dispatched ad-hoc | After DB-11 + cardinality close (independent) | After Engine + at least one full-reference |
| Route 2 (originally analyzed) | After XL+ Engine-Phase-1 ships — likely quarters | After DB-11 + cardinality close (independent) | After Engine + at least one full-reference |
| Route 4 (originally analyzed) | After PB-1 ships full scope — if scope naturally includes extdeps-load | After DB-11 + cardinality close (independent) | After Engine + at least one full-reference |

Note that Routes 1, 3, and 4 converge on the same Engine shape (sibling crate, structural Declaration walker, mirroring eliminated). They differ only in **how** the substrate ask is routed and **when** the dependency closes.

---

## What stays parked regardless

- **T-Ground-Rust / -Python / -Go full-reference dispatch** — independent gates (DB-11 + cardinality-substrate). Will hit the same extdeps-loader gap when those briefs dispatch; pre-emptively flagging here so the substrate close benefits all four lanes simultaneously.
- **T-Ground-Tests** — blocks on Engine + full-reference.
- **T-Ground-Dissolve** — blocks on everything.

---

## Decision Director needs to make

~~Original asks (resolved 2026-04-25 — see "Decision" section at top):~~

1. ~~**Routing** — Route 1, 2, 3, or 4? (Manager recommends Route 3.)~~ → **Route 1 chosen.**
2. ~~**If Route 3** — coordinate with Pure Bootstrap to Zero Manager.~~ → **N/A (Route 3 not chosen).**
3. ~~**If Route 1** — schedule a discrete substrate sub-lane. Manager defers to Director on which substrate program owns it.~~ → **Director ad-hoc dispatch; loader-close worker authored under Director scope. Manager supplies Engine consumer requirements (see manager-side input below).**
4. ~~**If Route 2 or 4** — manager surfaces the parking decision and updates ROADMAP / working state accordingly.~~ → **N/A.**

### Manager-side input for the loader-close brief

Engine-Phase-1's sharpened-(b) form requires the loader close to enable a **specific consumer interface**. Loader brief author should ensure these are satisfied so Engine can re-dispatch directly without a follow-up amendment:

1. **Bootstrap loads `dsl/extdeps/languages/rust/primitives.dag`** into the production Dag. (For initial close, Rust target only is sufficient — Python / Go primitives.dag files don't exist yet.)
2. **Public accessor returns `rust_pilot_primitives` as parsed `Declaration`/`Node`** to a downstream Rust caller, walkable structurally without re-implementing the parser. Audit characterizes this as the (b.i) shape — sibling-crate Engine reads `Declaration` and extracts `(target_name, algebra-variant, carrier-variant, is_copy[, overflow])` per element.
3. **Stable enough shape that Engine doesn't have to reach into private `Dag` internals.** If the public accessor returns something that requires Engine to depend on parser-internal types, the loader close hasn't fully unblocked Engine.
4. **Revisit `bootstrap.rs:16-19` comment.** The "Production bootstrap does not inject target-language realizations" assertion is the stale design Director called out. Loader-close PR should update or delete this comment in the same change (avoid leaving contradictory documentation).
5. **SG-0 ratchet handling.** Loader-close edits live in `src/v3/compiler/src/bootstrap.rs` — ratcheted hand-Rust territory. Either (a) expansion is acceptable as a Director-sanctioned bump (loader-close is small and serves a load-bearing thesis claim), or (b) loader-close lands in a transitional shape that PB-1 absorbs cleanly. Loader brief author should pick and document.

These are consumer-side requirements, not implementation prescriptions. Manager defers to substrate-side judgment on internal shape.

---

## What manager does in parallel (no Director input needed)

- Working state in `grounding-manager.md` updated: Pilot ✅, Engine-Phase-1 audit ✅ + parked pending loader-close PR, Route 1 decision recorded.
- Pilot receipt landed at [`grounding-pilot-receipt.md`](grounding-pilot-receipt.md).
- ~~Cross-manager notification queued~~: cancelled. Director chose Route 1 over Route 3; Director handles Pure Bootstrap to Zero Manager heads-up directly per cross-program coordination. No manager-side notification needed.
- Engine re-dispatch readiness: when loader-close PR is ready-for-review or merged, manager authors the sharpened-(b) Engine-Phase-1 re-dispatch brief against the actual public-accessor shape and dispatches a worker.

---

## Lineage

- Parent program: ROADMAP.md §"Post-R1 Program — Grounding Completeness" → T-Ground-Engine.
- Source evidence: [`t-ground-engine-substrate-audit.md`](t-ground-engine-substrate-audit.md) (worker-authored Phase 0 audit, PR #768).
- Pilot empirical record: [PR #765](https://github.com/gunb-ai/gunbc/pull/765), [`grounding-pilot-receipt.md`](grounding-pilot-receipt.md).
- Adjacent program: [`pure-bootstrap-zero-manager.md`](pure-bootstrap-zero-manager.md).
- Engine brief: [`t-ground-engine-phase-1.md`](t-ground-engine-phase-1.md) (parked; re-dispatch shape determined by routing decision).
