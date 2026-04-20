### Verification predicates are substrate consumers

Any predicate in `std/verification.dag` (or equivalent verification
authority) that asserts a property of the compiler's output must
reference the compiler's own authoritative fact as a typed edge.
Parallel enum/string schemas for the same concept the compiler
already models are forbidden — they are shadow taxonomies, not
verification.

**Why this is a separate invariant.** "No duplicate representations"
forbids duplicate facts in general. This invariant specializes that
rule to the verification surface, where the temptation is strongest:
testgen needs a "view" onto compiler facts, and the cheapest local
move is to sketch a small enum or string-extraction helper
("kind + detail_contains") that approximates the real authority.
Each such approximation becomes a bridge that drifts as the real
authority evolves, and because verification is downstream of every
compiler fact, the bridges multiply quickly.

**The test:** if a test assertion would change meaning after a
rename or restructure of the authoritative compiler type (Diagnostic,
PortState, Declaration), the predicate is consuming a shadow schema,
not the substrate fact. A predicate that references `DeclarationRef`
or equivalent typed edges cannot silently drift; one that references
`DiagnosticKind::TypeMismatch` as a parallel enum can.

**The fix:** replace parallel enums with typed references into
substrate. `DiagnosticReference.kind: DiagnosticKind` (parallel enum)
becomes `DiagnosticReference.kind: DeclarationRef` (typed edge to
the real diagnostic variant declaration). String extraction helpers
(`diagnostic_detail(...)`) become structural field reads.

**Bounded scaffolds are allowed with named triggers.** If substrate
doesn't yet expose the fact in a consumable form (e.g., Diagnostic
isn't reflected into std/ yet), a shadow taxonomy can exist as a
bounded scaffold — but only with an explicit `🟡 Scaffold` marker
naming the dissolution trigger. "Dissolves when substrate.Diagnostic
becomes consumable by std/verification.dag" is load-bearing
documentation; without it, the scaffold is indistinguishable from a
permanent parallel authority.

**Historical context (2026-04-16):** PR #481's verification surface
rework dissolved the port-state and cost-bound axes (flat variants
→ structural carriers using substrate's `ComparisonOp`), but
introduced `DiagnosticKind` as a parallel enum to the compiler's
native Diagnostic type, with string-extraction helpers
(`diagnostic_kind()`, `diagnostic_detail()`) in test code bridging
back to the real fact. Both reviewers (codex + ChatGPT-browser)
independently flagged the same pattern. This invariant graduates the
finding so future verification-surface changes have a named
structural rule to test against.

**Structural prevention:** typed edges are the shape. When a
verification predicate needs a compiler concept, the substrate
reflects the concept as a declaration, and the predicate carries a
`DeclarationRef` to it. The compiler's own `meta_tag` / `inhabits`
machinery makes this typed edge equivalent to "this predicate is
testing the same concept the compiler uses internally." The escape
hatch is parallel enums with matching variant names; the fix is
refusing to write them.

