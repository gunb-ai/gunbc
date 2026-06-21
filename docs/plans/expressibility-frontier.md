# The expressibility frontier — what a modeling discipline can gate, and what stays review

> Methodology doc. Generalizes DESIGN §5's decidability trichotomy (wall / ratchet / undecidable) from
> *one class* into a **frame for any modeling discipline**: locate each instance on a frontier between
> "expressible → gate by construction" and "known-inexpressible → permanent review." Two disciplines
> instantiate it today — **anemic modeling** and **algorithmic complexity** — and they share a shape.
> DESIGN refs: §1 (displaced cost), §2 (decomposition leaf-side), §5 (fail-closed; "never" trap;
> construction over validation), §6 (lenses as residue; the purity trap), §7 (recursion).

## 1. The pattern

A *modeling discipline* is a property we want every program to have — "measures aren't unwrapped to do
arithmetic," "no super-linear hidden cost," "no nicknamed concept." For each, the instinct is "gate it
so review doesn't have to." But you cannot gate a discipline uniformly: its instances spread across a
**range**, from ones you can make *unwritable* to ones that are *provably not mechanically decidable at
all*. The job, before building any enforcement, is to **locate each instance on that range** — because
mispricing the location is itself the failure mode (§4).

This is not new machinery; it is DESIGN §5's trichotomy turned from a property of *one* class into a
*method* applied to *every* discipline.

## 2. The frontier — three regions by decidability of membership

