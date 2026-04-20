### E-9: External realization lives on Arrow.body (2026-04-17)

A callable is externally realized if and only if its
`Arrow.body` is `ArrowBody::ExternalRealization(ref)`. **No
auxiliary mechanism** — `meta_tag`, naming heuristics, spec-side
lookup existence, module location, or convention-based stub-body
detection — can mark a callable as externally realized. Emission
MUST dispatch on `Arrow.body` variant, not on side markers.

**Why this is a separate invariant.** The thesis already says the
substrate has two meeting points — `Transform → Arrow` (the call)
and `Arrow → body` (the implementation kind). E-9 forbids
introducing a third meeting point at the side (meta_tag or
parallel lookup table) that duplicates the fact "this callable is
externally realized." A future reader navigating the substrate
must reach that fact through a single structural path: the Arrow's
body variant.

**Why the temptation is real.** Target dispatch for externally
realized callables depends on the active target, which the
substrate (correctly) doesn't know about at bootstrap or parse
time. The tempting move is to declare "external" as a
substrate-adjacent fact (meta_tag, name convention) and postpone
the structural choice until emission. That move looks like it
preserves substrate minimality, but it splits authority: the Arrow
body says "I'm a normal fn with a stub body," the side marker
says "actually I'm external." Two sources of truth, one guaranteed
to drift.

**The correct pattern.** `ArrowBody::ExternalRealization(ref)`
carries a DeclarationRef to a target-neutral identity declaration
(an "accessor marker" for substrate accessors; a
`CompilerHostRealization` for pipeline stages; analogous
structures for future cases). Per-target spec files declare
realizations that reference the same marker. Emission dispatches
on `Arrow.body`, walks to the marker, finds the matching
realization for the current target in the active spec.

**Historical context (2026-04-17).** DB-14 (substrate external
primitives) landed an earlier design where "external" was encoded
as `meta_tag` + per-target spec search, with the Arrow body
remaining an ordinary user-defined stub. ChatGPT-browser, codex,
and meta-review all independently flagged the same authority-split
shape. The PAUSE_AND_REGROUP meta-verdict prescribed banking the
rule before redesigning. This invariant is that rule; DB-14's
rewrite consumes it.

**Structural prevention.** At PR review, any new introduction of
"external realization" (or analogous host-backed callable pattern)
must place the structural fact on `Arrow.body`. If the design says
"Arrow body stays Foo and a side marker indicates externality," it
violates E-9 and needs rework before landing.

**Bounded exception.** None. This is a structural rule. The
substrate's two-meeting-point architecture is load-bearing for the
self-hosting cycle: `compiler.dag` walks Arrow bodies to translate
calls; a split authority means the self-hosted compiler either
duplicates the heuristic or silently misses externally realized
callables.

