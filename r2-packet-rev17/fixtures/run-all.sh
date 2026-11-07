#!/bin/bash
# Local fixture harness for the r2 packet (see README.txt). Exit 0 only if every case gives its expected outcome.
set -u
P="$(cd "$(dirname "$0")/.." && pwd)"; BASE="$(mktemp -d)"; FAILS=0
OWN="$BASE/own-pids"; : > "$OWN"
# own_alive / own_kill: ONLY pids this harness recorded at spawn, and only while /proc/<pid>/stat field 22 (starttime) still
# equals the captured value (pid-reuse guard). No global name/UID discovery is used as ownership anywhere in this harness.
own_alive() { local p st; while read -r p st; do [ "$(awk '{print $22}' /proc/$p/stat 2>/dev/null)" = "$st" ] && echo "$p"; done < "$OWN"; }
own_kill() { local p; for p in $(own_alive); do kill -TERM "$p" 2>/dev/null; done; sleep 1; for p in $(own_alive); do kill -KILL "$p" 2>/dev/null; done; }
trap 'own_kill; rm -rf -- "$BASE"' EXIT; trap 'exit 130' INT TERM
ok() { echo "$1 PASS ($2)"; }; bad() { echo "$1 FAIL (expected $2; got: $3)"; FAILS=$((FAILS+1)); }
patch() { # patch <src> <dst> <T> <H> <TR> [extra python statements...]
python3 - "$@" <<'EOF'
import sys,re,getpass
src,dst,T,H,TR=sys.argv[1:6]; extra=sys.argv[6:]
s=open(src).read()
s=re.sub(r'^\[ "\$\(grep -c .*/proc/self/cgroup.*$','true',s,flags=re.M); s=re.sub(r'^CG=.*$','CG=%s/cg/job'%T,s,flags=re.M); s=re.sub(r'^path_guard$',': path_guard-neutralized-in-harness',s,flags=re.M); s=re.sub(r'^\[ "\$CG" = "/sys/fs/cgroup.*$','true   # harness: fake cgroup (CG re-check tested by cg-recheck)',s,flags=re.M); s=re.sub(r'^want .*$',':',s,flags=re.M); s=re.sub(r'^contain_selftest$',': contain_selftest-runs-only-on-a-real-cgroup',s,flags=re.M)
for a,b in [('/home/briansrls/runs/ld449-r2work',T+'/wd'),('/opt/gunbc-lab/codex-provider-feedback-v0',T+'/lab'),('/home/briansrls/.codex@gunbc-roadmap',T+'/codex'),('/home/briansrls/.config/systemd/user',T+'/root/home/.config/systemd/user'),('/etc/systemd/system',T+'/root/etc/systemd/system'),('/etc/systemd/user',T+'/root/etc/systemd/user'),('/run/systemd/system',T+'/root/run/systemd/system'),('/usr/bin/install','xinstall'),('/usr/bin/mv','xmv'),('/usr/bin/rm','xrm'),('/usr/bin/systemctl','systemctl')]:
    s=s.replace(a,b)
s=re.sub(r'^HEAD_EXPECT=\S+; TREE_EXPECT=\S+','HEAD_EXPECT=%s; TREE_EXPECT=%s'%(H,TR),s,flags=re.M)
s=s.replace('[ "$(id -un)" = briansrls ]','[ "$(id -un)" = %s ]'%getpass.getuser()); s=s.replace('[ "$_un" = briansrls ]','[ "$_un" = %s ]'%getpass.getuser())
for e in extra: exec(e)
open(dst,'w').write(s)
EOF
}
base() { # base <T>: fake cgroup, work dir with a git worktree at a fresh commit, SHAS
  local T=$1; mkdir -p $T/cg/job $T/wd/target/release $T/wd/markers $T/bin $T/state
  printf 'oom 0\noom_kill 0\n' > $T/cg/job/memory.events; echo 0 > $T/cg/job/memory.peak; echo 0 > $T/cg/job/memory.swap.current; echo $$ > $T/cg/job/cgroup.procs
  git init -q $T/wd/wt && (cd $T/wd/wt && echo x > f && git add . && git -c user.name=x -c user.email=x@x commit -qm a)
  printf x > $T/wd/target/release/gunbc; printf y > $T/wd/target/release/claim_batch; sha256sum $T/wd/target/release/gunbc $T/wd/target/release/claim_batch > $T/wd/SHAS; }
# ---- frame + join-law self-tests in all 8 jobs ----
T=$BASE/frame; mkdir -p $T; base $T; H=$(git -C $T/wd/wt rev-parse HEAD); TR=$(git -C $T/wd/wt rev-parse HEAD^{tree})
for f in $P/ld449-r2-*.sh; do b=$(basename $f)
  case $b in *j1r.sh|*j2.sh|*j4.sh) patch $f $T/$b $T $H $TR "s=s.replace('echo \"fixtures: all held\"','echo \"fixtures: all held\"; echo NOJOIN_OK; exit 0',1)";;
             *) patch $f $T/$b $T $H $TR "s=s.replace('join_fixture || die2','join_fixture && { echo JOIN_OK; exit 0; } || die2')";; esac
  xa=""; case $b in *j1b1.sh) xa="FABRIC_POSTS_GRANT=msg_00000000-fixture";; *j2.sh) xa="$(printf '0%.0s' {1..64})";; *j4.sh) xa="/x $(printf '0%.0s' {1..64})";; esac
  o="$(cd $T && bash ./$b $T/out-$b $xa 2>&1 | tail -1)"; case "$o" in JOIN_OK|NOJOIN_OK) ok "frame[$b]" "self-tests held";; *) bad "frame[$b]" "self-tests held" "$o";; esac; done
# ---- join-neg ----
patch $P/ld449-r2-j1b1.sh $T/jn.sh $T $H $TR "s=s.replace('join_fixture || die2','join_fixture && { echo JOIN_OK; exit 0; } || die2')" "L=s.split(chr(10))
for msg in ('CR byte in the claim_batch log','a malformed outcome-like line','outcome-like line count != roster size','canonical outcome line count != roster size','unknown ALLCAPS line'):
    hits=[k for k,l in enumerate(L) if msg in l and 'die2' in l]
    assert len(hits)==1, ('join-neg: join-law line not found exactly once', msg, len(hits))
    L[hits[0]]='  true'
s=chr(10).join(L)"
# environment precondition: a job started with GUNBC_WORKSPACE_ROOT (or any GUNBC_* name) set refuses before any CLI call, and
# never prints the value
for v in GUNBC_WORKSPACE_ROOT GUNBC_EVAL_MEMO; do o="$(cd $T && env $v=fixture-value-must-not-print bash ./ld449-r2-j1b4.sh $T/out-env-$v 2>&1)"
  case "$o" in *fixture-value-must-not-print*) bad env-$v "value never printed" "value appeared in output";; *REFUSED_SETUP*GUNBC_*) ok env-$v "set $v refuses before any CLI call; value not printed";; *) bad env-$v "REFUSED_SETUP naming GUNBC_*" "$(printf '%s' "$o" | tail -1)";; esac; done
