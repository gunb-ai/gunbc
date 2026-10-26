set -u
cargo build --release -p v1-compiler --bin claim_executor --bin gunbc 2>&1 | grep -E "^error|Finished" | head -3
G=./target/release/gunbc
probe() { tag=$1; for e in kci_kernel kci_foreign; do echo "== $tag $e"; timeout 900 $G compile --source-root dag --entry dag/test/probe/$e.dag --output-dir /tmp/o_${tag}_${e} --target rust 2>&1 | grep -iE "error|refus|mismatch|crossing" | cut -c1-400 | head -8; done; }
selfhost() { tag=$1; rm -rf /tmp/sh_$tag; timeout 1500 $G compile --source-root dag --source-root src/v2 --entry src/v2/compiler/source_root_eval.dag --target rust --output-dir /tmp/sh_$tag > /tmp/sh_$tag.log 2>&1; echo "selfhost $tag exit=$? lines=$(wc -l < /tmp/sh_$tag.log)"; grep -E "blocking|^error" /tmp/sh_$tag.log | grep -v "^\[" | sort | uniq -c | head -8; grep -E "(\[E|error\[|refus)" /tmp/sh_$tag.log | grep -v "^\[pre" | sed 's/[0-9]\+ms//g' | sort > /tmp/sh_$tag.diags; ls /tmp/sh_$tag 2>/dev/null | head -3; }
probe base
selfhost base
python3 - <<'PY'
p='dag/std/literal_elaboration.dag'; s=open(p).read()
s=s.replace("import std.integer { Signedness, Signed, Unsigned }\n","",1)
s=s.replace("import std.measure { BitWidth, bit_width, bit_width_count }\n","",1)
s=s.replace("import std.machine_width_range { machine_width_admits }\n","",1)
s=s.replace("FixedWidthExtent { signedness: s, bits: b } => machine_width_admits(signedness: s, bits: b, value: value)","FixedWidthExtent { signedness: s, bits: b } => signedness_is_signed(s: s) || value >= 0",1)
s+="""
type Signedness = Signed | Unsigned
type BitWidth { count: Int }
fn bit_width(count: Int) -> BitWidth { BitWidth { count: count } }
fn bit_width_count(b: BitWidth) -> Int { b.count }
"""
open(p,'w').write(s)
PY
git status --short src/v1 | grep -v "src/v1/stage0/src/" | head
for gen in 1 2; do
  ./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | grep -E "first_generation_equal=|refus|Refus|error|->|edge" | head -12
  cp target/stage0-regen-candidate/src/*.rs src/v1/stage0/src/ 2>/dev/null
  cargo build --release -p v1-compiler --bin gunbc --bin claim_executor 2>&1 | grep -E "^error|Finished" -A6 | head -30
done
git status --short src/v1/stage0 | head -20
probe new
selfhost new
echo "== selfhost diag diff (base vs new)"
diff /tmp/sh_base.diags /tmp/sh_new.diags | head -60; diff <(grep -vE "\[pre-entry\]|done in" /tmp/sh_base.log) <(grep -vE "\[pre-entry\]|done in" /tmp/sh_new.log) | head -60
echo "== selfhost emitted diff"
diff -rq /tmp/sh_base /tmp/sh_new | head -20
echo "== claims"
for w in dag/test/claim/emitter_nat_int_cast_witness.dag dag/test/claim/nominal_cast_wall_witness.dag dag/test/claim/refinement_cast_wall_witness.dag; do
  echo "== $w"; timeout 900 $G run --claim-run --source-root dag --source-root src/v2 --entry $w 2>&1 | grep -E "^PASS|^FAIL|^error" -A12 | head -60; done
