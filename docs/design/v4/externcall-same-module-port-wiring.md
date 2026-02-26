# ExternCall Same-Module Port Wiring (NF-7)

**Status**: Draft  
**Date**: 2026-02-25  
**Depends on**: NF-4  
**Owned by**: NF-7

## Problem

Converting documented shadow `fn` items to `extern func` does not currently
land cleanly for same-module calls from function bodies. The lowerer does not
register extern declarations as callable endpoints, so downstream call-edge
wiring misses `ExternCall` nodes or wires non-existent ports. Codegen then sees
broken data flow.

This is a lowerer limitation, not a DSL design choice.

## Current Behavior

1. Typecheck validates `extern func` declarations and includes them in callable
   contract checks for expression typing.
2. Lowering endpoint registration currently covers `Fn`, `Func`, and `Pattern`
   signatures only.
3. Call wiring helpers (`add_dependency_edges`, `wire_fn_call_arguments`,
   `resolve_source_expr`, `collect_bound_callable_sources`) all rely on lowered
   endpoint maps.
4. Because extern declarations are absent from those maps, same-module call
   wiring can silently skip edges that should exist.

## Goals

1. Treat `extern func` as first-class lowered callable endpoints.
2. Ensure extern node input/output ports match DSL declarations exactly.
3. Make same-module function-body calls to externs wire identically to normal
   callable calls.
4. Preserve existing fail-closed extern resolution behavior in `resolve.rs`.
5. Keep shadow fn bodies as documented placeholders until migration is complete.

## Non-Goals

1. Removing extern infrastructure.
2. Changing resolver semantics for unknown extern symbols.
3. Redesigning cross-module name disambiguation beyond current endpoint map
   behavior.

## Design

### D1. Add typed signature coverage for extern funcs

Extend typed signatures so extern declarations participate in lowering setup.

```rust
pub enum TypedItemSignature {
    // existing variants...
    ExternFunc(TypedCallableSignature),
}
```

Typecheck should push `ExternFunc` signatures after existing type validation of
extern inputs/outputs.

### D2. Lower extern declarations to explicit `ExternCall` nodes

Add a dedicated lowering path:

1. Build deterministic node ID with existing `lowered_node_id(module, name)`.
2. Build symbol with `ProgramSymbolId::from_parts(module, name)`.
3. Create `Node<LoweredOp>` with:
   - `LoweredOp::ExternCall { symbol }`
   - input ports from declaration inputs plus `__deps: Any*`
   - output ports from declaration outputs (or `return: Unit` if empty)
4. Return a `LoweredEndpoint` whose `primary_output` is first output (or
   `return`).

This makes extern nodes structurally compatible with existing call-edge wiring.

### D3. Register extern endpoints in both endpoint maps

Register extern endpoints exactly like callable endpoints:

1. `endpoints_by_full[(module, name)]`
2. `endpoints_by_name[name]` with existing ambiguity handling (`None` on
   conflict)

No new endpoint map type is required.

### D4. Reuse existing edge-wiring logic

No new edge phase is needed. Existing logic will begin wiring extern calls once
extern endpoints exist:

1. `wire_fn_call_arguments` can connect argument sources to extern input ports.
2. `add_dependency_edges` can connect extern output to caller `__deps`.
3. `resolve_source_expr` and bound-source collection can resolve extern outputs
   for content/path/service argument wiring.

## Expected Behavior Example

```dag
module tools.example

extern func render(cfg: Json) -> { content: String }

func write(cfg: Json) -> { return: Unit } {
  let out = render(cfg: cfg)
  fs.write(path: "out.txt", content: out.content)
}
```

Lowering should include:

1. Extern node `tools.example::render` with input `cfg`, output `content`.
2. Argument edge from `cfg` source into `tools.example::render.cfg`.
3. Dependency edge from `tools.example::render.content` to
   `tools.example::write.__deps`.
4. Source resolution for `out.content` pointing at the extern endpoint output.

## Failure Modes and Mitigations

1. **Name collision across modules**: existing `endpoints_by_name` ambiguity
   semantics remain. Same-module resolution still prefers full-key lookups where
   available.
2. **Multi-output extern used as bare identifier**: behavior matches existing
   callable handling through `primary_output`.
3. **Unknown extern symbol at resolve**: unchanged hard error from
   `resolve_extern_call`.

## Test Plan

1. Lowerer unit: extern declaration lowers to `ExternCall` node with declared
   symbol and ports.
2. Lowerer unit: same-module function calling extern wires argument and
   dependency edges.
3. Lowerer unit: field access from extern call result wires the named output
   port.
4. Lowerer unit: duplicate extern names across modules keep deterministic
   ambiguity behavior in `endpoints_by_name`.
5. Resolver unit: lowered extern symbol still resolves/fails with existing
   `resolve_extern_call` contract.

## Cleanup Impact

NF-7 is the single gating dependency for eliminating the shadow bridge mechanism
across the entire codebase. Everything below becomes possible once NF-7 lands.

### Shadow bridges deleted (3 active)

These DSL `fn` items have placeholder bodies that are silently overridden by Rust
extern impls at resolve time. NF-7 converts them to honest `extern func` declarations.

