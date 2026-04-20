### Navigating the concept DAG: where to start

The `dsl/std/` directory IS the concept DAG. Files are ordered by
dependency depth. Start at the roots and follow imports.

**Layer 0 — Foundations (no imports):**
| File | Concept | External authority |
|------|---------|-------------------|
| `logic.dag` | Classical bivalent truth | Mathematical logic |
| `constructors.dag` | Product, Coproduct | Category theory |
| `algebra.dag` | Monoid → Semiring → Ring → Field, Lattice, BooleanAlgebra, FreeMonoid, PartialFunction | Abstract algebra |
| `iteration.dag` | fold, descend, repeat (bounded computation) | Catamorphism theory |
| `syntax.dag` | BinOp, Literal, Token, ExpectedToken | BNF grammar theory |

**Layer 1 — Compositions (import from Layer 0):**
| File | Concept | Imports from |
|------|---------|-------------|
| `bit.dag` | Word8..Word64 | logic |
| `integer.dag` | Int = Word64 + OrderedRing | algebra, bit |
| `float.dag` | Float = Word64 + ApproximateField | algebra, bit |
| `string_type.dag` | String = FreeMonoid<Char> | algebra, types |
| `types.dag` | Kernel types, container types | algebra |
| `termination.dag` | DescentEvidence, RankingDimension, TerminationProof | algebra |

**Layer 2 — Domain vocabularies (import from Layer 0-1):**
| File | Concept | For whom |
|------|---------|---------|
| `languages.dag` | Language specs (Rust, Python, Go, ...) | Emission |
| `coercion.dag` | TypeCheckpoint, InhabitantDecl | Type rendering |
| `primitives.dag` | PrimitiveContract (43 operation costs) | Complexity analysis |
| `unicode.dag` | Unicode blocks, display width | String handling |
| `resources.dag` | Acquirable capabilities | Service modeling |

**Layer 3 — Application domains (import from Layer 0-2):**
`cloud.dag`, `credentials.dag`, `encoding.dag`, `errors.dag`,
`fermi.dag`, `fidelity.dag`, `filesystem.dag`, `patterns.dag`,
`render.dag`, `behavioral.dag`

**The compiler pipeline** (`src/v2/`) imports from `std/` at Layer 0-1:
- `00_core.dag` ← std.types, std.syntax
- `04_types.dag` ← std.algebra (AlgebraTypeTemplate, profiles)
- `complexity.dag` ← std.types (kernel identity)

**Missing links (to be added):**
- `algebra.dag` should note conceptual dependency on `logic.dag` and `constructors.dag`
- `iteration.dag` should note that `fold` is a catamorphism over `FreeMonoid` from `algebra.dag`
- ~~`stack.dag` should import `algebra.dag`~~ — DONE: imports FreeMonoid, operations aligned to FreeMonoid vocabulary
- `containers.dag` should import `algebra.dag` or merge into it (documentation-only today)

---

