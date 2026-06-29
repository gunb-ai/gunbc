# Reference grounding -- migrate stringly-typed references to typed carriers

> ROADMAP follow-on to the #5943 construction-justification prose deletion. The carrier half of a two-part groundedness effort: this plan owns migrating stringly-typed REFERENCES to typed carriers; the DETECTOR half is neat-fox-279's v2.lens.grounding (the EXTRACT pass that enumerates every bare-String field coinciding with a concept). DESIGN refs: sec 2 (decompose to grounded atoms; net concepts must not grow by re-invention), sec 3 (single authority; nicknaming), sec 4 (a closed substrate makes references walkable, not text-matched), sec 5 (construction over validation; a typed reference makes 'points at nothing' unwritable), sec 6 (the mark on the carrier is the authority). Sibling frames: [expressibility-frontier.md](expressibility-frontier.md), [construction-justification-rule.md](construction-justification-rule.md), [axiom-syllogism-lens.md](axiom-syllogism-lens.md).

## 1. The displaced cost (DESIGN sec 1)

A reference stored as a String is an un-checkable edge. The compiler cannot see that the symbol it names exists, is imported, is in-layer, or is the single authority for that fact -- so a typo, a dead symbol, a layer inversion, and a nickname all pass silently and are paid later as a debugging session or a wrong answer. The deliverable here is the removal of that silent-failure surface: once a reference is a typed carrier, the substrate walks it. The same String also defers cost the DRY way (DESIGN sec 2) -- every consumer that wants to follow the edge re-implements string parsing / lookup, and every one can drift.

A holistic audit (three-region fan-out, 2026-06-29) found this is not a long tail of unique problems but ONE problem with ~80+ instances: a symbol, module, fn, path, or URL written as free text where a typed edge belongs. That is the good news -- a handful of carrier changes cover almost all of it, and almost every carrier ALREADY EXISTS. The work is migration, not invention.

## 2. The partition (expressibility-frontier regions)

Each stringly-typed field sorts into one of four buckets. Only the first three are work; the fourth is honest residue to leave alone.

| Bucket | What the String holds | Target -- reuse, do not mint | Why it is a wall once migrated |
| --- | --- | --- | --- |
| Reference | a qualified symbol / module / fn / decl name | std.decl_ref.DeclarationRef or a QualifiedName | 'points at nothing' / out-of-layer becomes a resolve error, not a runtime miss |
| Reference (URL) | a URL / locator | extdeps.uri.Uri (exists, already validated) | scheme + locator parsed once, at the carrier, not per consumer |
| Reference (path) | a file path / entry | the existing path-literal lane (resolver --entry accepts QualifiedName) | decided in that lane; this plan routes to it, does not re-solve it |
| Enum-like | a value from a small CLOSED set used as a discriminator | a closed sum type (e.g. UriScheme, ReasoningEffort) | an unmodeled member becomes an unhandled-variant compile error |
| Derivable | a property the compiler could analyze structurally | a check, not a stored field | the fact is computed by construction, so the stale-copy state is unwritable |
| Genuine prose | a real human explanation with no structure to extract | KEEP -- diagnostic detail, descriptions, fixtures, doc bodies | honest sec 6 residue, not a wall |

## 3. The single authorities to reuse (and the anti-goal)

DFS the concept DAG first. The entire reference class collapses onto carriers that already exist:

- std.decl_ref.DeclarationRef (module_path + decl_name + field) -- already the carrier for Disposition.Scaffold.bind and the commit-workflow entries. This is the home for the dominant class: module / import_module / fn_qualified_name / construction / entry / check_fns / floor_plan_function / ref / caller / callee / operation_name / method_name / from_type / to_type.
- QualifiedName -- where only a name (no field selector) is meant.
- extdeps.uri.Uri -- the ~22 url fields (std.markdown LinkInline/ImageInline, std.types call surfaces, extdeps validation).
- extdeps.uri.UriScheme -- raw_scheme (already imported in extdeps_external_authority.dag; lowest-effort win).

