# v4 Phase 2 CP-1/CP-2 Prep Audit

Date: 2026-05-17

Scope: read-only audit of `src/v4/compiler/01_tokenize.dag`,
`02_parse.dag`, `03_normalize.dag`, and `03_resolve.dag` against
`src/v4/TASKS.md` T-6 through T-8. This audit does not propose
load-bearing v3 pipeline edits or an implementation PR.

Reframe from parent on 2026-05-17: do not use bare alias-identity as a
modeling default. Every front-end fact below is classified as either
owned by the compiler layer or reused from `std/` with explicit evidence.
Merging two representations into one requires a proven identity claim,
not convenience.

## Summary

`01_tokenize.dag` and `02_parse.dag` already encode the Phase 2 pivot:
they are generic B2-OMNI ingest walkers over compiler-owned
`LexRules` / `Grammar` facts represented as `Node` data. Their only
realized paths are the Wave-1 void encodings (`E0` and `G0`).
`03_normalize.dag` and `03_resolve.dag` are still contract headers only.

The smallest safe next scope is therefore not a stage-body port from v2.
It is a `.dag`-only model/data proposal that makes the native `.dag`
`LanguageModel` carry the first real lexical and grammar facts consumed
by the existing generic walkers, after operator sync on the compositional
layer.

## T-6: `compiler/01_tokenize.dag`

TASKS contract: `FreeMonoid<Char> -> Result<TokenStream, Diagnostic>`.

Current declaration:

- Reused fact: `String` from `std/text.dag` is the source text carrier;
  its file declares it as finite `Char` sequence data, which is the T-6
  `FreeMonoid<Char>` boundary.
- The stage returns `Outcome<TokenStream>`, which is the ratified v4
  spelling for `Result<..., Diagnostic>` in `std/diagnostic.dag`.
- Owned compiler facts: `Token`, `TokenStream`, `LexRules`, and
  tokenizer diagnostics are declared in `01_tokenize.dag`.
- The actual function is `tokenize(text: String, file: Symbol, rules:
  LexRules) -> Outcome<TokenStream>`.
- Evidence for reusing `Node`: `LexRules` is declared as finite grammar
  data over `Node`; the current walker recognizes only an empty
  `TypeNode { connective: Conj }` with zero children.

Gaps against T-6 modeling decisions:

- Character class encoding is not modeled beyond the E0 void rule.
  There are no declared productions for identifier starts, digits,
  keywords, punctuation, string literals, comments, or whitespace.
- Whitespace/comment disposition is not encoded as data. T-6 asks
  preserve vs discard; the current realized path cannot answer because
  non-empty source rejects before token classification.
- Token boundary discipline is only specified for E0 exhaustion. There
  is no source span/extent model for tokens beyond raw `start: Int` and
  `end: Int`, which inherits the known `diagnostic.dag` raw-offset
  scaffold risk.
- `Token.class: Symbol` reuses the opaque identifier fact from
  `std/node.dag`; the compiler layer still owns the token-class facts
  that those symbols identify. Those token-class declarations do not
  yet exist in `extdeps/languages/dag.dag`.

T-3 dependency blockers:

- Required reused carriers exist: `String`, `Char`, `List`,
  `Diagnostic`, `Outcome`, `Locus`, `Node`, `Symbol`.
- The remaining blocker is not a missing carrier for the current
  signature; it is missing modeled lexical production data and a refined
  source-offset/extent story if token spans become load-bearing.

## T-7: `compiler/02_parse.dag`

TASKS contract: `TokenStream -> Result<ParseTree, Diagnostic>`.

Current declaration:

- `TokenStream` is imported from `01_tokenize.dag`.
- `ParseTree = Node`; `Grammar = Node`.
- The stage returns `Outcome<ParseTree>`.
- The actual function is `parse(tokens: TokenStream, grammar: Grammar)
  -> Outcome<ParseTree>`.
- The generic walker recognizes only the G0 void grammar: empty
  `TypeNode { connective: Conj }` plus an empty token stream.

Gaps against T-7 modeling decisions:

- Grammar productions are nominally `Node` trees, which matches the
  B2-OMNI direction, but the native `.dag` grammar currently contains
  only `dag_wave1_g0_void_grammar`.
- There is no modeled production data for modules, imports, `service`,
  `fn`, `type`, `operation`, expression forms, bodiless signatures, or
  pattern syntax.
- Error recovery is effectively single-diagnostic fail-fast by
  `Outcome`, but no non-G0 parse diagnostics exist for concrete grammar
  failures.
- ParseTree is layout-normalized to `Node` for the only realized path.
  No layout-preserving sidecar or declared-normalized disposition exists
  for whitespace/comments/trivia in the native `.dag` model.