o="$(cd $T && bash ./jn.sh $T/out-jn FABRIC_POSTS_GRANT=msg_00000000-fixture 2>&1 | grep -E 'join fixture|JOIN_OK' | head -1)"; case "$o" in *"join fixture foreign"*) ok join-neg "self-test refuses a broken join";; *) bad join-neg "refusal" "$o";; esac
# ---- chain + contain: the common block's REAL mark/need/contain, extracted from J1B4 ----
T=$BASE/chain; mkdir -p $T/out; base $T; H=$(git -C $T/wd/wt rev-parse HEAD); TR=$(git -C $T/wd/wt rev-parse HEAD^{tree})
python3 - $P/ld449-r2-j1b4.sh $T/h.sh $T $H $TR <<'EOF'
import sys,re
s=open(sys.argv[1]).read(); T,H,TR=sys.argv[3:6]
a=s.index('die2()'); b=s.index('REAL_COMPLETION=')
fn=s[a:b]; fn=re.sub(r'^\[ "\$\(grep -c .*/proc/self/cgroup.*$','true',fn,flags=re.M); fn=re.sub(r'^CG=.*$','CG=%s/cg/job'%T,fn,flags=re.M); fn=re.sub(r'^want .*$',':',fn,flags=re.M)
rg=s[s.index('roster_gate() {'):s.index('# ==== r2 pins')]
cm=s[s.index('# ==== r2 pins'):s.index('# ==== end r2 common ====')].replace('WD=/home/briansrls/runs/ld449-r2work','WD=%s/wd'%T)
cm=re.sub(r'^HEAD_EXPECT=\S+; TREE_EXPECT=\S+','HEAD_EXPECT=%s; TREE_EXPECT=%s'%(H,TR),cm,flags=re.M); cm=re.sub(r'^mapfile -t B0 < .*$','B0=($$)',cm,flags=re.M); cm=re.sub(r'^path_guard$',':',cm,flags=re.M); cm=re.sub(r'^\[ "\$CG" = "/sys/fs/cgroup.*$','true',cm,flags=re.M)
open(sys.argv[2],'w').write('JOB_RT=7200\n'+fn+rg+'R=%s/out\n'%T+cm)
EOF
S1=$(printf 'a%.0s' {1..64}); S2=$(printf 'b%.0s' {1..64}); S3=$(printf 'c%.0s' {1..64}); MAP="J1_GREEN:$S1 J1R_GREEN:$S2 J1B1_GREEN:$S3"
cd $T
chk() { local name=$1 want=$2; shift 2; local o; o="$( ( . ./h.sh; "$@" ) 2>&1 | tail -1 )"; case "$o" in *"$want"*) ok "$name" "$want";; *) bad "$name" "$want" "$o";; esac; }
mk3() { ( . ./h.sh; RUNS_ROOT=$T/runs; for r in J1_GREEN:none:$S1:ld449-r2-j1 J1R_GREEN:J1_GREEN:$S2:ld449-r2-j1r J1B1_GREEN:J1R_GREEN:$S3:ld449-r2-j1b1; do IFS=: read n pr sh job <<< "$r"; SELF_SHA=$sh; rm -f $MK/$n; d=$T/runs/$job/out; rm -rf $d; mkdir -p $d; R=$(readlink -e $d); W=$(readlink -e $WD); files=$(printf '%s\n' "$EXPECTED_RECEIPTS" | awk -F= -v k=$n '$1==k{print $2}' | tr ',' ' '); args=(); for e in $files; do loc=${e%%:*}; rel=${e#*:}; [ "$rel" = ignored-end.txt ] && continue; if [ $loc = W ]; then f=$W/$rel; else f=$R/$rel; fi; mkdir -p $(dirname $f); [ -e $f ] || echo $n-$rel > $f; args+=($f); done; mark $n $pr "${args[@]}" >/dev/null; done ); }
mk3; chk chain-genuine "chain verified to J1" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1B1_GREEN $S3 J1R_GREEN"
echo t >> runs/ld449-r2-j1/out/build.log; chk chain-old-tamper "changed" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1B1_GREEN $S3 J1R_GREEN"; mk3
chk chain-unreviewed "not written by reviewed script" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='J1_GREEN:$S3 J1R_GREEN:$S2 J1B1_GREEN:$S3'; need J1B1_GREEN $S3 J1R_GREEN"
echo z > wd/wt/.ign-x; echo .ign-x > wd/wt/.git/info/exclude; chk chain-ignored "ignored-path set changed" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1B1_GREEN $S3 J1R_GREEN"; rm -f wd/wt/.ign-x
mk3; mkdir -p $T/forgeX; cp runs/ld449-r2-j1/out/* $T/forgeX/ 2>/dev/null; sed -i "s#^OUT .*#OUT $(readlink -e $T/forgeX)#; s#$(readlink -e runs/ld449-r2-j1/out)/#$(readlink -e $T/forgeX)/#" wd/markers/J1_GREEN
chk anchor-foreign-out "not the launcher OUT" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1_GREEN $S1 none"
mk3; mkdir -p runs/ld449-r2-j1/out/deep; mv runs/ld449-r2-j1/out/build.log runs/ld449-r2-j1/out/deep/build.log; sed -i "s#/out/build.log#/out/deep/build.log#" wd/markers/J1_GREEN
chk anchor-moved-receipt "expected locations" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1_GREEN $S1 none"; mk3
printf 'HEAD=x\n' > wd/markers/J1B2_GREEN; chk chain-printf-forge "NO OBSERVATION" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP J1B2_GREEN:$S1'; need J1B2_GREEN $S1 J1B1_GREEN"
chmod 500 wd/markers; chk mark-write-fail "marker write" bash -c ". ./h.sh; SELF_SHA=$S1; echo r > $T/out/SHAS; mark J9 none $T/out/SHAS"; chmod 700 wd/markers
# the REAL mark with shas_id failing (keen-otter rev11): must refuse and write NO marker (rev11 wrote one with an empty SHAS)
chk mark-shas-fail "cannot hash SHAS" bash -c ". ./h.sh; SELF_SHA=$S1; shas_id() { return 1; }; echo r > $T/out/r1; mark J8 none $T/out/r1"
[ -e wd/markers/J8 ] && bad mark-shas-nomarker "no marker J8" "marker J8 was written" || ok mark-shas-nomarker "no marker written"
# the REAL mark with its readback hanging 20 s under an 8 s operation deadline: must refuse, bounded (rev11's readback was untimed)
mkdir -p $T/slowcat; printf '#!/bin/bash\nfor a in "$@"; do case "$a" in %s/wd/markers/*) sleep 20;; esac; done\nexec /usr/bin/cat "$@"\n' $T > $T/slowcat/cat; chmod +x $T/slowcat/cat
S0=$(date +%s); chk mark-deadline "marker readback" bash -c ". ./h.sh; SELF_SHA=$S1; PATH=$T/slowcat:\$PATH; OPDL=\$(( \$(date +%s) + 8 )); echo r > $T/out/r2; mark J7 none $T/out/r2"; el=$(( $(date +%s)-S0 ))
[ $el -lt 15 ] && [ ! -e wd/markers/J7 ] && ok mark-deadline-bound "refused in ${el}s < 15, no marker" || bad mark-deadline-bound "< 15 s and no marker J7" "${el}s"
# readback that prints the EXACT temp file and then fails (rc 1, and timeout's rc 124): the real mark must refuse, no marker
for rcv in 1 124; do mkdir -p $T/rbcat$rcv; printf '#!/bin/bash\nfor a in "$@"; do case "$a" in %s/wd/markers/.*.tmp) /usr/bin/cat "$a"; exit %s;; esac; done\nexec /usr/bin/cat "$@"\n' $T $rcv > $T/rbcat$rcv/cat; chmod +x $T/rbcat$rcv/cat
  chk mark-readback-rc$rcv "readback" bash -c ". ./h.sh; SELF_SHA=$S1; PATH=$T/rbcat$rcv:\$PATH; echo r > $T/out/r3; mark J6R$rcv none $T/out/r3"
  [ -e wd/markers/J6R$rcv ] && bad mark-readback-nomarker$rcv "no marker" "marker J6R$rcv written" || ok mark-readback-nomarker$rcv "no marker written"; done
# need with shas_id failing must refuse (an empty marker SHAS field and a failed shas_id must not compare equal)
mk3; chk need-shas-fail "cannot hash SHAS" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; shas_id() { return 1; }; need J1B1_GREEN $S3 J1R_GREEN"
o="$( ( . ./h.sh; B0=($$); mkdir -p cg/job/sub; echo 999 > cg/job/sub/cgroup.procs; chmod 000 cg/job/sub; survivors; echo "rc=$? SURV=[$SURV]" ) 2>&1 | tail -1)"; chmod 755 cg/job/sub; rm -rf cg/job/sub
case "$o" in rc=1*) ok subtree-unreadable "unreadable sub-cgroup dir refuses (not 'no survivors')";; *) bad subtree-unreadable "rc=1" "$o";; esac
# dot-named sub-cgroups (sleek-owl rev10): a nested sub/.deep and a top-level .hide must both be visited
mkdir -p cg/job/sub/.deep cg/job/.hide; : > cg/job/sub/cgroup.procs; echo 999 > cg/job/sub/.deep/cgroup.procs; echo 998 > cg/job/.hide/cgroup.procs
o="$( ( . ./h.sh; B0=($$); survivors; echo "rc=$? SURV=[$SURV]" ) 2>&1 | tail -1)"; rm -rf cg/job/sub cg/job/.hide
case "$o" in "rc=0 SURV=["*" 999"*"]"*) case "$o" in *" 998"*) ok subtree-dotdir "dot-named sub-cgroups visited: $o";; *) bad subtree-dotdir "998 and 999" "$o";; esac;; *) bad subtree-dotdir "998 and 999" "$o";; esac
# contain: the stub claim_batch starts a setsid'd child and a backgrounded child and records each EXACT pid with its
# /proc starttime in the harness's own-pids file at spawn, then exits 0. The fake cgroup lists ONLY this run's own recorded,
# identity-checked pids (plus the job shell), so contain() can signal nothing else. The postcheck tests those exact pids.
# No global name or UID discovery is used anywhere.
mkdir -p wd/wt/dag/gunbc/ci; printf '%s\n' 'data witness_layer_roots: List<String> = ["dag", "src/v2"]' > wd/wt/dag/gunbc/ci/ci_layer_roots.dag
{
  echo '#!/bin/bash'
  echo '[ "$1" = --print-entry-closure ] && { echo "[entry-closure] dag/e.dag: file=\"dag/e.dag\""; echo "[entry-closure-summary] state=complete entries=1 path_rows=1"; exit 0; }'
  echo 'setsid sleep 300 >/dev/null 2>&1 < /dev/null & p1=$!'
  echo "( sleep 301 >/dev/null 2>&1 < /dev/null & echo \$! > $T/p2 ); p2=\$(cat $T/p2)"
  echo "for p in \$p1 \$p2; do echo \"\$p \$(awk '{print \$22}' /proc/\$p/stat)\" >> $OWN; done"
  echo 'sleep 1; echo PASS a; echo PASS b'
  echo "echo '[cost-partition] {\"verdict\":{\"state\":\"Reconciled\",\"residual_nanos\":0,\"tolerance_nanos\":1}}'"
  echo "echo '[process-partition] total_ms=5 a_ms=1'; exit 0"
} > wd/target/release/claim_batch
chmod +x wd/target/release/claim_batch; mkdir -p wd/wt/dag; printf 'test fn a() -> Bool { true }\ntest fn b() -> Bool { true }\n' > wd/wt/dag/e.dag
( while :; do { echo $$; own_alive; } > cg/job/.p; mv cg/job/.p cg/job/cgroup.procs; sleep 0.2; done ) & REG=$!
o="$( ( . ./h.sh; declare -A EXPECT OBS; EXPECT[G.t.a]=PASS; EXPECT[G.t.b]=PASS; DEVIATIONS=0; closure_gate_all $T/wd/wt dag/e.dag && run_entry G t $T/wd/wt dag/e.dag a,b 0 60 ) 2>&1 | grep -c SURVIVORS )"; kill $REG; wait $REG 2>/dev/null
n=$(wc -l < $OWN); left=$(own_alive | wc -l)
if [ "$o" -ge 1 ] && [ "$n" = 2 ] && [ "$left" = 0 ]; then ok contain "own setsid+background children ($n) killed and gone"; else bad contain "own survivors killed" "seen=$o recorded=$n left=$left"; fi
# FULL-CAP-OR-REFUSE branches (sleek-owl rev12): with too little budget left, the claim run and the closure query each
# refuse BEFORE starting (exit 2, typed), and the stub claim_batch is never invoked
: > $T/cb-calls; printf '#!/bin/bash\necho called >> %s/cb-calls\nexit 0\n' $T > $T/cbstub; chmod +x $T/cbstub
o="$( ( . ./h.sh; TG=$T/tgx; mkdir -p $TG/release; cp $T/cbstub $TG/release/claim_batch; JOB_END=$(( $(date +%s) + 400 )); declare -A EXPECT OBS; DEVIATIONS=0; CLOSED["$T/wd/wt|dag/e.dag"]=1; run_entry G cap $T/wd/wt dag/e.dag a,b 0 1500 ) 2>&1 | tail -1)"
case "$o" in *"not starting a run that could end before its roster"*) [ ! -s $T/cb-calls ] && ok fullcap-run "claim run refused before start: $o" || bad fullcap-run "claim_batch never invoked" "invoked";; *) bad fullcap-run "pre-start refusal" "$o";; esac
o="$( ( . ./h.sh; TG=$T/tgx; JOB_END=$(( $(date +%s) + 700 )); closure_gate_all $T/wd/wt dag/e.dag ) 2>&1 | tail -1)"
case "$o" in *"< its full cap"*"not starting"*) [ ! -s $T/cb-calls ] && ok fullcap-closure "closure query refused before start: $o" || bad fullcap-closure "claim_batch never invoked" "invoked";; *) bad fullcap-closure "pre-start refusal" "$o";; esac
# CG re-check (rev15), the generated lines verbatim, against this container's real /proc/self/cgroup
CGL="$(/usr/bin/grep -m1 -A1 '^{ read -r _cgl < /proc/self/cgroup; }' $P/ld449-r2-j1.sh)"
o1="$(bash -c 'die2() { echo "REFUSED: $*"; exit 2; }; CG="/sys/fs/cgroup$(sed -n "s/^0:://p" /proc/self/cgroup)"; eval "$1"; echo CG_OK' _ "$CGL" 2>&1 | tail -1)"
o2="$(bash -c 'die2() { echo "REFUSED: $*"; exit 2; }; CG="/sys/fs/cgroup/some/other/scope"; eval "$1"; echo CG_OK' _ "$CGL" 2>&1 | tail -1)"
[ "$o1" = CG_OK ] && case "$o2" in *"is not this process's cgroup"*) true;; *) false;; esac && ok cg-recheck "right CG passes, a wrong CG refuses" || bad cg-recheck "CG_OK / refusal" "$o1 / $o2"
# ===== #15175 ancestry gate (keen-otter msg_cf03a808), the generated J1 functions verbatim, on LOCAL synthetic repos only
GF="$(sed -n '/^promisor_guard() {/,/^  case \$r in 0) die2 "#15175 is an ancestor";; 1) return 0;; \*) die2 "ancestry rc=\$r";; esac; }$/p' $P/ld449-r2-j1.sh)"
AG=$T/anc; rm -rf $AG; mkdir -p $AG; ( cd $AG && git init -q a && cd a && git commit -q --allow-empty -m p && git tag tP && git commit -q --allow-empty -m c2 && git commit -q --allow-empty -m c3 \
  && git checkout -q -b side tP && git commit -q --allow-empty -m side && git tag tS && git checkout -q - ) >/dev/null 2>&1
PSHA=$(git -C $AG/a rev-parse tP); SSHA=$(git -C $AG/a rev-parse tS)
git clone -q file://$AG/a $AG/full 2>/dev/null; git -C $AG/full fetch -q origin tag tS 2>/dev/null
git clone -q --depth 1 file://$AG/a $AG/shallow 2>/dev/null; git -C $AG/shallow fetch -q --depth 1 origin tag tP 2>/dev/null
ag() { bash -c 'die2() { echo "REFUSED: $*"; exit 2; }; eval "$1"; no15175_gate "$2" "$3"; echo GATE_ACCEPTED' _ "$GF" "$@" 2>&1 | tail -1; }
OLDL='timeout 120 git -C $WT merge-base --is-ancestor $NO15175 HEAD; r=$?; case $r in 0) die2 "#15175 is an ancestor";; 1) ;; *) die2 "ancestry rc=$r";; esac'
oold="$(WT=$AG/shallow NO15175=$PSHA bash -c 'die2() { echo "REFUSED: $*"; exit 2; }; eval "$1"; echo GATE_ACCEPTED' _ "$OLDL" 2>&1 | tail -1)"
git -C $AG/shallow cat-file -e "$PSHA^{commit}" && onew="$(ag $AG/shallow $PSHA)" || onew="(prohibited object not present in the shallow clone: fixture setup failed)"
[ "$oold" = GATE_ACCEPTED ] && case "$onew" in *"is shallow"*) true;; *) false;; esac && ok anc-shallow-hidden "shallow clone, prohibited ancestor present but hidden: rev15 line ACCEPTED it, rev16 gate refuses" || bad anc-shallow-hidden "old accepts / new refuses" "$oold / $onew"
o="$(ag $AG/full $PSHA)"; case "$o" in *"#15175 is an ancestor"*) ok anc-full-ancestor "full history, ancestor (rc 0) refuses";; *) bad anc-full-ancestor "ancestor refusal" "$o";; esac
o="$(ag $AG/full $SSHA)"; [ "$o" = GATE_ACCEPTED ] && ok anc-full-nonancestor "full history, non-ancestor (rc 1, non-shallow) accepted" || bad anc-full-nonancestor GATE_ACCEPTED "$o"
mkdir -p $T/gitsh1 $T/gitsh2 $T/gitsh3 $T/gitsh4
printf '#!/bin/bash\nfor a in "$@"; do [ "$a" = merge-base ] && exit 129; done\nexec /usr/bin/git "$@"\n' > $T/gitsh1/git
printf '#!/bin/bash\nfor a in "$@"; do [ "$a" = --is-shallow-repository ] && exit 1; done\nexec /usr/bin/git "$@"\n' > $T/gitsh2/git
printf '#!/bin/bash\nfor a in "$@"; do [ "$a" = --is-shallow-repository ] && exit 0; done\nexec /usr/bin/git "$@"\n' > $T/gitsh3/git
printf '#!/bin/bash\nfor a in "$@"; do [ "$a" = --get-regexp ] && exit 3; done\nexec /usr/bin/git "$@"\n' > $T/gitsh4/git; chmod +x $T/gitsh*/git
o="$(PATH=$T/gitsh1:$PATH ag $AG/full $SSHA)"; case "$o" in *"ancestry rc=129"*) ok anc-mergebase-error "merge-base rc 129 refuses";; *) bad anc-mergebase-error "rc 129 refusal" "$o";; esac
o="$(PATH=$T/gitsh2:$PATH ag $AG/full $SSHA)"; case "$o" in *"cannot read shallowness"*) ok anc-shallow-read-fail "rev-parse --is-shallow-repository failing refuses";; *) bad anc-shallow-read-fail "refusal" "$o";; esac
o="$(PATH=$T/gitsh3:$PATH ag $AG/full $SSHA)"; case "$o" in *"is shallow (or unexpected output []"*) ok anc-shallow-empty "empty shallowness output refuses";; *) bad anc-shallow-empty "refusal" "$o";; esac
git -C $AG/full config remote.origin.promisor true; o="$(ag $AG/full $SSHA)"; git -C $AG/full config --unset remote.origin.promisor
case "$o" in *"partial clone / promisor"*) ok anc-promisor "a promisor remote refuses";; *) bad anc-promisor "promisor refusal" "$o";; esac
o="$(PATH=$T/gitsh4:$PATH ag $AG/full $SSHA)"; case "$o" in *"git config read failed (rc 3)"*) ok anc-config-error "git config rc 3 refuses";; *) bad anc-config-error "refusal" "$o";; esac
# ===== rev15 real-caller error injections (sleek-owl/keen-otter rev13): correct-looking output with a nonzero status, and
# no-output errors, at the REAL call sites; each with its success control. Shims are PATH-prepended, act only on the named
# site (pattern AND file), and otherwise exec the real tool.
mkdir -p $T/gsh $T/ash $T/tgg/release
printf '#!/bin/bash\nhp=0; hf=0; for a in "$@"; do [ "$a" = "$GSHIM_PAT" ] && hp=1; case "$a" in *"$GSHIM_FILE") hf=1;; esac; done\n[ -n "$GSHIM_PAT" ] && [ $hp = 1 ] && [ $hf = 1 ] && exit 2\nexec /usr/bin/grep "$@"\n' > $T/gsh/grep
printf '#!/bin/bash\nfor a in "$@"; do [ -n "$AWK_FAILK" ] && [ "$a" = "k=$AWK_FAILK" ] && { /usr/bin/awk "$@"; exit 1; }; done\nexec /usr/bin/awk "$@"\n' > $T/ash/awk
{ echo '#!/bin/bash'; echo '[ "$1" = --print-entry-closure ] && { echo "[entry-closure] dag/e.dag: file=\"dag/e.dag\""; echo "[entry-closure-summary] state=complete entries=1 path_rows=1"; exit 0; }'
  echo 'echo PASS a; echo PASS b'; echo "echo '[cost-partition] {\"verdict\":{\"state\":\"Reconciled\",\"residual_nanos\":0,\"tolerance_nanos\":1}}'"; echo "echo '[process-partition] total_ms=5 a_ms=1'; exit 0"; } > $T/tgg/release/claim_batch
