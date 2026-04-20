### Scaffold boundaries

Every substrate scaffold variant must have an **explicit unreachability
gate** that rejects it for user-range declarations before the compile
boundary returns `Ok`. A scaffold is any substrate form introduced to
accommodate a parser or lowering limitation — typically a variant
that says "this position has a fact the compiler can't yet validate"
(e.g., `ArrowBody::Unparsed(SourceSpan)` for block-body fns whose
body the parser cannot lower). Scaffolds are load-bearing for
std/bootstrap files whose richer content outpaces the current
parser surface. They are never acceptable in user code: letting a
user-range scaffold survive means ordinary programs can ship an
opaque body that the compiler never validates, violating
fail-closed static grounding.

**The principle:** a new substrate variant added to accommodate a
compiler limitation is not done when the variant is defined and
populated. It is done when the code that rejects it in user code
also lands. "Deliberate, documented, with a dissolution trigger"
is necessary but not sufficient — a tracked scaffold that applies
to user code is still a fail-open leak.

**The diagnostic:** ask "where does this scaffold become
unreachable?" for every scaffold variant. If the answer is "at
bootstrap time only, everything else tolerates it," the scaffold
is unbounded. If the answer is "nowhere — it's a general
placeholder," the variant is not a scaffold at all; it's load-
bearing substrate and deserves a full 4-pattern check plus
dissolution trigger, not scaffold status.

**The test:** compile a minimal user program that deliberately
produces the scaffold state (e.g., `fn foo(x: Int) -> Int { junk }`
for `ArrowBody::Unparsed`, `data foo: Int = { junk }` for
`ValueBody::Unparsed`). If `compile_to_dag` returns `Ok`, the
boundary is missing.

**The fix:** add a post-lowering sweep that walks declarations
at id `>= user_start` (the range of user-lowered declarations
after bootstrap) and emits a fail-closed diagnostic for each
scaffold variant found. Bootstrap-range declarations stay
tolerated by design. The sweep runs alongside
`resolve_pending_identifiers_strict` as a sibling gate.

**Structural prevention:** a per-scaffold regression test that
pins the boundary. The test compiles the minimal user-code
reproducer and asserts `compile_to_dag` returns `Err`. Adding a
new scaffold variant without such a test is a violation; the
test IS the boundary. In v3's test suite these are the
`m18_r14_user_*_is_rejected` tests for the two R9/M1(2.7)
scaffold variants.

**Ratchet:** a PR that introduces a new substrate variant (new
enum arm in `ArrowBody`, `ValueBody`, `BranchPattern`,
`AtomPayload`, etc.) must, in the same PR, either (a) prove the
variant is terminal substrate — 4-pattern check passes, no
dissolution trigger — or (b) prove the variant is a bounded
scaffold — unreachability gate landed, regression test landed,
dissolution trigger named. A new variant without one of these
two receipts is a blocker.

**Background:** this invariant was codified after R14 of PR #445,
when a reviewer found that `ArrowBody::Unparsed` (introduced in
M1(2.7) R9 for std/bootstrap block-body fns) had no user-range
boundary. User code could ship `fn foo(x: Int) -> Int { junk }`
and it compiled cleanly — the scaffold was "deliberate,
documented, with a dissolution trigger" at the variant level but
had no landing gate in lowering. `ValueBody::Unparsed` had the
same shape for data items. R14 added the sweep
(`reject_user_unparsed_scaffolds` in `src/v3/compiler/src/lower.rs`)
as the canonical form of the fix; this invariant generalizes that
fix into a ratchet on all future scaffold additions.

