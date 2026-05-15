# `.dag` `fn` bodies — brace parse vs expression parse (opt-in)

Staged **`*.dag`** sources participate in bootstrap lowering with a deliberate split:

- **`fn name(...) -> Ty = <expression>`** — parsed as **`SurfaceExpr`** where the grammar permits; lowers to **`ArrowBody::UserDefined`** when the expression is fully modeled.
- **`fn name(...) -> Ty { ... }`** — often retained as **`ArrowBody::Unparsed`** at parse boundaries until the brace grammar is migrated for that authority (**`fn_brace_body_parse_as_expression`** and related knobs in **`src/v3/compiler/src/parse_generated.rs`**).

Implications:

1. **Bootstrap regen ripple:** touching std authorities that flip between `=` and `{...}` materially changes **`bootstrap_generated*.rs`** and any tests that fingerprint body classes.
2. **Parser surface gate:** widening brace parsing is intentionally coupled to corpus tests (e.g. **`m17_dag_corpus_brace_fn_stays_fn_external_body_at_parse_time`** in `src/v3/compiler/tests/integration/m1_substrate_test.rs`) so migrations stay explicit.
3. **Authoring ergonomics:** prefer expression bodies (`= match ...`) for small dispatch surfaces that must execute in-bootstrap today; brace bodies remain the escape hatch for not-yet-parsed scaffolding.