| Module | fn name | DSL body | Rust impl | What changes |
|--------|---------|----------|-----------|--------------|
| tools.gist | build_snapshot_content | `"{branch}"` (placeholder) | BuildSnapshotContentOp — full markdown doc | `fn` → `extern func`, delete placeholder body |
| tools.bootstrap | render_bootstrap_makefile | partial DSL scaffold | GenerateBootstrapMakefileOp — ignores inputs, calls full makegen | `fn` → `extern func`, delete scaffold body |
| tools.bootstrap | render_bootstrap_gitignore | partial DSL scaffold | GenerateBootstrapGitignoreOp — ignores inputs, calls full gitignore | `fn` → `extern func`, delete scaffold body |

### Pragma dual-truth resolved (3 fns)

These DSL `fn` items have *real* working bodies AND Rust extern impls registered
in extern_impls.rs. The Rust impl silently overrides the DSL body at resolve time
(Step 4 wins over Step 5). Both implementations exist, one runs, the other is dead
code that looks alive.

| Module | fn name | DSL body | Rust impl | What changes |
|--------|---------|----------|-----------|--------------|
| tools.pragma | render_clippy_toml | real: `render_document(render_clippy_toml_document(...))` | RenderClippyTomlOp | Delete Rust impl (DSL body is correct), OR convert to `extern func` |
| tools.pragma | render_disallowed_methods_allowlist | real: `render_document(render_disallowed_methods_allowlist_document(...))` | RenderAllowlistOp | Same choice |
| tools.pragma | render_pragma_lint_policy | real: `render_document(render_pragma_lint_policy_document(...))` | RenderLintPolicyOp | Same choice |

For pragma: the ideal outcome is to **delete the Rust impls** and let the DSL bodies
execute. NF-7 makes this possible because the DSL bodies call helper fns in the same
module — those calls need correct same-module wiring to work.

### Resolver simplification

Once all shadow bridges are converted, resolve_domain() Step 4 (`extern_impls::
lookup_extern_impl()` for `Callable` nodes) becomes dead code:

```
Before NF-7:
  resolve_op(Callable) → resolve_domain() → Step 4: extern_impls lookup → Step 5: passthrough
  resolve_op(ExternCall) → resolve_extern_call() → hard fail

After NF-7:
  resolve_op(Callable) → resolve_domain() → Step 5: passthrough (only DSL-body callables)
  resolve_op(ExternCall) → resolve_extern_call() → hard fail (all extern resolution)
```

Deletable after NF-7 cleanup:
- `resolve_domain()` Step 4 (extern_impls lookup for Callable nodes)
- Shadow bridge detection (`#[cfg(debug_assertions)]` eprintln)
- The `kind: CallableKind` parameter added to resolve_domain (no longer needed
  for shadow bridge detection)

### classify_handler() TODO(NF-5) closed

The obligation-gated passthrough in `classify_handler()` has TODO(NF-5) markers on
`PureRender` and `PureDataLoad` obligations. Once NF-7 converts these callables:
- PureRender/PureDataLoad callables become `ExternCall` nodes → skip classify_handler
  entirely (line 334: `LoweredOp::ExternCall => return None`)
- OR become pure DSL with SubDag bodies → skip classify_handler entirely
  (line 227: `NodeBody::SubDag(_) => continue`)

Either way, the PureRender/PureDataLoad passthrough arms become unreachable and can
be deleted, tightening classify_handler to only structural passthroughs (None,
PureGeneric, resource/service obligations).

### Deletion summary

| What | ~Lines deleted | Condition |
|------|---------------|-----------|
| 3 shadow fn placeholder bodies in DSL | 30 | Immediate after NF-7 |
| 3 pragma extern impls (or 3 pragma DSL bodies, one or the other) | 60-80 | Immediate after NF-7 |
| resolve_domain() Step 4 + shadow bridge detection | 15 | After all shadow→extern conversions |
| classify_handler PureRender/PureDataLoad arms + TODO comments | 10 | After all conversions |
| **Total resolver/emit cleanup** | **~120** | |

This is in addition to the ~1,850 lines from FC-P6/P7/P8 that NF-7 gates.

### Risk without NF-7

Every day shadow bridges exist:
- **Silent override**: Rust impl runs, DSL body is dead code that looks alive.
  Changes to the DSL body have zero effect. No test catches this.
- **Pragma dual-truth**: Two implementations, only one executes. If someone
  fixes a bug in the DSL render_clippy_toml, the fix doesn't ship.
- **Fail-open resolution**: Callable resolution falls through to passthrough
  (Step 5) for any callable without an extern impl, including typos and
  deleted callables. ExternCall resolution is fail-closed. Converting to
  extern func moves callables to the fail-closed path.
- **Audit opacity**: `extern_impls.rs` has 9 entries. Without NF-7, you can't
  tell which are honest externs vs shadow bridges without reading both the
  DSL and Rust code for each.

## Rollout

1. Land typed-signature + lowerer endpoint changes and tests.
2. Convert selected shadow `fn` items to `extern func` where bodies are currently
   placeholders.
3. Keep clear migration comments in DSL until all targeted conversions are done.
4. Close NF-7 once converted modules compile and codegen flow remains intact.
