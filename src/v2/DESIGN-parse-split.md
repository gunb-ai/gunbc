# Design: Splitting 02_parse.dag into Sub-Modules

## Motivation

02_parse.dag is 3700 lines with ~130 functions and ~55 type definitions. The compiled
v2 parser OOMs on this file because S76 (clone_if_needed) generates ~2000 .clone()
calls, and each ParserState.clone() copies the token Vec handle. With 3700 lines of
token input, memory usage exceeds available limits.

Splitting into smaller modules reduces per-module token list size, bringing each
sub-module below the OOM threshold.

## Complete Function Inventory (121 functions)

### Group A: Core Helpers (20 functions, lines 156-475)

These are the primitive operations on ParserState that every other function calls.

```
peek, peek_kind, at_end, current_span, advance,
parse_error, has_err,
kind_tag, check, expect, expect_ident, expect_name, keyword_to_name,
skip_newlines, skip_continuation_newlines, eat,
is_ident, is_keyword_name,
parse_dotted_ident, parse_dotted_ident_rest
```

Dependencies: Only v2.std.core types. No calls to any other group.

### Group B: Type Parsing (18 functions, lines 642-1093)

Parses type definitions, type expressions, fields, predicates, and variants.

```
parse_type_def, parse_type_body_after_eq,
try_where_clause, parse_predicates, parse_single_predicate,
parse_named_int_args, parse_single_named_int,
parse_variant_fields, parse_more_variants,
parse_type_expr, finish_type_expr_from_name, maybe_optional,
parse_field_list, parse_field, parse_optional_from_key,
parse_params, parse_param_list, parse_param
```

Dependencies:
- Group A (core helpers): all functions
- Group D (expressions): parse_expr (called by parse_single_predicate, parse_single_named_int, parse_field for default values, parse_param for default values)
- Circular: parse_type_expr <-> parse_expr (via parse_field -> parse_expr for defaults, and parse_primary -> parse_type_expr via cast)

### Group C: Item Parsing / Module Structure (14 functions, lines 481-636, 1099-1237, 2067-2225)

Top-level module structure: parse entry point, imports, item dispatch, fn/func/data/extern defs.

```
parse, parse_module,
parse_imports, parse_import, parse_import_names,
parse_items, parse_items_acc, parse_item,
parse_fn_def, parse_func_def,
parse_uses_clause, parse_uses_list, parse_uses_entry,
parse_optional_return_type,
parse_data_def, parse_extern_decl
```

Dependencies:
- Group A (core helpers): all functions
- Group B (type parsing): parse_type_expr, parse_params, parse_field_list
- Group D (expressions): parse_expr, parse_block
- Group E (service parsing): parse_service_def, parse_resource_def (via parse_item dispatch)

### Group D: Expression Parsing (44 functions, lines 2231-2846, 3219-3700)

The Pratt precedence climbing parser, statements, match, if, for, let, lambdas,
literals, call args, string interpolation, brace expressions.

```
parse_block, parse_stmts, parse_stmts_acc,
parse_stmt, peek_is_eq_after_ident, parse_bare_assignment,
parse_expr, parse_expr_bp, parse_expr_loop,
infix_bp, token_to_binop, parse_pipe_rhs,
parse_prefix, parse_primary,
parse_lambda_body, parse_lambda_stmts,
parse_ident_expr, is_uppercase_start,
try_postfix, make_call_expr,
parse_index_or_slice,
parse_call_args, parse_arg_list, parse_single_arg,
try_named_arg_label, keyword_to_arg_label,
parse_if, parse_let, parse_return, parse_for,
parse_record_literal, parse_field_init_list, parse_field_init,
parse_list_literal, parse_expr_list_until,
parse_paren_expr, parse_fn_lambda, collect_fn_lambda_params,
try_lambda_params, collect_lambda_idents,
parse_string_interp, parse_interp_parts,
parse_brace_expr, peek_is_colon_after_ident
```

Dependencies:
- Group A (core helpers): all functions
- Group B (type parsing): parse_type_expr (used by try_postfix for `as` casts)
- Group F (match parsing): parse_match (called from parse_primary)

### Group E: Service/Resource Parsing (25 functions, lines 1243-2061)

