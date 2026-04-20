### No short-term solutions (this is not a production codebase)

**gunbc is not production.** There are no external users running
compiled binaries in the wild, no uptime commitments to keep, no
downstream teams whose releases depend on a stable API surface, no
breaking-change negotiations to manage, and no migration windows
that need to span multiple releases. Every refactor can be atomic.
Every API change can land in one PR that updates every caller.
Every representation change can sweep every consumer in one push.

**There is therefore no legitimate reason to introduce short-term
solutions** — adapter functions, deprecated APIs preserved
alongside their replacements, compatibility shims, feature flags
that gate half a migration, `TODO(M2): remove` markers on whole
code paths, scaffolded states with tracked dissolution triggers,
bridges between old and new data shapes, fallback code paths that
"just work" while the real fix is built.

These patterns exist in production codebases because production
codebases can't afford to break N million users in one change.
gunbc cannot break anyone. The patterns have **no defensible
motivation here** and every observed instance has calcified
instead of dissolving. The specific rules below (no bridges, no
deprecations, no parallel implementations, no fallbacks that
fabricate) are instances of this meta-principle.

**The rule:** every representation change, API change, or
refactor lands as a single atomic PR that updates every affected
consumer. If that PR is too large, the fix is to **split the
change into smaller atomic changes** — never to introduce a
transitional state with tracked removal.

**The test:** does the PR introduce any of the following?

- A new representation alongside an old one, with an adapter
  between them ("no bridges" violation)
- An old API preserved alongside its replacement, marked
  deprecated or conditionally active ("no deprecations" violation)
- Two separate implementations of the same computation
  ("no parallel implementations" violation)
- A feature flag that enables half a migration with a plan to
  flip it later (any form of gating a half-done change)
- A code path labeled "scope-bound," "dissolves in M2+,"
  "transitional," or "until X lands" — where the cleanup is
  deferred to a future commit that isn't in this PR

If yes to any, the PR is introducing a short-term solution and
violates this invariant. The fix is to do the rework in the same
PR, or to split the representation change into something smaller
that doesn't need transitional state.

**The excuse filter:** "but the refactor would be too large for
one PR" is almost always wrong. The refactoring cost that the
short-term solution is supposed to defer is exactly the
refactoring cost the solution is written to avoid — the cost
isn't reduced, just rewritten as "someone else's later problem."
And in a codebase where "later" is "whenever the current
milestone finishes," that's equivalent to "indefinitely."

If the rework genuinely cannot fit in one PR, the representation
change is the wrong size. Split the representation change into a
smaller one whose consumers can all be updated atomically — not
into "new representation now, old representation also still
here, delete the old one later."

**The honest exceptions:** there are two cases where something
that looks like a short-term solution is allowed.

1. **Emission into a target language via a declared language
   spec.** The compiler emits target-language source code, which
   is a different representation from the internal Node tree.
   That "conversion" is the whole job of the emitter (see
   "coercion = emission" in THESIS.md). Test: if the output is
   consumed by another part of the compiler, it's a bridge and
   is forbidden. If the output is target source code via a
   language spec declaration, it is emission and is allowed.
2. **Scaffolded primitive realization** (see THESIS.md §"Two
   groundings" and the `ArrowBody` enum in
   `src/v3/compiler/src/dag.rs`). Primitive Arrows may carry
   `ArrowBody::Pending` in the short window between
   substrate-shape commitment and extdeps language spec
   declarations landing. This is tracked via a
   monotonic-decrease CI ratchet. The exception is narrow and
   explicit: only primitives, only during the specific
   M1(2.5) → M3 transition, only covered by a numeric CI
   ratchet that strictly decreases.

Any other pattern that looks like a short-term solution is not
one of these two exceptions and is forbidden.

**Encountering one in existing code is an alarm signal, not a
routine cleanup.** If you spot a bridge, a deprecation marker, a
`TODO: remove in M2` on a live code path, a `_legacy` suffix, or
any other short-term-solution pattern — even in code you were
not planning to touch, even in a file adjacent to the one you
are working on, even while you were reading the code for
entirely unrelated reasons — **stop and raise it.**

The correct response is NOT:

- "This is not in my immediate task, I will note it and keep
  going."
- "I will fix it quietly in this PR while I am here."
- "Someone probably knows about it, so it is fine."
- "It is tracked in a comment already, so I can trust the
  tracking."

The correct response IS:

- **Stop your current work** long enough to understand what you
  are looking at. Read the surrounding code. Figure out when the
  short-term solution was introduced and what it was meant to
  unblock.
- **Treat it as concerning**, not as a minor nit or a
  "could-clean-up." The tone matters: describe it as a
  **structural signal that something went wrong upstream**, not
  as cosmetic debt. Normalizing the language ("just a small
  bridge," "only a minor deprecation") is how these patterns
  calcify.
- **Raise it to the reviewer or the implementer** of the code
  that contains it — even if they are a different person than
  whoever is reviewing your current PR. The bridge exists
  because someone's earlier design decision did not close a
  migration cleanly; that person is the one who should hear
  about it first.
- **Back up and assess the damage.** Does this one instance
  exist in isolation, or is it a symptom of a broader pattern?
  Are there more? What does its existence tell us about the
  current state of the subsystem? What is the root cause that
  made the short-term solution seem necessary, and does that
  root cause still apply?
- **Work on the diagnosis before the fix.** "Here is the bridge
  and here is a quick patch" is the wrong move. "Here is the
  bridge, here is what I think went wrong upstream, here is
  what I think the full cleanup scope is, and here is what I
  propose to do about it" is the right move.

The instinct to "get my own work done and flag it later" is
exactly how short-term solutions calcify. Every bridge and every
deprecation that survived to dominate a subsystem was something
that someone noticed in passing and chose not to escalate. This
invariant is not satisfied by "I personally didn't add any new
ones" — it is satisfied by "nobody saw a bridge without flagging
it." **If you see one and do not raise it, you are endorsing
its continued existence.**

What "raise it" looks like concretely:

- **In PR review:** a blocking review comment citing this
  section. Not a nit. Not a "future work" tag. A stop-sign.
- **During your own implementation:** stop the implementation
  branch, open an issue or a discussion describing the bridge
  you found, estimate what it would take to remove it, and
  decide whether to fold the removal into your current PR or
  to block your PR on its removal first. Do not keep coding
  around it and circle back later.
- **In a file you were passing through for unrelated reasons:**
  a short note: "I was reading X for unrelated work and noticed
  Y. We should discuss before I go further." Silence is not
  correct.
- **When found by a reviewer agent or an automated scan:** a
  loud failure, not an informational note. Whatever mechanism
  caught it should halt the workflow, not log and proceed.

The pattern this rule is trying to eliminate is
**normalization** — the state where bridges and deprecations
exist in the codebase and everyone walks past them. Normalization
is the terminal stage of calcification. The escalation rule is
what keeps normalization from starting.

