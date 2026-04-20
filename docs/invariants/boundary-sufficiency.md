### Boundary sufficiency

A stage boundary is *sufficient* when the data it carries contains all
the structural facts the downstream stage needs, making name-based proxy
reads unnecessary. When a stage branches on a name to make a structural
decision, the boundary is insufficient — a fact is missing.

**The diagnostic:** scramble all user-defined names across a boundary.
If downstream decisions change, a structural fact is missing and the
name was used as a proxy. The scrambled-name test reveals exactly which
decisions depend on names, pointing to the missing facts.

**The fix:** always enrich the boundary, never restrict access. When
inference needs "has math methods," the fix is "put algebra membership
in the boundary data," not "hide the name." When emit needs "how to
declare a variable," the fix is "read LanguageSpec," not "prevent
hardcoding."

**Structural prevention:** Typed boundaries where insufficient data
is a compile error. If emit needs algebra membership and the boundary
doesn't carry it, emit can't compile — the field doesn't exist on the
boundary type. The escape hatch is `node.name` (any string, always
accessible, carries no structural guarantee); the fix is deleting
`Node.name` (M4/D6) so the only way to get information about a node
is through its structural properties and edges. The scrambled-name
tests are the diagnostic; `Node.name` deletion is the prevention.