chmod +x $T/gsh/grep $T/ash/awk $T/tgg/release/claim_batch
leg() { ( . ./h.sh; TG=$T/tgg; PATH=$T/gsh:$PATH; export GSHIM_PAT="$1" GSHIM_FILE="$2"; declare -A EXPECT OBS; EXPECT[G.t.a]=PASS; EXPECT[G.t.b]=PASS; DEVIATIONS=0
  closure_gate_all $T/wd/wt dag/e.dag && run_entry G t $T/wd/wt dag/e.dag a,b 0 40 ) 2>&1 | tail -1; }
o="$(leg '' x)"; case "$o" in *"leg G t ok (2 fns)"*) ok inj-control "no injection: closure + run_entry accept";; *) bad inj-control "leg ok" "$o";; esac
o="$(leg $'\r' claims-G-t.log)"; case "$o" in *"grep error"*) ok inj-cr-gate "grep rc 2 at run_entry's CR gate refuses: $o";; *) bad inj-cr-gate "grep error" "$o";; esac
o="$(leg 'closure diagnostics|resolve failed|resolve did not reach' claims-G-t.log)"; case "$o" in *"grep error"*) ok inj-closure-diag "grep rc 2 at the closure/resolve gate refuses";; *) bad inj-closure-diag "grep error" "$o";; esac
o="$(leg '^[A-Z]+ a( |$)' claims-G-t.log)"; case "$o" in *"grep error reading the log for 'a'"*) ok inj-verdict "grep rc 2 in verdict refuses (ERROR, never NONE or PASS)";; *) bad inj-verdict "verdict ERROR refusal" "$o";; esac
o="$(leg '^\[entry-closure-summary\] ' closure-wt.txt)"; case "$o" in *"closure wt: grep error"*) ok inj-closure-summary "grep rc 2 at the closure summary refuses";; *) bad inj-closure-summary "grep error" "$o";; esac
o="$(leg '^\[process-partition\] total_ms' claims-G-t.log)"; case "$o" in *"grep error"*) ok inj-completed "grep rc 2 in completed() refuses";; *) bad inj-completed "grep error" "$o";; esac
o="$(leg '^\[cost-partition\] ' claims-G-t.log)"; case "$o" in *"grep error"*) ok inj-cost-partition "grep rc 2 at the cost-partition gate refuses";; *) bad inj-cost-partition "grep error" "$o";; esac
# join self-test aggregate bound (sleek-owl rev13 m4): a `timeout` shim adds 8 s to EVERY call, so the 35 cases would need
# well over 500 s unbounded; the self-test must give up (non-zero) within its 240 s budget plus one kill grace (< 300 s)
mkdir -p $T/tsh; printf '#!/bin/bash\nsleep 8\nexec /usr/bin/timeout "$@"\n' > $T/tsh/timeout; chmod +x $T/tsh/timeout
S0=$EPOCHSECONDS; o="$( ( . ./h.sh; PATH=$T/tsh:$PATH; join_fixture; echo "JF_RC=$?" ) 2>&1 | tail -1)"; el=$(( EPOCHSECONDS - S0 ))
case "$o" in JF_RC=0) bad join-bound "non-zero within 300 s" "rc 0 after ${el}s";; *) [ $el -lt 300 ] && ok join-bound "slowed self-test gave up in ${el}s (< 300): $o" || bad join-bound "< 300 s" "${el}s";; esac
# PATH-shadow guard, the generated function verbatim: a clean PATH passes; a `mktemp` shadow first on PATH refuses
PGF="$(sed -n '/^path_guard() {/,/^path_guard$/p' $P/ld449-r2-j1.sh | sed '$d')"; mkdir -p $T/pgs; printf '#!/bin/bash\nexit 0\n' > $T/pgs/mktemp; chmod +x $T/pgs/mktemp
o1="$(env -i PATH=/usr/bin:/bin bash -c 'die2() { echo "REFUSED: $*"; exit 2; }; eval "$1"; path_guard; echo PG_OK' _ "$PGF" 2>&1 | tail -1)"
o2="$(env -i PATH=$T/pgs:/usr/bin:/bin bash -c 'die2() { echo "REFUSED: $*"; exit 2; }; eval "$1"; path_guard; echo PG_OK' _ "$PGF" 2>&1 | tail -1)"
[ "$o1" = PG_OK ] && case "$o2" in *"'mktemp' resolves to"*"PATH shadow"*) true;; *) false;; esac && ok path-guard "clean PATH passes; a mktemp shadow refuses" || bad path-guard "PG_OK / shadow refusal" "$o1 / $o2"
# tmp_list (calm-crab M1, keen-otter msg_dc650b95): a dangling LAST match is normal filtering, not a failure: a WET leg passes;
# its sort failing is a real failure and refuses. (A foreign-owned entry cannot be created by this user; the -O branch is the
# same `if` and is covered by the same status rule.)
DL=/tmp/wre3-zzzz-r2fixture.$$; ln -s /nonexistent-r2-fixture $DL
o="$( ( . ./h.sh; TG=$T/tgg; declare -A EXPECT OBS; EXPECT[G.t.a]=PASS; EXPECT[G.t.b]=PASS; DEVIATIONS=0; closure_gate_all $T/wd/wt dag/e.dag && run_entry G t $T/wd/wt dag/e.dag a,b 1 40 ) 2>&1 | tail -1)"; rm -f -- $DL
case "$o" in *"leg G t ok (2 fns)"*) ok tmp-list-dangling "a dangling last /tmp match does not refuse a wet leg";; *) bad tmp-list-dangling "leg ok" "$o";; esac
mkdir -p $T/sortsh; printf '#!/bin/bash\nexit 2\n' > $T/sortsh/sort; chmod +x $T/sortsh/sort; DL2=/tmp/wre3-zzzz-r2fixture2.$$; mkdir $DL2
o="$( ( . ./h.sh; PATH=$T/sortsh:$PATH; tmp_list > /dev/null; echo "TL_RC=$?" ) 2>&1 | tail -1)"; rmdir $DL2
[ "$o" = "TL_RC=2" ] && ok tmp-list-sort-fail "a failing sort is a real failure (rc 2)" || bad tmp-list-sort-fail "TL_RC=2" "$o"
# MUTATION TEST of the join self-test (independent review MAJOR 4): delete ONE join-law check at a time from a scratch copy of
# the real helpers; the self-test must then FAIL. The canonical-count check is excluded: given the malformed-line check (= 0)
# and the outcome-like count (= roster), it is implied (canonical lines are a subset of outcome-like lines), so no input can
# reach it first; it is kept as defence in depth and stated as not separately discriminable.
for m in 'CR byte in the claim_batch log' 'a malformed outcome-like line' 'outcome-like line count != roster size' 'unknown ALLCAPS line' \
         'has no SINGLE BARE PASS/FAIL verdict' 'its own verdict lines say' 'closure rows are under dag/' "a closure path has a '..' segment" \
         'no closure summary line' 'per-entry rows sum to'; do
  python3 - "$T/h.sh" "$T/h-mut.sh" "$m" <<'PYM'
import sys; s=open(sys.argv[1]).read().split('\n'); m=sys.argv[3]
hits=[i for i,l in enumerate(s) if m in l and 'die2' in l and not l.lstrip().startswith('#') and 'join fixture' not in l]
assert len(hits)==1, (m, len(hits)); s[hits[0]]='  true   # MUTATED: '+m; open(sys.argv[2],'w').write('\n'.join(s))
PYM
  o="$( ( . ./h-mut.sh; join_fixture; echo "JF_RC=$?" ) 2>&1 | tail -1)"
  case "$o" in JF_RC=0) bad "mutation[$m]" "self-test fails with the check deleted" "still held";; *) ok "mutation[$m]" "deleting the check fails the self-test";; esac; done
# fns_csv and roster_gate: grep rc 2 -> rc 1 (refuse); control reads the declared names
o="$( ( . ./h.sh; fns_csv $T/wd/wt/dag/e.dag; echo " rc=$?" ) 2>&1 | tail -1)"; [ "$o" = "a,b rc=0" ] && ok fns-control "fns_csv reads a,b" || bad fns-control "a,b rc=0" "$o"
o="$( ( . ./h.sh; PATH=$T/gsh:$PATH; export GSHIM_PAT='^[[:space:]]*(pub )?test fn [a-z0-9_]+' GSHIM_FILE=e.dag; fns_csv $T/wd/wt/dag/e.dag; echo " rc=$?"; roster_gate $T/wd/wt/dag/e.dag a b; echo "rg=$?" ) 2>&1 | tr '\n' ' ')"
case "$o" in *"rc=1"*"rg=1"*) ok inj-fns-roster "grep rc 2: fns_csv and roster_gate both return 1";; *) bad inj-fns-roster "rc=1 rg=1" "$o";; esac
# need_one's mfield: awk prints the right HEAD and exits 1 -> refuse; control: the genuine chain verifies
mk3; o="$( ( . ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; PATH=$T/ash:$PATH; export AWK_FAILK=HEAD; need_one J1B1_GREEN $S3 J1R_GREEN ) 2>&1 | tail -1)"
case "$o" in *"cannot read HEAD"*) ok inj-mfield "mfield: right output + awk rc 1 refuses";; *) bad inj-mfield "cannot read HEAD" "$o";; esac
# J1's built-identity line, extracted VERBATIM from the generated J1 and run against a stub gunbc: right line + rc 1 / rc 124
# refuse; rc 0 passes
VL="$(/usr/bin/grep -m1 '^capt V timeout 30 \$TG/release/gunbc --version' $P/ld449-r2-j1.sh)"; mkdir -p $T/tgv/release
for vrc in 1 124 0; do printf '#!/bin/bash\necho "gunbc %s"; echo trailing-noise; exit %s\n' "$H" $vrc > $T/tgv/release/gunbc; chmod +x $T/tgv/release/gunbc
  o="$( ( . ./h.sh; TG=$T/tgv; eval "$VL"; echo VERSION_OK ) 2>&1 | tail -1)"
  if [ $vrc = 0 ]; then [ "$o" = VERSION_OK ] && ok version-j1-rc0 "right line, rc 0: accepted" || bad version-j1-rc0 VERSION_OK "$o"
  else case "$o" in *"--version failed or timed out"*) ok version-j1-rc$vrc "right line, rc $vrc: refused";; *) bad version-j1-rc$vrc "refusal" "$o";; esac; fi; done
