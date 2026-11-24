# Sig-Unresolved Fallthrough Census

## Method

Counts call sites that hit the sig-unresolved fallthrough path: bare calls to
kernel method template names with labeled arguments, where the callee is
NEITHER locally declared NOR imported by the calling module.

These calls reach CallableUnresolved (sig=Absent) → method bridge → positional
binding → labels NEVER checked against any declared parameter list.

## Criteria

- Callee name matches a kernel method template name (algebra_method_template_name gate)
- Call has at least one labeled argument (pattern: `name(label:` with word-boundary anchor)
- The calling module does NOT declare `fn name(` locally (func_env.local won't have it)
- The calling module does NOT list the name in any import (parent_closure_callable_candidates won't have it)
- Excludes: method calls (.name(), pipe-receiver (|> name(), comments, imports, declarations

### Routes NOT filterable statically (error direction stated)

- parent_closure_callable_candidates (the chain walks imports; if not imported, the name
  cannot enter the parent chain, so the import filter subsumes this route)
- builtin/kernel table: 13 of the 31 names are in builtin_function_registry. However,
  builtin_positional_arity_diags filters OUT labeled args (line 8606: Present{value:_} => false),
  so the builtin path also leaves labels unchecked. The blind spot is the same whether the
  method bridge or the builtin path handles the call — labels are ignored either way.
  Error direction: if the builtin path adds ANY label validation in the future, those
  13 names would leave the census, making it conservative.

## Results at 1d8a1d397675 (main)

### OLD (INVALIDATED by the executed control):
  TOTAL: 4597 sites, 10 names, 1128 modules
  Predicate: imported-and-not-locally-declared
  Why invalidated: the executed red control (imported_kernel_name_mislabeled_call_is_refused)
  proves these resolve via parent_closure_callable_candidates, the wall fires, and they
  ARE protected. Counting them as blind spot was wrong.

### CORRECTED:
  TOTAL_SITES: 5890
  UNIQUE_MODULES: 1327
  UNIQUE_NAMES: 10 (split:1337, starts_with:1140, substring:900, to_string:497,
    to_upper:383, map_get:374, to_int:345, map_keys:303, map_has:122, skip:85)
  Predicate: NEITHER locally declared NOR imported

## Red Control (executed, green by execution in PR)

  PASS unimported_kernel_name_mislabeled_call_compiles_clean (RED: 0 diagnostics)
  PASS imported_kernel_name_mislabeled_call_is_refused (LICENSED TWIN: wall fires)
  PASS imported_correctly_labeled_call_compiles_clean (NEGATIVE CONTROL: no over-refusal)

  Binary: claim_batch sha256 fab6957cb4e614d42efb8e4ad00832c0a585ad7f37f10e6c874a3f6920d463fe
  Host: srv1 BOOT=c095c0cd
  Mode: Hermetic, 16 modules, 463 resolved items in closure
  RC=0

## Monotone

The corrected count falls as namespace-lane coverage lands (more callees resolve,
leaving CallableUnresolved). When coverage reaches a name, its calls move from
unimported (fallthrough) to imported-or-resolved (wall fires). The licensed twin
becomes the expected behavior for those sites.
