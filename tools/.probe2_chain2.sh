#!/bin/bash
set -u
B() { cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2; }
B
for round in 1 2 3 4; do
  echo "=== regen round $round $(date)"
  ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | grep -v '^\[' | tail -5 | cut -c1-200
  n=0
  C=target/stage0-regen-candidate/src
  for f in $(cd $C && find . -type f); do
    if ! cmp -s $C/$f src/v1/stage0/src/$f; then cp $C/$f src/v1/stage0/src/$f; n=$((n+1)); echo "installed $f"; fi
  done
  echo "installed $n"
  [ $n -eq 0 ] && break
  B
done
grep -c CHAIN2CENSUS src/v1/stage0/src/*.rs | grep -v ':0'
G=./target/release/gunbc
P=.probe_src; rm -rf $P; mkdir -p $P
HDR='import std.types { String, NonEmptyStr, Int, Bool, List }
import std.nat { Nat }
import std.process { ProcessExit, exit_failure }
type P { value: Int }
type R { s: String  p: P  n: Int }
fn mk_r() -> R { R { s: "x", p: mk_p(), n: 3 } }
type C = Red | Green { n: Int }
fn ne() -> NonEmptyStr { "ne" as NonEmptyStr }
fn nat() -> Nat { 5 }
fn mk_p() -> P { P { value: 7 } }
fn mk_c() -> C { Green { n: 7 } }
fn mk_cr() -> C { Red }
fn main() -> ProcessExit { exit_failure(reason: BODY) }'
probe() {
  mkdir -p $P/$1; printf 'module %s\n%s\n' "$1" "${HDR/BODY/$2}" > $P/$1/p.dag
  echo "=== $1 :: $2"
  timeout 600 $G run --source-root $P/$1 --source-root dag --entry $P/$1/p.dag --function main 2>&1 | grep -e CHAIN2 -e 'error' | head -4 | cut -c1-200
}
for pair in 'record|mk_p()' 'coproduct_unit|mk_cr()' 'coproduct_rec|mk_c()' 'int|7' 'nat|nat()' 'bool|true' 'nes|ne()' 'string|"s"'; do
  n=${pair%%|*}; e=${pair#*|}
  probe concat_$n "concat(\"id=\", $e)"
  if [ "$n" = "string" ] || [ "$n" = "nes" ]; then L="[\"a\", $e]"; else L="[$e, $e]"; fi
  probe join_$n "join($L, \",\")"
  probe joinsep_$n "join([\"a\", \"b\"], $e)"
done
probe listconcat "concat([1, 2], [3])"
probe listconcat_rec "concat([mk_p()], [mk_p()])"
probe joinlist_rec "join([mk_p()], \",\")"

probe field_s_first "concat(mk_r().s, \"x\")"
probe field_s_join "join([mk_r().s], \",\")"
probe field_s_sep "join([\"a\"], mk_r().s)"
probe field_p_first "concat(mk_r().p, \"x\")"
probe field_n_first "concat(mk_r().n, \"x\")"
probe field_s_if "concat(mk_r().s, if true { \" h\" } else { \" n\" })"