# contain self-test: the fake cgroup.procs is republished every 0.1 s as [the h.sh shell + its OWN direct children], read from
# (every 0.02 s) /proc/<that shell>/task/*/children (its own children only; no global process discovery). Modes: exact (as a real cgroup
# would list it) -> held; stray (an extra pid 999999 is listed) -> refuses; blind (children omitted) -> refuses.
cstcase() { local mode=$1 want=$2 W o; rm -f $T/hp
  ( while :; do hp=$(cat $T/hp 2>/dev/null); if [ -n "$hp" ]; then { echo $hp; [ $mode != blind ] && cat /proc/$hp/task/*/children 2>/dev/null | tr ' ' '\n' | grep .; [ $mode = stray ] && echo 999999; } > cg/job/.p; mv cg/job/.p cg/job/cgroup.procs; fi; sleep 0.02; done ) & W=$!
  o="$( ( echo $BASHPID > $T/hp; . ./h.sh; B0=($BASHPID); sleep 0.5; contain_selftest; echo CST_RC=$? ) 2>&1 | grep -E 'contain self-test' | tail -1)"; kill $W; wait $W 2>/dev/null; echo $$ > cg/job/cgroup.procs
  case "$want|$o" in held\|*"held on this job"*) ok cst-$mode "self-test holds on an exact listing";; refuse\|*REFUSED_SETUP*) ok cst-$mode "self-test refuses [$o]";; *) bad cst-$mode "$want" "$o";; esac; }
cd $T; cstcase exact held; cstcase stray refuse; cstcase blind refuse
# OPDL: wt_ok's three git calls each take 3 s (well under their 60/60/120 caps); under a 5 s operation deadline the phase must
# refuse, not finish at ~9 s (rev10 bounded each call separately).
mkdir -p $T/slowgit; printf '#!/bin/bash\nsleep 3\nexec /usr/bin/git "$@"\n' > $T/slowgit/git; chmod +x $T/slowgit/git
o="$( ( . ./h.sh; PATH=$T/slowgit:$PATH; OPDL=$(( $(date +%s) + 5 )); S0=$SECONDS; wt_ok; echo "WT_DONE $((SECONDS-S0))s" ) 2>&1 | tail -1)"
case "$o" in *"NO OBSERVATION"*"timed out"*|*"NO OBSERVATION"*deadline*) ok opdl-phase "3 x 3 s git under a 5 s deadline refuses [$o]";; *) bad opdl-phase "deadline refusal" "$o";; esac
cd - >/dev/null
# ---- J2: fake root, stub gunbc/sudo/systemctl/install/mv/rm ----
j2setup() { local T=$1; rm -rf $T; mkdir -p $T; base $T; H=$(git -C $T/wd/wt rev-parse HEAD); TR=$(git -C $T/wd/wt rev-parse HEAD^{tree})
  mkdir -p $T/root/etc/systemd/system/paths.target.wants $T/root/etc/systemd/user $T/root/run/systemd/system $T/root/home/.config/systemd/user $T/lab/bin $T/lab/receipts $T/lab/attempt-state/attempt-identities $T/codex $T/wd/render
  git clone -q $T/wd/wt $T/lab/source; echo old > $T/lab/bin/gunbc; echo prev > $T/lab/bin/gunbc.prev; echo a > $T/root/etc/systemd/system/other.service; ln -s ../other.service $T/root/etc/systemd/system/paths.target.wants/other.service; echo tok > $T/codex/auth.json
  printf '[Unit]\nDescription=x\n[Service]\nUser=briansrls\nExecStart=/x %s\nRestart=on-failure\n[Install]\nWantedBy=multi-user.target\n' $H > $T/wd/render/serve.unit; echo prior > $T/wd/markers/J1B4_GREEN
  local U=gunbc-srv1-lab-codex-feedback-v0.service
  cat > $T/bin/systemctl <<EOF
#!/bin/bash
if [ "\$1" = show ]; then u=\$2; p=\$4
  if [ "\$u" = $U ]; then if [ -f $T/root/etc/systemd/system/$U ]; then case \$p in LoadState) echo loaded;; ActiveState) echo inactive;; UnitFileState) echo disabled;; *) echo;; esac; else case \$p in LoadState) echo not-found;; *) echo;; esac; fi; exit 0; fi
  f=$T/state/\$u.\$p; [ -f \$f ] && cat \$f || echo; exit 0; fi
case " \$* " in *" list-units "*) [ -f $T/lu.txt ] && cat $T/lu.txt; exit 0;; esac
[ "\$1" = daemon-reload ] && echo reload >> $T/reload.log; exit 0
EOF
  printf '#!/bin/bash\nshift; [ "$1" = -ll ] && { cat %s/sudo-ll; exit 0; }; [ "$1" = -l ] && exit 0; exec "$@"\n' $T > $T/bin/sudo
  printf 'User briansrls may run the following commands on host:\n\nSudoers entry: /etc/sudoers.d/lab\n    RunAsUsers: root\n    Options: !authenticate\n    Commands:\n\tALL\n' > $T/sudo-ll
  printf '#!/bin/bash\n[ "$1" = -G ] && { cat %s/ssh-g; exit 0; }; exit 255\n' $T > $T/bin/ssh
  printf 'hostname localhost\nuser briansrls\nport 22\nbatchmode yes\nuserknownhostsfile %s/kh\nstricthostkeychecking ask\n' $T > $T/ssh-g
  for b in install mv rm; do printf '#!/bin/bash\nargs=(); for a in "$@"; do case "$a" in -o|-g) skip=1; continue;; esac; [ -n "$skip" ] && { skip=; continue; }; args+=("$a"); done; exec /usr/bin/%s "${args[@]}"\n' $b > $T/bin/x$b; done
  for u in belt-exit belt-discovery attempt-unit-drain; do n=gunbc-srv1-lab-codex-feedback-v0-$u.path; echo inactive > $T/state/$n.ActiveState; echo disabled > $T/state/$n.UnitFileState; echo loaded > $T/state/$n.LoadState; echo $n > $T/state/$n.Id; done
  : > $T/reload.log; mkapply $T "${2:-:}"; chmod +x $T/bin/*
  patch $P/ld449-r2-j2.sh $T/j2.sh $T $H $TR "s=s.replace('prior_scope_gone ld449-r2-j1b4; need J1B4_GREEN \"\$J1B4_SHA\" J1B3_GREEN',':')"; }
mkapply() { local T=$1 act=${2:-:} H; H=$(git -C $T/wd/wt rev-parse HEAD); cat > $T/wd/target/release/gunbc <<EOF
#!/bin/bash
[ "\$1" = --version ] && { echo "gunbc $H"; exit 0; }
case "\$*" in *repository_converge_owner_wet*) git -C $T/lab/source update-ref refs/gunbc/srv1-lab-admitted $H && git -C $T/lab/source reset -q --hard $H; exit \$?;; *srv1_lab_dashboard_apply_wet*) cp -- "\$0" $T/lab/bin/gunbc && chmod 0755 $T/lab/bin/gunbc && $act; exit \$?;; esac; exit 3
EOF
  chmod +x $T/wd/target/release/gunbc; sha256sum $T/wd/target/release/gunbc $T/wd/target/release/claim_batch > $T/wd/SHAS; }
run2() { local T=$1; shift; ( cd $T && PATH=$T/bin:$PATH timeout 300 bash ./j2.sh "$@" 2>&1 ); }
T=$BASE/j2a; j2setup $T; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU | tail -1)"
U2=gunbc-srv1-lab-codex-feedback-v0.service; RQ="$(cat $T/out/k1-request.txt 2>/dev/null)"; IVF="$(readlink -e $T/out/units-after-apply.txt)"; IVS=$(sha256sum < "$IVF" 2>/dev/null | cut -d' ' -f1); RQW="$(printf 'K1_REQUEST unit=%s dest=%s\nK1_UNIT_SHA %s\nK1_UNIT_TEXT %s\nK1_J2_INVENTORY %s %s' $U2 $T/root/etc/systemd/system/$U2 $SU "$(readlink -e $T/out/serve.unit)" "$IVS" "$IVF")"
[ "$o" = DONE ] && [ -f $T/wd/markers/J2_READY ] && [ "$RQ" = "$RQW" ] && [ ! -s $T/reload.log ] && [ ! -e $T/root/etc/systemd/system/$U2 ] && [ ! -e $T/wd/J2_PARTIAL ] && ok j2-happy "J2_READY + exact 4-line K1 request (incl. the J2 inventory); nothing installed, 0 reloads" || bad j2-happy "J2_READY + request" "$o reloads=$(wc -l < $T/reload.log) req=[$RQ]"
# served identity (sleek-owl rev13 M2): the served gunbc prints the right version line and exits 1 / 124 -> J2 refuses at identity
for vrc in 1 124; do T=$BASE/j2v$vrc; j2setup $T; sed -i "s/echo \"gunbc \([0-9a-f]*\)\"; exit 0;/echo \"gunbc \1\"; exit $vrc;/" $T/wd/target/release/gunbc
  sha256sum $T/wd/target/release/gunbc $T/wd/target/release/claim_batch > $T/wd/SHAS; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU)"
  case "$o" in *"served gunbc --version failed or timed out"*) ok j2-version-rc$vrc "served version right line + rc $vrc: refused at identity";; *) bad j2-version-rc$vrc "identity refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac; done
# after-apply inventory failure (rev16): `find` fails on the unit tree once the apply ran -> LAB_STATE_CHANGED step=apply-units, no request
T=$BASE/j2if; j2setup $T "touch $T/applied"; printf '#!/bin/bash\n[ -e %s/applied ] && for a in "$@"; do case "$a" in */etc/systemd/system) exit 2;; esac; done\nexec /usr/bin/find "$@"\n' $T > $T/bin/find; chmod +x $T/bin/find
SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU)"
case "$o" in *"step=apply-units"*"unit inventory after apply failed"*) [ ! -e $T/out/k1-request.txt ] && ok j2-inv-fail "after-apply inventory error refuses; no K1 request" || bad j2-inv-fail "no request" "request written";; *) bad j2-inv-fail "apply-units refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
# full budget before each mutation (rev16): too little time before admit -> PRE_MUTATION, nothing admitted; before apply -> step=admit, apply not started
T=$BASE/j2ba; j2setup $T; sed -i 's/^NEED_ADMIT=6640; NEED_APPLY=5520$/NEED_ADMIT=999999; NEED_APPLY=5520/' $T/j2.sh; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU)"
case "$o" in *"PRE_MUTATION: "*"refused before admit"*) [ -z "$(git -C $T/lab/source rev-parse -q --verify refs/gunbc/srv1-lab-admitted)" ] && ok j2-budget-admit "insufficient budget refuses BEFORE admit" || bad j2-budget-admit "no admit" "admitted";; *) bad j2-budget-admit "pre-admit budget refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
T=$BASE/j2bp; j2setup $T; sed -i 's/^NEED_ADMIT=6640; NEED_APPLY=5520$/NEED_ADMIT=6640; NEED_APPLY=999999/' $T/j2.sh; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU)"
case "$o" in *"step=admit"*"apply NOT started"*) [ ! -e $T/out/apply.log ] && [ -e $T/wd/J2_PARTIAL ] && ok j2-budget-apply "insufficient budget refuses BEFORE apply (admit done, J2_PARTIAL kept)" || bad j2-budget-apply "no apply + partial" "apply ran or no partial";; *) bad j2-budget-apply "pre-apply budget refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
# live attempt unit whose NAME contains "drain" (independent review BLOCKER 1): must refuse before any mutation
T=$BASE/j2lu; j2setup $T; echo 'gunbc-attempt-node-drain7-k1.service loaded active running attempt (until exit)' > $T/lu.txt
SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU)"
case "$o" in *"PRE_MUTATION: live attempt/review/compute units present"*) [ -z "$(git -C $T/lab/source rev-parse -q --verify refs/gunbc/srv1-lab-admitted)" ] && ok j2-live-unit-drain "a live unit named *drain* refuses before admit" || bad j2-live-unit-drain "no admit" "admitted";; *) bad j2-live-unit-drain "live-unit refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
echo 'gunbc-attempt-node-a-k1.service loaded inactive dead x' > $T/lu.txt; T=$BASE/j2lu2; j2setup $T; echo 'gunbc-attempt-node-a-k1.service loaded failed failed x' > $T/lu.txt
SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU | tail -1)"; [ "$o" = DONE ] && ok j2-live-unit-failed-ok "a FAILED (not running) attempt unit does not block" || bad j2-live-unit-failed-ok DONE "$o"
D=$BASE/j2b/root/etc/systemd/system; T=$BASE/j2b; j2setup $T "mkdir -p $D/multi-user.target.wants && ln -s $D/gunbc-srv1-lab-codex-feedback-v0.service $D/multi-user.target.wants/gunbc-srv1-lab-codex-feedback-v0.service"
SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU | grep -c 'step=apply-units')"; [ "$o" -ge 1 ] && [ ! -e $T/out/k1-request.txt ] && ok j2-enable "an enable link made during apply refuses at apply-units; no K1 request" || bad j2-enable "apply-units refusal" "$o"
T=$BASE/j2c; j2setup $T "sleep 30"; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); ( cd $T && PATH=$T/bin:$PATH timeout -s KILL 10 bash ./j2.sh $T/out $SU >/dev/null 2>&1 ); o="$(run2 $T $T/out2 $SU | tail -1)"
[ -e $T/wd/J2_PARTIAL ] && case "$o" in *"already ran"*) true;; *) false;; esac && ok j2-kill "partial kept, rerun refused" || bad j2-kill "rerun refused" "$o"
# (rev16: the sudo-* parser cases are deleted with the parser; the root acts are K1/K2, below)
T=$BASE/j2e; j2setup $T; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); sed -i 's/batchmode yes/batchmode no/' $T/ssh-g; : > $T/kh; printf '#!/bin/bash\n[ "$1" = -F ] && exit 0; exit 1\n' > $T/bin/ssh-keygen; chmod +x $T/bin/ssh-keygen
o="$(run2 $T $T/out $SU | grep 'NO OBS')"; case "$o" in *PRE_MUTATION*BatchMode*) ok j2-ssh "BatchMode no refused pre-mutation even with a known_hosts entry";; *) bad j2-ssh "ssh readiness refusal" "$o";; esac
# ---- K1/K2 (root acts, rev17): run as this NON-root user by patching the EUID check, PATH, the fixed roots and the cgroup root
#      into a fake tree. Every case is tagged `#case <name>` so fixtures/inversion-check.sh can run it alone against a mutated script.
pk() { python3 - "$1" "$2" "$3" <<'PYK'
import sys; src,dst,T=sys.argv[1:4]; s=open(src).read()
import os; s=s.replace('[ "$EUID" = 0 ] ||','true ||').replace('export PATH=/usr/sbin:/usr/bin:/sbin:/bin','export PATH=%s/bin:/usr/sbin:/usr/bin:/sbin:/bin'%T)
assert s.count('PRIV_OWNER=0:0')==1; s=s.replace('PRIV_OWNER=0:0','PRIV_OWNER=%d:%d'%(os.getuid(),os.getgid()))
for a,b in [('/etc/systemd/system',T+'/root/etc/systemd/system'),('/etc/systemd/user',T+'/root/etc/systemd/user'),('/home/briansrls/.config/systemd/user',T+'/root/home/.config/systemd/user'),('/run/systemd/system',T+'/root/run/systemd/system'),('/run/r2-k1',T+'/run/r2-k1'),('/run/r2-k2',T+'/run/r2-k2'),('/home/briansrls/runs/ld449-r2work',T+'/wd'),('CGR=/sys/fs/cgroup','CGR=%s/cg'%T)]: s=s.replace(a,b)
open(dst,'w').write(s)
PYK
}
j2inv() { local T=$1; { echo 'OPDL=$(( EPOCHSECONDS + 600 ))'; sed -n '/^optc() {/p' $P/ld449-r2-j2.sh; sed -n '/^inv_roots() {/,/rm -f -- "\$out.l" "\$out.h" "\$out.err"; }$/p' $P/ld449-r2-j2.sh; } > $T/inv2.sh
  bash -c '. "$1"; inv_roots "$2" "$3/root/etc/systemd/system" "$3/root/etc/systemd/user" "$3/root/home/.config/systemd/user" "$3/root/run/systemd/system:gunbc-srv1-lab-codex-feedback-v0*"' _ $T/inv2.sh $T/j2out/units-after-apply.txt $T; }
