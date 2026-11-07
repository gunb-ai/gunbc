#!/bin/bash
# The rev17 inversion set: each row = one guard removed (a literal mutation of K1.sh or K2.sh) and the fixtures that must go
# RED against it. Runs fixtures/inversion-check.sh per row; prints CAUGHT/MISSED per row; exit 0 iff every row is CAUGHT.
cd "$(dirname "$0")"; M=0
row() { local id=$1 s=$2 old=$3 new=$4; shift 4; local o; o="$(./inversion-check.sh "$s" "$old" "$new" "$@" 2>&1)"; local rc=$?
  if [ $rc = 0 ]; then echo "CAUGHT $id ($s) by: $*"; else echo "MISSED $id ($s): $(printf '%s' "$o" | tr '\n' ' ' | cut -c1-400)"; M=$((M+1)); fi; }
row I1-k1-priv-verify K1.sh '[ "$_ps" = "$PRIV_OWNER:700:directory" ] && [ ! -L "$PRIV" ] || die B0' 'true || die B0' k1-priv-mode k1-priv-symlink
row I2-k1-export-drops-privilege K1.sh 'timeout -k $GRACE $c runuser -u $EXPORT_USER -- tee -- "$OUT/$1"' 'timeout -k $GRACE $c tee -- "$OUT/$1"' k1-no-user-writes
row I3-k1-begin-export K1.sh 'xport k1-begin.txt "K1_BEGIN request=$REQS unit=$USHA priv=$PRIV" || die B0' 'true || die B0' k1-out-unusable
row I4-k1-export-after-mutation K1.sh 'xport k1-receipt.txt "$LINE" || {' 'true || {' k1-export-fail k1-out-renamed
row I5-k1-install-contract K1.sh '[[ $_ic =~ ^[0-9]+$ ]] && [ "$_ic" -ge 1 ] || die B0' 'true || die B0' k1-static k1-static-empty-install
row I6a-k1-parse-snapshot K1.sh 'mapfile -t RL < "$PRIV/request"' 'mapfile -t RL < "$REQ"' k1-snapshot-race
row I6b-k1-install-snapshot K1.sh 'T install -m 0644 -o root -g root -- "$PRIV/unit" $STAGE' 'T install -m 0644 -o root -g root -- "$USRC" $STAGE' k1-snapshot-race
row I7-k1-inventory-format K1.sh '-type f -printf "F %m %u:%g %s %Ts %i %p\n"' '-type f -printf "F %m %u:%g %s %T@ %i %p\n"' k1-e2e-j2
row I8-k2-priv-verify K2.sh '[ "$_ps" = "$PRIV_OWNER:700:directory" ] && [ ! -L "$PRIV" ] || die "the private dir' 'true || die "the private dir' k2-priv-mode
row I9-k2-export-drops-privilege K2.sh 'timeout -k $GRACE $c runuser -u $EXPORT_USER -- tee -- "$OUT/$1"' 'timeout -k $GRACE $c tee -- "$OUT/$1"' k2-no-user-writes
row I10-k2-begin-export K2.sh 'xport k2-begin.txt "K2_BEGIN k1_receipt=$K1S j3_receipt=$J3S priv=$PRIV" || die' 'true || die' k2-out-unusable
row I11-k2-export-after-start K2.sh 'xport k2-receipt.txt "$LINE" || {' 'true || {' k2-export-fail
row I12-k2-post-stop-attribution K2.sh '[ -n "$cur2" ] && [ "$cur2" = "$cur" ] && [ "$nr2" = "$nr" ] || unresolved' 'true || unresolved' k2-post-check-restart
row I13-k2-fail-ours-origin K2.sh 'origin "$cur" "$nr" || unresolved "$1; invocation' 'true || unresolved "$1; invocation' k2-inv-foreign-at-stop k2-inv-auto-at-stop
row I14-k2-origin-auto-restart K2.sh '[ "$2" -gt "$NRL" ] && {' '[ "$2" -gt 999999 ] && {' k2-inv-auto-at-stop k2-auto-restart
row I15-k2-start-rc K2.sh '[ $r = 0 ] || fail_ours "start rc=$r"' 'true || fail_ours "start rc=$r"' k2-start-rc
row I16-k2-empty-invocation K2.sh 'if [ -z "$INV" ]; then b=' 'if false; then b=' k2-empty-invocation
row I17-k2-invocation-read K2.sh 'INV="$(sprop $UNIT InvocationID)" || unresolved "start rc=$r; InvocationID read failed"' 'INV="$(sprop $UNIT InvocationID)" || true' k2-inv-read-fail
row I18a-k2-parse-k1-snapshot K2.sh 'mapfile -t KL < "$PRIV/k1-receipt"' 'mapfile -t KL < "$K1R"' k2-snapshot-race
row I18b-k2-parse-j3-snapshot K2.sh 'mapfile -t J3L < "$PRIV/j3-receipt"' 'mapfile -t J3L < "$J3R"' k2-snapshot-race
echo "INVERSIONS: $M missed"; exit $M