Service definitions, transport bindings, operations, resources, capabilities.

```
parse_service_def, parse_service_body, parse_service_entries,
parse_service_config_block, parse_config_fields,
parse_transport_binding,
parse_rest_binding_body, parse_rest_fields,
parse_shell_binding_body, parse_shell_fields,
parse_file_binding_body, parse_file_fields,
parse_operation_def, parse_operation_v2_inline, parse_operation_v1_body,
parse_op_body_entries,
parse_exit_entries, parse_operation_modifiers,
parse_status_pattern,
parse_optional_response_block, parse_response_entries,
parse_optional_mock_response_block, parse_mock_response_entries,
parse_resource_def, parse_resource_entries,
skip_until_rbrace,
parse_capability, parse_input_output_blocks, parse_io_blocks_acc
```

Dependencies:
- Group A (core helpers): all functions
- Group B (type parsing): parse_type_expr, parse_field_list, parse_optional_return_type
- Group D (expressions): parse_expr, parse_expr_list_until

### Group F: Match/Pattern Parsing (17 functions, lines 2852-3213)

Match expressions, patterns, arms, guards, lookahead for arm boundaries.

```
parse_match,
parse_expr_no_brace, parse_expr_bp_no_brace, parse_expr_loop_no_brace,
parse_match_arms, parse_match_arms_acc, parse_match_arm,
parse_match_arm_body, parse_match_arm_stmts,
looks_like_arm_start,
peek_is_fat_arrow_at, peek_is_tag_at,
scan_for_fat_arrow_after_braces, scan_braces_depth,
parse_optional_guard,
parse_pattern, parse_variant_pattern, parse_variant_bindings_brace
```

Dependencies:
- Group A (core helpers): all functions
- Group D (expressions): parse_expr, parse_block, parse_stmt, parse_call_args, make_call_expr, parse_pipe_rhs, parse_index_or_slice, infix_bp, token_to_binop, parse_prefix (the no_brace variants duplicate the expr loop logic)

## The Circular Dependency Problem

The critical challenge is that Groups B, C, D, E, and F have circular call relationships:

```
C (items) -> D (exprs)    : fn/func bodies are expressions
C (items) -> E (services) : parse_item dispatches to service/resource
D (exprs)  -> B (types)   : `as` casts need parse_type_expr
D (exprs)  -> F (match)   : parse_primary calls parse_match
F (match)  -> D (exprs)   : match arm bodies are expressions
B (types)  -> D (exprs)   : field defaults, predicate args need parse_expr
E (service)-> D (exprs)   : config values, mock bodies are expressions
E (service)-> B (types)   : operation inputs/outputs are fields/types
```

The v2 .dag language does not support forward declarations or mutual imports.
Each module must be self-contained or import from already-defined modules.
This means we cannot split into 6 separate modules with circular imports.

## Proposed Module Split: 4 Modules

The solution is to merge the circular groups into a single "grammar" module and
keep only the truly independent groups separate.

### Module 1: `02a_parse_core.dag` (~350 lines)

Module path: `v2.compiler.parse.core`

**Contains:** All type definitions (55 types, lines 64-151) + all core helper
functions (Group A, 20 functions).

**Types defined here:**
- ParserState, ParseResult
- All *Result types: AdvanceResult, EatResult, TokenResult, NameResult, ExprResult,
  ItemResult, TypeResult, ModuleResult, ImportResult, VariantResult, PredResult,
  ParamResult, TransportResult, OpResult, CapResult, PatternResult, ArmResult,
  ArgResult, FieldResult, FieldInitResult, ResUseResult, ConfigResult
- All list result types: ImportsResult, ItemsResult, NamesResult, FieldsResult, etc.
- All optional/multi-field result types: OptRetResult, GuardResult, PostfixResult, etc.
- NamedArgLabelResult, DescResult, BindingPower (new explicit type)

**Functions defined here (20):**
```
peek, peek_kind, at_end, current_span, advance,
parse_error, has_err,
kind_tag, check, expect, expect_ident, expect_name, keyword_to_name,
skip_newlines, skip_continuation_newlines, eat,
is_ident, is_keyword_name,
parse_dotted_ident, parse_dotted_ident_rest
```

