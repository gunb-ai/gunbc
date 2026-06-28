#!/bin/bash
BIN="$1"; shift
"$BIN" "$@" >/tmp/wlwt_out.txt 2>/tmp/wlwt_err.txt &
PID=$!
PEAK=0; LAST=0
while kill -0 $PID 2>/dev/null; do
  if [ -r /proc/$PID/status ]; then
    RSS=$(awk '/^VmRSS:/{print $2}' /proc/$PID/status 2>/dev/null)
    HWM=$(awk '/^VmHWM:/{print $2}' /proc/$PID/status 2>/dev/null)
    [ -n "$RSS" ] && LAST=$RSS
    [ -n "$HWM" ] && PEAK=$HWM
    echo "$(date +%s.%N) rss=${RSS} hwm=${HWM}"
  fi
  sleep 0.2
done
wait $PID; RC=$?
echo "EXIT=$RC PEAK_HWM_kB=$PEAK LAST_RSS_kB=$LAST"
