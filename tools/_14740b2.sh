set -uo pipefail
R=/tmp/gf740r
GB=/root/workspace/repo-root/.t740/release/gunbc
for n in head_alone head_plus_main_merge control_merge control_merge_b; do
  echo "===== $n ====="; ls $R/c_$n 2>/dev/null | head -5
  echo "--- comp.out ($(wc -c < $R/c_$n/comp.out 2>/dev/null) bytes) ---"
  head -20 $R/c_$n/comp.out 2>/dev/null | cut -c1-240
done
