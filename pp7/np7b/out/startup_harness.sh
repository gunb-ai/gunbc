#!/bin/bash
# Bash-only harness: runs the ACTUAL np7b.sh (top-level startup + leg()) with git, cargo and claim_batch STUBBED and the cgroup gate pointed at a fake cgroup dir. No gunbc/claim_batch/cargo is run.
set -u
SRC="$1"; W="$(mktemp -d)"; trap '[ -n "${KEEP:-}" ] || rm -rf "$W"' EXIT
eval "$(sed -n '25,26p' "$SRC")"; [ -n "$PIN_P$TREE_P$PIN_A$TREE_A$BLOB_PROBE$BLOB_IPMI" ] && [ "$(sed -n 25p "$SRC" | cut -c1-6)" = PIN_P= ] || { echo HARNESS_PIN_FAIL; exit 9; }
mkdir -p "$W/cg" "$W/bin"
for kv in "memory.max 34359738368" "memory.high 30064771072" "memory.swap.max 0" "pids.max 4096" "cpu.max 3200000 100000"; do set -- $kv; f=$1; shift; echo "$*" > "$W/cg/$f"; done
printf 'oom 0\noom_kill 0\n' > "$W/cg/memory.events"; echo 1 > "$W/cg/memory.peak"; echo 0 > "$W/cg/memory.swap.current"
# harness copy: ONLY the two cgroup-discovery lines are replaced (fake CG); everything else, incl. startup() and leg(), is the real text
awk -v cg="$W/cg" '/^\[ "\$\(grep -c .. \/proc\/self\/cgroup\)" = 1 \]/{print "# (harness: cgroup discovery replaced)"; next} /^CG="\/sys\/fs\/cgroup/{print "CG=\"" cg "\""; next} {print}' "$SRC" > "$W/np7b_h.sh"
[ "$(grep -c 'harness: cgroup discovery replaced' "$W/np7b_h.sh")" = 1 ] && [ "$(grep -c "^CG=\"$W/cg\"" "$W/np7b_h.sh")" = 1 ] || { echo "HARNESS_SETUP_FAIL"; exit 9; }
cat > "$W/bin/git" <<GS
#!/bin/bash
L() { echo "\$*" >> "$W/gitlog"; }
case "\$*" in
 "init -q wt") L "\$*"; mkdir -p wt;;
 fetch*|checkout*) L "\$*";;
 "rev-parse HEAD") L "\$*"; echo $PIN_P;; "rev-parse HEAD^{tree}") L "\$*"; echo $TREE_P;; "rev-parse HEAD^1") echo $PIN_A;; "rev-parse $PIN_A^{tree}") echo $TREE_A;;
  "cat-file -e $PIN_A^{commit}"|"cat-file -e $PIN_A^{tree}") L "\$*";;
 diff*) L "\$*"; echo src/v2/test/claim/shape_probe7b.dag;;
 "hash-object src/v2/test/claim/shape_probe7b.dag") echo $BLOB_PROBE;; "hash-object dag/extdeps/bmc/ipmi.dag") echo $BLOB_IPMI;;
 "-C . status --porcelain") L "\$*";;
 *) exec /usr/bin/git "\$@";;
esac
GS
cat > "$W/bin/cargo" <<GC
#!/bin/bash
echo "cargo \$*" >> "$W/gitlog"
mkdir -p "\$CARGO_TARGET_DIR/release"
cat > "\$CARGO_TARGET_DIR/release/claim_batch" <<'CB'
#!/bin/bash
f="\${@: -1}"
case "\$f" in IPMI_s1) s=accepted-tokenize;; IPMI_s2) s=accepted-parse;; IPMI_s3) s=accepted-normalize;; IPMI_s4) s="accepted-collect sites=7";; esac
echo "FAIL \$f (returned ProcessExit::ExitFailure, code 3: P7B code=1 stage=\$s)"
echo "[process-partition] total_ms=1000 pre_run_ms=0 args_ms=0"
exit 1
CB
chmod +x "\$CARGO_TARGET_DIR/release/claim_batch"
GC
chmod +x "$W/bin/git" "$W/bin/cargo"
run() { # NAME args... ; runs in fresh cwd, env R poisoned to prove it is not read
  local n="$1"; shift; mkdir -p "$W/cwd-$n"; : > "$W/gitlog"
  ( cd "$W/cwd-$n" && PATH="$W/bin:$PATH" R="$W/poisoned" bash "$W/np7b_h.sh" "$@" > "$W/out-$n" 2>&1 ); echo $? > "$W/rc-$n"
}
fail=0; ck() { if eval "$2"; then echo "ok   $1"; else echo "FAIL $1"; fail=1; fi; }
refuse() { ck "$1: exit 2, REFUSED_SETUP, no git/cargo invoked" "[ \"\$(cat $W/rc-$1)\" = 2 ] && grep -q 'REFUSED_SETUP' $W/out-$1 && [ ! -s $W/gitlog ]"; }
run noargs; refuse noargs
run twoargs a b; refuse twoargs
run emptyarg ""; refuse emptyarg
mkdir "$W/exdir"; run existdir "$W/exdir"; refuse existdir; ck "existdir untouched (still empty)" "[ -z \"\$(ls -A $W/exdir)\" ]"
: > "$W/exfile"; run existfile "$W/exfile"; refuse existfile
ln -s "$W/nowhere" "$W/dangle"; run dangling "$W/dangle"; refuse dangling; ck "dangling symlink still a symlink, target not created" "[ -L $W/dangle ] && [ ! -e $W/nowhere ]"
run fresh "$W/FRESH"
ck "fresh: exit 0 and DONE" "[ \"\$(cat $W/rc-fresh)\" = 0 ] && grep -q '^DONE' $W/out-fresh"
ck "fresh: wt/ under OUT" "[ -d $W/FRESH/wt ]"
ck "fresh: oom-raw.log under OUT" "[ -s $W/FRESH/oom-raw.log ]"
ck "fresh: git init was the first git call" "[ \"\$(head -1 $W/gitlog)\" = 'init -q wt' ]"
ck "fresh: four CASE lines" "[ \$(grep -c '^CASE IPMI_s' $W/out-fresh) = 4 ]"
ck "env R was NOT used (poisoned path never created)" "[ ! -e $W/poisoned ]"
ck "nothing but OUT created in fresh cwd" "[ -z \"\$(ls -A $W/cwd-fresh)\" ]"
[ $fail = 0 ] && echo "STARTUP_HARNESS: all held" || { echo "STARTUP_HARNESS: FAILURES"; exit 1; }
