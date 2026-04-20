### Single-authority metadata

The compiler should provide all metadata (tool definitions, output
paths, type registries) through its own output types (`CompileOutput`,
`InferredEntrypoint`, etc.), not through runtime callbacks, string
conventions, or hardcoded lists. Each piece of metadata should have
exactly one producer.

**Structural prevention:** Guarantee receipt. The compiler emits a
machine-readable receipt on every run that records what was discovered,
what was proven, what was tested, and what's uncertain. If a guarantee
isn't in the receipt, it doesn't exist. Markdown dashboards are derived
from the receipt — never the source of truth. The escape hatch is
metadata scattered across log output, comments, and separate scripts;
the fix is one structured artifact that CI can enforce.

