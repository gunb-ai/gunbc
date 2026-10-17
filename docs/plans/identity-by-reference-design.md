# Identity by reference, not by spelling

Status: **draft for review: problem statement and open questions.** Not for merge. Companion to
`docs/plans/carrier-structure-boundary-design.md` (#14192), whose §15 survey surfaced this problem. That doc
settles where types and structures live; this one is about how code refers to *things*.

## 1. Problem

A large amount of higher-level `.dag` code identifies declarations, modules, nodes and entities by **name
strings** rather than by reference to the thing itself:

- `Map<String, X>` keyed by a declaration or module name: ~331 lines in `dag/gunbc`, ~232 in `src/v2`, 4 in
  `dag/product`.
- Fields like `decl_name: String`, `module_name: String`, `type_name: String`: ~150 lines. The broader
  `*name / *_ref / *_id: String` pattern is ~2,000 lines.
- Counts are rough grep line counts (comments included) at HEAD `6eab655b13`.

Representative sites:

- `dag/gunbc/instruments/prose_citation_census.dag:171,191,291,391` keys declarations by `module_path` +
  `decl_name` joined into one string. The comment at `:366` records a real failure: two rows sharing a key
  silently collapsed.
- `dag/gunbc/legacy_binding_tap_coverage_census.dag:60,77` (`decl_name: String`)
- `dag/gunbc/instruments/module_impact_query_front_door.dag:144` (`module_name: String`)
- `dag/product/printed_chassis/assembly_review_checks.dag:75` (`seen: Map<String, Bool>`)

## 2. Why it is a correctness problem, not style

A spelling is not an identity:

1. **Collisions.** Two distinct things with the same spelling (shadowing, the same name in two modules,
   ad-hoc joining of path and name) become one key. Rows are silently lost or merged. This has happened at
   least once (`prose_citation_census.dag:366`).
2. **Dangling references.** Nothing checks that a named thing exists. A rename or deletion leaves a stale
   string that still type-checks.
3. **Renames don't propagate.** Every rename becomes a text search, and the namespace cutover (N0–N6)
   multiplies renames.
4. **Two identities for one thing.** The same declaration is keyed differently in different places (bare
   name, qualified path, joined path+name), which is the §1.1 fork pattern again, applied to identity.

## 3. Root cause

v1's spelling-keyed identity (e.g. `kernel_algebra_profile: Map<String, AlgebraProfile>`, spelling rosters)
leaked into v2-era code. It is concentrated in `dag/gunbc`'s self-describing code (censuses, instruments,
roadmap), where the subject *is* the program's own declarations, but no first-class "reference to a
declaration" type was available or conventional.

## 4. Candidate direction (to be decided)

> A value that refers to a declaration, module or node holds a **reference to it by identity**, not its
> spelling. Spellings are for display and for the moment of resolution; they are not keys.

This needs answers to the questions below before any sweep.

## 5. Open questions

| # | Question | Notes |
|---|---|---|
| Q1 | What is the reference type? Does one exist in v2 already (`Node`, a declaration handle), or is a new one needed? | Survey the existing candidates first; don't add a second |
| Q2 | Where is a spelling resolved to a reference, and what happens when it fails or is ambiguous? | Fail closed (DESIGN): an unresolved or ambiguous name refuses, never picks |
| Q3 | Which string-keyed sites are genuinely about text (user-facing labels, external IDs, file paths), and stay strings? | Needs classification; not every `name: String` is wrong. External identifiers from outside the program are legitimately strings |
| Q4 | Census/instrument rows that *report* on declarations: do they store references, or references plus a display spelling? | Probably both, with the reference as the key |
| Q5 | How do references survive the namespace cutover and declaration moves? | Ties to N3/N4 and the one generator (#14017) |
| Q6 | Check: refuse new `Map<String, X>` / `*_name: String` where X or the field denotes a declaration? | Hard to detect mechanically; maybe convention plus a narrow lint on known shapes |
| Q7 | Order relative to #14192's migration | Probably after the List/Map cut, so the sweep only rewrites each site once |

## 6. Next step

Classify the ~560 `Map<String, …>` sites and the `*_name: String` fields into (a) references to program
entities, (b) genuine text or external identifiers, (c) unclear. That turns the counts above into a real
scope.
