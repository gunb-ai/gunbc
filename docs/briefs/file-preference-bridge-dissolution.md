# File-preference bridge dissolution `(M)`

## Problem

The compiler has a **file-preference resolution rule** that prefers `src/v3/` declarations over `dsl/` duplicates for same-named declarations:

- `src/v3/compiler/src/dag.rs:1985-2015` — `declaration_name_preference_rank`: ranks v3 above dsl (`v3=2, dsl=0, other=1`), and `declaration_by_name` uses it to select among same-named declarations.
- `src/v3/compiler/src/lower.rs:1258-1274` — duplicates the same policy while seeding the symbol table.

The policy exists because *there are live duplicate module authorities* in the tree:

- `module std.effects` declared in both `dsl/std/effects.dag:39` **and** `src/v3/std/effects.dag:112`.
- `module std.verification` declared in both `dsl/std/verification.dag:22` **and** `src/v3/std/verification.dag:17`. (The v3 file at `:8-15` explicitly comments that the duplication is intentional-scaffold and should converge.)
- Inside `src/v3/std/effects.dag:118-260`, an *embedded mirror* of `dsl/std/http_path.dag` exists because "the v3 staged-file set does not yet include `http_path.dag`" (verbatim from the file's own comment).

This is a C-checkpoint-scale P2 Boundary Discipline violation: parallel authority at compiler-config level, *ratified* by a lookup policy rather than dissolved. The policy doesn't just tolerate duplicates — it encodes a stable resolution rule that normalizes them, which is exactly what "no bridges" forbids.

It also catches multiple invariants at once: P1 Modeling Faithfulness (the preference rule is an internal taxonomy not grounded externally), P2 Boundary Discipline (two authorities for one fact), and P5 Progress Is Dissolution (the v3 verification file's own comment admits it's a scaffold with convergence pending).

## Read first

- `src/v3/compiler/src/dag.rs:1985-2015` (rank fn + declaration_by_name)
- `src/v3/compiler/src/lower.rs:1258-1274` (duplicate policy in symbol-table seeding)
- `dsl/std/effects.dag` vs `src/v3/std/effects.dag` (diff the two module declarations)
- `dsl/std/verification.dag` vs `src/v3/std/verification.dag` (same; v3 file's :8-15 comment is the dissolution receipt)
- `src/v3/std/effects.dag:118-260` — the embedded http_path mirror
- INVARIANTS.md §P2 Boundary Discipline (especially *Parallel authority* problem shape)

## Work

1. **Consolidate `module std.effects`.** One canonical location. The v3 file has additional imports (`v3.std.substrate`) and the embedded http_path mirror; the dsl file is the older authority. Pick the canonical home — merge any v3-only content into the canonical file, delete the other — such that only one `module std.effects` declaration remains in the tree.
2. **Consolidate `module std.verification`.** Same pattern. The v3 file's `:8-15` comment names convergence-to-one as the target.
3. **Resolve the embedded `http_path` mirror.** Two options (pick based on scope):
   - **(a) Stage `http_path.dag` in v3** and let the consolidated `std.effects` import from it. (Prerequisite lane if scope is too big.)
   - **(b) Have `std.effects` depend on the canonical `std.http_path`** directly without an embedded mirror. (Smaller, preferred if imports allow.)
4. **Delete `declaration_name_preference_rank`** in `dag.rs:1985-2015`. Update `declaration_by_name` to fail-closed on multiple matches: `None` for zero, `Some` for exactly one, **typed diagnostic (not `None`)** for multiple. "Return the first match" is a cryptic preference rule — iteration order becomes the hidden selection criterion, which is strictly worse than the explicit rank since the behavior is invisible to readers. Fail-closed is the only principled target.
5. **Delete the duplicate policy** in `lower.rs:1258-1274`. Symbol-table seeding also fail-closes on multiple matches; any remaining duplicate surfaces as a structural error, not a silently-resolved scaffold.
6. **Run the full test suite.** Any remaining duplicate-name failures surface as typed diagnostics now — each must be resolved structurally, not by re-introducing a preference rule.

## Acceptance

- `git grep "^module std\.effects"` returns exactly one result
- `git grep "^module std\.verification"` returns exactly one result
- No embedded path-template mirror inside `src/v3/std/effects.dag`
- `declaration_name_preference_rank` deleted from `dag.rs` and `lower.rs`
- **`declaration_by_name` emits a typed diagnostic (not `None`, not silent first-match) when multiple declarations share a name** — `None` for zero, `Some` for exactly one, diagnostic otherwise. This is the dissolution target's load-bearing invariant, not a nice-to-have: returning any "winner" from multiple candidates re-introduces the preference rule by another name.
- **`collect_symbols` in `lower.rs` similarly fail-closes on duplicate names** — symbol-table seeding surfaces a structural error, not a silently-resolved scaffold.
- A regression test exists that constructs two same-named declarations and asserts both lookup paths diagnose (defensive against future preference-rule re-introduction).
- Full test suite green (including self-parse / bootstrap)

## STOP-AND-ESCALATE

- **If removing either v3 duplicate requires moving substrate content** that doesn't exist in the dsl copy (e.g., `v3.std.substrate` imports, `PortId` / `ElementRef` references), STOP. That's a substrate-staging lane, not this cleanup. Surface the specific missing v3-substrate types.
- **If option (a) — staging `http_path.dag` in v3 — turns into a multi-file restructure**, STOP. Switch to option (b); if (b) also fails, surface the blocker.
- **If deleting `declaration_name_preference_rank` breaks self-parse / bootstrap / emit tests via other duplicate names I didn't catalog here**, STOP. Dump the failing cases; each is an additional dissolution item, likely trivial once identified, but worth getting agreement on scope before chasing.

## Non-goals

- Not fixing the broader `declaration_by_name` debt (emit paths that still do name-lookup lookups instead of DeclarationId) — that's tracked separately.
- Not restructuring v3 substrate imports.
- Not touching other duplicate-module cases beyond the two named here.
- Not touching tests beyond what's needed to get green after the dissolution.

## Size

M. Two module consolidations + embedded-mirror resolution + two small deletions in Rust. Could land smaller if the duplicated modules turn out to be structurally identical (straightforward delete-the-loser). STOP-AND-ESCALATE is the scope safety net.

## Dispatch note

This is a direct P2 Boundary Discipline receipt candidate post-merge: the PR body should frame the change as "deleted a ratified-parallel-authority pattern" and cite the invariant. If the lane reveals that convergence requires prerequisite work (substrate staging, http_path migration), raise them as separate lanes rather than expanding scope here.
