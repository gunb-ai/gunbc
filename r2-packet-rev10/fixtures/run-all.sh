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
s=re.sub(r'^\[ "\$\(grep -c .*/proc/self/cgroup.*$','true',s,flags=re.M); s=re.sub(r'^CG=.*$','CG=%s/cg/job'%T,s,flags=re.M); s=re.sub(r'^want .*$',':',s,flags=re.M); s=re.sub(r'^contain_selftest$',': contain_selftest-runs-only-on-a-real-cgroup',s,flags=re.M)
for a,b in [('/home/briansrls/runs/ld449-r2work',T+'/wd'),('/opt/gunbc-lab/codex-provider-feedback-v0',T+'/lab'),('/home/briansrls/.codex@gunbc-roadmap',T+'/codex'),('/home/briansrls/.config/systemd/user',T+'/root/home/.config/systemd/user'),('/etc/systemd/system',T+'/root/etc/systemd/system'),('/etc/systemd/user',T+'/root/etc/systemd/user'),('/run/systemd/system',T+'/root/run/systemd/system'),('/usr/bin/install','xinstall'),('/usr/bin/mv','xmv'),('/usr/bin/rm','xrm'),('/usr/bin/systemctl','systemctl')]:
    s=s.replace(a,b)
s=re.sub(r'^HEAD_EXPECT=\S+; TREE_EXPECT=\S+','HEAD_EXPECT=%s; TREE_EXPECT=%s'%(H,TR),s,flags=re.M)
s=s.replace('[ "$(id -un)" = briansrls ]','[ "$(id -un)" = %s ]'%getpass.getuser())
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
patch $P/ld449-r2-j1b1.sh $T/jn.sh $T $H $TR "s=s.replace('join_fixture || die2','join_fixture && { echo JOIN_OK; exit 0; } || die2')" "s=re.sub(r'^  (grep -aq .....r|grep -aiE \"|\\[ \"\\\$\\(grep -ac).*die2 \"leg .*\$','  true',s,flags=re.M)" "assert s.count('  true\\n')>=5, 'join-neg: did not neutralize all 5 join-law lines'"
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
cm=re.sub(r'^HEAD_EXPECT=\S+; TREE_EXPECT=\S+','HEAD_EXPECT=%s; TREE_EXPECT=%s'%(H,TR),cm,flags=re.M); cm=re.sub(r'^mapfile -t B0 < .*$','B0=($$)',cm,flags=re.M)
open(sys.argv[2],'w').write('JOB_RT=7200\n'+fn+rg+'R=%s/out\n'%T+cm)
EOF
S1=$(printf 'a%.0s' {1..64}); S2=$(printf 'b%.0s' {1..64}); S3=$(printf 'c%.0s' {1..64}); MAP="J1_GREEN:$S1 J1R_GREEN:$S2 J1B1_GREEN:$S3"
cd $T
chk() { local name=$1 want=$2; shift 2; local o; o="$( ( . ./h.sh; "$@" ) 2>&1 | tail -1 )"; case "$o" in *"$want"*) ok "$name" "$want";; *) bad "$name" "$want" "$o";; esac; }
mk3() { ( . ./h.sh; RUNS_ROOT=$T/runs; for r in J1_GREEN:none:$S1:ld449-r2-j1 J1R_GREEN:J1_GREEN:$S2:ld449-r2-j1r J1B1_GREEN:J1R_GREEN:$S3:ld449-r2-j1b1; do IFS=: read n pr sh job <<< "$r"; SELF_SHA=$sh; rm -f $MK/$n; d=$T/runs/$job/out; rm -rf $d; mkdir -p $d; R=$(readlink -e $d); W=$(readlink -e $WD); files=$(printf '%s\n' "$EXPECTED_RECEIPTS" | awk -F= -v k=$n '$1==k{print $2}' | tr ',' ' '); args=(); for e in $files; do loc=${e%%:*}; rel=${e#*:}; [ "$rel" = ignored-end.txt ] && continue; if [ $loc = W ]; then f=$W/$rel; else f=$R/$rel; fi; mkdir -p $(dirname $f); [ -e $f ] || echo $n-$rel > $f; args+=($f); done; mark $n $pr "${args[@]}" >/dev/null; done ); }
mk3; chk chain-genuine "chain verified to J1" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1B1_GREEN $S3 J1R_GREEN"
echo t >> runs/ld449-r2-j1/out/build.log; chk chain-old-tamper "changed or missing" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1B1_GREEN $S3 J1R_GREEN"; mk3
chk chain-unreviewed "not written by reviewed script" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='J1_GREEN:$S3 J1R_GREEN:$S2 J1B1_GREEN:$S3'; need J1B1_GREEN $S3 J1R_GREEN"
echo z > wd/wt/.ign-x; echo .ign-x > wd/wt/.git/info/exclude; chk chain-ignored "ignored-path set changed" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1B1_GREEN $S3 J1R_GREEN"; rm -f wd/wt/.ign-x
mk3; mkdir -p $T/forgeX; cp runs/ld449-r2-j1/out/* $T/forgeX/ 2>/dev/null; sed -i "s#^OUT .*#OUT $(readlink -e $T/forgeX)#; s#$(readlink -e runs/ld449-r2-j1/out)/#$(readlink -e $T/forgeX)/#" wd/markers/J1_GREEN
chk anchor-foreign-out "not the launcher OUT" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1_GREEN $S1 none"
mk3; mkdir -p runs/ld449-r2-j1/out/deep; mv runs/ld449-r2-j1/out/build.log runs/ld449-r2-j1/out/deep/build.log; sed -i "s#/out/build.log#/out/deep/build.log#" wd/markers/J1_GREEN
chk anchor-moved-receipt "expected locations" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP'; need J1_GREEN $S1 none"; mk3
printf 'HEAD=x\n' > wd/markers/J1B2_GREEN; chk chain-printf-forge "NO OBSERVATION" bash -c ". ./h.sh; RUNS_ROOT=$T/runs; REVIEWED_SCRIPTS='$MAP J1B2_GREEN:$S1'; need J1B2_GREEN $S1 J1B1_GREEN"
chmod 500 wd/markers; chk mark-write-fail "marker write" bash -c ". ./h.sh; SELF_SHA=$S1; echo r > $T/out/SHAS; mark J9 none $T/out/SHAS"; chmod 700 wd/markers
o="$( ( . ./h.sh; B0=($$); mkdir -p cg/job/sub; echo 999 > cg/job/sub/cgroup.procs; chmod 000 cg/job/sub; survivors; echo "rc=$? SURV=[$SURV]" ) 2>&1 | tail -1)"; chmod 755 cg/job/sub; rm -rf cg/job/sub
case "$o" in rc=1*) ok subtree-unreadable "unreadable sub-cgroup dir refuses (not 'no survivors')";; *) bad subtree-unreadable "rc=1" "$o";; esac
# contain: the stub claim_batch starts a setsid'd child and a backgrounded child and records each EXACT pid with its
# /proc starttime in the harness's own-pids file at spawn, then exits 0. The fake cgroup lists ONLY this run's own recorded,
# identity-checked pids (plus the job shell), so contain() can signal nothing else. The postcheck tests those exact pids.
# No global name or UID discovery is used anywhere.
{
  echo '#!/bin/bash'
  echo 'setsid sleep 300 >/dev/null 2>&1 < /dev/null & p1=$!'
  echo "( sleep 301 >/dev/null 2>&1 < /dev/null & echo \$! > $T/p2 ); p2=\$(cat $T/p2)"
  echo "for p in \$p1 \$p2; do echo \"\$p \$(awk '{print \$22}' /proc/\$p/stat)\" >> $OWN; done"
  echo 'sleep 1; echo PASS a; echo PASS b'
  echo "echo '[cost-partition] {\"verdict\":{\"state\":\"Reconciled\",\"residual_nanos\":0,\"tolerance_nanos\":1}}'"
  echo "echo '[process-partition] total_ms=5 a_ms=1'; exit 0"
} > wd/target/release/claim_batch
chmod +x wd/target/release/claim_batch; mkdir -p wd/wt/dag; printf 'test fn a() -> Bool { true }\ntest fn b() -> Bool { true }\n' > wd/wt/dag/e.dag
( while :; do { echo $$; own_alive; } > cg/job/.p; mv cg/job/.p cg/job/cgroup.procs; sleep 0.2; done ) & REG=$!
o="$( ( . ./h.sh; declare -A EXPECT OBS; EXPECT[G.t.a]=PASS; EXPECT[G.t.b]=PASS; DEVIATIONS=0; run_entry G t $T/wd/wt dag/e.dag a,b 0 60 ) 2>&1 | grep -c SURVIVORS )"; kill $REG; wait $REG 2>/dev/null
n=$(wc -l < $OWN); left=$(own_alive | wc -l)
if [ "$o" -ge 1 ] && [ "$n" = 2 ] && [ "$left" = 0 ]; then ok contain "own setsid+background children ($n) killed and gone"; else bad contain "own survivors killed" "seen=$o recorded=$n left=$left"; fi
# contain self-test: the fake cgroup.procs is republished every 0.1 s as [the h.sh shell + its OWN direct children], read from
# (every 0.02 s) /proc/<that shell>/task/*/children (its own children only; no global process discovery). Modes: exact (as a real cgroup
# would list it) -> held; stray (an extra pid 999999 is listed) -> refuses; blind (children omitted) -> refuses.
cstcase() { local mode=$1 want=$2 W o; rm -f $T/hp
  ( while :; do hp=$(cat $T/hp 2>/dev/null); if [ -n "$hp" ]; then { echo $hp; [ $mode != blind ] && cat /proc/$hp/task/*/children 2>/dev/null | tr ' ' '\n' | grep .; [ $mode = stray ] && echo 999999; } > cg/job/.p; mv cg/job/.p cg/job/cgroup.procs; fi; sleep 0.02; done ) & W=$!
  o="$( ( echo $BASHPID > $T/hp; . ./h.sh; B0=($BASHPID); sleep 0.5; contain_selftest; echo CST_RC=$? ) 2>&1 | grep -E 'contain self-test' | tail -1)"; kill $W; wait $W 2>/dev/null; echo $$ > cg/job/cgroup.procs
  case "$want|$o" in held\|*"held on this job"*) ok cst-$mode "self-test holds on an exact listing";; refuse\|*REFUSED_SETUP*) ok cst-$mode "self-test refuses [$o]";; *) bad cst-$mode "$want" "$o";; esac; }
cd $T; cstcase exact held; cstcase stray refuse; cstcase blind refuse
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
[ "$o" = DONE ] && [ -f $T/wd/markers/J2_INSTALLED ] && [ "$(wc -l < $T/reload.log)" = 1 ] && [ ! -e $T/wd/J2_PARTIAL ] && ok j2-happy "J2_INSTALLED, 1 reload" || bad j2-happy "J2_INSTALLED" "$o reloads=$(wc -l < $T/reload.log)"
D=$BASE/j2b/root/etc/systemd/system; T=$BASE/j2b; j2setup $T "mkdir -p $D/multi-user.target.wants && ln -s $D/gunbc-srv1-lab-codex-feedback-v0.service $D/multi-user.target.wants/gunbc-srv1-lab-codex-feedback-v0.service"
SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); o="$(run2 $T $T/out $SU | grep -c 'step=pre-reload')"; [ "$o" -ge 1 ] && [ "$(wc -l < $T/reload.log)" = 0 ] && ok j2-enable "refused pre-reload, 0 reloads" || bad j2-enable "pre-reload refusal" "$o reloads=$(wc -l < $T/reload.log)"
T=$BASE/j2c; j2setup $T "sleep 30"; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); ( cd $T && PATH=$T/bin:$PATH timeout -s KILL 10 bash ./j2.sh $T/out $SU >/dev/null 2>&1 ); o="$(run2 $T $T/out2 $SU | tail -1)"
[ -e $T/wd/J2_PARTIAL ] && case "$o" in *"already ran"*) true;; *) false;; esac && ok j2-kill "partial kept, rerun refused" || bad j2-kill "rerun refused" "$o"
sudocase() { local name=$1 body=$2 T=$BASE/j2s-$1; j2setup $T; local SU; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); printf '%b' "$body" > $T/sudo-ll
  local o; o="$(run2 $T $T/out $SU | grep 'NO OBS')"; case "$o" in *PRE_MUTATION*"no sudoers rule permits"*) [ -z "$(git -C $T/lab/source rev-parse -q --verify refs/gunbc/srv1-lab-admitted)" ] && ok "sudo-$name" "refused pre-mutation" || bad "sudo-$name" "no admit" "admitted";; *) bad "sudo-$name" "per-rule refusal" "$o";; esac; }
H0='User briansrls may run the following commands on host:\n\n'
sudocase wrong-runas "${H0}Sudoers entry: a\n    RunAsUsers: nobody\n    Options: !authenticate\n    Commands:\n\tALL\nSudoers entry: b\n    RunAsUsers: root\n    Commands:\n\t/usr/bin/install\n"
sudocase mixed-tags "${H0}Sudoers entry: a\n    RunAsUsers: root\n    Options: !authenticate\n    Commands:\n\t/usr/bin/ls\nSudoers entry: b\n    RunAsUsers: root\n    Options: authenticate\n    Commands:\n\t/usr/bin/install\n"
sudocase prefix "${H0}Sudoers entry: a\n    RunAsUsers: root\n    Options: !authenticate\n    Commands:\n\t/usr/bin/rmdir\n\t/usr/bin/install\n\t/usr/bin/mv\n\t/usr/bin/systemctl\nSudoers entry: b\n    RunAsUsers: root\n    Commands:\n\t/usr/bin/rm\n"
sudocase other-args "${H0}Sudoers entry: a\n    RunAsUsers: root\n    Options: !authenticate\n    Commands:\n\t/usr/bin/install -m 0600 /a /b\n\t/usr/bin/mv\n\t/usr/bin/systemctl\n\t/usr/bin/rm\nSudoers entry: b\n    RunAsUsers: root\n    Commands:\n\t/usr/bin/install\n"
T=$BASE/j2e; j2setup $T; SU=$(sha256sum < $T/wd/render/serve.unit | cut -d' ' -f1); sed -i 's/batchmode yes/batchmode no/' $T/ssh-g; : > $T/kh; printf '#!/bin/bash\n[ "$1" = -F ] && exit 0; exit 1\n' > $T/bin/ssh-keygen; chmod +x $T/bin/ssh-keygen
o="$(run2 $T $T/out $SU | grep 'NO OBS')"; case "$o" in *PRE_MUTATION*BatchMode*) ok j2-ssh "BatchMode no refused pre-mutation even with a known_hosts entry";; *) bad j2-ssh "ssh readiness refusal" "$o";; esac
# ---- J4: stub systemctl/sudo/ss; fake unit cgroup with a sub-cgroup ----
j4setup() { local T=$1 RT=$2; rm -rf $T; mkdir -p $T; base $T; H=$(git -C $T/wd/wt rev-parse HEAD); TR=$(git -C $T/wd/wt rev-parse HEAD^{tree}); mkdir -p $T/cg/unit/sub
  echo prior > $T/wd/markers/J2_INSTALLED; local U=gunbc-srv1-lab-codex-feedback-v0.service S=$T/state
  cat > $T/bin/systemctl.real <<EOF
#!/bin/bash
S=$S; if [ "\$1" = show ]; then f=\$S/\$2.\$4; [ -f \$f ] && cat \$f || echo; exit 0; fi
case "\$1" in start) echo active > \$S/\$2.ActiveState; echo inv-9 > \$S/\$2.InvocationID; echo /unit > \$S/\$2.ControlGroup; touch \$S/listening;; stop) echo inactive > \$S/\$2.ActiveState; rm -f \$S/listening;; esac; exit 0
EOF
  printf '#!/bin/bash\nexec %s/bin/systemctl.real "$@"\n' $T > $T/bin/systemctl
  printf '#!/bin/bash\nshift; [ "$1" = -ll ] && { printf "User briansrls may run:\\n\\nSudoers entry: lab\\n    RunAsUsers: root\\n    Options: !authenticate\\n    Commands:\\n\\tALL\\n"; exit 0; }; case "$1" in /usr/bin/systemctl|systemctl) shift;; esac; exec systemctl "$@"\n' > $T/bin/sudo
  printf '#!/bin/bash\n[ -f %s/listening ] && echo '"'"'LISTEN 0 4096 0.0.0.0:18081 0.0.0.0:* users:(("gunbc",pid=424242,fd=7))'"'"'; exit 0\n' $S > $T/bin/ss
  chmod +x $T/bin/*; echo loaded > $S/$U.LoadState; echo inactive > $S/$U.ActiveState; echo $T/root/etc/systemd/system/$U > $S/$U.FragmentPath; echo no > $S/$U.NeedDaemonReload; : > $S/$U.DropInPaths; echo 0 > $S/$U.NRestarts
  : > $T/cg/unit/cgroup.procs; echo 424242 > $T/cg/unit/sub/cgroup.procs; : > $T/j3.txt
  for u in belt-exit belt-discovery attempt-unit-drain; do n=gunbc-srv1-lab-codex-feedback-v0-$u; echo active > $S/$n.path.ActiveState; echo enabled > $S/$n.path.UnitFileState; echo pinv-$u > $S/$n.path.InvocationID; echo 1234$u > $S/$n.path.ActiveEnterTimestampMonotonic
    echo "J3_OK $n.path ActiveState=active UnitFileState=enabled Triggers=$n.service InvocationID=pinv-$u ActiveEnterTimestampMonotonic=1234$u" >> $T/j3.txt; done
  patch $P/ld449-r2-j4.sh $T/j4.sh $T $H $TR "s=s.replace('prior_scope_gone ld449-r2-j2; need J2_INSTALLED \"\$J2_SHA\" J1B4_GREEN',':').replace('sha256sum -c --quiet \$WD/INSTALLED_SHAS >/dev/null 2>&1 || die2','true || die2')" "s=s.replace('JOB_END=\$(( T0 + 2400 - 60 ))','JOB_END=\$(( T0 + $RT - 60 ))').replace('JOB_RT=2400','JOB_RT=$RT')"; }
run4() { local T=$1 R3=${2:-$1/j3.txt}; ( cd $T && PATH=$T/bin:$PATH CGROOT=$T/cg timeout 900 bash ./j4.sh $T/out $R3 $(sha256sum < $R3 | cut -d' ' -f1) 2>&1 ); }
T=$BASE/j4a; j4setup $T 2400; o="$(run4 $T | tail -1)"; [ "$o" = DONE ] && [ -f $T/wd/markers/J4_UP ] && ok j4-happy "J4_UP" || bad j4-happy "J4_UP" "$o"
T=$BASE/j4b; j4setup $T 1070; rm -f $T/bin/ss; printf '#!/bin/bash\nexit 0\n' > $T/bin/ss; chmod +x $T/bin/ss; : > $T/cg/unit/sub/cgroup.procs; o="$(run4 $T | grep 'NO OBS')"
case "$o" in *DEADLINE*"verified down"*) ok j4-deadline "deadline, stop verified";; *) bad j4-deadline "deadline + verified stop" "$o";; esac
T=$BASE/j4c; j4setup $T 1070; printf '#!/bin/bash\nexit 0\n' > $T/bin/ss; chmod +x $T/bin/ss; o="$(run4 $T | grep 'NO OBS')"
case "$o" in *UNRESOLVED*) ok j4-subtree "sub-cgroup survivor -> UNRESOLVED";; *) bad j4-subtree "UNRESOLVED" "$o";; esac
T=$BASE/j4d; j4setup $T 1070; printf '#!/bin/bash\n[ -f %s/state/listening ] && { echo '"'"'LISTEN 0 4096 0.0.0.0:18081 0.0.0.0:* users:(("gunbc",pid=424242,fd=7))'"'"'; echo '"'"'LISTEN 0 4096 [::]:18081 [::]:*'"'"'; }; exit 0\n' $T > $T/bin/ss; chmod +x $T/bin/ss
o="$(run4 $T | grep 'NO OBS')"; case "$o" in *"carries no pid"*) ok j4-nopid "pid-less listener refused";; *) bad j4-nopid "refusal" "$o";; esac
T=$BASE/j4e; j4setup $T 2400; sed 's/pinv-belt-exit/pinv-OLD/' $T/j3.txt > $T/j3f.txt; cp $T/j3.txt $T/j3x.txt; echo 'J3_FAILED x' >> $T/j3x.txt
o1="$(run4 $T $T/j3f.txt | grep 'NO OBS')"; rm -rf $T/out; o2="$(run4 $T $T/j3x.txt | grep 'NO OBS')"
case "$o1|$o2" in *"does not equal the live reading"*"|"*"not exactly 3 lines"*) ok j4-receipt "stale and extra-line receipts refused";; *) bad j4-receipt "both refused" "$o1 / $o2";; esac
T=$BASE/j4f; j4setup $T 1120; printf '#!/bin/bash\n[ "$1" = show ] && [ "$(echo "$@" | grep -o " -p " | wc -l)" -gt 3 ] && sleep 600\nexec %s/bin/systemctl.real "$@"\n' $T > $T/bin/systemctl; chmod +x $T/bin/systemctl
S0=$(date +%s); o="$(run4 $T | grep 'NO OBS')"; el=$(( $(date +%s)-S0 )); case "$o" in *"post-start property read"*) [ $el -lt 1060 ] && ok j4-stall "bounded (${el}s < 1060)" || bad j4-stall "bounded" "${el}s";; *) bad j4-stall "post-start read refusal" "$o";; esac
T=$BASE/j4g; j4setup $T 1150
# late listener: the stub ss reports a listener only once the start marker is >= 100 s old (WORK_END is ~130 s here)
printf '#!/bin/bash\nf=%s/state/listening; [ -f $f ] && [ $(( $(date +%%s) - $(stat -c %%Y $f) )) -ge 100 ] && echo '"'"'LISTEN 0 4096 0.0.0.0:18081 0.0.0.0:* users:(("gunbc",pid=424242,fd=7))'"'"'; exit 0\n' $T > $T/bin/ss
# slow post-listen bookkeeping: every git call on the worktree sleeps 50 s (under each cap) before answering
printf '#!/bin/bash\nfor a in "$@"; do [ "$a" = %s/wd/wt ] && sleep 50; done\nexec /usr/bin/git "$@"\n' $T > $T/bin/git
chmod +x $T/bin/ss $T/bin/git
S0=$(date +%s); o="$(run4 $T | grep -E '^DONE|NO OBS' | head -1)"; el=$(( $(date +%s)-S0 ))
case "$o" in DONE|*"serve is UP"*) [ $el -lt 1090 ] && ok j4-late "late listener + slow bookkeeping: outcome [$o] in ${el}s < 1090" || bad j4-late "outcome before JOB_END" "${el}s";; *) bad j4-late "DONE or a serve-UP outcome" "$o";; esac
T=$BASE/fab; mkdir -p $T; base $T; H=$(git -C $T/wd/wt rev-parse HEAD); TR=$(git -C $T/wd/wt rev-parse HEAD^{tree}); echo k > $T/key
patch $P/ld449-r2-j1b1.sh $T/f.sh $T $H $TR "s=s.replace('/etc/gunbc-fabric-storage/state-writer-mac-key','$T/key').replace('join_fixture || die2','true || die2').replace('fresh_own J1B1_GREEN','echo GATE_PASSED; exit 0')"
o1="$(cd $T && bash ./f.sh $T/o1 NOARG 2>&1 | tail -1)"; o2="$(cd $T && bash ./f.sh $T/o2 FABRIC_POSTS_GRANT=msg_12345678-abcd 2>&1 | tail -1)"
case "$o1|$o2" in *"arg 2 must be FABRIC_POSTS_GRANT"*"|GATE_PASSED") ok fabric-gate "readable key without grant arg refuses; with it the gate passes";; *) bad fabric-gate "refuse then pass" "$o1 / $o2";; esac
echo "--- cases done; failures: $FAILS"; [ $FAILS = 0 ]
