### No deprecations

A **deprecation** is any pattern that keeps an old API, data
structure, or representation alive alongside its replacement with
the intent to remove it "later." Concrete forms: `@deprecated`
annotations, `_v1` / `_legacy` / `_old` suffixes next to fresh
names, feature flags that toggle between "new" and "old"
behaviors, type aliases that re-export old names from new
modules, `TODO: delete this function when X lands` comments,
parallel function bodies selected by a runtime flag.

**Deprecations are forbidden. Do not introduce them. And if you
find one already in the codebase — even in a file you were
passing through — do not silently work around it. Raise it as
an alarm signal per §"No short-term solutions."** See that
section for the meta-principle and the escalation procedure.
Production codebases tolerate deprecations because they can't
afford to break external consumers in one release, and gunbc
has no external consumers. The refactoring cost a deprecation
defers is exactly the refactoring cost the deprecation was
written to avoid.

Deprecations are the close cousin of bridges: a **bridge**
translates between two representations of a fact that exist
simultaneously; a **deprecation** keeps an old callable/type
alive alongside its replacement so callers have time to
migrate. The failure mode is identical — the old form calcifies
because every new caller learns the presence of both
alternatives and becomes dependent on whichever one was
convenient at the time of writing. By the time the "delete old
form" commit lands, reworking every consumer has become a
bigger task than reworking every consumer at the introduction
time would have been.

**The test:** does the change introduce two versions of the
same function, type, API, or module name, where one is labeled
or intended as the "new" one and the other is labeled or
intended as the "old" one? Signs to look for:

- `@deprecated`, `#[deprecated]`, `// DEPRECATED`, or similar
  annotations.
- Names with `_v1` / `_v2` / `_old` / `_legacy` / `_new` suffixes.
- Re-exports of the form `pub use new_module::NewName as OldName`.
- Function signatures that take a boolean flag named `use_new`,
  `legacy_mode`, `v2_dispatch`, or similar.
- Match arms labeled `// old path` and `// new path` in the same
  function.
- Comments saying "keep this until X migrates" or "delete after
  M3" on callable code (not comments on data that's actively
  being transitioned via a ratchet).

**The rule:** when you rename a function, change an API shape,
or replace a type, every caller updates in the same PR. There is
no "introduce the new form, migrate callers over N PRs, delete
the old form at the end." There is only "introduce the new
form with every caller already using it, in one PR, with the old
form deleted."

**The fix when you've already written one:** back out the
deprecation and do the rename/replacement as a single atomic
change. If the rename touches many callers, the refactor is big,
but it is exactly the refactor you were going to do eventually
— doing it now is cheaper than doing it later with additional
consumers that learned the deprecated form in between.

**The fix when the rename genuinely spans multiple independent
subsystems:** the representation or API change is the wrong
size. Split it into smaller changes where each rename is
atomic within its subsystem. Do not split by "new name first,
old name deleted later" — split by "these five callers get the
new name in PR A, these four get it in PR B, and the PRs are
independent because A's callers don't import B's."

**Structural prevention (future):** CI audit on every PR that
grep-matches the deprecation signals above (annotations,
suffixes, naming patterns, comment phrases). Zero matches
required. Until the audit lands, enforced by code review —
any reviewer can veto with a reference to this section.

**Exception:** versioned external protocols (wire formats,
persisted data schemas with existing content on disk). These
are outside the closed system — a change to them can't be
atomic because the other side is outside gunbc's control.
Protocol versioning is the one place where keeping an old form
alive alongside a new form is the honest answer. Test: if the
"old" thing is consumed only by gunbc itself, it's a
deprecation and is forbidden. If the "old" thing is consumed by
an external protocol peer, by a file format with existing data
on disk, or by a declared language spec target, it's protocol
versioning and is allowed.

