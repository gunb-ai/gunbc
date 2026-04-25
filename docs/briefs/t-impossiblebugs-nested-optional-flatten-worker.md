# T-ImpossibleBugs — nested-optional flatten **(DESIGN/SCOPING brief, S — produces substrate proposal, NOT implementation)**

> **Director ad-hoc dispatch.** R2 T-ImpossibleBugs class 1 of 3 per
> [`docs/r2-structure.md`](../r2-structure.md) §"Goal 4". Independent
> of the other two impossible-bug classes — any worker can dispatch
> in parallel. Reports to Director (`zesty-bear-812`).
>
> **🔄 REFRAMED 2026-04-25 (post-`bright-moth-390` STOP-AND-ESCALATE).**
> Original brief framed this as an implementation lane against a
> presumed-existing "type-algebra constructor flatten" mechanism. Worker
> verified at HEAD: (a) there is no `Option<T>` type in `dsl/std/`;
> Option lives as `OptionalOf { inner: AlgebraTypeTemplate }` algebra-
> template variant at `dsl/std/algebra.dag:423`, with surface syntax `T?`
> as sugar; (b) the user-facing `T??` parsing question was never asked;
> (c) THESIS.md:343 *explicitly* gates this on "cardinality refinement
> substrate work" — which is *exactly* the substrate prerequisite this
> brief presumed could be bypassed; (d) "idempotent under self-nesting"
> was invented vocabulary not grounded in current algebra.dag facts;
> (e) DB-11 alias-RHS `where` is a predicate-filter pattern, not a
> structural-rewrite precedent. The worker correctly STOP-and-escalated
> with the recommendation to redirect to a design/scoping brief or park
> until cardinality substrate lands. **Director picked redirect.** This
> brief now produces a substrate proposal + bypass-or-substrate decision,
> NOT implementation.

## Read first

- **[`THESIS.md` lines 342-344](../../THESIS.md)** — class definition + the *"Gated on cardinality refinement substrate work"* gate.
- **[`docs/r2-structure.md` §"Goal 4"](../r2-structure.md)** — sub-lane scoping; tagged `[R2+]`.
- **[`dsl/std/algebra.dag:423`](../../dsl/std/algebra.dag)** — `OptionalOf { inner: AlgebraTypeTemplate }` — the *actual* representation of "Optional" in v3 substrate. **NOT** an `Option<T>` type declaration. This is an algebra-template variant; the surface form `T?` desugars into this.
- **[`dsl/std/types.dag:29`](../../dsl/std/types.dag)** — *"syntax keeps it simple: T, T?, List<T>"*. Confirms `T?` is sugar; consumers everywhere use `T?` (`first() -> T?`, `last() -> T?`, etc. in `dsl/std/algebra.dag`).
- **[`docs/architecture.md` §"How the compiler knows it" — cardinality bridge](../architecture.md)** — `return_cardinality` enum on Node; ~142 construction sites. The dissolution path THESIS names: cardinality refines into edge-existence patterns; once refined, `T??` becomes un-expressible at construction time. **None of this is in v3 substrate today.**
- **[`MODELING.md`](../../MODELING.md)** — especially M9 (DFS the concept DAG); also `feedback_audit_adjacent_authority_first` (audit existing facts before authoring new vocabulary).
- **`feedback_surface_brief_upstream_check`** — the dissolution may be one layer upstream.

## Frame — design-scoping, not implementation

Output of this lane is a **scoping document** (lands as `docs/briefs/t-impossiblebugs-nested-optional-flatten-design.md` or similar — worker picks placement), **NOT** a code change to v3 substrate. The scoping document answers four questions and produces a recommendation:

1. **Surface-upstream check.** Does `T??` parse today? What AST does it produce? Does it desugar to nested `OptionalOf`? Is it rejected at parse / desugar / lower? Find the answer with concrete file:line citations. **If `T??` is rejected at parse / desugar, the dissolution lives one layer upstream of any algebra-level flatten** and the rest of this scoping is moot.
2. **Substrate-attachment question.** If `T??` does parse + desugar to nested `OptionalOf`, where does the flatten attach structurally? Cardinality-substrate is named in THESIS as the obligation; *what specifically* on the cardinality substrate would express idempotent-flatten under self-nesting? (No reusable invented vocabulary: cite existing or proposed facts.)
3. **Bypass feasibility.** Is there a narrower bypass that closes the bug class without the full cardinality-substrate? E.g., a per-`OptionalOf` template-level rule that flattens `OptionalOf<OptionalOf<T>> → OptionalOf<T>` at template-resolution time. If yes, scope it concretely. If no, that's the answer — substrate is the prereq.
4. **Recommendation.** Either (a) bypass is feasible, here's the implementation brief shape; or (b) bypass is not feasible, this lane parks until cardinality-substrate lands; or (c) needs a substrate-design PR upstream first. Worker picks one with reasoning.

