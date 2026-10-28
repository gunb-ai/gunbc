# Primitive runtime body check

**Goal:** reject registry bridges without emitted Rust bodies. `v1.compiler.primitive_runtime_body.check` reads the live `rt_function_registry` and `rust_runtime_source()` and joins on bridge identity through `std.primitive_identity.primitive_runtime_body_check_in`; it carries no copied population oracle.

**Run:** with a built compiler and an enforceable memory bound:

```sh
gunbc run --source-root dag --source-root src/v2 --source-root src/v1 \
  --source-root test/primitive_runtime_body \
  --entry test/primitive_runtime_body/producer.dag --function check
```

**Controls:** the producer checks the combined surface join, replaces `map_insert`'s bridge with a missing-body specimen, and appends a missing bridge. Each must refuse exactly its intended subject; unread source, missing controls or unrelated missing rows refuse. Fixture witnesses cover aliases, enrollment, bodyless declarations, scanner refusals and identity joins.

**Limits:** this proves file-level body presence in supported scanner syntax, not signatures, visibility, cfg availability or behavior. Mirror freshness and emitted-edge coverage are separate facts. The producer stays outside ordinary floor roots and the v1-only regeneration sweep to avoid widening either compilation subject. DESIGN owns current required CI; the old witnesses-phase enrollment described in [historical qualification](https://github.com/gunb-ai/gunbc/blob/d919d662ddf59813f9dab24ca3b25ed5b54ec236/test/primitive_runtime_body/README.md) is not a current merge-gate claim. No checker execution is claimed by this cleanup.
