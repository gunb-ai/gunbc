## Root-Cause Depth Invariant

This codebase is a DAG — both the language it compiles and its own
internal architecture. The purpose of every invariant in this document
is not just to flag violations where they appear, but to DFS the
dependency graph upstream from each violation until a sound node is
found. The violation lives at the deepest unsound node, not at the
leaf where the symptom was observed.

A downstream symptom — a heuristic, a duplicate, a fabrication — is
often correct code doing the best it can with what it received. The
bug is upstream: a missing fact, an incomplete type, a structure that
was never surfaced. Fixing the leaf treats the symptom. Fixing the
deepest unsound ancestor treats the disease.

**The rule:** when reviewing or diagnosing, do not stop at the first
node that looks wrong. Walk every parent in the dependency chain and
check each for violations. The fix belongs at the deepest node where
an invariant is broken. If a downstream stage needs information that
isn't available, the fix is to surface that information from its
origin — not to re-derive, guess, or hardcode it at the consumption
site.

**The test:** for any proposed fix, ask: "does this fix a root cause,
or does it compensate for a broken ancestor?" If the fact it relies on
originates in a producer (core types, parse, resolve, infer) but the
fix is in a consumer (emit, complexity, ownership), the fix is in the
wrong place. Move the fact upstream.

**The connection to other invariants:** "No duplicate representations"
is a consequence — duplicates arise when a downstream stage re-derives
what should come from upstream. "Heuristics indicate lost structure" is
a consequence — heuristics arise when upstream structure was never
surfaced. "No fallbacks that fabricate" is a consequence — fabrication
fills the gap left by a missing upstream fact. This invariant is the
shared root cause of all three.

