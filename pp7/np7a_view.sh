# np7a_view: off-host summary of np7a .full files. usage: bash np7a_view.sh <dir with {H1,H2,K2}_s{1,2}.full>
# Parser tested on real H1_s1 rows (calm-crab-645, msg_e1644c9c). Rows repeat (fn,site) with distinct keys, so
# rows are AGGREGATED per (kind,fn,first-site): n_rows, sum_calls (count/calls), max_total_ms (inclusive ms is never summed).
# A fn+site absent from a capped list is UNKNOWN. Delta = s2 - s1 on those aggregates.
D=${1:-.}
agg(){ grep -a '^\[recompute-trace\] \(dup\|partial\) ' "$1" 2>/dev/null | sed -n 's/^\[recompute-trace\] \(dup\|partial\) fn=\([^ ]*\) \(count\|calls\)=\([0-9]*\) total_ms=\([0-9]*\).* @\([^ ]*\).*/\1 \2 \6 \4 \5/p' | awk '{k=$1" "$2" "$3; n[k]++; c[k]+=$4; if($5>m[k])m[k]=$5} END{for(k in n)print k, n[k], c[k], m[k]}' | sort; }
for k in H1 H2 K2; do
  [ -f $D/${k}_s1.full ] && [ -f $D/${k}_s2.full ] || { echo "-- $k: missing s1/s2 file"; continue; }
  echo "-- $k  (kind fn site | rows s1,s2 | sum_calls s1,s2,delta | max_ms s1,s2)"
  agg $D/${k}_s1.full > $D/.a1; agg $D/${k}_s2.full > $D/.a2
  awk 'NR==FNR{key=$1" "$2" "$3; r[key]=$4; c[key]=$5; m[key]=$6; next} {key=$1" "$2" "$3; a=(key in c)?c[key]:"-"; printf "%s | rows %s,%s | calls %s,%s,%s | max_ms %s,%s\n",key,(key in r)?r[key]:"-",$4,a,$5,(key in c)?$5-c[key]:"-",(key in m)?m[key]:"-",$6}' $D/.a1 $D/.a2 | sort -t, -k5 -nr | head -40
done
