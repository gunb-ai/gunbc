#!/bin/bash
# Lane adhoc-9102e2d6-92f scratch (never lands): re-spell proof.
# Canonical tree digests over the RE-SPELLED corpus: pristine mirror (old binding 12) vs the same
# mirror with ONLY the two prefix arms patched 12->18 (measurement only; never committed).
# The diff must contain ONLY this lane's own fixture items (the values claim's RED arms), which
# proves every re-spell reproduced the exact tree the corpus has today: no expression changes meaning.
set -uo pipefail
MEMTOTAL_KB=$(awk "/MemTotal/{print \$2}" /proc/meminfo)
[ "$MEMTOTAL_KB" -ge 60000000 ] || { echo "runner memory too low: $MEMTOTAL_KB kB"; exit 3; }

echo "=== BUILD A (pristine mirror = old binding 12) ==="
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -2
./target/release/unary_prefix_digest dag src > /tmp/digest_old.txt
echo "items_a $(wc -l < /tmp/digest_old.txt)"

echo "=== PATCH mirror prefix arms 12 -> 18 (measurement only) ==="
python3 - <<'PY'
p = "src/v1/stage0/src/v1_compiler_parse.rs"
s = open(p).read()
pat = "let r = parse_expr_bp(token_stream_advance(tokens.clone(), 1), ctx.clone(), 12);"
n = s.count(pat)
assert n == 2, f"expected 2 prefix arms, found {n}"
s = s.replace(pat, pat.replace(" 12);", " 18);"))
open(p, "w").write(s)
print("patched 2 arms")
PY

echo "=== BUILD B (binding = the declared value) ==="
cargo build --release -p v1-compiler --bin unary_prefix_digest 2>&1 | tail -2
./target/release/unary_prefix_digest dag src > /tmp/digest_new.txt
echo "items_b $(wc -l < /tmp/digest_new.txt)"

echo "=== RESTORE mirror ==="
git checkout -- src/v1/stage0/src/v1_compiler_parse.rs && echo restored

echo "=== DIFF (want ONLY unary_prefix_values.dag fixture items) ==="
diff /tmp/digest_old.txt /tmp/digest_new.txt | grep -E "^[<>]" || true
echo "diff-lines $(diff /tmp/digest_old.txt /tmp/digest_new.txt | grep -cE '^[<>]' || true)"
echo "non-fixture diff-lines: $(diff /tmp/digest_old.txt /tmp/digest_new.txt | grep -E '^[<>]' | grep -vc 'unary_prefix_values\.dag' || true)"
