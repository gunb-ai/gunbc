# Allocation-store planning integration

The allocation-store CLI previously wrote an artifact through the legacy planner without the run-bound receipt consumed by the current apply CLI. It now delegates to the existing scoped planner with `FabricAllocationStoreOnly`, preserving host/revision admission and receipt publication.

The workflow supplies the expected host and revision and uses the existing 15-minute planning allowance. Run 36604979501 exhausted the old five-minute limit; full-host planning in run 36603426389 took 6m34s. No memory limits change.

Validation: `git diff --check` passed. Parsed workflow comparison establishes that only the allocation-store step's two environment bindings and timeout changed. The checked-in YAML is a projection candidate, awaiting canonical generated-artifact CI comparison; local canonical emission is not claimed. A production-step witness checks shared binding and timeout authorities. Its bounded 6 GiB/no-swap execution is pending at this receipt.

No host state has been applied by this cut. A read-only live plan must prove the receipt is emitted before any apply is attempted.

## Review 5357395973: lost effects

The successful read-only run 36616988377 exposed an unsafe artifact: missing directories but an empty apply script marked FullyApplied. That artifact must not be applied. No apply was dispatched.

The shared wet spine now selects the specialized allocation-store constructor whenever either directory needs convergence. A fully observed desired directory pair selects the generic no-op projection. Artifact construction refusal stops receipt publication. Both paths use the same extracted run-bound receipt constructor.

Executable controls cover both exact privileged directory ensures for absent prestate and an operation-free converged prestate. Both round-trip the receipt made by the production receipt constructor, including run, bundle and generation bindings. Bounded local execution and a fresh read-only host plan are pending; source repair alone does not clear HOLD.