**Imports:** Only `v2.std.core` (AST types, Token, TokenKind, etc.)

**Dependency direction:** Pure leaf module. No calls to any other parse sub-module.

**Estimated lines:** ~350 (151 lines of types + ~200 lines of helper functions)

### Module 2: `02b_parse_service.dag` (~820 lines)

Module path: `v2.compiler.parse.service`

**Contains:** Service definitions, transport bindings, operations, resources,
capabilities (Group E). Also includes parse_optional_return_type since it is
shared between items and services.

**Functions defined here (29):**
```
parse_optional_return_type,
parse_service_def, parse_service_body, parse_service_entries,
parse_service_config_block, parse_config_fields,
parse_transport_binding,
parse_rest_binding_body, parse_rest_fields,
parse_shell_binding_body, parse_shell_fields,
parse_file_binding_body, parse_file_fields,
parse_operation_def, parse_operation_v2_inline, parse_operation_v1_body,
parse_op_body_entries,
parse_exit_entries, parse_operation_modifiers,
parse_status_pattern,
parse_optional_response_block, parse_response_entries,
parse_optional_mock_response_block, parse_mock_response_entries,
parse_resource_def, parse_resource_entries,
skip_until_rbrace,
parse_capability, parse_input_output_blocks, parse_io_blocks_acc
```

**Imports:**
- `v2.std.core` (AST types)
- `v2.compiler.parse.core` (ParserState, all result types, all helpers)
- `v2.compiler.parse.grammar` (parse_type_expr, parse_field_list, parse_expr, parse_expr_list_until, peek_is_colon_after_ident)

**Problem:** This module calls parse_expr and parse_type_expr from the grammar
module, creating a potential ordering issue. However, the service module does NOT
need to be called BY the grammar module -- only parse_item in the grammar module
calls parse_service_def and parse_resource_def.

**Resolution approach:** The grammar module (02c) will import from the service
module. The service module needs parse_expr/parse_type_expr/parse_field_list
which are in the grammar module. This is a circular dependency.

**Alternative:** Merge service parsing into the grammar module. This eliminates
the circular dependency but makes the grammar module larger (~2530 lines).

### Module 2 (revised): Service parsing stays in the grammar module.

Given the circular dependency, the cleaner split is:

### Module 2: `02b_parse_grammar.dag` (~2530 lines)

Module path: `v2.compiler.parse.grammar`

**Contains:** Everything that has circular dependencies -- type parsing (Group B),
expression parsing (Group D), match/pattern parsing (Group F), service/resource
parsing (Group E), plus shared utilities like parse_optional_return_type and
parse_params.

**Functions defined here (98):**
All functions from Groups B, D, E, and F.

**Imports:**
- `v2.std.core` (AST types)
- `v2.compiler.parse.core` (ParserState, all result types, all helpers)

**Dependency direction:** Imports only from core. All internal circular calls
are within this single module.

**Estimated lines:** ~2530 (lines 642-3700 minus types)

### Module 3: `02c_parse_entry.dag` (~200 lines)

Module path: `v2.compiler.parse`

**Contains:** Entry point and module-level structure (Group C). This is the
"facade" module that external consumers (06_pipeline.dag) import from.

**Functions defined here (14):**
```
parse, parse_module,
parse_imports, parse_import, parse_import_names,
parse_items, parse_items_acc, parse_item,
parse_fn_def, parse_func_def,
parse_uses_clause, parse_uses_list, parse_uses_entry,
parse_data_def, parse_extern_decl
```

**Imports:**
- `v2.std.core` (AST types)
- `v2.compiler.parse.core` (ParserState, all result types, all helpers)
- `v2.compiler.parse.grammar` (parse_type_expr, parse_params, parse_field_list,
  parse_block, parse_expr, parse_service_def, parse_resource_def,
  parse_optional_return_type)

**Dependency direction:** Top of the dependency chain. Not imported by any other
parse sub-module.

**Estimated lines:** ~200

## Revised Assessment: 3-Module Split Is Insufficient

The grammar module at ~2530 lines is still too large for the OOM problem.
The original 3700-line file causes OOM; we need modules well under 2000 lines.

## Proposed Module Split: 5 Modules (Breaking Circular Dependencies)

