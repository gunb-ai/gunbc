#!/usr/bin/env bash
# Full-table both-reader sweep over frozen v14 definitions (no gunbc; pure kit functions).
set -o pipefail
die2() { echo "REFUSED_SETUP: $*"; return 2; }
for d in FX RTD RS PGD R RUNDIR; do declare "$d=$(mktemp -d)"; done
source ./probe-kit-lib.sh >/dev/null 2>&1
ROWS=0; FAIL=0
for ctx in OK RF; do
for k in "${CENSUS37[@]}"; do i=$((ROWS)); :; done
for i in $(seq 0 36) X; do
  for pos in arm field binder body; do
    if [ "$ctx" = OK ]; then
      tokA="$(probe_token "$i" U T 1)"; tokR="R"
      row=""
      for p in arm field binder body; do
        v="$tokR"; [ "$p" = "$pos" ] && v="$tokA"
        row="$row$(printf '%s' "$(probe_mod_token_v14 "$v")")"
      done
      ROWLINE="WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=${row%%|*}"
      # build directly with separators for exactness
      t_arm="$tokR"; t_field="$tokR"; t_binder="$tokR"; t_body="$tokR"
      case $pos in arm) t_arm="$tokA";; field) t_field="$tokA";; binder) t_binder="$tokA";; body) t_body="$tokA";; esac
      ROW="WCVERIFY ctx=OK ctl_ok=r=0 ctl_out=r=X arm=$t_arm field=$t_field binder=$t_binder body=$t_body"
    else
      [ "$i" = 0 ] || continue   # RF form only needs one pass
      ROW="WCVERIFY ctx=RF ctl_ok=r=0 ctl_out=r=X arm=- field=- binder=- body=-"
    fi
    probe_validate "$ROW" >/dev/null 2>&1 || { echo "VALIDATOR-REFUSED i=$i pos=$pos ctx=$ctx: $ROW"; FAIL=1; continue; }
    B="$({ probe_read_line "$ROW"; })"; rb=$?
    printf '%s\n' "$ROW" > "$RS/rr.txt"; P="$(python3 ./probe_read.py.v14 "$RS/rr.txt")"; rp=$?
    [ "$rb" = 0 ] && [ "$rp" = 0 ] || { echo "READER-ERR i=$i pos=$pos rb=$rb rp=$rp"; FAIL=1; continue; }
    nb="$(printf '%s\n' "$B" | tr -s '[:space:]' ' ')"; np="$(printf '%s\n' "$P" | tr -s '[:space:]' ' ')"
    [ "$nb" = "$np" ] || { echo "DISAGREE i=$i pos=$pos ctx=$ctx"; diff <(printf '%s\n' "$B") <(printf '%s\n' "$P"); FAIL=1; }
    if [ "$i" != X ] && [ "$ctx" = OK ]; then
      want="${CENSUS37[$i]}"
      case $pos in arm) key="$want";; *) key="$want";; esac
      printf '%s\n' "$B" | grep -qF "$want" || { echo "NAME-MISS i=$i pos=$pos want=$want"; FAIL=1; }
    fi
    ROWS=$((ROWS+1))
  done
done
done
echo "SWEEP rows=$ROWS FAIL=$FAIL"
