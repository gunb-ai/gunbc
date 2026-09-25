#!/bin/bash
# usage: drive.sh row:module ...  (repo root; needs target/release/cssl_assemble and probe/emit_<row>.tgz)
set -u
R=$(pwd)
for spec in "$@"; do
  row=${spec%%:*}; mod=${spec#*:}
  O=$R/target/drive_$row; rm -rf $O; mkdir -p $O; tar xzf probe/emit_$row.tgz -C $O
  echo "######## $row"
  ./target/release/cssl_assemble --out-dir $O --entry-dag src/v2/compiler/$mod.dag --root $R 2>&1 | tail -1
  printf '[package]\nname = "v1_compiled"\nversion = "0.1.0"\nedition = "2021"\n\n[features]\ntext_lookup_work_counter = []\n\n[[bin]]\nname = "witness"\npath = "src/main.rs"\n\n[dependencies]\nim = { version = "15.1", features = ["serde"] }\nserde = { version = "1", features = ["derive", "rc"] }\nserde_json = "1"\nstacker = "0.1"\nlazy_static = "1"\nunicode-ident = "1"\nunicode-properties = { version = "0.1", features = ["emoji"] }\nv1-compiler = { path = "%s/src/v1/stage0" }\n' "$R" > $O/Cargo.toml
  cp dag/gunbc/instruments/self_host_${row}_shims/witness_main.rs $O/src/main.rs
  (cd $O && CARGO_TARGET_DIR=$R/target/witness_$row cargo build --release 2>&1 | grep -E '^error|^ *--> ' | head -40)
  B=$R/target/witness_$row/release/witness
  if [ -x $B ] && [ $B -nt $O/src/main.rs ]; then $B | tail -4; echo "exit=$?"; $B --inject-fault | tail -1; else echo BUILD_FAIL; fi
done