To break the circular dependencies, we use a technique: **late binding via
function parameters.** Instead of Module E importing parse_expr directly,
it receives a `parse_expr_fn` parameter. This is not viable in the current
DSL -- functions are not first-class values that can be passed as typed
parameters in .dag files.

**Alternative approach: Re-export facade.** Each module defines its functions
and exports them. The entry module re-imports and re-exports everything.
Circular calls are resolved by having all the circularly-dependent functions
in a single module.

## Final Proposed Split: 4 Modules

After careful analysis, the most practical split given the language constraints:

```
02a_parse_core.dag      (~350 lines)  -- types + helpers
02b_parse_expr.dag      (~1100 lines) -- expressions, match, patterns, lambdas
02c_parse_items.dag     (~1350 lines) -- types, services, resources, items
02d_parse.dag           (~200 lines)  -- entry point, module structure
```

The key insight: the circular dependency between expressions and types is
actually small. Only two paths create it:

1. `parse_type_expr` -> (field defaults) -> `parse_expr` : field default values
2. `parse_expr` -> (cast `as`) -> `parse_type_expr` : cast expressions

**Resolution:** Move parse_type_expr and its cluster into the items module
(02c), and have parse_expr call parse_type_expr from 02c. This means 02b
imports from 02c (for parse_type_expr), and 02c imports from 02b (for
parse_expr). This is still circular.

## Final Design: 3 Modules (Practical Minimum)

Given that .dag modules cannot have circular imports, the practical minimum
split that respects this constraint is:

```
02a_parse_core.dag     (~350 lines)   -- types + core helpers
02b_parse_grammar.dag  (~3150 lines)  -- all grammar productions
02c_parse.dag          (~200 lines)   -- entry point + module structure facade
```

But this does not solve the OOM. The grammar module is still too large.

## Recommended Design: Break the Cycle with Duplication

The only way to get modules under ~1000 lines while respecting the no-circular-
imports constraint is to duplicate a small number of functions. Specifically,
the `parse_expr_no_brace` family (3 functions, ~50 lines) used by match parsing
is a near-duplicate of `parse_expr_bp` + `parse_expr_loop`. By inlining these
already-duplicated functions into the match module, we can separate match from expr.

Similarly, the cast path (parse_type_expr called from try_postfix) can be
handled by having the expression module include a minimal `parse_type_expr`
or by restructuring `as` casts to use a name-only parser.

### Module 1: `02a_parse_core.dag` (~380 lines)

Module path: `v2.compiler.parse.core`

Contents: 55 type definitions + 20 core helper functions.

Lines 1-475 of current file.

Imports: `v2.std.core`

### Module 2: `02b_parse_type.dag` (~650 lines)

Module path: `v2.compiler.parse.types`

Contains: Type expressions, fields, predicates, variants, params.
Also includes parse_expr since type parsing needs it for field defaults,
and parse_expr needs parse_type_expr for casts. The solution is to put
the small, self-contained parts here.

**Functions (18 + shared utilities):**
```
parse_type_expr, finish_type_expr_from_name, maybe_optional,
parse_type_def, parse_type_body_after_eq,
try_where_clause, parse_predicates, parse_single_predicate,
parse_named_int_args, parse_single_named_int,
parse_variant_fields, parse_more_variants,
parse_field_list, parse_field, parse_optional_from_key,
parse_params, parse_param_list, parse_param,
parse_optional_return_type
```

**Note:** parse_single_predicate and parse_field both call parse_expr.
These are the only calls from this module into expression territory.
We handle this by having parse_field and parse_single_predicate accept
a simplified expression: for field defaults, the expression is always
a literal or simple value; for predicates, it is always a string literal.
We can add a `parse_simple_expr` that handles only literals and variables,
avoiding the need to import the full expression parser.

**Revised approach:** Actually, field defaults can be arbitrary expressions
(e.g., `field: Type = some_func(arg: val)`). We cannot simplify this.

### FINAL RECOMMENDED DESIGN

Given the fundamental constraint that .dag modules cannot circularly import,
and expressions/types/services all call each other, the **correct** split is:

## Proposed Modules (6 files)

### `02a_parse_core.dag` -- Types and Helpers (~380 lines)

