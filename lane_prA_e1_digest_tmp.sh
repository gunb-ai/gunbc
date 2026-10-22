#!/bin/bash
# Scratch (never lands; mirrors untracked): prA lane E1 = DIGEST PROOF, re-run at the
# assembled head with the manager-mandated loud skip counter (fail-open silent skip let
# the two #14559-broken grammar files vanish from census jc1a7cd3c; #14608 makes them
# parseable, so this run must report SKIPPED 0 over the FULL corpus at both bindings).
# Method faithful to jc1a7cd3c: same corpus, same bin, only the prefix binding differs:
# new = mirror as regenerated (reads declared 18); old = same tree with the two data
# rows' binding_power patched 18 -> 12 (measurement only, restored after).
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }
ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
git clean -x -d --force >/dev/null

echo "=== HEAD ==="
git rev-parse --short HEAD; git branch --show-current

echo "=== BUILD bin (new binding: mirror reads declared 18) ==="
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1
./target/release/unary_prefix_digest dag src > /tmp/digest_new.txt 2>/tmp/summary_new.txt
echo "NEW: $(cat /tmp/summary_new.txt)"
grep -c "^SKIPPED" /tmp/digest_new.txt | sed 's/^/NEW skipped-lines: /'

echo "=== PATCH binding 18 -> 12 (old binding; measurement only) ==="
python3 - <<'PY'
import re
hits = 0
for p in ["src/v1/stage0/src/extdeps_languages_dag_syntax.rs",
          "src/v1/stage0/src/extdeps_languages_rust_syntax.rs"]:
    s = open(p).read()
    n = s.count('\\"binding_power\\": 18')
    s2 = s.replace('\\"binding_power\\": 18', '\\"binding_power\\": 12')
    open(p, "w").write(s2)
    hits += n
    print(f"{p}: patched {n}")
assert hits == 4, f"expected 4 prefix rows (2 per language), got {hits}"
PY

echo "=== BUILD bin (old binding 12) ==="
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -1
./target/release/unary_prefix_digest dag src > /tmp/digest_old.txt 2>/tmp/summary_old.txt
echo "OLD: $(cat /tmp/summary_old.txt)"
grep -c "^SKIPPED" /tmp/digest_old.txt | sed 's/^/OLD skipped-lines: /'

echo "=== RESTORE ==="
git checkout -- src/v1/stage0/src/extdeps_languages_dag_syntax.rs src/v1/stage0/src/extdeps_languages_rust_syntax.rs && echo restored
git status --porcelain | grep -v "^??" | head -3

echo "=== DIFF (expect ONLY unary_prefix_values.dag fixture items; non-fixture MUST be 0) ==="
diff /tmp/digest_old.txt /tmp/digest_new.txt > /tmp/digest_diff.txt || true
grep -cE "^[<>]" /tmp/digest_diff.txt | sed 's/^/diff-lines: /'
grep -E "^[<>]" /tmp/digest_diff.txt | grep -vc "unary_prefix_values\.dag" | sed 's/^/non-fixture diff-lines: /'
grep -E "^[<>]" /tmp/digest_diff.txt | head -30
