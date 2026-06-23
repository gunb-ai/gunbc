# Self-applying lenses — detect → generalize → emit → write

**Thesis.** A lens that only *flags* concedes the bad state is writable and leaves the fix to a
human (who pays the §1 time, and re-introduces fail-open arms by hand). The next form of the lens
**produces the correct pattern and applies it through our own write API** — it does not report a
violation, it removes it. This is the §7 recursion (the dedup principle applied to the dedup-*tools*)
and the apotheosis of the Ergonomics lane ("make the fold the path of least resistance" → the lens
*writes* the fold).

## The unifying concept: redundant intent

Every member of this lens family detects **specification complexity above the essential minimum**:
the *intent* has a minimal generative description; the *code* spells it out at higher complexity.

- hand-unrolled fold — intent "do X to each of N" (O(1) intent + O(n) data), code O(n) statements.
- 2-D / nested unroll — intent "fold over a grid" (O(1) + O(n²) data), code **O(n²)** statements.
- if/else-ladder dispatch — intent "look up key→value" (a table: O(n) data + O(1) dispatch), code O(n) branches.
- duplicate type decls (`structural_similarity`) — intent "one parameterized type", code N type decls.

This is §1/§2 made measurable. Anti-unification yields the **generalization** (template + per-element
substitution); the redundancy is `spec_size − generalization_size`. The generalization *is* the
minimal-intent code — so the same read that **measures** the gap **produces** the fix (§4: one
decision procedure run in different directions, N models not N×M).

## The engine: anti-unification (one kernel, two binders)

`congruent`/`anti_unify` (seeded in `v2.lens.simulated_relationship`) is the shared kernel. It serves:

- **term layer** — N near-identical statements → the varying part becomes a **list element** (a fold).
- **type layer** (`structural_similarity`, currently an unrealized scaffold) — N near-identical type
  decls → the varying part becomes a **type parameter** (a generic; the `Int8…Int128` = one `Compose`
  axis). Same move, different binder. These should be **one engine imported by both lenses**, not two.

## Three refinements proven by stress test (scratch S1–S4)

1. **Fractal recursion (S2).** A 2-D unroll is a fold-of-folds; a single application removes one layer
   (outer flagged *and* inner-row flagged independently). The producer-and-applier must **recurse into
   its own output's holes** until the residue is irreducible (§6: a finished stage is one fold; the
   bottom is a named irreducible kernel). The recursion is the O(n²)→O(n)→O(1)-spec reduction.
2. **Type-homogeneous holes need resolve, not parse (S4).** A fold over a heterogeneous coproduct —
   `handle(Read{path}); handle(Write{path,data}); handle(Close{})` — is **missed** by structural
   congruence (arms differ in shape). The real criterion is "the hole ranges over inhabitants of one
   type" (§2-deep, §3-grounding), which requires the **resolved type** at each hole. (Producer scope
   note sent to the parse/resolve-walk lane.)
3. **The species are distinct schemes; one isn't a fold (taxonomy).** foldl (gate chain) · mapAccumL/
   scan (interleaved byte-decomp) · reduce (n-ary binary) · **table-lookup (if/else ladder)**. The
   ladder's correct fix is "these cited rows belong in `extdeps/` + one generic dispatch" (§3: dispatch
   lives in extdeps, not std) — **not** a fold. The lens must name the scheme so it emits the right form.

## Dependencies

- **emit** (§6, `serialize_target ∘ translate`) to render the generalized `Node` back to source.
- a **filesystem write effect** to apply it (the write twin of the lenses' existing `filesystem_read`).
- **resolve** facts on the corpus walk (refinement 2) — the shared grounding authority.

## Retrofit path (fix all current lenses)

Each existing analytical lens is upgraded from `-> Bool`/`-> count` to *also* produce a corrected
`Node` and (behind a flag) write it. Order by displaced cost, not taxonomy. The detect-only form stays
valid where the fix is undecidable (the ratchet residue) — produce-and-apply is for the **decidable
wall** classes, where the generalization is unambiguous.
