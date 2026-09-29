# Allocation-store planning integration

The allocation-store CLI previously wrote an artifact through the legacy planner without the run-bound receipt consumed by the current apply CLI. It now delegates to the existing scoped planner with `FabricAllocationStoreOnly`, preserving host/revision admission and receipt publication.

The workflow supplies the expected host and revision and uses the existing 15-minute planning allowance. Run 36604979501 exhausted the old five-minute limit; full-host planning in run 36603426389 took 6m34s. No memory limits change.

Validation: `git diff --check` passed. Parsed workflow comparison establishes that only the allocation-store step's two environment bindings and timeout changed. The checked-in YAML is a projection candidate, awaiting canonical generated-artifact CI comparison; local canonical emission is not claimed. A production-step witness checks shared binding and timeout authorities. Its bounded 6 GiB/no-swap execution is pending at this receipt.

No host state has been applied by this cut. A read-only live plan must prove the receipt is emitted before any apply is attempted.
