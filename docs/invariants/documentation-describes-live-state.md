## Documentation Describes Live State

Repository documentation (design notes under `docs/`, roadmap sections, and
similar) must read as a **current specification**: what the system is, which
constraints bind it, and what work remains. Docs must not double as **project
chronicles** — PR-by-PR narration, §Revision history blocks, round-by-round
convergence records, changelogs of review threads, or full preserved copies of
superseded proposals.

**Why:** Readers and automated reviewers pattern-match stagnant prose. A doc
that foregrounds history can read abandoned even when the technical content is
current; history also diverges from git, which remains the authoritative record
of how decisions evolved.

**Narrow exceptions:**

- **Stable external names** — A single line tying a doc to a roadmap milestone
  or design ID (e.g. “Supersedes DB-9 R1”) is fine when other artifacts point
  here by that name.
- **Load-bearing rationale** — When newcomers might still propose a rejected
  approach, a short table row or paragraph explaining *why* it stays out of
  scope is live content. Prefer that over archiving entire prior drafts in place.
- **Stamped checks** — A dissolution receipt or similar ritual that states the
  **current** structural verdict (pass/fail and why) belongs in the doc; the
  review conversation that prompted the check does not.

**Rule of thumb:** If removing a paragraph would not change anyone’s ability to
implement or review the **present** design, it belongs in git history or the
tracking issue, not the doc body.

**Relation to tests:** This aligns with source-audit tests anchoring on live
syntax and declarations — comments and historical notes are not structural
evidence (see Testing Invariants).

**Scope — implementation comments:** This section governs **checked-in
documentation** (`docs/`, roadmap, similar). It does not impose the same
mechanical bar on every inline comment in `src/`. The Testing Invariants
cross-reference means **tests** must not treat comments as authoritative
structure; it is not a substitute for a future, explicit rule on chronicle-style
`//` prose. Audit passes that apply “Documentation Describes Live State” need
not rewrite implementation comments unless a companion invariant or team
standard says otherwise.

