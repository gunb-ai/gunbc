# R3 Wave-1 S2 — Quantifier + Generator carriers (#85 + #86 bundled)

**Owner**: Wave-1 Substrate worker (to be assigned by spawn)
**Authored by**: warm-wolf-698 (R3 Substrate Mgr)
**Authoring date**: 2026-05-12
**Bundle rationale**: per `feedback_bundle_workstreams_per_pr` + `feedback_single_bundle_ratification_uniform_substrate_cause` — Tests-as-Data Cluster M Phase 1; locked-design-resolved; substrate-cause uniform (carrier landings for quantifier/generator surface).

---

## §0. Status — DISPATCH-READY (no prerequisites)

Both gates' shape is locked per `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md`. No canvas needed. Substrate-cause is uniform: land typed carriers for two adjacent Tests-as-Data axes.

In-flight precedent:
- **#85 substrate**: PR #2734 in flight (close out / coordinate; the carrier set may already be partially landed). Verify state of PR #2734 BEFORE adding any duplicate carrier; if PR #2734 has already landed `Quantifier` / `ForAll` / `Exists` / `QuantifiedTestClaim`, scope narrows to verification + completion gates.
- **#86 substrate**: PR #2752 DRAFT — close out / supersede. Same verification step.

## §1. Scope

**Authority**: `docs/design-tests-as-data-completeness.md` §2.1 + §2.2 + §2.3 (locked carrier shapes). Substrate lives in **`src/v3/std/verification.dag`** (existing single-authority extension per P2 — NOT a new module).

### #85 — `forall_exists_quantifier_substrate_landed`

Land the canonical carriers from `docs/design-tests-as-data-completeness.md`:

```
// §2.2 — closed two-variant Quantifier sum (forall / exists are the only structurally meaningful quantifications over a ProgramGenerator's output)
type Quantifier = ForAll | Exists

// §2.3 — QuantifiedTestClaim references a ProgramGenerator + property declaration
type QuantifiedTestClaim {
  quantifier: Quantifier
  generator: ProgramGenerator
  // property surface fields per design §2.3 (see source doc for full shape)
}

// §2.4 — SuiteClaim aggregator over List<QuantifiedTestClaim>
type SuiteClaim {
  claims: List<QuantifiedTestClaim>
  // suite-level fields per design §2.4
}
```

**Acceptance**: `Quantifier` + `QuantifiedTestClaim` + `SuiteClaim` carriers present in `src/v3/std/verification.dag` per design §2 shapes + at least one consumer-side wiring point per carrier (no scaffold-without-consumer per INVARIANTS P5 / modeling-discipline Practice 4). Closes gate #85.

### #86 — `program_generator_carrier_landed`

Land canonical carriers from `docs/design-tests-as-data-completeness.md` §2.1:

```
// §2.1 — ProgramGenerator is a structural reference to a generator declaration
// (NOT a closed roster of shape kinds — that's the lens-extensibility failure flagged by
//  docs/lens-library-design.md §1.5)
type ProgramGenerator {
  generator: DeclarationRef
}

// §2.1 — ProgramShape coproduct; single-variant at bootstrap (LiteralProgram)
// per INVARIANTS P1 coproduct-vs-coordinate honesty; future variants
// (ParameterizedProgram, SubstrateDerivedProgram) added when consumer demand exists
type ProgramShape
  = LiteralProgram { source: String, file_name: String }
  // future: ParameterizedProgram { ... }
  // future: SubstrateDerivedProgram { ... }
```

**Acceptance**: `ProgramGenerator` + `ProgramShape` declared in `src/v3/std/verification.dag` per design §2.1 shape + at least one consumer-side wiring exercising the `LiteralProgram` variant. Closes gate #86.

## §2. STOP conditions (per brief P2/P3 discipline established in PR #2762 / PR #2774)

1. **Pre-existing carrier collision** — if `Quantifier` / `ProgramGenerator` / `ProgramShape` name is already declared in `src/v3/SELF_HOSTING.md`, `dsl/std/`, or `dsl/extdeps/`, **STOP**. Surface to warm-wolf-698 with the collision details. (Run authority-audit grep per `feedback_self_hosting_md_authority_audit_before_naming` BEFORE declaration.)
2. **Locked-design ambiguity** — if `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md` does NOT spell out the field shape for either carrier at the level of detail you can land without inference, **STOP** and surface — locked-design-resolved means substrate-canvas isn't needed, not that detail is missing.
3. **Consumer-wiring absence** — if you can't find a single legitimate consumer-side wiring point (test, fold, projection), **STOP** — scaffold-without-consumer is a P5 violation per `feedback_pattern_a_scaffold_sentinel_per_instance_ratification`.

## §3. Verification before PR-ready

- `cargo test --workspace`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt --all --check`
- Grep audit receipt in PR body: `grep -n "type Quantifier\|type ProgramGenerator\|type ProgramShape\|QuantifiedTestClaim" src/v3/SELF_HOSTING.md dsl/std/ dsl/extdeps/` returns the new declarations and only the new declarations.

## §4. PR body framing

- Cite gates #85 + #86 explicitly as closures
- Cite `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md` as authority for the locked design
- If PR #2734 / PR #2752 are superseded by this PR, explicit cross-link with "supersedes #NNNN" framing per `feedback_redirect_noop_prs`

## §5. Out of scope

- #84 Phase 3 bulk-port (Verification lane V2 worker; depends on these carriers)
- #87 Phase 2 cementing-discipline pattern (Verification lane V1 worker)
- Cluster-F gates (#81/#82/#83/#95) — separate Substrate workers

## §6. Reference

- `docs/audit/r3-cluster-m-sequencing-plan-2026-05-09.md` — Cluster M sequencing authority
- `docs/r3-remaining-work-dependency-graph.md:34,120-121` — gate-row metadata
- PR #2734 (in flight #85 attempt) + PR #2752 (DRAFT #86 attempt) — coordinate / supersede
