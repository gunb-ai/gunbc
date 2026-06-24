# DRAFT for operator review — a candidate DESIGN.md articulation

> Not a DESIGN.md edit. This is prose to hold against §1/§2/§7 so you can decide whether intent-linearity earns a place in the authority doc (it lands on your open thread "model §1's axioms in `.dag` + enforce the syllogism"). Nothing here is committed to DESIGN.md.

## §X. Fractal intent-linearity (the enforceable shadow of §1's limit)

§1 closes on a limit: *replace convention with necessity until nothing arbitrary survives.* That limit has, until now, been a direction, not a check. Intent-linearity is its **decidable, enforceable form.**

**The claim.** A program's *description length should equal its irreducible information content* — the minimal generative template plus the genuinely-distinct data it ranges over — **recursively, at every nesting level.** Equivalently: a program's intent is **1:1 with its own inputs**. The description grows *only* with distinct information; never with repetition.

**Why this is §1, not a new axiom.** Redundancy is convention surviving where a *reference* was available — the same intent stated twice by hand instead of modeled once and pointed at. Super-linear description (N hand-copies of one pattern; one concept under two names) is exactly "the arbitrary that has not yet been replaced by necessity." So intent-linearity is not added to the axioms — it is what §1's limit *says*, made into a quantity: `redundancy = description − minimal`, and the wall is `redundancy = 0`.

**Why this is §2, made a wall.** §2 names the master move (minimize redundancy) along two directions — horizontal (one concept, every scale) and deep (decompose to atoms). Intent-linearity is those two directions stated as one recursive measure: *horizontal* = the template is O(1) in the data's cardinality (a 10-gate CI is O(1) logic + 10 rows, not O(10) logic); *deep* = the recursion bottoms out only at genuine atoms (§6: a finished stage is one fold; residue is a named irreducible kernel or un-migrated modeling — there is no third). The **base case is the fixed point**: when every hole is an atom, the program is 1:1 with its inputs. That is the executable definition of "done modeling."

**Why this is §7, measured.** The principle governs the system that implements it (the lenses, the compiler, this argument). A compiler 1:1 with its inputs *is* the seed shrunk to its irreducible core — intent-linearity at whole-compiler scale is the self-host seed-shrink at micro scale; one law, every scale. And it makes "language design opens up" a **number**: the wall reaches exactly as far as the substrate's expressible abstractions, and every new combinator/catalog-row advances it.

**The bound (the part that keeps it honest — §5).** Globally, "is this the minimal description?" is Kolmogorov-uncomputable — a ratchet, by the same Rice argument as optimality. What makes it a *wall* at all is the closed, grounded substrate (§4): anti-unification computes the **structural minimum relative to the available abstractions**. So intent-linearity is a **wall up to the substrate's abstraction power, a ratchet beyond it.** The frontier (the expressibility-frontier partition) is the honest object; "linear, full stop" would be the §5 "never" trap — a ratchet masquerading as a wall. Two consequences: (1) the enforcer must name where it stops; (2) you can only be 1:1 with a minimal form that is *expressible and ergonomic to reference* — so making folds reachable (the Ergonomics lane) is upstream of, and the same project as, linearity. Linearity and ergonomics are one principle from two ends.

**One mechanism, every instance.** redundancy = `(actual − minimal)` along a §1 time-axis, computed by a catamorphism over the closed substrate, closed by an anti-unification `(pattern → minimal-form)` registry, applied via the write API. Parameterized by (representation walked) × (axis minimized):

- source-AST × change-time → hand-unrolled fold, duplicate type decl (the model-time redundancy).
- cost-recurrence × run-time → the `O(n²)→O(n)` rewrite catalog (the runtime redundancy).
- whole-program × the seed → self-host shrink.

The **model↔realization fork** (the named systemic debt) is the value-level instance: one intent described twice (model + realization) reconciled by per-site bridges. "Intent 1:1 with inputs" *is* the de-fork goal — the model IS the realization, one description, no bridges.

## Dissolution trigger (DESIGN §6)

Delete this draft when the operator rules on it: either intent-linearity is absorbed into DESIGN.md (the §1 axioms modeled in `.dag` with a syllogism-enforcing lens, the open thread this draft lands on) — at which point the authority doc carries the articulation and holding a separate draft against it is the §6 parallel-ledger violation — or it is rejected as off-thesis, at which point it is §5 dead scaffold. Either way the ruling retires the draft; prose held against the authority doc has no standing once the authority doc has spoken.
