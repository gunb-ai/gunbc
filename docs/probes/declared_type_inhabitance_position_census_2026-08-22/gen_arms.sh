#!/usr/bin/env bash
# Generates one .dag arm per (position, specimen) into $ARMDIR/<arm>/probe.dag
set -u
ARMDIR="$1"
rm -rf "$ARMDIR"; mkdir -p "$ARMDIR"

PRELUDE='module probe.inhabit

type Inner { v: Int }

type Rel = | Wrapped(Inner) | SameRev

type Boxed<T> { inner: T }

type Holder { rel: Rel }

fn mk_inner() -> Inner { Inner { v: 1 } }

fn take_rel(r: Rel) -> Int { 1 }
'

emit() { # name body
  d="$ARMDIR/$1"; mkdir -p "$d"
  { printf '%s\n' "$PRELUDE"; printf '%s\n' "$2"; } > "$d/probe.dag"
}

# value specimens
#   pos  = correctly wrapped member          -> expect ACCEPT
#   nega = plain kernel value                -> expect REFUSE
#   negb = arm payload at parent position    -> expect REFUSE
#   reach= undefined name                    -> expect REFUSE (proves position reached)
V_pos='SameRev'
V_nega='7'
V_negb='mk_inner()'
V_reach='nosuchname_zzz'

for s in pos nega negb reach; do
  eval "v=\$V_$s"
  emit "field_$s"   "data a_field: Holder = Holder { rel: $v }"
  emit "data_$s"    "data a_data: Rel = $v"
  emit "return_$s"  "fn a_ret() -> Rel { $v }"
  emit "arg_$s"     "fn a_arg() -> Int { take_rel(r: $v) }"
  emit "let_$s"     "fn a_let() -> Int {
  let x: Rel = $v
  1
}"
  emit "cast_$s"    "fn a_cast() -> Int {
  let x = $v as Rel
  1
}"
  emit "listelem_$s" "data a_list: List<Rel> = [$v]"
  emit "mapval_$s"  "data a_map: Map<String, Rel> = { \"a\": $v }"
  emit "generic_$s" "data a_gen: Boxed<Rel> = Boxed { inner: $v }"
  emit "variantpayload_$s" "data a_vp: Rel = Wrapped($v)"
  emit "lambdaret_$s" "data a_fn: fn(Int) -> Rel = fn(n) { $v }"
  emit "paramdefault_$s" "fn a_pd(r: Rel = $v) -> Int { 1 }"
  emit "callableparam_$s" "fn a_cb(cb: fn(Rel) -> Int) -> Int { cb($v) }"
done

# map KEY position (declared key type Rel)
emit "mapkey_pos"  'data a_mk: Map<Rel, Int> = { SameRev: 1 }'
emit "mapkey_nega" 'data a_mk: Map<Rel, Int> = { 7: 1 }'
emit "mapkey_negb" 'data a_mk: Map<Rel, Int> = { mk_inner(): 1 }'
emit "mapkey_reach" 'data a_mk: Map<Rel, Int> = { nosuchname_zzz: 1 }'
