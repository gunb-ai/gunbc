set -uo pipefail
export GUNBC_MEMORY_BUDGET_BYTES=24000000000 GITHUB_ACTIONS=true
cargo build --release -p v1-compiler --bin gunbc --bin claim_executor --bin claim_batch 2>&1|tail -1
./target/release/gunbc test //gunbc/instruments:v2-native-census-scoped > /tmp/census_full.txt 2>&1; echo "census exit $?"


grep -oiE "(declaration_?identity_?unavailable|callee_?unresolved)" /tmp/census_full.txt | sort | uniq -c
awk '/census_residual/ && /[Dd]eclaration_?[Ii]dentity_?[Uu]navailable|[Cc]allee_?[Uu]nresolved/{match($0,/[Dd]eclaration_?[Ii]dentity_?[Uu]navailable|[Cc]allee_?[Uu]nresolved/); pend=pend "\n  " substr($0,RSTART,RLENGTH)} /\[native-census-infer\]/{match($0,/"module":"[^"]*"/); if(pend!=""){print substr($0,RSTART,RLENGTH) pend}; pend=""}' /tmp/census_full.txt | head -60
grep -oE '"_variant":"[A-Za-z]*Unavailable"|"_variant":"[A-Za-z]*Unresolved"' /tmp/census_full.txt | sort | uniq -c
