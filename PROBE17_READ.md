# Probe v17 — the static read replaced it; this records the reasoning + the repair

## Parent ruling (msg_2d4b0512): "(ii) then (i), both inside #14152 ... Cheaper read first,
## since it may make v17 unnecessary ... If one site returns it without substitution through
## the unify result, that is the answer. Run v17 only if the read is ambiguous."

## THE READ — ONE SITE. v17 NOT RUN.

Producer 1 (39 of 46 Map-family rows): the bound node is `dag/std/algebra.dag:73644-73647`,
the `Map` token of line 1406 `fn empty_map<K, V>() -> Map<K, V> {` — the callee's DECLARED
return type, children K,V parsed at 73648-73649/73651-73652, each carrying
DeclarationRef{module_path:"std.algebra", decl_name:"empty_map", field:TypeParameter{...}}.

The site that hands it to the fold is `borrowed_callable_call_type` (src/v1/04_infer.dag:6499).
Mechanism, every link verified against the tree at 8d8d8aac0e1:

1. tp_names = fn_type_param_names(item: gnode) = ["K","V"] (04_resolve.dag:1311), so the
   `count == 0` degenerate arm at 6502-6504 does NOT fire.
2. value_params = gnode.params filtered to NON-generic params (6506) = [] for empty_map — its
   only params are the generic K,V.
3. the fold at 6507-6520 therefore runs over an EMPTY list -> unified.subst = empty_map().
4. substitute_generics (15292-15296) short-circuits: `if map_is_empty(subst) { n }` —
   the declared return type is returned **literally, un-instantiated**.
5. that node becomes resolved_type -> the `actual` at the 8227 fold site (bind A), and a later
   call with a concrete Map<Int,String> rebinds A -> my Acc's rebind diagnostic -> the 77/67.

Sibling arms ruled out: 9652 `callable_return_type(n: gnode)` never substitutes either, but it
requires `(gnode.params |> count) == 0` — empty_map HAS params, so it cannot be that arm.
The type_match arm (9658) returns an env-registered type node, not a declaration read.
=> unambiguous, so per the ruling v17 was NOT dispatched.

Producer 2 (7 rows): minter settled by ELIMINATION (already reported): bare_map_node has
exactly ONE caller in the real tree (04_infer.dag:9430, the seeded empty_map constructor arm).
Probe detail corrected here: the children print as `K@.../decl=None` — rebind16_names records
n.name, and bare_map_node's children have name = container_param_name("Map",i) = "K"/"V" from
std data (algebra_type_param_names: FinitelySupportedFunctionProfile => ["K","V"],
dag/std/algebra.dag:1177) with inferred TypeVariable{id:"K"}. Consistent.

## THE REPAIR (i) — reuse, do not add an instantiation path

Parent: "A declared generic return type is instantiated with fresh unsettled variables at each
use, through the instantiation path the fold already uses for other generic calls; do not add a
new one. bare_map_node mints fresh variables, not the literal names K/V."
MEASURED CORRECTION (already reported, accepted): bare_map_node's variables ARE keyed by the
literal formal names (TypeVariable{id:"K"}), because container_param_name reads std data.
There is no counter-based fresh-id source in the tree (`fn .*fresh` -> 0 hits); freshness in
this codebase is Rc identity, not id uniqueness. So "fresh" here = per-use distinct variable
NODES keyed by the slot name, exactly what the existing record-literal path mints.

The instantiation path the fold already uses for other generic calls is
`unbound_owner_param_variables` (04_infer.dag:11728-11731):
    fold(names, init: empty_map(), f: (acc, g) => map_insert(acc, g, type_variable_node(id: g)))
The repair seeds borrowed_callable_call_type's fold with exactly that map over tp_names,
inserting ONLY for slots the argument fold left unbound (never overwriting a solved slot).
Then the 6521 substitute_generics call is the ONLY instantiation mechanism (unchanged); the
bare name nodes K,V in the declared return type hit its bare-name arm and become
type_variable_node children, which my (c) arm (binding_has_type_variable_leaf) already treats
as unsettled -> skip -> the family's refusals disappear WITHOUT touching any fold arm.

Why not touch bare_map_node: it has one caller; its literal-K/V keying is load-bearing for the
seeded empty map's key/value lookup; and producer 2 is downstream of the SAME fold skip once
the bound side is unsettled, so minter surgery is neither necessary nor in the ruling.

## RED (the parent's own spec, msg_2d4b0512)
- 77 self-host hard diagnostics at the pre-repair commit 8d8d8aac0e1  [measured, gate3]
- 0 Map-family rows after the repair                    [census + gate, this dispatch]
- per-module two-closure comparison against main          [gate_two_closures_14152.sh]
If the Map-family rows do NOT go to 0, the repair is wrong and gets reverted, not widened.
