> Part of: [post-l15-phase-plan.md](./post-l15-phase-plan.md) > [single-emitter-design.md](./single-emitter-design.md)

# Lane 1 Stage 1d — Single-emitter consolidation build plan

**Lane:** 1 (Emission unification)
**Stage:** 1d (last design stage; gates Stage 1e implementation start)
**Time budget:** ~1 week
**Status:** Plan. Pure design, no code changes.

> Role in the plan: produces the file-by-file build plan for Stage 1e's
> consolidation execution. Stage 1e does NOT start until this
> stage signs off on the inventory, gap list, bridges, and pilot
> target choice.

---

## Motivation

`docs/single-emitter-design.md` establishes the principle: one emitter,
reads target specs. But it stops at principles. P2's consolidation
needs a **file-by-file build plan** with:

- Which functions in `emit_rust.rs` / `emit_go.rs` / `emit_python.rs`
  become target-agnostic (move into generic walker) vs target-declared
  (move into spec)
- What new spec fields each current hardcoded behavior needs
- Where substrate gaps block the consolidation
- Which target to pilot with (P2-L1 choice)
- A bridge list: every piece of name-based dispatch, hardcoded variant
  name, or target-specific convention currently in Rust code

This lane produces that plan. **No code changes.** The plan's quality
is the gate for P2 starting.

---

## Scope

Four deliverables, all design artifacts:

### 1. Emitter function inventory

Output: `docs/emit-functions-inventory.md`

For every `fn render_*` and `fn emit_*` across the three current
emitters:

| Function | Current home | Classification | Destination |
|---|---|---|---|
| `render_operator` | emit_rust.rs | spec-driven | generic walker + spec template |
| `render_field_project` | emit_rust.rs | spec-driven | generic walker + spec template |
| `InputUseFacts::build` | emit_rust.rs | lens | ownership lens (own file) |
| `decl_is_copy` | emit_rust.rs | lens | copy-type lens (own file) |
| `algebra_field_for_operator` | emit_rust.rs | substrate walk | shared primitive |
| ... (30+ more) | | | |

**Classifications:**
- **spec-driven** → template substitution, walker reads spec, dissolves
- **lens** → extract into its own `lens_*.dag`, walker consumes facts
- **substrate walk** → generic primitive, moves to `std/substrate_walks.dag`
- **per-target integration** → stays target-specific (rustfmt invocation,
  file extension choice, etc.) — typically <5% of total

**Expected split:** ~90% spec-driven or lens, ~5% substrate walk, ~5%
per-target integration.

### 2. Spec field gap list

Output: `docs/spec-field-gaps.md`

For every spec-driven function, enumerate what the target spec currently
declares vs what the function needs. Example:

```
render_branch (match expression emission):
  Current rust.dag: match_arm template, match_expr template
  Needed: match_discriminator rule (does the target dispatch on
          enum tag? struct field? runtime type check?)
  Gap: new `data rust_match: MatchStrategy = DiscriminatorTag` needed

render_transform (callable dispatch):
  Current rust.dag: CallableRealization with template
  Needed: nothing — already sufficient
  Gap: none
```

The gap list drives what spec extensions P2 adds BEFORE the generic
walker can replace the Rust function.

### 3. Bridge inventory

Output: `docs/emit-bridges.md`

Every place in the current emitters where Rust code does something the
spec should do. Examples already known:

- `name.starts_with("rust_")` for target realization filtering (B11)
- `v.label == "Empty"` / `"Cons"` in Python pattern matching (B13)
- `algebra_field_for_operator` always resolves via OrderedRing (B14)
- `#[derive(...)]` injection in type emission (Rust-specific hardcode)
- Literal syntax variations (`"str"` vs `'str'` vs `"""str"""`)

Each bridge is scoped: what kills it, what spec field replaces it,
whether the substrate supports it today.

### 4. Pilot target decision

Output: a section in this file (or a separate `docs/p2-pilot-target-choice.md`).

Evaluate SPICE vs English as the P2 pilot. Criteria:

| Criterion | SPICE | English |
|---|---|---|
| Ownership complexity | None (analog) | None (prose) |
| Scope model | None | None |
| Pattern emission | Sum types → mux synthesis | Sum types → prose |
| Verifier availability | `ngspice --syntax-check` | Oracle diff vs golden |
| Substrate coverage | Needs analog algebra — might be a gap | Needs text templates — already covered |
| Test cost | Medium (ngspice runtime) | Low (text comparison) |

**Recommendation framework, not decision:** prefer the target that
exposes the *fewest* new substrate gaps for its pilot run. If SPICE
needs analog algebra additions, English may be safer. If English's text
templates prove too weak, SPICE earlier.

This lane produces the evaluation; P2-L1 owner makes the final call
informed by it.

### 5. Pessimistic fallbacks from Half B to revisit

Half B's merge reconciliation (2026-04-17) reverted two optimizations
back to pessimistic behavior because they misbehaved under merge with
main's cached-bootstrap state. These are **known-pessimistic areas**,
not silent regressions — explicitly catalog them here so the walker
design revisits them rather than inheriting the revert:

**A. `decl_is_copy` structural walk**
- **Reverted:** the structural walk over user-defined sum types was
  over-eagerly classifying variants with all-Copy payload as Copy.
  Reverted to main's "sum types are non-Copy" conservative behavior.
- **Revisit during consolidation:** structural copy-detection for
  user-defined sums is legitimately decidable. The walker can read
  each variant's payload fields and compute Copy-ness from the Conj
  structure. This should re-land during Lane 1e as part of the
  copy-type lens.