This lane is sized **S** because it's design-scoping, not implementation. Output is a doc PR, not a code PR.

## Three consumer-side requirements

1. **Surface-upstream answer documented.** Section in the scoping doc with file:line citations characterizing how `T??` is handled today (parse → desugar → lower → algebra). If rejected somewhere upstream, that's the dissolution-locus answer; document and stop here.
2. **Substrate-attachment proposal OR park-decision documented.** Section walking through cardinality-substrate's named obligation in THESIS + `architecture.md`, and either: (a) proposing a concrete attachment point (with cited substrate facts, no invented vocabulary), or (b) declaring the proposal blocks on substrate-design that's out-of-scope and parking the lane.
3. **Director-actionable recommendation.** Section closing with one of the three outcomes (bypass-feasible / lane-parks / needs-upstream-substrate-design). If (a), name the implementation-brief shape (what reqs, what STOPs, what dispatch profile). If (b) or (c), name the unblock-trigger.

## Slice — design-scoping doc

1. Surface-upstream investigation: grep parse + desugar + lower for `??` / nested-`OptionalOf` handling. Cite file:line for whatever you find (or whatever's absent). Concrete: *"`parse_type_atom` at `parse_parser_body.txt:NNN` rejects double-`?` with diagnostic XYZ"* — or the reverse, with the AST it produces.
2. Substrate-attachment investigation: cardinality-substrate's existing scaffolding per `architecture.md`; whatever facts are declared today; what would need to be declared for idempotent-flatten under self-nesting; cite, don't invent.
3. Author the scoping doc (location worker's call).
4. PR description: cite this brief; cite the scoping doc; close with the three-outcomes recommendation.

## Acceptance

- [ ] Scoping doc landed with all 3 consumer-side requirements addressed.
- [ ] Director-actionable recommendation: bypass-feasible / lane-parks / needs-upstream-substrate-design — pick one, cite reasoning.
- [ ] No code changes to v3 substrate (this is design-scoping, not implementation).
- [ ] `cargo fmt --all --check` clean (doc-only PR; CI gates trivially pass).

## STOP-AND-ESCALATE

Surface to Director.

- **Surface-upstream investigation reveals `T??` parses + lowers without rejection AND substrate-attachment investigation reveals there IS a clean per-`OptionalOf` template-level flatten that bypasses cardinality-substrate** — this is the "good outcome" for bypass-feasibility. NOT a STOP, but worker should explicitly flag it in the recommendation so Director can fast-track an implementation brief.
- **Surface-upstream investigation surprises** (e.g., `T??` parses to something other than nested `OptionalOf`; or only some-but-not-all consumers see the nested form) — STOP. Director-call on which surface to dissolve.
- **Substrate-attachment requires inventing fundamental new substrate vocabulary** beyond what cardinality-substrate already names — STOP. May indicate the lane is mis-scoped; substrate design is its own program.

## Non-goals

- **Not implementing the flatten.** This is scoping, not implementation.
- **Not modifying v3 substrate.** Doc-only output.
- **Not closing other T-ImpossibleBugs classes** — independent briefs.
- **Not re-authoring cardinality-substrate.** That's its own R2 sub-lane (or post-R2) work.

## Reporting

- Single PR. Title: `docs(briefs): T-ImpossibleBugs nested-optional-flatten — design/scoping doc (post-bright-moth-390 redirect)`.
- PR body cites this brief + the scoping-doc receipt + the chosen recommendation.
- On merge: signal Director with the recommendation; Director either authors the bypass implementation brief, parks the lane, or routes to substrate-design.

## Cross-manager note

- **Zero-Floor Manager**: heads-up if the recommendation lands on "needs-upstream-substrate-design" — that's substrate-territory potentially.
- **Grounding Manager**: no current overlap.
