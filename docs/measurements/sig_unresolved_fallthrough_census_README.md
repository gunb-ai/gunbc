# Sig-Unresolved Fallthrough Census

## Method

Counts call sites that hit the sig-unresolved fallthrough path: bare calls to
kernel method template names with labeled arguments, where the callee is
imported by the calling module (so its identity IS known) but not declared
locally (so func_env_declared_func_sig cannot resolve it via func_env.local).

These calls are routed to the method bridge (sig=Absent → DeclaredCallableIdentityResolved
→ method bridge) instead of being checked by direct_call_shape_wall_note.

## Criteria

- Callee name matches a kernel method template name (is_kernel_method_name gate)
- Call has at least one labeled argument (pattern: `name(label:`)
- The calling module explicitly imports the name (import line lists it)
- The calling module does NOT declare `fn name(` locally (func_env.local won't have it)
- Excludes: method calls (.name(, pipe-receiver (|> name(), comments, imports, declarations

## Results at 1d8a1d397675 (main)

TOTAL_SITES: 4597
UNIQUE_MODULES: 1128
UNIQUE_NAMES: 10

Per-name:
  562 length
  377 trim
  107 map_insert
   75 contains
   61 is_empty
   55 skip
   48 chars
   26 map_contains_key
   14 list_push
    3 substring

## Red control

The count monotonically falls as the namespace lane's resolution coverage
improves (more callees resolve their sig via func_env, exiting the fallthrough).

## Reproduction

Run the census script (light, uses git grep — no expensive acquisitions).