U1=gunbc-srv1-lab-codex-feedback-v0.service
# mkrun T ST: harness stand-in for `runuser -u briansrls -- cmd...`. The harness is ONE user, so it cannot drop privilege; it
# LOGS every call (argv must be exactly `-u briansrls -- ...`) and runs the command with R2_AS_USER=1. Modes (files in ST):
#   export-sandbox: the user-side tee/cat act on ST/exported/<basename> instead of OUT (so the real tree shows ONLY what the
#                   script itself, i.e. root, wrote); export-fail-at N: call N and later exit 1; rename-out-at N: before call
#                   N, OUT is renamed to OUT.moved.
mkrun() { local T=$1 ST=$2; cat > $T/bin/runuser <<EOF
#!/bin/bash
echo "\$*" >> $ST/runuser.log; n=\$(wc -l < $ST/runuser.log)
[ "\$1" = -u ] && [ "\$2" = briansrls ] && [ "\$3" = -- ] || { echo "runuser stub: unexpected argv [\$*]" >&2; exit 97; }; shift 3
[ -e $ST/export-fail-at ] && [ \$n -ge \$(cat $ST/export-fail-at) ] && exit 1
[ -e $ST/rename-out-at ] && [ \$n -ge \$(cat $ST/rename-out-at) ] && [ -d $T/out ] && /usr/bin/mv $T/out $T/out.moved
if [ -e $ST/export-sandbox ]; then mkdir -p $ST/exported; last="\${@: -1}"; case "\$1" in tee) exec /usr/bin/cat > $ST/exported/\${last##*/};; cat) exec /usr/bin/cat -- $ST/exported/\${last##*/};; esac; exit 98; fi
export R2_AS_USER=1; exec "\$@"
EOF
  cat > $T/bin/mktemp <<EOF
#!/bin/bash
if [ -e $ST/priv-symlink ]; then d=\$(/usr/bin/mktemp -d $T/elsewhere.XXXXXX) || exit 1; ln -s \$d $T/run/r2-link.\$\$; echo $T/run/r2-link.\$\$; exit 0; fi
exec /usr/bin/mktemp "\$@"
EOF
  cat > $T/bin/chmod <<EOF
#!/bin/bash
[ -e $ST/priv-mode ] && [ "\$1" = 0700 ] && { shift; exec /usr/bin/chmod 0755 "\$@"; }; exec /usr/bin/chmod "\$@"
EOF
  cat > $T/bin/tee <<EOF
#!/bin/bash
[ "\${R2_AS_USER:-}" = 1 ] || echo "tee-without-runuser \$*" >> $ST/violations; exec /usr/bin/tee "\$@"
EOF
  chmod +x $T/bin/runuser $T/bin/mktemp $T/bin/chmod $T/bin/tee; }
k1stubs() { local T=$1 U=$U1
  cat > $T/bin/systemctl <<EOF
#!/bin/bash
if [ "\$1" = show ]; then case "\$4" in LoadState) [ -e $T/st/reloaded ] && [ -e $T/root/etc/systemd/system/$U ] && echo loaded || echo not-found;; ActiveState) echo inactive;; UnitFileState) awk '/^\\[/ {s = (\$0 == "[Install]")} s && /^(WantedBy|RequiredBy|UpheldBy|Alias)=./ {n++} END {exit !n}' $T/root/etc/systemd/system/$U 2>/dev/null && echo disabled || echo static;; *) echo;; esac; exit 0; fi
[ "\$1" = daemon-reload ] && { [ -e $T/st/reload-fail ] && exit 1; [ -e $T/st/lock-priv ] && for d in $T/run/r2-k1.*; do mkdir "\$d/k1-receipt.txt"; done; touch $T/st/reloaded; echo r >> $T/st/reloads; exit 0; }; exit 0
EOF
  cat > $T/bin/install <<EOF
#!/bin/bash
[ -e $T/st/term-ignore ] && { trap '' TERM; sleep 100; }
[ -e $T/st/fail-install ] && exit 1; a=(); while [ \$# -gt 0 ]; do case "\$1" in -o|-g) shift 2;; *) a+=("\$1"); shift;; esac; done
/usr/bin/install "\${a[@]}" || exit \$?; [ -e $T/st/corrupt-install ] && echo X >> "\${a[-1]}"; exit 0
EOF
  cat > $T/bin/mv <<EOF
#!/bin/bash
[ -e $T/st/fail-mv ] && exit 1; /usr/bin/mv "\$@" || exit \$?; [ -e $T/st/corrupt-dest ] && echo X >> $T/root/etc/systemd/system/$U
[ -e $T/st/foreign ] && echo stray > $T/root/etc/systemd/system/stray.conf; exit 0
EOF
  cat > $T/bin/rm <<EOF
#!/bin/bash
/usr/bin/rm "\$@"; r=\$?; [ -e $T/st/foreign-transient ] && /usr/bin/rm -f $T/root/etc/systemd/system/stray.conf; exit \$r
EOF
  cat > $T/bin/cp <<EOF
#!/bin/bash
/usr/bin/cp "\$@" || exit \$?; [ -e $T/st/race ] && for a in "\$@"; do case "\$a" in $T/j2out/*) printf 'replaced-after-snapshot\n' > "\$a";; esac; done; exit 0
EOF
  cat > $T/bin/cat <<EOF
#!/bin/bash
[ -e $T/st/bad-readback ] && for a in "\$@"; do case "\$a" in */k1-receipt.txt) echo tampered; exit 0;; esac; done; exec /usr/bin/cat "\$@"
EOF
  chmod +x $T/bin/*; mkrun $T $T/st; }
k1setup() { local T=$1 U=$U1 INSTALL=${2:-yes}; rm -rf $T; mkdir -p $T/out $T/root/etc/systemd/system/paths.target.wants $T/root/etc/systemd/user $T/root/home/.config/systemd/user $T/root/run/systemd/system $T/run $T/wd/markers $T/j2out $T/bin $T/st
  echo a > $T/root/etc/systemd/system/other.service; ln -s ../other.service $T/root/etc/systemd/system/paths.target.wants/other.service; echo u > $T/root/home/.config/systemd/user/x.service
  case $INSTALL in yes) printf '[Unit]\nDescription=lab\n[Service]\nExecStart=/x\n[Install]\nWantedBy=multi-user.target\n' > $T/j2out/serve.unit;;
    empty) printf '[Unit]\nDescription=lab\n[Service]\nExecStart=/x\n[Install]\n' > $T/j2out/serve.unit;; *) printf '[Unit]\nDescription=lab\n[Service]\nExecStart=/x\n' > $T/j2out/serve.unit;; esac
  KU=$(sha256sum < $T/j2out/serve.unit | cut -d' ' -f1); j2inv $T || echo "k1setup: J2 inventory failed"; KI=$(sha256sum < $T/j2out/units-after-apply.txt | cut -d' ' -f1)
  printf 'K1_REQUEST unit=%s dest=%s\nK1_UNIT_SHA %s\nK1_UNIT_TEXT %s\nK1_J2_INVENTORY %s %s' $U $T/root/etc/systemd/system/$U $KU $T/j2out/serve.unit $KI $T/j2out/units-after-apply.txt > $T/j2out/k1-request.txt; KR=$(sha256sum < $T/j2out/k1-request.txt | cut -d' ' -f1)
  printf 'HEAD x\nRECEIPT %s %s\nRECEIPT %s %s\nRECEIPT %s %s\n' $KR $T/j2out/k1-request.txt $KU $T/j2out/serve.unit $KI $T/j2out/units-after-apply.txt > $T/wd/markers/J2_READY
  k1stubs $T; pk $P/K1.sh $T/k1.sh $T; }
runk1() { local T=$1 us=${2:-}; [ -n "$us" ] || us=$(sha256sum < $T/j2out/serve.unit | cut -d' ' -f1); local rs; rs=$(sha256sum < $T/j2out/k1-request.txt | cut -d' ' -f1); ( cd $T && timeout 900 bash ./k1.sh $T/j2out/k1-request.txt $rs $us $T/out 2>&1 ); }
k1rc() { cat $1/run/r2-k1.*/k1-receipt.txt 2>/dev/null; }
k1neg() { local name=$1 flag=$2 want=$3 chk=$4 T=$BASE/k1-$1; k1setup $T; [ -n "$flag" ] && touch $T/st/$flag; local o; o="$(runk1 $T)"
  case "$o" in *"$want"*) if eval "$chk"; then ok "k1-$name" "$want"; else bad "k1-$name" "state after [$want]" "$(k1rc $T)"; fi;; *) bad "k1-$name" "$want" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac; }
