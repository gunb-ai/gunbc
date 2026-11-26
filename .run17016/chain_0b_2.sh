#!/usr/bin/env bash
H=a8d7e7a4eab815982f75a3b086bb084ad822f942
bash run_stage.sh $H STAGE0B stage0b_floor_bin.sh
RC=$?
echo "CHAIN: stage0b rc=$RC"
[ $RC -ne 0 ] && exit $RC
bash run_stage.sh $H STAGE2 stage2_module.sh
RC2=$?
echo "CHAIN: stage2 rc=$RC2"
exit $RC2
