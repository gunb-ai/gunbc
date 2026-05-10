# R3 Omni OpenAPI Backend Bridge Dissolution

Status: QUEUED

Owner lane: T-PB-B, coordinated with T-Omni-Shape-B.

## Scope

Retire the R3 gate #25 hand-Rust demo bridge introduced for the omni OpenAPI
backend emission receipt:

- `src/v3/compiler/src/omni_shape_b_openapi.rs`
  `project_openapi_yaml`, `project_markdown_documentation`, and
  `project_rust_backend_service`
- `src/v3/compiler/tests/integration/m1_5_omni_shape_b_openapi_test.rs`
  direct `rustc` generated-backend roundtrip helper

These are fixture-scoped demo projections, not compiler targets and not a
general backend framework.

## Dissolution Trigger

Retire this bridge when `.dag` TestClaim execution can materialize a generated
artifact, invoke `ExecuteCommand` against it, and reuse the produced binary or
artifact handle across multiple assertions. At that point:

- OpenAPI and Markdown projection move to Shape B `.dag` artifact emitters.
- The runnable backend uses the normal Shape A Rust emission path or generated
  artifact materialization path, not a compiler-local string projector.
- The route roundtrip moves from Rust-authored `Command::new("rustc")` to an
  `ExecuteCommand`-based `.dag` TestClaim.

## ROADMAP Receipt

`ROADMAP.md:170` — Hand-Rust census split; the test-driven bridge retires
through T-PB-B by migrating the generated-backend roundtrip to a `.dag`
TestClaim declaration. The adjacent production projection is in this brief's
scope only because that test bridge is its sole consumer for the R3 gate #25
receipt; it retires with the same T-PB-B TestClaim migration.
