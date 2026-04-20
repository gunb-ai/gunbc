### Design commitments must name the substrate target

**Design isn't done until every claimed non-commitment is proven.**
When a design doc claims "no substrate change needed for feature X,"
the doc must explicitly name the substrate element(s) that express
X's semantic. If it can't name them, there IS a substrate change and
the doc is smuggling it — and the implementation will surface it as
one of four routes, all of which fail review later at higher cost
than fixing the design now.

**The four smuggling routes** (the option space when a feature
requires substrate expression and the design refuses to name it):

1. **Synthesized declarations at lowering time.** The lowerer
   allocates fresh declarations per-occurrence (or deduplicated
   per-unique-shape) to carry the feature's semantic. The new
   declarations are compiler artifacts mixed into the declaration
   table, creating layering opacity for any lens that walks it.
2. **Flat coproduct growth on existing substrate variants.** An
   existing coproduct gains new variants that encode the feature,
   often as a flat expansion of an orthogonal dimensional
   distinction (e.g., `{A, B}` becomes `{A_bare, A_with, B_bare,
   B_with}`). Fails 4-pattern check at Pattern 4 (dimensional).
3. **Surface state preserved in the substrate.** Parse-level
   shapes (`SurfaceExpr::Path`, raw tokens, unlowered AST nodes)
   survive into the substrate so downstream consumers re-parse
   them at their boundary. Breaks the substrate/surface
   separation and makes "lenses walk a Dag, not a parse tree"
   structurally impossible.
4. **Parallel hand-maintained representation.** The feature's
   semantic is tracked in a second data structure outside the
   substrate, kept in sync by convention. Diverges immediately;
   the `feedback_parallel_representation_debt` concern applies.

**The review rule:** for every design doc that claims "no substrate
change needed," the reviewer must force the author to name the
existing substrate element(s) that express the feature's semantic.
"It lowers to X" is the answer; X must be a specific, existing
substrate form. If the author cannot produce X, the claim is
unverified and the design is not done.

**The check against the codebase:** thesis-level claims about
what's expressible in the substrate drift from implementation,
especially around categorical vocabulary that reads as intuitive
but isn't wired in. Before recommending a design option that
depends on an existing substrate form, verify that form exists by
reading the current code (the file and line number), not by
reciting the thesis. Reading the thesis out loud is not the same
as reading `dsl/std/algebra.dag:276-302`.

**Background:** this rule was added after PR #453, where Prereqs
1 and 2 of the substrate reflection slate each claimed "pure
lowering extension, no substrate change needed" in §11 of
`docs/substrate-reflection-design.md` without naming the
substrate target. The implementation hit all four smuggling
routes simultaneously: synthesized `FieldAccessor` accessor
declarations (route 1), flat 4-way `BranchPattern` growth
(route 2), scattered parallel representations between
`substrate.dag` and `dag.rs` (route 4), and near-adoption of
preserving `SurfaceExpr::Path` through to emit (route 3,
considered and rejected at implementation time). The design
gap was the review miss: each prereq deserved the
§3.5-level option walk Prereq 0 received. §3.6 and §3.7 now
codify those walks; this invariant generalizes the lesson
into a ratchet on all future design-doc reviews.

**Ratchet:** a design doc that adds a prereq / feature scope
saying "no substrate change needed" must, in the same doc,
either (a) name the existing substrate element that expresses
the feature (with file and line citation), or (b) provide a
§3.5-style option walk that enumerates the substrate
commitments and commits to one. A claimed non-commitment
without one of these two receipts is a blocker.

