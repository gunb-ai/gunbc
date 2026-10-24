#!/usr/bin/env bash
# THROWAWAY step-0 measurement (never merge): which declared route admits NON-literal kernel Ints
# at std.integer.int_to_decimal_string's coproduct-typed formal.
set -u
export CARGO_TARGET_DIR=$PWD/target RUSTC_WRAPPER=
echo "HEAD $(git rev-parse HEAD)"
cargo build -q --release -p v1-compiler --bin gunbc || { echo BUILD-FAIL; exit 2; }
run() { n=$1; body=$2; d=.probe-tostring/$n; rm -rf $d; mkdir -p $d
  printf 'module %s\nimport std.types { String, Int, Bool, Nat }\nimport std.integer { int_to_decimal_string }\n%s\nfn main() -> String { "x" }\n' "$n" "$body" > $d/p.dag
  echo "=== $n ==="
  ./target/release/gunbc run --source-root dag --source-root src/v2 --source-root $d --entry $d/p.dag > $d/out.log 2>&1
  echo "exit=$?"
  grep -Ei "error|diagnostic|refus" $d/out.log | head -4 | cut -c1-260
}
run p1_nonliteral 'fn f(n: Int) -> String { int_to_decimal_string(value: n) }'
run p2_literal    'fn f() -> String { int_to_decimal_string(value: 42) }'
run p3_call       'fn f() -> String { int_to_decimal_string(value: list_length(items: [1, 2, 3])) }'
run p4_neg_bool   'fn f() -> String { int_to_decimal_string(value: true) }'
run p5_neg_local  'type Int = Alpha { a: Nat } | Beta
fn f(x: Int) -> String { int_to_decimal_string(value: x) }'
