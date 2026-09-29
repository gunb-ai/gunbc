# Preserve the established dependency retirements

Source d352e651c1ff93786cae0462328c60a5d49fa41d. CI run 36514441574 had
cleared the previous bare-provider findings but refused seven resurrected debt
rows relative to origin/main. The attached prior-ci-refusal.json is the actual
published receipt. This source supplies the missing explicit dependencies and
receiver call, and preserves those retirements. No compiler/gate implementation
or debt expansion was introduced.

Under 6 GiB/no swap, existing authorization-state controls pass (2), census
controls pass (13), and compile-ledger ownership controls pass (5). Synthesis
initially exposed a missing AdvisoryLens import; the same explicit import already
on main was added. Its next attempt typechecked but the selected fixture defines
`test data`, not `test fn`, so --claim-run did not execute a claim. The retained
one-function adapter calls that fixture's existing synthesis_optimality_family_gate
without reimplementing it; this control passes (1). The adapter is additional to
87 byte-identical dependency modules; other closures were 54 and 264 modules.
The two unsuccessful synthesis invocations are retained rather than counted as
passes. No Rust binary change was needed.

Integrated run 36519516794 is pinned to this source and remains pending. Full
server startup on this exact clean source also passed; the included preview
receipt records its timing, binary identity and memory measurements under the
unchanged 12 GiB/no-swap serving test cap. No deployment or VM launch
has occurred. This receipt does not establish the missing production consumer,
commissioning or access-refresh connections.
