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

## 6. Survey results (origin/main `fb6f04c403`, 2026-10-17)

### 6.1 Q1: identity types that already exist

| Type | Declared | Identity or spelling? |
|---|---|---|
| `^decl` caret reference | syntax; compiler-resolved, produces a `Symbol` | Identity |
| `Symbol` | `src/v2/std/node.dag:32` (interned, `symbol_eq`) | Identity |
| `QualifiedName = FreeMonoid<Symbol>` | `src/v2/std/qualified_name.dag:32` | Path of identities; equality is structural |
| `DeclarationRef {module_path, decl_name: NonEmptyStr, field}` | `dag/std/decl_ref.dag:15` (~7.9k uses) | **Right shape, spelling payload** |
| `ResolvedDeclRef {module: String, name: Symbol}` | `src/v2/compiler/discovery_enumeration.dag:5` | Half identity, half spelling |
| `ModuleId` (branded `NonEmptyStr`), `ModuleIdentity {name: String}` | `dag/gunbc/crate_partition.dag:34`, `src/v2/lens/module_impact_query.dag:47` | Spelling wrappers: forks of one concept |
| `ContentHash` | `dag/std/content_hash.dag:53` | Identity by content |
| `Node` + `NodeOccurrenceIdentity` / `Path` | `src/v2/std/node.dag` | A value with positional identity |
| `RoadmapNodeId` | roadmap tree | Branded string, its own domain |

**Candidate answer to Q1: no new type is needed.** Re-key `DeclarationRef` on identities
(`module_path: QualifiedName`, `decl_name: Symbol`) so the compiler builds it from `^decl`. Then:
- `ResolvedDeclRef`, `ModuleId` and `ModuleIdentity` collapse into it or into `QualifiedName` (a carrier-boundary
  fork in #14192's terms);
- SCM objects key by `ContentHash`;
- roadmap code keys by `RoadmapNodeId`, not bare `String`.

### 6.2 Classification

Each site is classed as (a) a reference to an in-program entity, (b) genuine text or an external identifier, or
(c) unclear. Map counts come from a heuristic over all lines, hand-corrected on a sample: roughly ±10%. The 870
`*_name / *_ref / *_id` fields were classified by name and spot-checked.

| Population | (a) convert | (b) stays a string | (c) unclear |
|---|---|---|---|
| `Map<String, …>` (572 lines; 51 are not code) | ~340 | ~106 | ~76 |
| `*_name / *_ref / *_id: String` fields (870) | ~225 | ~560 | ~80 |

The (a) work is concentrated in about 20 files:
- ~160 module/declaration-keyed indexes in `src/v2` (`name_resolve`, `module_impact_query`, `module_graph`,
  `module_acquisition`, …);
- ~120 roadmap maps plus 94 `node_id` fields in `dag/gunbc/roadmap`, a typing-only change to `RoadmapNodeId`;
- ~15 SCM object indexes, which move to `ContentHash`.

(b) is mostly cloud, GitHub and Google identifiers, paths and labels, all correctly strings.

**Scope:** ~550 sites truly need converting. The work is incremental, and the `DeclarationRef` re-key comes first
because every declaration-keyed conversion depends on it.

### 6.3 New open question

| # | Question | Notes |
|---|---|---|
| Q8 | Are dashboard sessions (`session_name` / `session_id`, ~40 sites) in-program entities? | They are modelled in the program but named outside it. Leaning: external identifiers, so they stay a branded string |

## 7. Decisions (2026-10-17, via sleek-owl-397)

**R1: `DeclarationRef` is the one canonical reference.** It is re-keyed on identities
(`module_path: QualifiedName`, `decl_name: Symbol`) and built by the compiler from `^decl`. This follows from DESIGN
§3 (single authority) and the §6b root generalization: three identity carriers for one fact is itself the fork.
- `ResolvedDeclRef`, `ModuleId` and `ModuleIdentity` are **deleted**, not kept beside it. Module identity is
  `QualifiedName`.
- The migration population is the fork census of §6.2: the ~550 (a) sites in ~20 files. The `DeclarationRef`
  re-key lands first.

**R2: dashboard sessions are external branded strings (Q8).** The dashboard is an independently governed
upstream (DESIGN §3, external upstream decomposition), and its ids are opaque to us. The brand belongs in a
modelled `extdeps` home for the dashboard. None exists on main today (checked `fb6f04c403`), so creating that
home is part of the conversion.

**Interlock: alias identity (Family A, `adhoc-65526d74-43f`).** The type relation it keys must use the re-keyed
`DeclarationRef` (identity: `QualifiedName` + `Symbol`). It must not use a spelling, `ResolvedDeclRef` or
`ModuleId`.

This is doc text only. Implementation goes back through plan approval.

## 8. Next step

Plan the `DeclarationRef` re-key as the first change, for plan approval.