- **Path forward:** copy-type becomes a dedicated lens (it's one of
  the extractions identified in the function inventory). The lens
  does the structural walk correctly once; every consumer reads
  from it. This eliminates the local-reconstruction issue that
  caused the Half B revert.

**B. `OwnedConstructLastUse` optimization**
- **Reverted:** the optimization that emitted move-on-last-use
  (`value` instead of `value.clone()`) was unsound under template
  reordering. Reverted to always-clone behavior.
- **Revisit during consolidation:** this is the "template-aware
  ownership emission" problem. The consolidated walker has full
  template context (CleanEmissionContract + LocalScope with
  position tracking per DB-2). A template-aware move-or-clone
  decision is tractable once the walker is authority for emission
  order.
- **Path forward:** the ownership lens (extracted from
  `analyze_user_defined_callable`, per Lane 1e function inventory)
  computes borrow/move/clone decisions from substrate facts. The
  walker consumes those decisions without needing to re-derive
  ownership during emission — which is what made the Half B
  optimization unsound.

**C. Clone ratchet 1 → 5 (main was 6)**
- Current state: 5 clones in `lens_unused_parameters_generated.rs`.
  Main was at 6; Half B initially got to 1 but reverted to 5 due
  to A and B above.
- **Target after Lane 1e:** ≤ 1 (Half B's original target).
  Achievable once the ownership lens is the single authority and
  the walker reads from it.

These are **explicit revisit items** during Stage 1e execution. Flag in
the build plan's bridge inventory so the walker design accounts for
them.

---

## Out-of-scope

- Any implementation of the generic walker — that's P2-L1.
- Actually fixing any bridge (B11, B13, B14) — those follow in P2.
- Deciding the generic walker's Rust vs `.dag` home. The walker is
  Rust in P2 (stays in `src/v3/compiler/`), becomes `.dag` in P3
  (self-hosting cycle). Don't design for self-hosting yet.
- New substrate types beyond what the gap list identifies — if a
  type is needed by the walker but isn't in scope of any existing
  spec, note it in the gap list; don't design it here.

---

## Direction

**Inventory first, synthesis second.** Start with the mechanical work
of listing every emitter function. Patterns emerge from volume: once
every function is classified, the walker's shape is visible as the
intersection of "what's common across all classifications."

**Be ruthless about substrate gaps.** If a function classification
would require "the spec doesn't have a way to say X", write it down.
P2's first week will be spec additions; the gap list is the scope.

**Don't over-plan the walker's implementation.** This lane's output is
enough material for an implementer to write the walker next phase — not
the walker itself. Designs that specify APIs too tightly often need to
be thrown out once implementation starts.

---

## Escalation criteria

Stop work and surface if:

1. **>30% of emitter functions fall into "per-target integration"**
   — that means the consolidation premise is wrong. The theory says
   ~95% should be spec-driven or lens. If the fraction is materially
   lower, either the classification is too conservative (re-evaluate)
   or the targets are more divergent than the thesis assumed (escalate
   to thesis-level review).

2. **Substrate gap list exceeds ~10 new type additions** — that's
   roughly a phase of work by itself. Either the gap list is
   overspecified (aggregate), or consolidation needs a prerequisite
   phase that extends the substrate. Surface.

3. **No safe pilot target candidate exists** — if both SPICE and
   English have significant substrate gaps, the pilot itself becomes
   a multi-lane effort. Surface; reconsider whether P2 starts with
   Rust-only consolidation (proving against the existing target)
   before adding a new one.

4. **Name-based dispatch is pervasive beyond expectation** — if the
   bridge inventory finds 20+ distinct name-prefix or
   string-comparison sites, B11 is the tip of the iceberg. Surface;
   this may need a dedicated debridge phase before consolidation.

---

## Acceptance gates

Lane is done when all five hold:

- `docs/emit-functions-inventory.md` classifies every `fn render_*` /
  `fn emit_*` across the three emitters (count verified by `grep`).
- `docs/spec-field-gaps.md` enumerates each needed spec extension,
  tagged by priority (blocks consolidation vs nice-to-have).
- `docs/emit-bridges.md` lists every known bridge (name-based
  dispatch, hardcoded convention, per-target Rust branch) with
  dissolution target.
- Pilot target evaluation written and linked.
- P2-L1 owner (whoever takes it) reviews and signs off on the plan
  before P2 starts.

---

## Dependencies

- **Requires:** Half A + Half B merged (so the inventory reflects
  the current state, not a moving target).
- **Blocks:** P2-L1 (consolidation implementation needs this plan).
  Hard gate.
- **Does not block:** P1-L1 or P1-L2.

---

## Estimate

- Emitter function inventory: 2 days (mechanical but thorough)
- Spec field gap list: 1.5 days (requires reading each function body)
- Bridge inventory: 1 day (grep-driven)
- Pilot target evaluation: 0.5 day
- Review + sign-off cycle: 1 day

Total: ~6 implementer-days.

---

## Success signal

When P2-L1 starts, the implementer reads these three design docs and
can:

1. Open each current emit_* file knowing which functions to delete,
   extract, or keep
2. Add the needed spec extensions in a single batch commit (no
   "oh wait, I need another field" mid-implementation)
3. Write the generic walker against a clear API: "read each node's
   Behavior, look up the CallableRealization / MatchStrategy /
   whatever, substitute into the declared template, recurse on inputs"

If P2-L1 needs to pause and redesign mid-implementation, this lane
under-specified something. The escalation criteria above are designed
to catch those before they become P2's problem.
