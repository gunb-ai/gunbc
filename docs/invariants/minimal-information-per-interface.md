### Minimal information per interface

Every function, helper, and modeling unit should receive the minimum
information it needs — nothing more. Passing an entire collection to a
function that only inspects one element couples the function to state
it doesn't use, creates ambiguity about which instance of the state is
current, and hides the function's true dependency.

**The test:** if a function takes a parameter and immediately projects
one field or element from it, the function should take the projection
directly. `fn check_token(tokens: List<Token>)` that does `tokens |>
first` should be `fn check_token(tok: Token?)`.

**Subtle examples:**
- `peek_is_newline(tokens: List<Token>)` → only needs `Token?`
  (the current token). Passing the list creates ambiguity about
  WHICH list when the caller has multiple remaining lists in scope.
- `function_size_effects(name: String)` → only needs the function's
  structural contract, not a string key into a lookup table. The
  string forces the caller to know the name; a direct reference to
  the contract would be unambiguous.
- `classify_argument(arg_expr: Node, param_name: String, ctx: DescentContext)` →
  DescentContext bundles 7 fields, but most call sites only need 2-3.
  The bundling hides which facts the function actually depends on.

**Structural prevention:** Design function signatures from the
function's body outward: what does it READ? Pass exactly that. When a
helper only inspects a single value, take that value — not the
collection it came from, not the struct it's embedded in, not the
context that happens to carry it.

**Escape hatch:** convenience structs that bundle unrelated state
("context" objects, "environment" bags). These make it easy to pass
everything and hard to see what matters. Prefer explicit parameters
over context bundles; group into a bundle only when 3+ consumers need
the exact same set of fields.

