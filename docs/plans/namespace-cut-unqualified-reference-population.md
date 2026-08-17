# The unqualified-reference population: what the cut actually left behind

**2026-08-17.** Supersedes the framing in which the branch's remaining work was
"182 diagnostics in four classes". It is not. That count is an observation over
a biased surface.

## The bias

The qualification pass only ever rewrote a site that PRODUCED A DIAGNOSTIC. A
bare reference whose name is ambiguous corpus-wide, but which happened to
resolve because exactly one declarer was in that compile's pool, was left
untouched and reported nothing.

So the diagnostic count never measured binding correctness. It measured how many
ambiguous bindings were also unlucky enough to be type errors. Reducing it
196 -> 182 removed noisy bindings and told us nothing about silent ones.

This is DESIGN's absorbing fallback with the polarity inverted: not a failure arm
that widens, but a SUCCESS arm that narrows. The instrument reported the subset
that complained.

## Why silence is not safety here

`v1_interpreter.rs` registers every named item twice:

    fn_nodes.insert(name.clone(), item.clone());       // bare leaf
    fn_nodes.insert(qualified.clone(), item.clone());  // module.name

One map. The bare slot is overwritten as modules are traversed, so a bare name
with N declarers resolves to whichever module was visited last. The qualified
key cannot collide, because a module path is unique.

The interpreter also computes

    ambiguous_bare_function_names = bare_name_counts.filter(count > 1)

stores it on `InterpContext`, and never consults it in `lookup_fn`. Its only
consumer is `selected_function_identity`, whose only callers are one unit test.
The refusal is available at the exact site of the silent pick and is discarded.

## The population, measured — and the grain correction

An earlier draft of this note said "5,783 reference sites". **That was wrong and
the error was 6.5x.** `tools/namespace_cut/binding_identity_oracle.py` emits one
row per `(file, imported name)` pair and decides membership with `re.search`, so
it records PRESENCE, not occurrences. Corrected:

    (file, name) pairs                     5,783
    bare occurrences, total               38,694
      inside string literals                1,047   (2.7%)
      in code                              37,647
    distinct names                            348
    files                                   1,913

Of those pairs, only 6 had been qualified.

## Two names are half the population, and neither is a rewrite subject

    Node   9,893
    List   7,573

46% of in-code occurrences. `List` is part of the canonical container surface
(`List`, `Set`, `Map`, `Witness`) rather than an ordinary runtime-dispatched
module item, and blanket-qualifying it because the declaration census finds
homonyms would be wrong. `Node` needs the same scrutiny before it is touched.

This is why the raw census is an upper-bound INDEX and not an edit manifest.

## `import_said` is a candidate generator, NOT an oracle

Each row carries `import_said` — the module the file's import block named for
that spelling before the cut. It is strong evidence of intended authority and
the right way to PROPOSE a replacement. It cannot CERTIFY one:

- it is file-grain while binding is occurrence-grain, so one imported spelling
  can coexist in the same file with a generic parameter, a lambda parameter, a
  pattern binder, a local declaration, and prose (the `C` class is exactly this);
- import syntax records exposure, not resolved-node identity — it establishes
  no declaration kind, variant parent, expected type, or lexical precedence;
- the declarer index it is checked against is itself regex-derived, so the 207
  rows where `import_said` is absent from the declarers may be index omissions
  rather than judgement calls.

### And "a wrong qualifier fails loud" is FALSE as a general property

This note previously argued a bulk rewrite was safe because a bad qualifier
announces itself, citing `parse_diagnostic` — qualified to `v2.std.algebra` from
main's import binding, refused, corrected to `std.algebra`.

That was one lucky case, not a law. It failed loudly because the wrong authority
was INCOMPATIBLE. A wrong target survives silently when it has the same
parameter surface, a structurally compatible record shape, a same-shaped variant,
the same broad return type, or is simply never reached by the measured consumer.

The `emit` incident proves the point rather than refuting it: five declarations
shared one leaf name and the runtime picked by map insertion order. That
particular winner had incompatible parameters, so it complained. A
compatible wrong winner would have executed.

## The oracle that does hold

For each authored occurrence o:

    pre-cut resolved declaration identity(o)
      == proposed qualified target
      == post-cut resolved declaration identity(o)

where identity carries declaring module, declaration node or stable symbol,
declaration kind, and parent coproduct where relevant. `import_said` supplies the
middle term; the two resolvers verify it.

## Sequence

1. Expand pairs into exact occurrences with parser-provided spans and reference
   roles. Prose then disappears by construction instead of by regex exclusion.
2. Partition structurally BEFORE resolving: prose/comment, embedded DAG-source
   payload, lexical/local/generic binding, kernel primitive, canonical container,
   runtime function/data reference, type reference, record constructor, variant
   constructor, match-pattern constructor, unknown position.
3. Capture pre-cut binding identities from the pinned import-era resolver — do
   not infer them from the import block when the compiler can report them.
4. Rewrite the runtime-dispatchable population first (calls, function values,
   data reads, entry references). Highest harm: it can silently execute the
   wrong code. The nine `emit` qualifications are the exemplar.
5. Rewrite types and constructors using declaration kind, expected type and
   parent coproduct — NOT the function-runtime rule.
6. Only then recompute diagnostics and treat what survives as modeling work.

## Acceptance, at the right grain

    ambiguous bare runtime-dispatch occurrences = 0
    runtime selected-identity mismatches        = 0
    ordinary string literals changed            = 0
    comments/annotations changed                = 0

Not a diagnostic count, and not the peer's 72,480 `(name x scope)` exposure
census — that moves with reach as well as with resolution.

## Code-as-data is a separate, real defect

Some `String` rows carry DAG source or generated source for another consumer, so
a blanket string-literal exclusion is necessary but not sufficient. Source text
and prose are both undifferentiated `String`, forcing every tool to infer the
consumer. The durable fix is a branded carrier (`DagSourceText` / `ProseText` /
`GeneratedSourceText`) or an explicit specimen roster. It is recorded here and
deliberately NOT solved inside this rewrite.
