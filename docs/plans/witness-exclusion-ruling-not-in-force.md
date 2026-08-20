# An operator exclusion ruling that is no longer in force, with nothing saying so

**Filed against the exclusion mechanism, not against the module that exposed it.**

## The observation

`src/v2/lens/complexity_accumulator_copy/roster_gate.dag` carries a module-scope data row
declaring itself, verbatim:

> Operator-ruled OFFLINE (NOT CI discovery-enrolled; witness_exclusion_substrings row
> `claim/complexity/accumulator_copy_roster_gate` in `gunbc.ci_layer_roots` — owned by #6452, not
> duplicated on #6440). Serial wet claim_batch only.

The required floor discovered and offered its witnesses anyway. Four of the six non-resolving
identities in run `32358048539` are from this module's test carriers.

## Why this is its own defect

An operator made a decision — this module runs serially and wet, offline, never in CI discovery
— and that decision is **silently not being honoured**. Nothing refused, nothing counted, and
nothing reported that an exclusion row had stopped excluding.

This is the same class as the stale live-tree premise that the witness-execution-closure work
removed, one layer over, and it is **worse than a stale premise**: a stale premise is a
prediction that stopped being true, whereas this is a decision someone deliberately made that is
no longer in force while its carrier still asserts it. A reader of that module today is told the
opposite of what happens.

## What is NOT claimed

Not measured here: whether `witness_exclusion_substrings` still exists in `gunbc.ci_layer_roots`,
whether the floor cut deleted the consumer that read it, or whether other exclusion rows are in
the same state. The floor cut (2026-08-15) deleted `gunbc.ci_spec`'s discovery machinery
wholesale, so the likely shape is that the exclusion mechanism died with it and its rows were
left standing — but that is a hypothesis, not a finding, and it is stated as one.

## The obligation

The class is: **a declared exclusion whose enforcing consumer no longer exists.** An exclusion
row with no consumer must refuse or be deleted, never sit as prose asserting a control that is
not applied. The census — how many such rows exist and which mechanism was meant to read each —
is the first work, and it must be DERIVED from the rows rather than recalled, per the DESIGN §5
oracle rule.

Rung: **mitigatable, and only barely** — the state is writable, nothing detects it, and it was
found by accident while routing unrelated witness failures. Next-rung trigger: an exclusion row
that names its consuming mechanism, checked where the row is parsed, so a row whose consumer is
gone refuses at ingestion instead of being discovered by someone reading a comment.
