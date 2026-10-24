set -o pipefail
cargo build --release -p v1-compiler --bin gunbc || exit 1
./target/release/gunbc run --source-root dag --source-root src/v2 --entry src/v2/test/claim/name_resolve/declarer_refused_lambda.dag --function dl_probe > lam.out 2>&1; echo lam_exit=$?
echo ---OUT; tail -n 15 lam.out | cut -c1-600