| Region | Membership | Enforcement | Can the fix be *presented*? |
| --- | --- | --- | --- |
| **① Wall** | decidable **and** a single authority makes the bad state unwritable | construction (the bad state cannot be authored) | n/a — it can't be written |
| **② Lens-residue** | decidable, but not yet unwritable (a seam you can't fence) | a pure reader over the `Node` tree (§6 residue) | **sometimes** — if the correct form is *determined*, not *searched* |
| **③ Inexpressible** | **undecidable** — needs domain knowledge or hits Rice | honest, permanent **review** (no complete gate, ever) | no — a constructive alternative is impossible |

Two laws govern the frontier:

- **Decidability is the boundary between ① ② and ③.** Region ③ is permanent: by Rice / by missing
  domain knowledge, no wall and no *complete* lens exists, ever. Pretending otherwise is the §5 **"never"
  trap** — a ratchet masquerading as a wall.
- **Presentability of the fix is a property of the region, not the discipline.** A lens can *present the
  alternative* (not just flag) exactly when the corrected form is **determined** — the RHS of an equation
  you already proved — rather than **searched**. That is a ② phenomenon; it never reaches into ③.
  Within ②, presentability has two flavors: a **single determined rewrite** (one safe RHS — `a+b ⇒
  measure_add`), or a **surfaced choice** (≥2 determined-but-safety-distinct RHSs, where picking is a
  §5 decision the anemic form *hid*). The lens presents the set and forces intent; it does not auto-pick.
  Division is the canonical case — `byte_size(count ÷ n)` buried whether to **ceil** (a per-shard
  *demand*: under-estimate ⇒ over-fit ⇒ OOM) or **floor** (a fit *count*: over-estimate ⇒ OOM): opposite
  directions, *both* fail-closed. So "present alternatives" is literal — the round-trip hid a safety
  decision, not merely a redundancy.

## 3. The two instances, side by side

This is the pattern the frontier names — the same three regions, in two unrelated disciplines:

| Region | **Anemic modeling** | **Algorithmic complexity** |
| --- | --- | --- |
| **① Wall** | *round-trip through representation* — `wrap(op_M(unwrap a, unwrap b))` is provably `op_T(a,b)` (homomorphism law); provide the lifted op + fence the projector ⇒ the unwrap-form is unwritable. Dimensional safety: the `Measure<Q,S>` phantom types already wall it. | *budget-dominance* — a cost-fold verdict that a step's cost is dominated by its declared budget is a decidable hard-gate (the complexity lens *can* gate this). |
| **② Lens-residue** | the round-trip at the **grounding seam** you can't fence (host `Int` → `ByteSize`); a pure reader flags a constructor whose magnitude derives from a projector — **and presents the exact `op_T`** (the op→op_T table is finite). Lifted algebra (`+`,`max`) is one determined rewrite; the two divisions (`Measure÷scalar`, `Measure÷Measure`) **surface a ceil/floor choice** the round-trip hid (demand ceils, width floors) | the cost-budget lens over the corpus where construction isn't available yet — flags, but the *cheaper algorithm* is **searched**, so it cannot present it |
| **③ Inexpressible** | *leaf under-decomposition* — "`LGA4926`" should be a record; needs **domain knowledge**, undecidable ⇒ stays review (DESIGN's parked §2 open thread) | *optimality* — "is this the minimal cost?" is undecidable (**Rice**); *synthesis* — "produce a faster version" has no constructive patch ⇒ permanent ratchet/review |

The reading: anemic-modeling and complexity are not analogous by coincidence — they are **two samples of
one structure**. Each has a constructive core (①), a detectable-but-not-yet-walled middle (②), and an
undecidable tail (③) that is honestly review-bound forever. The win in each is the **same move**: drag
instances leftward — review → lens → construction — and *stop* at the decidability boundary.

## 4. The methodology (and its two failure modes)

For any discipline D we propose to enforce: **partition D's instances into ① ② ③ *before* building
enforcement.** The partition is the deliverable; the gate is downstream of it.

The two ways to get the partition wrong are exactly DESIGN's two named traps:

- **Region ③ priced as ①** → the **"never" trap** (§5): a ratchet sold as a wall. You build a "gate"
  that can never be complete (optimality, leaf-decomposition), then either it silently lets things
  through (coverage-by-illusion) or it grows without bound chasing completeness (the **purity trap**,
  §6 — the economic twin). Tell: the gate's success criterion contains "never" over an undecidable set.
- **Region ① or ② left in ③** → **coverage-by-illusion** (§6): a decidable, walkable class left to
  review because nobody located it. The round-trip homomorphism sat in review for exactly this reason
  until it was placed in ①.

The value is denominated in §1's displaced cost: each leftward move (review→lens→construction) is a pain
someone stops paying. The frontier tells you **how far left a given instance can possibly go** — so you
neither over-invest (purity trap) nor under-invest (coverage-by-illusion).

## 5. Why now / consumers

This frame is the decision procedure *under* the lockdown work, not a new lane beside it:

- **§0 (what to lock down)** — for each fail-open class, the frontier says whether to build a wall, a
  lens, or accept review. It is why "cache trustworthy" can be construction but "complexity-budget
  completeness" is residue.
- **§2 (decomposition)** — region ③'s anemic-modeling row *is* the parked "can a lens diagnose the
  leaf-side of §2?" question; the frontier answers it honestly: the **round-trip** sub-case is ①/②, the
  **leaf-under-decomposition** sub-case is ③. Not "yes" or "no" — *which part*.
- **§3 (complexity)** — the complexity column above; budget-dominance is gateable, optimality is not.
- **§5/§6 (wall vs lens)** — the frame *is* the rule for choosing, generalized past the cases §5
  enumerates.

## 6. Open (the §7 recursion)

Can the partition itself be mechanized — a check that, given a proposed discipline + gate, classifies
the gate's target as ①/②/③ and **fails closed if a gate claims ① over an undecidable set**? That would
turn the "never"-trap detector into a wall. Likely region ③ on itself (deciding decidability is
undecidable in general) — so honest residue, but a *high-value* residue: it is the lint that catches a
ratchet wearing a wall's badge. First mechanizable shard: flag any gate whose pass-condition quantifies
"never / for all" over a set the gate enumerates by search rather than by construction.
