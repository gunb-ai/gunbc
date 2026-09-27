set -u
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
G=./target/release/gunbc
for fn in v2_zzp_arg_imp_green v2_zzp_arg_imp_red v2_zzp_ret_imp_green v2_zzp_ret_imp_red; do
  echo "=== CENSUS $fn"
  $G run --source-root dag --source-root src/v2 --source-root zzprobe --entry zzprobe/runner.dag --function $fn 2>&1 | tail -c 2500
done
