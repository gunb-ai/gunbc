## Every Dependency Is A Substrate Fact

Whenever a downstream consumer needs a fact, that fact must have a
structural home in the substrate or in a declared realization. Consumers
must not reconstruct dependencies from names, parallel tables, or hidden
calling convention knowledge.

Applied to the reflection work, this means:

- "Which field does this projection read?" is a substrate fact carried
  by the field label and realized via `FieldBinding`.
- "Which port is the primary result of this node?" is a substrate fact
  surfaced compositionally as `result_port`, not a five-arm Rust helper.
- "How do I read this reflected field from Rust?" is a realization fact
  in `rust.dag`, not a hardcoded emitter branch.

If a consumer cannot answer its question by following typed edges and
declared bindings, the fix is upstream: add the missing substrate fact
or realization binding. Reconstructing the fact locally is a violation
of Single Authority and a precursor to drift.