> ANTI-GOAL (DESIGN sec 2 / sec 3): do NOT mint a fresh branded NonEmptyStr per field (FieldName, ParamName, MethodName, ImportPath, TypeIdentity, GitRefLabel ...). That FAILS the sec 2 test -- 'net concepts must not grow by re-invention' -- and a per-field brand is exactly the nickname sec 3 forbids. The right move is the OPPOSITE: collapse the reference class onto the two authorities that already exist (DeclarationRef, QualifiedName). A genuinely new type earns its place only where the thing is NOT a decl reference -- a Uri, or a real closed enum (UriScheme, ReasoningEffort, the Docker mode enums).

## 4. The worklist is generated, not hand-maintained (DESIGN sec 6)

neat-fox-279's v2.lens.grounding EXTRACT pass enumerates every bare-String field whose name coincides with an existing concept -- which IS the reference bucket above. So the migration is driven by the detector's output, not a hand-kept list in this doc (a hand-list would be the parallel-ledger sec 6 forbids). The detector finds the sites; this carrier plan reshapes them. The two are the two halves of one groundedness effort: detector (neat-fox) and carrier (this plan).

Concretely the division: the grounding LENS reports the worklist and stays the permanent backstop; the carrier migration consumes it. Neither subsumes the other -- a reshaped carrier with no detector still lets the NEXT bare-String slip in; a detector with no reshape only reports.

## 5. Sequencing

1. #5933 (adds v2.lens.grounding) lands, then #5943 (deletes the unread rationale / undecidable_because prose) lands. Prerequisite: the carriers are already in their post-deletion shape.
2. PROOF PR -- WallNow construction:String becomes mechanism: ConstructionMechanism plus authority: DeclarationRef (reusing std.disposition.ConstructionMechanism + std.decl_ref.DeclarationRef). This is the first concrete reference reshape and the pattern proof: it turns 'every lens chains to a real construction' from a prose review into a walkable graph property (feeding the axiom-syllogism open thread). Small, self-contained, ~3 live sites.
3. BULK migration -- String becomes DeclarationRef / QualifiedName across the reference bucket, scoped by the EXTRACT worklist, in reviewable slices (one carrier family per slice, so a slice is one fold over N call sites).
4. QUICK WINS -- url becomes Uri; raw_scheme becomes UriScheme; the small closed enums (ReasoningEffort, Docker modes, medium).
5. PATH-LITERAL debt folds into its existing lane (resolver --entry accepts QualifiedName), not a fresh effort.

## 6. The wall, and what stays residue

Each reshape is a sec 5 wall: a DeclarationRef that resolves to nothing is a compile error, so the bad state is unwritable by construction -- strictly stronger than a lens that flags a dangling String after the fact. What stays honest residue: (a) genuine-prose fields (kept); (b) the grounding LENS itself, which remains the permanent detector for the NEXT bare-String (a reshape makes one field a wall; only the lens keeps the CLASS closed); (c) deciding whether a given String is a reference vs genuine prose is itself a judgment -- the EXTRACT pass over-approximates and a human confirms, exactly the sec 6 residue the grounding lens is built to be.

### Non-goals

- No new branded-String types (section 3 anti-goal).
- No reshape of the derivable bucket into a stored field -- those become checks (e.g. the cost 'totality over the closed kernel' prose is a totality analysis, not a String).
- No hand-maintained site list in this doc -- the EXTRACT pass is the worklist authority.

## Dissolution trigger (DESIGN §6)

Delete this doc when the reference buckets are migrated and the property is witnessed rather than planned: every reference-class String field carries a typed reference (DeclarationRef / QualifiedName / Uri) or a closed enum, the WallNow construction field is a DeclarationRef, and v2.lens.grounding's EXTRACT pass returns empty over the corpus (no bare-String field coincides with a concept) -- at which point groundedness is a walked graph property, the grounding lens is the standing backstop for any regression, and this migration plan is redundant.