Module: `v2.compiler.parse.core`

**Types (55):** All type definitions currently at lines 64-151 and line 2785.
Add explicit `type BindingPower { left: Int, right: Int }`.

**Functions (20):**
- peek, peek_kind, at_end, current_span, advance
- parse_error, has_err
- kind_tag, check, expect, expect_ident, expect_name, keyword_to_name
- skip_newlines, skip_continuation_newlines, eat
- is_ident, is_keyword_name
- parse_dotted_ident, parse_dotted_ident_rest

**Imports:** `v2.std.core`

**Imported by:** All other parse modules.

### `02b_parse_expr.dag` -- Expression Parser (~900 lines)

Module: `v2.compiler.parse.expr`

**Functions (51):**
- Pratt core: parse_expr, parse_expr_bp, parse_expr_loop, infix_bp,
  token_to_binop, parse_pipe_rhs, parse_prefix, parse_primary
- Postfix: try_postfix, make_call_expr, parse_index_or_slice
- Call args: parse_call_args, parse_arg_list, parse_single_arg,
  try_named_arg_label, keyword_to_arg_label
- Statements/blocks: parse_block, parse_stmts, parse_stmts_acc,
  parse_stmt, peek_is_eq_after_ident, parse_bare_assignment
- Ident expr: parse_ident_expr, is_uppercase_start
- Match: parse_match, parse_expr_no_brace, parse_expr_bp_no_brace,
  parse_expr_loop_no_brace, parse_match_arms, parse_match_arms_acc,
  parse_match_arm, parse_match_arm_body, parse_match_arm_stmts,
  looks_like_arm_start, peek_is_fat_arrow_at, peek_is_tag_at,
  scan_for_fat_arrow_after_braces, scan_braces_depth,
  parse_optional_guard, parse_pattern, parse_variant_pattern,
  parse_variant_bindings_brace
- If/let/for/return: parse_if, parse_let, parse_return, parse_for
- Literals: parse_record_literal, parse_field_init_list, parse_field_init,
  parse_list_literal, parse_expr_list_until
- Lambda: parse_paren_expr, parse_fn_lambda, collect_fn_lambda_params,
  try_lambda_params, collect_lambda_idents, parse_lambda_body,
  parse_lambda_stmts
- String: parse_string_interp, parse_interp_parts
- Brace: parse_brace_expr, peek_is_colon_after_ident
- Type (inlined): parse_type_expr, finish_type_expr_from_name, maybe_optional

**Why type parsing is here:** parse_type_expr is called by try_postfix (for `as`
casts), parse_field (for field types), and parse_param (for param types). Rather
than creating a circular dependency, we include the 3 core type-expression
functions here. The type *definition* parsing (parse_type_def, variants, predicates)
stays in the items module and calls parse_type_expr from this module.

**Imports:**
- `v2.std.core`
- `v2.compiler.parse.core` (all types + helpers)

**Imported by:** 02c, 02d, 02e

### `02c_parse_items.dag` -- Item Definitions (~650 lines)

Module: `v2.compiler.parse.items`

**Functions (25):**
- Type definitions: parse_type_def, parse_type_body_after_eq,
  try_where_clause, parse_predicates, parse_single_predicate,
  parse_named_int_args, parse_single_named_int,
  parse_variant_fields, parse_more_variants
- Fields (shared): parse_field_list, parse_field, parse_optional_from_key
- Params (shared): parse_params, parse_param_list, parse_param
- Fn/func: parse_fn_def, parse_func_def,
  parse_uses_clause, parse_uses_list, parse_uses_entry,
  parse_optional_return_type
- Data/extern: parse_data_def, parse_extern_decl

**Imports:**
- `v2.std.core`
- `v2.compiler.parse.core` (all types + helpers)
- `v2.compiler.parse.expr` (parse_expr, parse_block, parse_type_expr)

**Imported by:** 02d, 02e

### `02d_parse_service.dag` -- Service/Resource Parsing (~820 lines)

Module: `v2.compiler.parse.service`

**Functions (29):**
- Service: parse_service_def, parse_service_body, parse_service_entries,
  parse_service_config_block, parse_config_fields
