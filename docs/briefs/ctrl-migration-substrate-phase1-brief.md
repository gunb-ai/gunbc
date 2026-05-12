# Ctrl-Migration Substrate Phase 1 Brief

**Status**: READY-FOR-DISPATCH once the Substrate Mgr exists.

**Authority**: PR #2775 companion doc `docs/design-decomposition-algebra.md`, especially §2, §5, §6, and §9.

## Output

Author `dsl/std/process_algebra.dag` as the staged substrate for ctrl decomposition algebra:

- `ProcessNodeId`
- `ProcessNodeMode` (`Leaf`, `Composite`, `Bucket`, and staged undeclared/null handling if needed)
- `ProcessOperation`
- `ProcessClosureDecision`
- `ProcessClosureRefusal`
- `Attestation`
- typed drain/replan/escalation evidence

## Scope

This is not a ctrl runtime cut-over. The file is 🟡 STAGED until:

1. an emission target consumes it,
2. parity tests prove the generated behavior matches ctrl PRs #1192/#1193/#1195/#1197 behavior,
3. the ctrl TS decomposition files are deleted or made generated-only.

## Mandatory Audits

Before naming any carrier, grep and cite non-reuse/reuse against:

- `dsl/std/`
- `src/v3/std/`
- `src/v3/lenses/`
- `dsl/gunbc/workflow/types.dag`

Known hazards:

- Do not define a new `Witness`; v3 already has `Witness<Carrier>` in `src/v3/std/dimensions.dag`.
- Do not define a parallel bidirectional `Lens<S,A>`; v3 already has `Lens<C>` in `src/v3/std/lens.dag`.
- Preserve workflow phase and run-keyed facts as separate axes; do not collapse them into node mode.

## Acceptance Gates

1. Every enum/sum with at least two variants has a Practice 4 receipt.
2. The file explicitly marks each staged carrier and its dissolution trigger.
3. Closure checks are fail-closed: undeclared post-cutoff mode, undrained bucket, and composite-with-open-children are representable refusals.
4. Operation effects are typed per operation; no blanket monotonicity claim.
5. The PR body names the ctrl TS files that remain current runtime authority.

