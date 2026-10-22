set -u
cd /root/workspace/repo-root 2>/dev/null || true
# #14666 instrument arms, DETACHED pattern: launch (or poll) gunbc instrument runs at head and
# base. Runner filesystem at /root/workspace persists across dispatches; per-run deadlines no
# longer kill the 29-min instrument. Idempotent: launch once, poll each dispatch until DONE.
HEADSHA=d3d44cfed33af0feede633829f7cf574f648ecc7
BASESHA=2538f832391c23c0209ce39c42df00f7492906c4
W=/root/workspace/ver666
mkdir -p $W
sha() { sha256sum "$1" | cut -d' ' -f1; }
export CARGO_INCREMENTAL=0 RUSTC_WRAPPER=
arm () { # T OID
  T=$1; S=$2
  WT=$W/$T
  if [ ! -d $WT ]; then
    git worktree prune >/dev/null 2>&1 || true
    git cat-file -e "$S^{commit}" 2>/dev/null || git fetch -q origin $S
    git worktree add -q --detach $WT $S || { echo "$T WT_FAIL"; return; }
    ( cd $WT && CARGO_TARGET_DIR=$W/$T cargo build --release -p v1-compiler --bin gunbc > $W/$T-build.log 2>&1 )
    [ -x $W/$T/release/gunbc ] && echo "$T BUILT commit=$(cd $WT && git rev-parse HEAD) sha=$(sha $W/$T/release/gunbc | cut -c1-12)" || { echo "$T BUILD_FAIL"; tail -5 $W/$T-build.log | sed 's/^/    B| /'; return; }
  else
    echo "$T worktree exists"
  fi
  if [ -f $W/$T.done ]; then
    echo "$T DONE rc=$(cat $W/$T.done)"
    L=$W/$T.log
    echo "  ambiguity_total=$(grep -ac 'ambiguous exact Rust binding' $L)"
    echo "  DeclarationRef_hits=$(grep -a 'ambiguous exact Rust binding' $L | grep -ac DeclarationRef)"
    echo "  GcpPolicy_hits=$(grep -a 'ambiguous exact Rust binding' $L | grep -ac GcpPolicy)"
    grep -a 'ambiguous exact Rust binding' $L | head -6 | sed 's/^/    AMB| /' | cut -c1-190
    echo "  PASS_lines=$(grep -ac 'PASS ' $L) SKIPPED=$(grep -ac 'SKIPPED' $L)"
    tail -6 $L | sed 's/^/    T| /' | cut -c1-170
  elif [ -f $W/$T.running ]; then
    echo "$T POLL: $(wc -l < $W/$T.log 2>/dev/null || echo 0) log lines so far, instrument $(ps aux | grep -c '[g]unbc test' || true) proc(s)"
  else
    echo "$T LAUNCHING"
    ( cd $(dirname $WT)/$T 2>/dev/null || cd $WT )
    cd $WT || { echo "$T CD_FAIL"; return; }
    touch $W/$T.running
    setsid bash -c "CARGO_TARGET_DIR=$W/$T $W/$T/release/gunbc test //gunbc/instruments:native-process-memory-census > $W/$T.log 2>&1; echo \$? > $W/$T.done; rm -f $W/$T.running" > /dev/null 2>&1 &
    echo "$T launched detached"
  fi
}
arm h $HEADSHA
arm b $BASESHA
echo DONE_666_D $(date -u +%H:%M:%S)Z