T-3 dependency blockers:

- Required carriers exist: `Outcome`, `Diagnostic`, `Node`, `Symbol`,
  and `TokenStream`.
- The blocking dependency is `extdeps/languages/dag.dag` production
  facts: T-7 can only advance after T-6 produces non-empty token streams
  and the grammar data has corresponding productions.

## T-8: `compiler/03_normalize.dag` and `03_resolve.dag`

TASKS contract: `ParseTree -> NormalizedTree -> ResolvedTree`.

Current declaration:

- `03_normalize.dag` has only the module header. It declares, in prose,
  the ratified C3 contract: dissolve exactly `service`, `fn`, `type`,
  and `operation` into `Node`.
- `03_resolve.dag` has only the module header. It declares, in prose,
  the B-4 contract: use-site `Atom` symbols must canonicalize to the
  exact binder declaration `Symbol`; `04_infer` must preserve this fact.

Gaps against T-8 modeling decisions:

- No `NormalizedTree` type/reuse decision is declared.
- No `normalize(ParseTree) -> Outcome<NormalizedTree>` function is
  declared or scaffolded.
- The four sugar forms are named in the header, but their source
  representation is not yet available as parse output data. Until
  `extdeps/languages/dag.dag` declares those productions, normalize has
  nothing concrete to dissolve.
- No `ResolvedTree` type/reuse decision is declared.
- No `resolve(NormalizedTree) -> Outcome<ResolvedTree>` function is
  declared or scaffolded.
- Identifier binding is specified as exact `Symbol` canonicalization,
  but there is no scope/import/name-reference carrier local to T-8.
  The right source of those facts should be the normalized `Node` shape
  plus the native `.dag` grammar data, not a v2 `ModuleGraph` copy.

T-3 dependency blockers:

- Required generic carriers exist: `Node`, `Symbol`, `Outcome`, and
  `Diagnostic`.
- A practical blocker remains in T-7: without non-empty parse output
  for the four bounded surface-sugar forms, normalize/resolve bodies
  would either be no-op stubs or `.dag`-specific hardcoding, both of
  which contradict the ratified headers.

Ownership/reuse decision still needed:

- If `NormalizedTree` is just `Node`, the PR must state the evidence:
  parse output has already represented the four sugar forms as `Node`
  data and normalize rewrites those forms into the post-C3 kernel.
- If `ResolvedTree` is just `Node`, the PR must state the evidence:
  resolve's only additional fact is the canonicalized opaque `Symbol`
  on use-site `Atom`s, and no separate binding table is needed.
- If either evidence claim is false, declare a new fact instead of
  collapsing the representation.

## v2 Reference Check

The v2 files are useful cautionary references, not direct templates:

- `src/v2/01_tokenize.dag` declares `.dag`-specific token shapes, keyword
  tables, interpolation handling, and newline policy.
- `src/v2/02_parse.dag` is a recursive-descent parser over v2 token
  shapes with many parser-local result records.
- `src/v2/03_normalize.dag` mostly validates post-resolve invariants
  such as bare containers, rather than implementing v4's C3 sugar
  dissolution.
- `src/v2/03_resolve.dag` builds a module graph, resolves imports, and
  topologically sorts modules using string-keyed maps.

Copying those shapes into v4 would reintroduce `.dag` special cases and
parallel authorities. The v4 path should keep the generic walkers and
model `.dag` specificity as native `LanguageModel` facts, with owned vs
reused facts named explicitly.

## Proposed Smallest Next Scope

Title: `v4: add native dag Wave-2 lexical and grammar model data`

Scope:

- Plan-only until parent relays operator decision on the compositional
  layer 1 through 4.
- Candidate implementation, if approved: edit only
  `src/v4/extdeps/languages/dag.dag` plus any directly required
  comments/imports in `01_tokenize.dag` / `02_parse.dag`.
- Declare the first non-empty lexical facts needed for a minimal `.dag`
  source fragment, naming which facts are owned by the `.dag`
  LanguageModel and which reuse `std/` carriers with evidence.
- Declare the matching first grammar facts as `Node` shape,
  including a disposition for bodiless `fn` signatures per PARSE-1.
- Keep tokenizer/parser bodies generic: they may grow only by matching
  ratified production shapes, never by branching on `.dag` spelling.
- Do not edit v3 pipeline Rust or v3 load-bearing stage files.

This scope would unblock a later T-8 PR that declares `NormalizedTree`,
`ResolvedTree`, and the stage function signatures over real parse-output
shapes instead of no-op scaffolds, with explicit proof for any reuse of
`Node` as the carrier.
