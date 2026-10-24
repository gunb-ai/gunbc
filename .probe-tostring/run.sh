#!/usr/bin/env bash
# THROWAWAY step-0 measurement (never merge).
# Question (manager tidy-ant-251): the KernelGrounding row is keyed on source_kind KernelIntLiteral.
# Which declared route admits NON-literal kernel Int values at std.integer.int_to_decimal_string's
# coproduct-typed formal? Discriminators:
#  M1/M3  non-literal kernel Int (parameter / call result)      -> admitted?
#  M4     Bool at the Int formal (negative: arm is not blanket)  -> must REFUSE
#  M5     a LOCAL `type Int` coproduct value at the same formal  -> discriminates IDENTITY-keyed
#         admission (KernelGrounding arm, same_declaration_site) from NAME-keyed admission
#         (declared_realizes_as_kernel_numeric, keyed on numeric_realization_declaring_modules
#          + kernel_carrier_admits_numeral(produced_name)).
set -u
export CARGO_TARGET_DIR=$PWD/target RUSTC_WRAPPER=
echo "HEAD $(git rev-parse HEAD)"
cargo build -q --release -p v1-compiler --bin gunbc || { echo BUILD-FAIL; exit 2; }
echo "BUILD-OK"
run() { n=$1; body=$2; d=.probe-tostring/$n; rm -rf $d; mkdir -p $d
  printf 'module %s\nimport std.types { String, Int, Bool, List }\nimport std.nat { Nat }\nimport std.integer { int_to_decimal_string }\n%s\nfn main() -> String { "x" }\n' "$n" "$body" > $d/p.dag
  ./target/release/gunbc run --source-root dag --source-root src/v2 --source-root $d --entry $d/p.dag > $d/out.log 2>&1
  rc=$?
  diag=$(grep -Eo "DeclaredTypeNotInhabited[^\"]*|expected [A-Za-z]+, found [A-Za-z]+|error\[E[0-9]+\]" $d/out.log | sort -u | head -3 | tr '\n' ';')
  echo "RESULT $n rc=$rc diag=${diag:-NONE}"
}
run m1_var       'fn f(n: Int) -> String { int_to_decimal_string(value: n) }'
run m2_literal   'fn f() -> String { int_to_decimal_string(value: 42) }'
run m3_call      'fn f() -> String { int_to_decimal_string(value: list_length(items: [1, 2, 3])) }'
run m4_bool_neg  'fn f() -> String { int_to_decimal_string(value: true) }'
run m5_local_int 'type Int = Alpha { a: Nat } | Beta
fn f(x: Int) -> String { int_to_decimal_string(value: x) }'
run m6_untagged  'fn f(n: Int) -> String { int_to_decimal_string(n) }'
run m7_nat_at_int 'fn f(n: Nat) -> String { int_to_decimal_string(value: n) }'
run m8_local_named 'type MyInt = Alpha { a: Nat } | Beta
fn f(x: MyInt) -> String { int_to_decimal_string(value: x) }'
