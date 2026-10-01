#!/bin/bash
# usage: census_run.sh native|claims
set -u
MODE=$1; BASE=66383cf58c048702a87c4cc51c24f4ff62541b02; HEAD_SHA=63fa734a975068079dc49ff41626365c3605f6f0
R=$(pwd); OUT=/tmp/cen; rm -rf $OUT; mkdir -p $OUT
echo "MemTotal: $(grep MemTotal /proc/meminfo)"
git fetch -q origin $BASE $HEAD_SHA 2>&1 | tail -1
mkdir -p /sys/fs/cgroup/gb 2>/dev/null; echo +memory > /sys/fs/cgroup/cgroup.subtree_control 2>/dev/null
echo 21474836480 > /sys/fs/cgroup/gb/memory.max && echo $$ > /sys/fs/cgroup/gb/cgroup.procs && echo "cgroup memory.max=$(cat /sys/fs/cgroup/gb/memory.max)"
export GUNBC_MEMORY_BUDGET_BYTES=${GUNBC_MEMORY_BUDGET_BYTES:-20000000000}
SIDES=${2:-"base head"}
for side in $SIDES; do
  S=$BASE; [ $side = head ] && S=$HEAD_SHA
  W=/tmp/wt_$side; rm -rf $W; git worktree add -qf --detach $W $S
  echo "$side rev-parse=$(git -C $W rev-parse HEAD) dirty=$(git -C $W status --porcelain | wc -l)"
  (cd $W && CARGO_TARGET_DIR=/tmp/tgt cargo build --release -q -p v1-compiler --bin gunbc --bin claim_batch 2>&1 | grep -E '^error' | head -3)
  mkdir -p /tmp/bin_$side; cp /tmp/tgt/release/gunbc /tmp/tgt/release/claim_batch /tmp/bin_$side/
  echo "$side gunbc sha256=$(sha256sum /tmp/bin_$side/gunbc | cut -c1-64)"
  cd $W
  if [ $MODE = native ]; then
    t0=$(date +%s); /tmp/bin_$side/gunbc test //gunbc/instruments:v2-native-frontier > $OUT/nat_$side.log 2>&1; echo "$side native exit=$? secs=$(( $(date +%s)-t0 )) peak=$(cat /sys/fs/cgroup/gb/memory.peak 2>/dev/null)"
    sed -n 's/^\[native-frontier-roster\] //p' $OUT/nat_$side.log | sort > $OUT/roster_$side.txt
    sed -n 's/^\[native-frontier\] //p' $OUT/nat_$side.log > $OUT/summary_$side.txt
    echo "$side roster_lines=$(wc -l < $OUT/roster_$side.txt)"; grep -avE "^\[native-frontier-roster\]" $OUT/nat_$side.log | tail -15 | cut -c1-300; head -c 1500 $OUT/summary_$side.txt; echo
  else
    : > $OUT/cl_$side.txt
    while read f; do
      [ -f "$f" ] || { echo "ABSENT $f" >> $OUT/cl_$side.txt; continue; }
      fns=$(grep -oP '(?<=^test fn )\w+' "$f" | paste -sd,)
      timeout 1800 /tmp/bin_$side/claim_batch --source-root dag --source-root src/v2 --entry "$f" --functions "$fns" > $OUT/one.log 2>&1; rc=$?
      for fn in ${fns//,/ }; do
        v=$(grep -oE "^(PASS|FAIL) $fn\b" $OUT/one.log | head -1 | cut -d' ' -f1); [ -z "$v" ] && v="NOVERDICT(rc=$rc)"
        echo "$v $f::$fn" >> $OUT/cl_$side.txt
      done
    done < $R/census_claim_roster.txt
    echo "$side claims: $(cut -d' ' -f1 $OUT/cl_$side.txt | sort | uniq -c | tr '\n' ' ')"; echo "ROSTER_B64_BEGIN $side"; grep -v "^PASS" $OUT/cl_$side.txt | gzip -9 | base64 -w0; echo; echo "ROSTER_B64_END"; echo "PASS_SET_SHA256 $side $(grep "^PASS" $OUT/cl_$side.txt | sort | sha256sum | cut -c1-64) count=$(grep -c "^PASS" $OUT/cl_$side.txt)"
  fi
  cd $R; git worktree remove -f $W
done
if [ $MODE = native ]; then
  echo "== native verdict diff (base< head>)"; diff $OUT/roster_base.txt $OUT/roster_head.txt > $OUT/d.txt; echo "diff_lines=$(wc -l < $OUT/d.txt)"; head -c 5000 $OUT/d.txt
else
  echo "== base PASS not PASS on head"
  join -j2 <(awk '{print $1, $2}' $OUT/cl_base.txt | sort -k2) <(awk '{print $1, $2}' $OUT/cl_head.txt | sort -k2) -o 0,1.1,2.1 | awk '$2=="PASS" && $3!="PASS"' | head -50
  echo "== base PASS absent on head"; comm -23 <(awk '$1=="PASS"{print $2}' $OUT/cl_base.txt | sort) <(awk '{print $2}' $OUT/cl_head.txt | sort) | head -20
  echo "== head non-PASS"; grep -v '^PASS' $OUT/cl_head.txt | head -60
  echo "== head_only PASS count $(comm -13 <(awk '$1=="PASS"{print $2}' $OUT/cl_base.txt | sort) <(awk '$1=="PASS"{print $2}' $OUT/cl_head.txt | sort) | wc -l)"
fi
echo DONE
