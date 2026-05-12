# Ctrl-Migration Day-1 Manager Dispatch

**Status**: ACTIVE — Director dispatch artifact for `node://adhoc-dc298bc7-9f7` (2026-05-12).

**Charter authority**: PR #2775 (`docs/design-decomposition-algebra.md` + `docs/r4-ctrl-dag-migration-project-plan.md` on branch `design/decomposition-algebra-draft`). The charter is binding from PM dispatch even while the doc PR is still open.

**Mission**: replace `ctrl/` TypeScript implementation with `.dag` substrate as fast as the substrate/emission/parity trio allows. Every `.dag` subsystem model lands as 🟡 STAGED until emission target + parity test + cut-over PR deletes the TS authority.

## Manager Work Items

The Director attempted to create the four Day-1 manager work items with `dashboard-ops work-items create`, but the dashboard CLI currently fails from this container with:

```text
dashboard-ops: curl failed (network/DNS/connect error)
```

Once dashboard transport is restored, create these four work items:

1. **Ctrl-Migration Substrate Mgr — Phase 1 process algebra substrate**
   - Owns `dsl/std/process_algebra.dag` plus the `Attestation` carrier.
   - First output: Phase 1 worker brief from `docs/design-decomposition-algebra.md` §9.
   - Gate: M9 DFS trace to `dsl/std/`, `src/v3/std/`, and `src/v3/lenses/` for every carrier name.

2. **Ctrl-Migration Subsystem-Modeling Mgr — Phase 1.5 subsystem .dag contracts**
   - Owns the 16-subsystem catalog from project-plan §3.
   - First wave: items #3, #5, #8, #10, #11, #12, #14, #16.
   - Gate: each subsystem file names the current ctrl TS authority and the STAGED -> AUTHORITY trigger.

3. **Ctrl-Migration Emission-Targets Mgr — Phase 3 HTTP SQL audit extdeps**
   - Owns `dsl/extdeps/http/server.dag`, `dsl/extdeps/sql/migration.dag`, and `dsl/extdeps/audit/event.dag`.
   - First output: HTTP server extdeps brief.
   - Gate: no runtime cut-over claims until a subsystem projection consumes the target and parity passes.

4. **Ctrl-Migration Verification Mgr — Phase 4 parity tests and cut-over PRs**
   - Owns per-subsystem parity tests and TS deletion cut-over PRs.
   - First output: parity harness plan for review verdict, inbox, and API reviewer.
   - Gate: generated consumer receipt required before any staged model becomes authority.

## Director Enforcement Rules

- M9 DFS before naming any new carrier.
- Grep `dsl/std/`, `src/v3/std/`, and `src/v3/lenses/` before accepting a substrate proposal.
- Practice 4 receipts on every enum/sum with at least two variants, including closed sums.
- No parallel authority: reuse existing `src/v3/std/lens.dag::Lens<C>` and `src/v3/std/dimensions.dag::Witness<C>` rather than defining similarly named substitutes.
- `.dag` declarations are staging until the realization trio converges: emission target, parity test, and TS cut-over deletion.