- Transport: parse_transport_binding, parse_rest_binding_body, parse_rest_fields,
  parse_shell_binding_body, parse_shell_fields,
  parse_file_binding_body, parse_file_fields
- Operations: parse_operation_def, parse_operation_v2_inline,
  parse_operation_v1_body, parse_op_body_entries,
  parse_exit_entries, parse_operation_modifiers,
  parse_status_pattern
- Response/mock: parse_optional_response_block, parse_response_entries,
  parse_optional_mock_response_block, parse_mock_response_entries
- Resource: parse_resource_def, parse_resource_entries, skip_until_rbrace,
  parse_capability, parse_input_output_blocks, parse_io_blocks_acc

**Imports:**
- `v2.std.core`
- `v2.compiler.parse.core` (all types + helpers)
- `v2.compiler.parse.expr` (parse_expr, parse_expr_list_until,
  parse_type_expr, peek_is_colon_after_ident)
- `v2.compiler.parse.items` (parse_field_list, parse_optional_return_type)

**Imported by:** 02e

### `02e_parse.dag` -- Entry Point / Facade (~200 lines)

Module: `v2.compiler.parse`

This is the module that 06_pipeline.dag imports from. It keeps the same
module path so the pipeline import `v2.compiler.parse { parse, ParseResult }`
continues to work unchanged.

**Functions (7):**
- parse, parse_module
- parse_imports, parse_import, parse_import_names
- parse_items, parse_items_acc, parse_item

**Imports:**
- `v2.std.core`
- `v2.compiler.parse.core` (ParserState, ParseResult, ModuleResult, etc.)
- `v2.compiler.parse.expr` (parse_block)
- `v2.compiler.parse.items` (parse_type_def, parse_fn_def, parse_func_def,
  parse_data_def, parse_extern_decl)
- `v2.compiler.parse.service` (parse_service_def, parse_resource_def)

**Imported by:** `v2.compiler.pipeline` (06_pipeline.dag)

## Dependency Graph

```
v2.std.core
     |
02a_parse_core  (types + helpers)
     |
02b_parse_expr  (expressions, match, type_expr)
     |       \
02c_parse_items  02d_parse_service
     |       /        |
     +------+---------+
     |
02e_parse  (entry point)
     |
06_pipeline
```

Arrows represent import direction (importer -> imported):
- 02b imports 02a
- 02c imports 02a, 02b
- 02d imports 02a, 02b, 02c
- 02e imports 02a, 02b, 02c, 02d

No circular imports. Strict DAG ordering.

## Estimated Line Counts

| Module | Functions | Types | Est. Lines |
|--------|-----------|-------|------------|
| 02a_parse_core.dag | 20 | 56 | ~380 |
| 02b_parse_expr.dag | 54 | 0 | ~950 |
| 02c_parse_items.dag | 25 | 0 | ~650 |
| 02d_parse_service.dag | 29 | 0 | ~820 |
| 02e_parse.dag | 7 | 0 | ~200 |
| **Total** | **135** | **56** | **~3000** |

Note: Total is ~3000 vs. original 3700 because each module needs its own
module/import declarations, but the function bodies are the same. The 700-line
reduction comes from the type definitions being shared rather than duplicated
(they live only in 02a). The actual total with import boilerplate will be
closer to ~3200.

## What Moves Where: Key Decisions

1. **parse_type_expr in 02b (expr) not 02c (items):** This is the most important
   decision. parse_type_expr is needed by try_postfix (cast), parse_field, and
   parse_param. If it were in 02c, then 02b would need to import from 02c, but
   02c already imports from 02b -- creating a cycle. By placing parse_type_expr
   in 02b, we keep the dependency DAG clean.

2. **parse_field_list in 02c (items) not 02b (expr):** Fields are primarily an
   item/type concept. parse_field calls parse_type_expr (from 02b) and
   parse_expr (from 02b), so 02c importing 02b is correct.

3. **parse_optional_return_type in 02c (items):** Used by fn/func defs (02c)
   and operations/capabilities (02d). Since 02d imports 02c, this works.

4. **peek_is_colon_after_ident in 02b (expr):** Used by parse_brace_expr,
   parse_field_init, parse_op_body_entries, and parse_resource_entries. Since
   02d imports 02b, this works.

