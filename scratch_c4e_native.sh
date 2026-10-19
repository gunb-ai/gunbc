set -u
cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
export GUNBC_BIND_MEMORY_CGROUP_BYTES=21474836480
s=$(date +%s)
timeout 3000 target/release/gunbc test //gunbc/instruments:v2-native-census > /tmp/census_out.txt 2>/tmp/census_err.txt
echo "rc=$? wall_s=$(( $(date +%s) - s ))"
wc -l /tmp/census_out.txt /tmp/census_err.txt
grep -n "claim_coverage_census\|instrument_targets\|local_repo_wet\|module_acquisition" /tmp/census_out.txt | head -40
tail -40 /tmp/census_out.txt
tail -5 /tmp/census_err.txt
