# Design: Typed AST Extension for v2 Typechecker

## Representation

Flat wrapper, not parallel sum type:

```dag
type TypedExpr {
  expr: Expr
  resolved_type: TypeExpr
}
```

The emitter pattern-matches `typed_expr.expr` for variant structure and reads
`typed_expr.resolved_type` for type-driven decisions. Avoids duplicating 17
Expr variants.

## Scope

Every expression node gets a type annotation. No partial typing. If the emitter
can't ask "do I have type info here?" the heuristic paths are structurally
eliminated.

## Typed containers

```dag
type TypedMatchArm {
  pattern: MatchPattern
  guard: TypedExpr?
  body: TypedExpr
}

type TypedNamedArg {
  name: String?
  value: TypedExpr
}

type TypedFieldInit {
  name: String
  value: TypedExpr
}
```

## TypedItem

```dag
type TypedItem
  = TypedTypeDef { name: String, body: TypeBody, span: SourceSpan }
  | TypedFnDef { name: String, params: List<Param>, return_type: TypeExpr,
                 body: TypedExpr, span: SourceSpan }
  | TypedFuncDef { name: String, params: List<Param>, return_type: TypeExpr,
                   uses: List<ResourceUse>, body: TypedExpr, span: SourceSpan }
  | TypedServiceDef { name: String, transport: TransportBinding,
                      config: ServiceConfig?, operations: List<OperationDef>,
                      span: SourceSpan }
  | TypedResourceDef { name: String, properties: List<FieldInit>,
                       capabilities: List<CapabilityDef>, span: SourceSpan }
  | TypedDataDef { name: String, type_expr: TypeExpr, value: TypedExpr,
                   span: SourceSpan }
  | TypedExternFuncDecl { name: String, params: List<Param>,
                          return_type: TypeExpr, span: SourceSpan }
```

## FuncEnv

```dag
type FuncSig {
  name: String
  params: List<Param>
  return_type: TypeExpr
  is_async: Bool
}

type FuncEnv {
  signatures: List<FuncSig>
}
```

## TypedModule

```dag
type TypedModule {
  name: String
  imports: List<Import>
  items: List<TypedItem>
  type_env: TypeEnv
  func_env: FuncEnv
  span: SourceSpan
}
```

## Inference

Single top-down pass per function body. Not Hindley-Milner — .dag has
explicit type annotations on params and fields. Inference is propagation.

```dag
type InferScope {
  type_env: TypeEnv
  func_env: FuncEnv
  locals: List<TypeBinding>
  module_name: String
}

type InferResult {
  typed: TypedExpr
  diagnostics: List<Diagnostic>
}
```

See task #35 design output for full inference rules per Expr variant,
lambda bidirectional typing, pipe method signatures, and migration plan.

## Implementation sequence

1. Add types to 00_core.dag (TypedExpr, TypedItem, etc.)
2. Add inference types + build_func_env to 04_typecheck.dag
3. Implement infer_expr (recursive walk)
4. Implement infer_item (wraps Items into TypedItems)
5. Update typecheck_module to produce new TypedModule
6. Update emitter forward declarations (C5)
7. Migrate emit_module to read TypedItem
8. Migrate emit_expr to accept TypedExpr
9. Delete ItemInfo registry (replaced by FuncEnv)