D1='$T/root/etc/systemd/system/'$U1; S1K='$T/root/etc/systemd/system/.'$U1'.k1-stage'
# tree fingerprint OUTSIDE root's legitimate write set (the unit tree, PRIV, the stub state): must be unchanged by K1/K2
fpr() { ( cd $1 && find . -path ./root/etc/systemd/system -prune -o -path ./run -prune -o -path ./st -prune -o -path ./state -prune -o -path ./cg -prune -o -printf '%y %m %s %T@ %p -> %l\n' | LC_ALL=C sort ); }
k2setup() { local T=$1 X=$2 FAST=${3:-} U=$U1 S=$1/state; rm -rf $T; mkdir -p $T/out $T/cg/unit/sub $T/bin $S $T/root/etc/systemd/system $T/wd/markers $T/run $T/j2out
  printf '[Unit]\nDescription=lab\n[Install]\nWantedBy=multi-user.target\n' > $T/root/etc/systemd/system/$U; local IS RS=$(printf 'f%.0s' {1..64}); IS=$(sha256sum < $T/root/etc/systemd/system/$U | cut -d' ' -f1)
  printf 'K1_OK unit=%s dest=%s installed_sha=%s request=%s request_path=%s j2_inventory=%s pre_inventory=%s post_inventory=%s LoadState=loaded ActiveState=inactive UnitFileState=disabled priv=/run/r2-k1.fixture\n' $U $T/root/etc/systemd/system/$U $IS $RS $T/j2out/k1-request.txt $(printf 'c%.0s' {1..64}) $(printf 'a%.0s' {1..64}) $(printf 'b%.0s' {1..64}) > $T/k1r.txt
  printf 'HEAD x\nRECEIPT %s %s\n' $RS $T/j2out/k1-request.txt > $T/wd/markers/J2_READY
  cat > $T/bin/systemctl.real <<EOF
#!/bin/bash
S=$S; if [ "\$1" = show ]; then
  if [ "\$4" = InvocationID ] && [ -f \$S/started ]; then [ -f \$S/inv-read-fail ] && exit 1
    if [ -f \$S/flip ] && [ ! -f \$S/flipped ]; then c=\$(( \$(cat \$S/icount 2>/dev/null || echo 0) + 1 )); echo \$c > \$S/icount
      [ \$c -ge \$(cat \$S/flip-at 2>/dev/null || echo 3) ] && { echo inv-10 > \$S/\$2.InvocationID; touch \$S/flipped; [ -f \$S/flip-nr ] && echo 1 > \$S/\$2.NRestarts; }; fi; fi
  f=\$S/\$2.\$4; [ -f \$f ] && cat \$f || echo; exit 0; fi
case "\$1" in start) touch \$S/started; if [ -f \$S/no-inv ]; then echo inactive > \$S/\$2.ActiveState; exit 1; fi
  echo active > \$S/\$2.ActiveState; echo inv-9 > \$S/\$2.InvocationID; echo /unit > \$S/\$2.ControlGroup; touch \$S/listening; exit \$(cat \$S/start-rc 2>/dev/null || echo 0);;
  stop) echo stop >> \$S/stops; [ -f \$S/flip-at-stop ] && echo inv-11 > \$S/\$2.InvocationID; echo inactive > \$S/\$2.ActiveState; rm -f \$S/listening;; esac; exit 0
EOF
  printf '#!/bin/bash\nexec %s/bin/systemctl.real "$@"\n' $T > $T/bin/systemctl
  printf '#!/bin/bash\n[ -f %s/listening ] && echo '"'"'LISTEN 0 4096 0.0.0.0:18081 0.0.0.0:* users:(("gunbc",pid=424242,fd=7))'"'"'; exit 0\n' $S > $T/bin/ss
  cat > $T/bin/cp <<EOF
#!/bin/bash
/usr/bin/cp "\$@" || exit \$?; [ -e $S/race ] && for a in "\$@"; do case "\$a" in $T/k1r.txt|$T/j3.txt|$T/wd/markers/J2_READY) printf 'replaced-after-snapshot\n' > "\$a";; esac; done; exit 0
EOF
  chmod +x $T/bin/*; mkrun $T $S; echo loaded > $S/$U.LoadState; echo inactive > $S/$U.ActiveState; echo disabled > $S/$U.UnitFileState; echo $T/root/etc/systemd/system/$U > $S/$U.FragmentPath; echo no > $S/$U.NeedDaemonReload; : > $S/$U.DropInPaths; echo 0 > $S/$U.NRestarts
  : > $T/cg/unit/cgroup.procs; echo 424242 > $T/cg/unit/sub/cgroup.procs; : > $T/j3.txt
  for u in belt-exit belt-discovery attempt-unit-drain; do n=gunbc-srv1-lab-codex-feedback-v0-$u; echo active > $S/$n.path.ActiveState; echo enabled > $S/$n.path.UnitFileState; echo pinv-$u > $S/$n.path.InvocationID; echo 1234$u > $S/$n.path.ActiveEnterTimestampMonotonic
    echo "J3_OK $n.path ActiveState=active UnitFileState=enabled Triggers=$n.service InvocationID=pinv-$u ActiveEnterTimestampMonotonic=1234$u" >> $T/j3.txt; done
  pk $P/K2.sh $T/k2.sh $T; sed -i "s/^K2_END=\$(( EPOCHSECONDS + 2340 ));/K2_END=\$(( EPOCHSECONDS + $X ));/" $T/k2.sh; grep -q "EPOCHSECONDS + $X ))" $T/k2.sh || echo "k2setup: deadline patch failed"
  [ -n "$FAST" ] && { sed -i "s/^GRACE=5; STOP_RESERVE=360; RECEIPT_RESERVE=60; START_CAP=120; OBS_ALLOW=900$/GRACE=5; STOP_RESERVE=90; RECEIPT_RESERVE=30; START_CAP=10; OBS_ALLOW=$FAST/" $T/k2.sh; grep -q "OBS_ALLOW=$FAST\$" $T/k2.sh || echo "k2setup: FAST patch failed"; }; true; }
runk2() { local T=$1 R3=${2:-$1/j3.txt} K1S J3S; K1S=$(sha256sum < $T/k1r.txt | cut -d' ' -f1); J3S=$(sha256sum < $R3 | cut -d' ' -f1); ( cd $T && timeout 900 bash ./k2.sh $T/k1r.txt $K1S $R3 $J3S $T/out 2>&1 ); }
noss() { printf '#!/bin/bash\nexit 0\n' > $1/bin/ss; chmod +x $1/bin/ss; }
slowcat() { printf '#!/bin/bash\nfor a in "$@"; do case "$a" in %s/cg/*) sleep %s;; esac; done\nexec /usr/bin/cat "$@"\n' $1 ${2:-12} > $1/bin/cat; chmod +x $1/bin/cat
  mkdir -p $1/cg/unit/sub2 $1/cg/unit/sub3; : > $1/cg/unit/sub2/cgroup.procs; : > $1/cg/unit/sub3/cgroup.procs; }
#case k1-happy
T=$BASE/k1a; k1setup $T; o="$(runk1 $T | tail -1)"; KU=$(sha256sum < $T/j2out/serve.unit | cut -d' ' -f1); DU=$(sha256sum < $T/root/etc/systemd/system/$U1 2>/dev/null | cut -d' ' -f1)
case "$o" in K1_OK*installed_sha=$KU*priv=*) [ "$DU" = "$KU" ] && [ "$(wc -l < $T/st/reloads)" = 1 ] && [ "$(k1rc $T)" = "$o" ] && [ "$(cat $T/out/k1-receipt.txt)" = "$o" ] && [ ! -e $T/st/violations ] && ok k1-happy "J2's inventory projection = K1's pre-inventory; installed bytes = reviewed sha; 1 reload; K1_OK persisted in PRIV, exported to OUT as the user, read back" || bad k1-happy "dest sha + 1 reload + persisted + exported receipt" "$DU / $(cat $T/out/k1-receipt.txt 2>&1) / $(cat $T/st/violations 2>/dev/null)";; *) bad k1-happy K1_OK "$o";; esac
#case k1-b1-install
k1neg b1-install fail-install "B1: install to stage failed" "[ ! -e $D1 ] && [ ! -e $S1K ] && [ ! -e \$T/st/reloads ]"
#case k1-b2-stage-readback
k1neg b2-stage-readback corrupt-install "B2: stage readback is not the reviewed sha" "[ ! -e $D1 ] && [ ! -e $S1K ] && [ ! -e \$T/st/reloads ]"
#case k1-b3-rename
k1neg b3-rename fail-mv "B3: rename failed" "[ ! -e $D1 ] && [ ! -e $S1K ] && [ ! -e \$T/st/reloads ]"
#case k1-b4-dest-readback
k1neg b4-dest-readback corrupt-dest "B4: installed bytes are not the reviewed sha" "[ ! -e $D1 ] && [ ! -e \$T/st/reloads ]"
#case k1-b5-verified
T=$BASE/k1-b5v; k1setup $T; touch $T/st/foreign $T/st/foreign-transient; o="$(runk1 $T)"
case "$o" in *"B5: an entry other than"*"VERIFIED equal to pre; NOT reloaded"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && [ ! -e $T/st/reloads ] && ok k1-b5-verified "foreign change -> dest removed, inventory VERIFIED equal to pre, no reload" || bad k1-b5-verified "removed + no reload" "state";; *) bad k1-b5-verified "B5 verified" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-b5-unresolved
T=$BASE/k1-b5u; k1setup $T; touch $T/st/foreign; o="$(runk1 $T)"
case "$o" in *"B5: an entry other than"*"did NOT verify"*) k1rc $T | grep -q '^K1_UNRESOLVED step=B5' && [ ! -e $T/st/reloads ] && ok k1-b5-unresolved "a persistent foreign change -> UNRESOLVED, no reload" || bad k1-b5-unresolved "UNRESOLVED receipt + no reload" "$(k1rc $T)";; *) bad k1-b5-unresolved "B5 unresolved" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-b6-reload
T=$BASE/k1-b6; k1setup $T; touch $T/st/reload-fail; o="$(runk1 $T)"
case "$o" in *"B6: daemon-reload failed"*) k1rc $T | grep -q '^K1_UNRESOLVED step=B6 dest=present' && ok k1-b6-reload "reload failure -> UNRESOLVED with dest present, no further action" || bad k1-b6-reload "UNRESOLVED receipt" "$(k1rc $T)";; *) bad k1-b6-reload "B6" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-text-moved
T=$BASE/k1-moved; k1setup $T; KU0=$(sha256sum < $T/j2out/serve.unit | cut -d' ' -f1); echo moved >> $T/j2out/serve.unit; o="$(runk1 $T $KU0)"; case "$o" in *"B0: the snapshot of the unit text is not the cited bytes"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && ok k1-text-moved "a unit text changed after the request refuses at B0" || bad k1-text-moved "nothing installed" "installed";; *) bad k1-text-moved "B0 sha refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-unbound
T=$BASE/k1-unbound; k1setup $T; printf 'HEAD x\n' > $T/wd/markers/J2_READY; o="$(runk1 $T)"; case "$o" in *"does not bind exactly this request"*) ok k1-unbound "a request J2_READY does not bind refuses at B0";; *) bad k1-unbound "B0 binding refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-partial
T=$BASE/k1-partial; k1setup $T; echo p > $T/wd/J2_PARTIAL; o="$(runk1 $T)"; case "$o" in *"J2_PARTIAL exists"*) ok k1-partial "J2_PARTIAL present refuses at B0";; *) bad k1-partial "B0 partial refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-receipt-unwritable
T=$BASE/k1-runw; k1setup $T; touch $T/st/lock-priv; o="$(runk1 $T)"
case "$o" in *K1_OK*) bad k1-receipt-unwritable "no success text" "printed K1_OK";; *"K1_UNRESOLVED step=R the install and reload HAPPENED"*) ok k1-receipt-unwritable "an unwritable receipt -> UNRESOLVED exit 2, no K1_OK text";; *) bad k1-receipt-unwritable "UNRESOLVED step=R" "$(printf '%s' "$o" | tail -1)";; esac
#case k1-receipt-readback
T=$BASE/k1-rbad; k1setup $T; touch $T/st/bad-readback; o="$(runk1 $T)"
case "$o" in *K1_OK*) bad k1-receipt-readback "no success text" "printed K1_OK";; *"K1_UNRESOLVED step=R"*) ok k1-receipt-readback "a receipt read back different -> UNRESOLVED, no K1_OK text";; *) bad k1-receipt-readback "UNRESOLVED step=R" "$(printf '%s' "$o" | tail -1)";; esac
#case k1-term-ignore
T=$BASE/k1-term; k1setup $T; touch $T/st/term-ignore; sed -i 's/^capv 60 B1; if ! T install/capv 6 B1; if ! T install/' $T/k1.sh; S0=$EPOCHSECONDS; o="$(runk1 $T)"; el=$(( EPOCHSECONDS - S0 ))
case "$o" in *"B1: install to stage failed"*) [ $el -lt 40 ] && ok k1-term-ignore "a TERM-ignoring install is KILLed at cap+grace (${el}s); B1 disposition" || bad k1-term-ignore "< 40 s" "${el}s";; *) bad k1-term-ignore "B1 failure" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-late-boundary
T=$BASE/k1-late; k1setup $T; sed -i 's/^K1_END=\$(( EPOCHSECONDS + 2400 ))$/K1_END=$(( EPOCHSECONDS + 150 ))/' $T/k1.sh; o="$(runk1 $T)"
case "$o" in *"K1_REFUSED step=B0"*"refused before the step"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && ok k1-late-boundary "remaining time below a step's cap refuses BEFORE that step (nothing installed)" || bad k1-late-boundary "nothing installed" "installed";; *) bad k1-late-boundary "pre-step refusal" "$(printf '%s' "$o" | grep -E 'K1_|NO OBS' | tail -1)";; esac
#case k1-snapshot-race
T=$BASE/k1-race; k1setup $T; touch $T/st/race; o="$(runk1 $T | tail -1)"; DU=$(sha256sum < $T/root/etc/systemd/system/$U1 2>/dev/null | cut -d' ' -f1); KU2=$(sha256sum < $T/run/r2-k1.*/unit 2>/dev/null | cut -d' ' -f1)
case "$o" in K1_OK*) [ "$(cat $T/j2out/serve.unit)" = replaced-after-snapshot ] && [ "$DU" = "$KU2" ] && ok k1-snapshot-race "originals replaced after the snapshot: K1 hashed, parsed and acted on the snapshots only (installed = the snapshot's reviewed bytes)" || bad k1-snapshot-race "acted on snapshots" "dest=$DU";; *) bad k1-snapshot-race "K1_OK on snapshots" "$o";; esac
#case k1-unit-sha-arg
T=$BASE/k1-usha; k1setup $T; o="$(runk1 $T $(printf 'e%.0s' {1..64}))"; case "$o" in *"not the reviewed unit sha given as argument"*) ok k1-unit-sha-arg "the request's unit sha must equal the independent argument";; *) bad k1-unit-sha-arg "refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-inv-changed
T=$BASE/k1-invchg; k1setup $T; sleep 1; echo b > $T/root/etc/systemd/system/other.service; o="$(runk1 $T)"
case "$o" in *"changed since J2's post-apply inventory"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && ok k1-inv-changed "a file touched between J2 and K1 refuses at B0" || bad k1-inv-changed "nothing installed" "installed";; *) bad k1-inv-changed "B0 inventory refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-static
T=$BASE/k1-static; k1setup $T no; o="$(runk1 $T)"; case "$o" in *"K1_REFUSED step=B0"*"would load as static"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && [ ! -e $T/st/reloads ] && ok k1-static "a reviewed unit without [Install] refuses at B0 (read from the rendered bytes), nothing installed" || bad k1-static "nothing installed" "installed";; *) bad k1-static "B0 static refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-static-empty-install
T=$BASE/k1-static2; k1setup $T empty; o="$(runk1 $T)"; case "$o" in *"K1_REFUSED step=B0"*"would load as static"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && ok k1-static-empty-install "an [Install] section naming no WantedBy/RequiredBy/UpheldBy/Alias refuses at B0, nothing installed" || bad k1-static-empty-install "nothing installed" "installed";; *) bad k1-static-empty-install "B0 static refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-no-user-writes
T=$BASE/k1-nuw; k1setup $T; touch $T/st/export-sandbox; for f in k1-receipt.txt k1-begin.txt; do ln -s $T/victim $T/out/$f; ln -s $T/victim $T/j2out/$f; ln -s $T/victim $T/wd/$f; done; F0="$(fpr $T)"; o="$(runk1 $T | tail -1)"; F1="$(fpr $T)"
case "$o" in K1_OK*) [ ! -e $T/victim ] && [ "$F0" = "$F1" ] && [ "$(grep -c '^-u briansrls -- ' $T/st/runuser.log)" = 4 ] && [ "$(cat $T/st/exported/k1-receipt.txt)" = "$o" ] && ok k1-no-user-writes "planted symlinks at OUT/k1-receipt.txt etc. untouched: with the user-side writer sandboxed, the tree outside PRIV and the unit tree is byte-identical (root wrote nothing there); the 4 export calls all ran as -u briansrls" || bad k1-no-user-writes "no root writes in user paths" "victim=$([ -e $T/victim ] && echo written) tree-changed=$([ "$F0" = "$F1" ] || echo yes) calls=$(wc -l < $T/st/runuser.log)";; *) bad k1-no-user-writes K1_OK "$o";; esac
#case k1-out-renamed
T=$BASE/k1-oren; k1setup $T; echo 3 > $T/st/rename-out-at; o="$(runk1 $T)"
case "$o" in *$'\n'K1_OK*|K1_OK*) bad k1-out-renamed "no success line" "printed K1_OK";; *"K1_UNRESOLVED step=X the install and reload HAPPENED"*"evidence retained in PRIV=$T/run/r2-k1."*) k1rc $T | grep -q '^K1_OK' && [ -e $T/root/etc/systemd/system/$U1 ] && [ ! -e $T/out/k1-receipt.txt ] && ok k1-out-renamed "OUT renamed between B0 and the final export: UNRESOLVED step=X exit 2, PRIV path printed, the PRIV receipt still records K1_OK, nothing written into the renamed dir by root" || bad k1-out-renamed "PRIV receipt K1_OK" "$(k1rc $T)";; *) bad k1-out-renamed "UNRESOLVED step=X" "$(printf '%s' "$o" | tail -2 | tr '\n' ' ')";; esac
#case k1-export-fail
T=$BASE/k1-xf; k1setup $T; echo 3 > $T/st/export-fail-at; o="$(runk1 $T)"
case "$o" in *$'\n'K1_OK*|K1_OK*) bad k1-export-fail "no success line" "printed K1_OK";; *"K1_UNRESOLVED step=X"*"PRIV=$T/run/r2-k1."*) ok k1-export-fail "the export of K1_OK fails after the mutation: UNRESOLVED step=X, exit 2, PRIV retained and printed";; *) bad k1-export-fail "UNRESOLVED step=X" "$(printf '%s' "$o" | tail -2 | tr '\n' ' ')";; esac
#case k1-out-unusable
T=$BASE/k1-ou; k1setup $T; rmdir $T/out; o="$(runk1 $T)"
case "$o" in *"K1_REFUSED step=B0"*"is not writable as briansrls"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && [ ! -e $T/st/reloads ] && ok k1-out-unusable "an unusable OUT refuses at B0 (k1-begin export), nothing installed, no reload" || bad k1-out-unusable "nothing installed" "installed";; *) bad k1-out-unusable "B0 refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-priv-mode
T=$BASE/k1-pm; k1setup $T; touch $T/st/priv-mode; o="$(runk1 $T)"
case "$o" in *"B0"*"is not owner"*|*"B0: the private dir"*"mode 700"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && [ ! -e $T/out/k1-begin.txt ] && ok k1-priv-mode "a private dir that is not mode 700 refuses at B0 before any snapshot, export or install" || bad k1-priv-mode "nothing done" "installed or exported";; *) bad k1-priv-mode "B0 PRIV refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-priv-symlink
T=$BASE/k1-ps; k1setup $T; touch $T/st/priv-symlink; o="$(runk1 $T)"
case "$o" in *"B0"*"symbolic link"*) [ ! -e $T/root/etc/systemd/system/$U1 ] && [ -z "$(ls -A $T/elsewhere.* 2>/dev/null)" ] && ok k1-priv-symlink "a private dir path that is a symlink refuses at B0; nothing written through it" || bad k1-priv-symlink "nothing written" "written";; *) bad k1-priv-symlink "B0 PRIV refusal" "$(printf '%s' "$o" | grep 'NO OBS' | tail -1)";; esac
#case k1-e2e-j2
T=$BASE/k1e2e; j2setup $T; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o2="$(run2 $T $T/out $SU | tail -1)"; mkdir -p $T/st $T/run $T/k1out
if [ "$o2" != DONE ]; then bad k1-e2e-j2 "J2 DONE" "$o2"; else k1stubs $T; pk $P/K1.sh $T/k1.sh $T; RS=$(sha256sum < $T/out/k1-request.txt | cut -d' ' -f1)
  o="$( cd $T && timeout 900 bash ./k1.sh $T/out/k1-request.txt $RS $SU $T/k1out 2>&1 | tail -1 )"; DU=$(sha256sum < $T/root/etc/systemd/system/$U1 2>/dev/null | cut -d' ' -f1)
  case "$o" in K1_OK*installed_sha=$SU*) [ "$DU" = "$SU" ] && [ "$(cat $T/k1out/k1-receipt.txt)" = "$o" ] && ok k1-e2e-j2 "J2's REAL request, J2_READY and post-apply inventory consumed by K1: K1_OK, installed = the reviewed render" || bad k1-e2e-j2 "installed + exported" "$DU";; *) bad k1-e2e-j2 K1_OK "$o";; esac; fi