5. **parse_expr_list_until in 02b (expr):** Used by parse_list_literal (02b)
   and parse_shell_fields (02d). Since 02d imports 02b, this works.

## Migration Plan

### Step 1: Create 02a_parse_core.dag

Extract all type definitions and core helper functions. Add the explicit
BindingPower type. Verify this compiles independently.

### Step 2: Create 02b_parse_expr.dag

Move all expression, match, pattern, lambda, literal, string interpolation,
and brace expression functions. Include parse_type_expr, finish_type_expr_from_name,
and maybe_optional. Add proper module/import declarations.

### Step 3: Create 02c_parse_items.dag

Move type definitions (parse_type_def cluster), field/param parsing, fn/func/data/
extern definitions, and uses clause parsing.

### Step 4: Create 02d_parse_service.dag

Move all service, transport, operation, resource, and capability parsing.

### Step 5: Create 02e_parse.dag

Move entry point (parse, parse_module) and module-structure functions (parse_imports,
parse_items, parse_item).

### Step 6: Update v1 emitter

Update `V2_MODULE_MAP` and `MODULE_PATH_TO_RUST_MOD` in
`src/v1/07_emit/daglang-emit/src/v2_crate_emit.rs`:

```rust
const V2_MODULE_MAP: &[(&str, &str)] = &[
    ("00_core", "v2_core"),
    ("01_tokenize", "tokenize"),
    ("02a_parse_core", "parse_core"),
    ("02b_parse_expr", "parse_expr"),
    ("02c_parse_items", "parse_items"),
    ("02d_parse_service", "parse_service"),
    ("02e_parse", "parse"),
    ("03_resolve", "resolve"),
    ("04_typecheck", "typecheck"),
    ("05_emit", "emit"),
    ("06_pipeline", "pipeline"),
];

const MODULE_PATH_TO_RUST_MOD: &[(&str, &str)] = &[
    ("v2.std.core", "v2_core"),
    ("v2.compiler.tokenize", "tokenize"),
    ("v2.compiler.parse.core", "parse_core"),
    ("v2.compiler.parse.expr", "parse_expr"),
    ("v2.compiler.parse.items", "parse_items"),
    ("v2.compiler.parse.service", "parse_service"),
    ("v2.compiler.parse", "parse"),
    ("v2.compiler.resolve", "resolve"),
    ("v2.compiler.typecheck", "typecheck"),
    ("v2.compiler.emit", "emit"),
    ("v2.compiler.pipeline", "pipeline"),
];
```

### Step 7: Update v2 pipeline imports

06_pipeline.dag already imports `v2.compiler.parse { parse, ParseResult }`.
Since 02e_parse.dag keeps the module path `v2.compiler.parse`, this import
continues to work. However, ParseResult is now defined in 02a_parse_core.dag
(module `v2.compiler.parse.core`). The v1 evaluator needs to resolve this
re-export -- the entry module 02e should either re-export ParseResult or
the pipeline should import it from parse.core.

Simplest approach: 06_pipeline.dag changes to:
```
import v2.compiler.parse { parse }
import v2.compiler.parse.core { ParseResult }
```

### Step 8: Update test file references

The self-parse test in v2_crate_emit.rs that skips 02_parse.dag due to OOM
should be updated to include all 5 parse sub-modules in the test, verifying
that the split resolved the OOM.

### Step 9: Delete 02_parse.dag

Once all tests pass with the split modules, remove the original monolithic file.

## Risks and Mitigations

**Risk:** The v1 evaluator's import resolution may not handle sub-module paths
like `v2.compiler.parse.core`.
**Mitigation:** The evaluator already handles dotted paths (v2.std.core,
v2.compiler.tokenize). Adding one more level should work. Verify in Step 1.

**Risk:** The compiled crate's `use` statements may conflict when multiple
parse modules are in scope.
**Mitigation:** The Rust module names are distinct (parse_core, parse_expr, etc.)
and each module's public symbols are different. No name conflicts expected.

**Risk:** parse_type_expr being in 02b (expr) rather than with type definitions
may be confusing.
**Mitigation:** Add a clear comment in 02b explaining why parse_type_expr lives
here (dependency ordering), and a comment in 02c pointing to 02b for type
expression parsing.
