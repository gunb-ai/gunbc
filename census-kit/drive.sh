#!/bin/bash
S=${KIT_DIR:?set KIT_DIR}
w=1; prev=$(python3 $S/recompute.py $S); echo "start remaining=$prev"
while [ "$prev" -gt 0 ]; do
  if [ "$prev" -gt 100000 ]; then $S/wave.sh $w 40 5 11811160064 4 34 > /dev/null
  else $S/wave.sh $w 9 1 22548578304 2 34 > /dev/null; fi
  now=$(python3 $S/recompute.py $S); echo "wave $w remaining=$now"
  [ "$now" -ge "$prev" ] && { echo "NO PROGRESS at remaining=$now"; break; }
  prev=$now; w=$((w+1))
done
