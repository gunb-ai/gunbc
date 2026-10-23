#!/bin/bash
# PARENT RULING PROBE (msg_0e84a10b + msg_812952da). At the merged tree, compile the
# corpus through the seed's v2 self-compile path and answer, with execution:
#  (Q1) are the 2 same-text FreeMonoid diagnostics at 02_parse.dag:1960/1979 the
#       rebind refusal? (print each full message + which emitter)
#  (Q2) do the 220 CI fold diagnostics reproduce here, and do they include the
#       same-text-under-shape pattern (equals=false while shapes print equal)?
#  (Q3) for one 02_parse site, print node_type_equals and node_type_shape(left) vs
#       node_type_shape(right) directly (throwaway test), so identity-vs-shape is
#       decided on the nodes, not on the printed pair.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
export RUSTUP_HOME=/opt/rustup
cd .
echo "Q: HEAD(runner mirror base)=$(git rev-parse HEAD)"
echo "Q: fold in .dag U=$(grep -c UnifyGenericsAcc src/v1/04_infer.dag) seed infer U=$(grep -c UnifyGenericsAcc src/v1/stage0/src/v1_compiler_infer.rs)"
if [ ! -x target/release/gunbc ] || [ ! -x target/release/claim_executor ]; then
  cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | tail -2
fi
echo "########## Q-regen: full refusal text ##########"
target/release/claim_executor --regen-round-cost --source-root dag --source-root src/v2 > /tmp/q_regen.log 2>&1
echo "rc=$?"
grep -E "refused|hard diagnostic|type mismatch|bound as" /tmp/q_regen.log | head -8
echo "bound-as count in regen log: $(grep -c 'bound as' /tmp/q_regen.log)"
echo "########## Q-v2boot: v2-native-cli instrument (CI-failing lane) ##########"
target/release/gunbc test //gunbc/instruments:v2-native-cli > /tmp/q_v2.log 2>&1
echo "rc=$?"
grep -c "bound as" /tmp/q_v2.log | sed 's/^/Q: fold-message diagnostics in v2 lane: /'
grep "bound as" /tmp/q_v2.log | head -3 | cut -c1-220
echo "########## Q-selfhost: self-host instrument (02_parse judge) ##########"
target/release/gunbc test //gunbc/instruments:self-host > /tmp/q_sh.log 2>&1
echo "rc=$?"
grep -c "bound as" /tmp/q_sh.log | sed 's/^/Q: fold-message diagnostics in self-host: /'
grep -E "FreeMonoid,Primitive\(element\)" /tmp/q_sh.log | head -4 | cut -c1-200
echo "Q_DONE"
