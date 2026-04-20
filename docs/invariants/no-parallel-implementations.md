### No parallel implementations

When the same computation exists in two forms (e.g., an AST interpreter
AND a resolved DAG op), they will diverge as the language evolves.
Every new expression form must be implemented in both, and the one that
lags will be masked by a fallback (see above).

**The test:** if a code path exists only to provide a result that
another code path also produces, one of them should be deleted.

**Structural prevention:** Single source + derivation. Stage0 is
generated from `.dag` source — never hand-edited. The regeneration
script is the only path from `.dag` to `.rs`. Committed binary approach
means CI verifies regenerate → diff → empty. The escape hatch is
hand-editing generated code; the fix is making regeneration the only
write path and failing CI if the generated output doesn't match.

