#!/bin/bash
# usage: rows.sh row:module ...   (run from repo root)
set -u
R=$(pwd)
cargo build --release -p v1-compiler --bin gunbc --bin cssl_assemble 2>&1 | tail -3
for spec in "$@"; do
  row=${spec%%:*}; mod=${spec#*:}
  O=$(mktemp -d); echo "######## $row ($mod)"
  ./target/release/gunbc compile --source-root dag --source-root src/v2 --entry src/v2/compiler/$mod.dag --output-dir $O --target rust --dependency-pool-index primary-precedence >$O.emit 2>&1 || { echo EMIT_FAIL; tail -5 $O.emit; }
  mkdir -p $O/src
  ./target/release/cssl_assemble --out-dir $O --entry-dag src/v2/compiler/$mod.dag --root $R 2>&1 | tail -2
  printf '[package]\nname = "v1_compiled"\nversion = "0.1.0"\nedition = "2021"\n\n[features]\ntext_lookup_work_counter = []\n\n[[bin]]\nname = "witness"\npath = "src/main.rs"\n\n[dependencies]\nim = { version = "15.1", features = ["serde"] }\nserde = { version = "1", features = ["derive", "rc"] }\nserde_json = "1"\nstacker = "0.1"\nlazy_static = "1"\nunicode-ident = "1"\nunicode-properties = { version = "0.1", features = ["emoji"] }\nv1-compiler = { path = "%s/src/v1/stage0" }\n' "$R" > $O/Cargo.toml
  cp dag/gunbc/instruments/self_host_${row}_shims/witness_main.rs $O/src/main.rs
  (cd $O && CARGO_TARGET_DIR=$R/target/witness_$row cargo build --release 2>&1 | grep -E '^(error|warning: unused)|-->' | sort | uniq -c | sort -rn | head -25)
  if [ -x $R/target/witness_$row/release/witness ]; then $R/target/witness_$row/release/witness | tail -3; $R/target/witness_$row/release/witness --inject-fault | tail -1; fi
done