#case k2-happy
T=$BASE/k2a; k2setup $T 2340; o="$(runk2 $T | tail -1)"; case "$o" in K2_UP*invocation=inv-9*listener_pids=424242*priv=*) [ "$(cat $T/run/r2-k2.*/k2-receipt.txt)" = "$o" ] && [ "$(cat $T/out/k2-receipt.txt)" = "$o" ] && [ ! -e $T/state/violations ] && ok k2-happy "K2_UP as this start's invocation, persisted in PRIV, exported as the user" || bad k2-happy "persisted + exported" "receipt differs";; *) bad k2-happy K2_UP "$o";; esac
#case k2-deadline
T=$BASE/k2b; k2setup $T 170 20; noss $T; : > $T/cg/unit/sub/cgroup.procs; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *DEADLINE*"pre-stop invocation=inv-9"*) ok k2-deadline "no listener by the deadline: this invocation stopped and verified down (same run), pre/post readings recorded";; *) bad k2-deadline "deadline + verified stop" "$o";; esac
#case k2-subtree
T=$BASE/k2c; k2setup $T 170 20; noss $T; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"stop failed or could not be verified"*) ok k2-subtree "sub-cgroup survivor after stop -> UNRESOLVED";; *) bad k2-subtree UNRESOLVED "$o";; esac
#case k2-cg-missing
T=$BASE/k2c2; k2setup $T 170 20; noss $T; rm -rf $T/cg/unit; o="$(runk2 $T | grep -E 'NO OBS|not present before the stop' | tr '\n' ' ')"
case "$o" in *"not present before the stop"*"could not be verified"*) ok k2-cg-missing "a unit cgroup absent BEFORE the stop never reads as 'gone' -> UNRESOLVED";; *) bad k2-cg-missing "UNRESOLVED (cgroup absent before stop)" "$o";; esac
#case k2-nopid
T=$BASE/k2d; k2setup $T 2340; printf '#!/bin/bash\n[ -f %s/state/listening ] && { echo '"'"'LISTEN 0 4096 0.0.0.0:18081 0.0.0.0:* users:(("gunbc",pid=424242,fd=7))'"'"'; echo '"'"'LISTEN 0 4096 [::]:18081 [::]:*'"'"'; }; exit 0\n' $T > $T/bin/ss; chmod +x $T/bin/ss
: > $T/cg/unit/sub/cgroup.procs; o="$(runk2 $T | grep 'NO OBS')"; case "$o" in *"carries no pid"*) ok k2-nopid "pid-less listener refused (and this invocation stopped)";; *) bad k2-nopid "refusal" "$o";; esac
#case k2-receipt
T=$BASE/k2e; k2setup $T 2340; sed 's/pinv-belt-exit/pinv-OLD/' $T/j3.txt > $T/j3f.txt; cp $T/j3.txt $T/j3x.txt; echo 'J3_FAILED x' >> $T/j3x.txt
o1="$(runk2 $T $T/j3f.txt | grep 'NO OBS')"; o2="$(runk2 $T $T/j3x.txt | grep 'NO OBS')"
case "$o1|$o2" in *"does not equal the live reading"*"|"*"not exactly 3 lines"*) ok k2-receipt "stale and extra-line J3 receipts refused";; *) bad k2-receipt "both refused" "$o1 / $o2";; esac
#case k2-dest-moved
T=$BASE/k2e2; k2setup $T 2340; echo X >> $T/root/etc/systemd/system/$U1; o="$(runk2 $T | grep 'NO OBS')"; case "$o" in *"is not the bytes K1 installed"*) ok k2-dest-moved "installed bytes changed since K1 refuses before start";; *) bad k2-dest-moved "refusal" "$o";; esac
#case k2-binding-exact
T=$BASE/k2be; k2setup $T 2340; printf 'HEAD x\nRECEIPT %s /elsewhere/k1-request.txt\n' $(printf 'f%.0s' {1..64}) > $T/wd/markers/J2_READY; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"does not bind the EXACT request"*) [ ! -f $T/state/started ] && ok k2-binding-exact "J2_READY binding the same sha at ANOTHER path refuses (exact path, not a pattern)" || bad k2-binding-exact "not started" "started";; *) bad k2-binding-exact "refusal" "$o";; esac
#case k2-stall
T=$BASE/k2f; k2setup $T 170 20; printf '#!/bin/bash\n[ "$1" = show ] && [ "$(echo "$@" | grep -o " -p " | wc -l)" -gt 3 ] && sleep 600\nexec %s/bin/systemctl.real "$@"\n' $T > $T/bin/systemctl; chmod +x $T/bin/systemctl; : > $T/cg/unit/sub/cgroup.procs
S0=$EPOCHSECONDS; o="$(runk2 $T | grep 'NO OBS')"; el=$(( EPOCHSECONDS-S0 )); case "$o" in *"post-start property read"*) [ $el -lt 180 ] && ok k2-stall "bounded inside the whole-act deadline (${el}s < 180)" || bad k2-stall "bounded" "${el}s";; *) bad k2-stall "post-start read refusal" "$o";; esac
#case k2-late
T=$BASE/k2g; k2setup $T 260 100
printf '#!/bin/bash\nf=%s/state/listening; [ -f $f ] && [ $(( $(date +%%s) - $(stat -c %%Y $f) )) -ge 100 ] && echo '"'"'LISTEN 0 4096 0.0.0.0:18081 0.0.0.0:* users:(("gunbc",pid=424242,fd=7))'"'"'; exit 0\n' $T > $T/bin/ss; chmod +x $T/bin/ss
S0=$EPOCHSECONDS; o="$(runk2 $T | grep -E '^K2_UP|NO OBS' | head -1)"; el=$(( EPOCHSECONDS-S0 )); case "$o" in K2_UP*) [ $el -lt 260 ] && ok k2-late "late listener (100 s): K2_UP in ${el}s < 260" || bad k2-late "before the deadline" "${el}s";; *) bad k2-late K2_UP "$o";; esac
#case k2-slowcg-listen
T=$BASE/k2h; k2setup $T 2340; slowcat $T; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"cannot read the unit cgroup subtree"*"could not be verified"*) ok k2-slowcg-listen "slow 3-subgroup read refuses attribution, stop unverified -> UNRESOLVED";; *) bad k2-slowcg-listen "attribution refused + UNRESOLVED" "$o";; esac
#case k2-slowcg-stop
T=$BASE/k2i; k2setup $T 170 20; noss $T; : > $T/cg/unit/sub/cgroup.procs; slowcat $T 40; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *DEADLINE*"could not be verified"*) ok k2-slowcg-stop "slow 3-subgroup read on the stop path (slower than the remaining stop reserve) -> UNRESOLVED, never verified down";; *) bad k2-slowcg-stop "UNRESOLVED (not verified down)" "$o";; esac
#case k2-too-little-time
T=$BASE/k2t; k2setup $T 1000; o="$(runk2 $T | grep 'NO OBS')"; case "$o" in *"refused before start"*) [ ! -f $T/state/started ] && ok k2-too-little-time "less than the full start+observation+stop+receipt allowance refuses BEFORE start" || bad k2-too-little-time "not started" "started";; *) bad k2-too-little-time "pre-start refusal" "$o";; esac
#case k2-foreign-actor
T=$BASE/k2x; k2setup $T 200 40; noss $T; touch $T/state/flip; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"WITHOUT an NRestarts increase"*"NOT stopped"*) [ ! -f $T/state/stops ] && ok k2-foreign-actor "invocation changed without an NRestarts increase while waiting: another actor -> NO stop, UNRESOLVED" || bad k2-foreign-actor "no stop" "stopped";; *) bad k2-foreign-actor "UNRESOLVED no stop" "$o";; esac
#case k2-auto-restart
T=$BASE/k2y; k2setup $T 200 40; noss $T; : > $T/cg/unit/sub/cgroup.procs; touch $T/state/flip $T/state/flip-nr; o="$(runk2 $T | grep -E 'NO OBS|automatic restart' | tr '\n' ' ')"
case "$o" in *"automatic restart"*"pre-stop invocation=inv-10 NRestarts=1"*) [ -f $T/state/stops ] && ok k2-auto-restart "invocation changed WITH an NRestarts increase: an automatic restart of K2's start -> stopped and verified down" || bad k2-auto-restart "stopped" "no stop";; *) bad k2-auto-restart "stop of the auto-restarted invocation" "$o";; esac
# sleek-owl msg_cfe1a18a M3: fail_ours' own invocation scoping, through the REAL start path (no observation loop involved)
#case k2-inv-foreign-at-stop
T=$BASE/k2m1; k2setup $T 2340; echo 1 > $T/state/start-rc; touch $T/state/flip; echo 2 > $T/state/flip-at; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"start rc=1"*"is NOT from K2's start; NOT stopped"*) [ ! -f $T/state/stops ] && ok k2-inv-foreign-at-stop "start rc!=0, then the invocation changed WITHOUT an NRestarts increase before the stop: NO stop, UNRESOLVED" || bad k2-inv-foreign-at-stop "no stop" "stopped";; *) bad k2-inv-foreign-at-stop "UNRESOLVED, not stopped" "$o";; esac
#case k2-inv-auto-at-stop
T=$BASE/k2m2; k2setup $T 2340; echo 1 > $T/state/start-rc; touch $T/state/flip $T/state/flip-nr; echo 2 > $T/state/flip-at; : > $T/cg/unit/sub/cgroup.procs; o="$(runk2 $T | grep 'NO OBS')"; rc2="$(cat $T/run/r2-k2.*/k2-receipt.txt 2>/dev/null)"
case "$rc2" in K2_STOPPED_VERIFIED_DOWN*auto_restarts=1*"pre-stop invocation=inv-10 NRestarts=1"*) [ -f $T/state/stops ] && ok k2-inv-auto-at-stop "start rc!=0, then an automatic restart (NRestarts +1) before the stop: stopped and verified, auto_restarts=1 recorded" || bad k2-inv-auto-at-stop "stopped" "no stop";; *) bad k2-inv-auto-at-stop "STOPPED_VERIFIED_DOWN auto_restarts=1" "$rc2 / $o";; esac
#case k2-start-rc
T=$BASE/k2m3; k2setup $T 2340; echo 1 > $T/state/start-rc; : > $T/cg/unit/sub/cgroup.procs; o="$(runk2 $T | tail -1)"; rc2="$(cat $T/run/r2-k2.*/k2-receipt.txt 2>/dev/null)"
case "$rc2" in K2_STOPPED_VERIFIED_DOWN*"start rc=1"*) ok k2-start-rc "start rc=1 (a listener even present): this invocation stopped and verified down, never K2_UP";; *) bad k2-start-rc "STOPPED_VERIFIED_DOWN start rc=1" "$rc2 / $o";; esac
#case k2-empty-invocation
T=$BASE/k2m4; k2setup $T 2340; touch $T/state/no-inv; o="$(runk2 $T | grep 'NO OBS')"; rc2="$(cat $T/run/r2-k2.*/k2-receipt.txt 2>/dev/null)"
case "$rc2" in K2_NOT_STARTED*"produced no invocation"*) [ ! -f $T/state/stops ] && ok k2-empty-invocation "start produced no invocation, unit inactive: K2_NOT_STARTED, nothing stopped" || bad k2-empty-invocation "no stop" "stopped";; *) bad k2-empty-invocation K2_NOT_STARTED "$rc2 / $o";; esac
#case k2-inv-read-fail
T=$BASE/k2m5; k2setup $T 2340; touch $T/state/inv-read-fail; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"start rc=0; InvocationID read failed"*) [ ! -f $T/state/stops ] && ok k2-inv-read-fail "the InvocationID read after start fails: UNRESOLVED, no stop" || bad k2-inv-read-fail "no stop" "stopped";; *) bad k2-inv-read-fail "UNRESOLVED (read failed)" "$o";; esac
#case k2-no-user-writes
T=$BASE/k2nuw; k2setup $T 2340; touch $T/state/export-sandbox; for f in k2-receipt.txt k2-begin.txt; do ln -s $T/victim $T/out/$f; ln -s $T/victim $T/j2out/$f; ln -s $T/victim $T/wd/$f; done; F0="$(fpr $T)"; o="$(runk2 $T | tail -1)"; F1="$(fpr $T)"
case "$o" in K2_UP*) [ ! -e $T/victim ] && [ "$F0" = "$F1" ] && [ "$(grep -c '^-u briansrls -- ' $T/state/runuser.log)" = 4 ] && ok k2-no-user-writes "planted symlinks at OUT/k2-receipt.txt etc. untouched; with the user-side writer sandboxed the tree outside PRIV is byte-identical; 4 export calls as -u briansrls" || bad k2-no-user-writes "no root writes in user paths" "victim=$([ -e $T/victim ] && echo written) tree-changed=$([ "$F0" = "$F1" ] || echo yes)";; *) bad k2-no-user-writes K2_UP "$o";; esac
#case k2-export-fail
T=$BASE/k2xf; k2setup $T 2340; echo 3 > $T/state/export-fail-at; o="$(runk2 $T)"
case "$o" in *$'\n'K2_UP*|K2_UP*) bad k2-export-fail "no success line" "printed K2_UP";; *"K2_UNRESOLVED step=X the serve is UP"*"NOT stopped"*"PRIV=$T/run/r2-k2."*) [ ! -f $T/state/stops ] && grep -q '^K2_UP' $T/run/r2-k2.*/k2-receipt.txt && ok k2-export-fail "the K2_UP export fails after the start: UNRESOLVED step=X exit 2, PRIV printed, the PRIV receipt records K2_UP, NOT stopped" || bad k2-export-fail "PRIV K2_UP + no stop" "state";; *) bad k2-export-fail "UNRESOLVED step=X" "$(printf '%s' "$o" | tail -2 | tr '\n' ' ')";; esac
#case k2-out-unusable
T=$BASE/k2ou; k2setup $T 2340; rmdir $T/out; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"is not writable as briansrls"*"refused before start"*) [ ! -f $T/state/started ] && ok k2-out-unusable "an unusable OUT refuses before start" || bad k2-out-unusable "not started" "started";; *) bad k2-out-unusable "refusal" "$o";; esac
#case k2-priv-mode
T=$BASE/k2pm; k2setup $T 2340; touch $T/state/priv-mode; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"the private dir"*"mode 700"*) [ ! -f $T/state/started ] && [ ! -e $T/out/k2-begin.txt ] && ok k2-priv-mode "a private dir that is not mode 700 refuses before any snapshot, export or start" || bad k2-priv-mode "nothing done" "done";; *) bad k2-priv-mode "PRIV refusal" "$o";; esac
#case k2-post-check-restart
T=$BASE/k2pc; k2setup $T 2340; echo 1 > $T/state/start-rc; touch $T/state/flip-at-stop; : > $T/cg/unit/sub/cgroup.procs; o="$(runk2 $T | grep 'NO OBS')"
case "$o" in *"changed between the last pre-stop read and the stop"*) [ "$(wc -l < $T/state/stops)" = 1 ] && ! grep -q STOPPED_VERIFIED_DOWN $T/run/r2-k2.*/k2-receipt.txt && ok k2-post-check-restart "InvocationID changed between the last pre-stop check and the stop: UNRESOLVED (never STOPPED_VERIFIED_DOWN), one stop, no widened stop" || bad k2-post-check-restart "UNRESOLVED, 1 stop" "$(cat $T/run/r2-k2.*/k2-receipt.txt)";; *) bad k2-post-check-restart "UNRESOLVED (post-stop attribution)" "$o";; esac
#case k2-snapshot-race
T=$BASE/k2sr; k2setup $T 2340; touch $T/state/race; o="$(runk2 $T | tail -1)"
case "$o" in K2_UP*) [ "$(cat $T/k1r.txt)" = replaced-after-snapshot ] && [ "$(cat $T/j3.txt)" = replaced-after-snapshot ] && ok k2-snapshot-race "K1 receipt, J3 receipt and J2_READY replaced right after their snapshots: K2 hashed and parsed the snapshots only" || bad k2-snapshot-race "originals replaced" "not replaced";; *) bad k2-snapshot-race "K2_UP on snapshots" "$o";; esac
#endcases
# J1B1 empty-mktemp bound: an existing top-level /attempt-state-like path refuses before any run (the path is redirected
# into the fixture tree; the check is otherwise the generated J1B1's own line)
T=$BASE/emk; mkdir -p $T/fr/attempt-state; base $T; H=$(git -C $T/wd/wt rev-parse HEAD); TR=$(git -C $T/wd/wt rev-parse HEAD^{tree})
patch $P/ld449-r2-j1b1.sh $T/e.sh $T $H $TR "s=s.replace('for _p in /attempt-state ','for _p in %s/fr/attempt-state '%T)"
o="$(cd $T && bash ./e.sh $T/o FABRIC_POSTS_GRANT=msg_00000000-fixture 2>&1 | tail -1)"; case "$o" in *"fr/attempt-state exists"*) ok j1b1-emptymktemp "a pre-existing /attempt-state-like path refuses before any run";; *) bad j1b1-emptymktemp "exists refusal" "$o";; esac
T=$BASE/fab; mkdir -p $T; base $T; H=$(git -C $T/wd/wt rev-parse HEAD); TR=$(git -C $T/wd/wt rev-parse HEAD^{tree}); echo k > $T/key
patch $P/ld449-r2-j1b1.sh $T/f.sh $T $H $TR "s=s.replace('/etc/gunbc-fabric-storage/state-writer-mac-key','$T/key').replace('join_fixture || die2','true || die2').replace('fresh_own J1B1_GREEN','echo GATE_PASSED; exit 0')"
o1="$(cd $T && bash ./f.sh $T/o1 NOARG 2>&1 | tail -1)"; o2="$(cd $T && bash ./f.sh $T/o2 FABRIC_POSTS_GRANT=msg_12345678-abcd 2>&1 | tail -1)"
case "$o1|$o2" in *"arg 2 must be FABRIC_POSTS_GRANT"*"|GATE_PASSED") ok fabric-gate "readable key without grant arg refuses; with it the gate passes";; *) bad fabric-gate "refuse then pass" "$o1 / $o2";; esac
# the fabric receipt is written (rc-checked, read back) with the gate's exact line, and BOUND: J1B1_GREEN expects it and mark passes it
[ "$(cat $T/o2/fabric-posts.txt 2>/dev/null)" = "FABRIC_POSTS=up-to-4 key=$T/key FABRIC_POSTS_GRANT=msg_12345678-abcd" ] && grep -q 'J1B1_GREEN=.*O:fabric-posts.txt' $P/ld449-r2-j1b1.sh && grep -q '^mark J1B1_GREEN J1R_GREEN .*\$R/fabric-posts.txt' $P/ld449-r2-j1b1.sh \
  && ok fabric-receipt "fabric-posts.txt holds the gate's line and is a bound J1B1_GREEN receipt" || bad fabric-receipt "exact receipt + binding" "[$(cat $T/o2/fabric-posts.txt 2>/dev/null)]"
echo "--- cases done; failures: $FAILS"; [ $FAILS = 0 ]
