# np7a: per-function attribution of the N7 normalize cost on H1, H2, K2 (H1 = additive control).
# Uses claim_batch's OWN recompute-trace (GUNBC_RECOMPUTE_TRACE=1): per fn CALL COUNTS and INCLUSIVE ms,
# NOT eval steps (steps exist only per claim). The trace prints a capped top list (20 dup rows, 10
# unkeyed-partial rows per process), so a fn absent from a table is UNKNOWN, never zero.
# One claim_batch PROCESS per (case, stage): the pure-call memo is cross-claim within a process.
# Verdict channel = Filesystem.Write under --wet (np6w lesson); the steps line is the second channel.
# Nothing is fetched, split or rewritten: np5's pieces are read as they are on disk.
# MANIFEST_ONLY=1 stops after preflight.
set -u
export PATH=$HOME/.cargo/bin:$PATH RUSTC_WRAPPER=
B=$HOME/runs/sp1/bin
SRCPIN=799697f5c76b2e86dc34f3408878aef9e0d3d750
cd ~/runs/sp1/wt5 || { echo "REFUSE: no wt5"; exit 2; }
echo HOST=$(hostname)
[ "$(git rev-parse HEAD)" = "$SRCPIN" ] || { echo "REFUSE: wt5 HEAD != $SRCPIN"; exit 2; }
case "$(sha256sum $B/gunbc | cut -c1-8)" in a82bc081) ;; *) echo "REFUSE: gunbc pin"; exit 2;; esac
case "$(sha256sum $B/claim_batch | cut -c1-8)" in 9266fcdc) ;; *) echo "REFUSE: claim_batch pin"; exit 2;; esac
[ "$(sha256sum ~/runs/sp1/np5.full | cut -c1-12)" = "9d2d41c49c00" ] || { echo "REFUSE: np5.full changed"; exit 2; }
bad=0; for f in syn/H1.dag syn/H2.dag syn/K2.dag; do want=$(grep -a "^$f " ~/runs/sp1/np5.sum | head -1 | cut -d' ' -f2); have=$(wc -c < $f); [ "$want" = "$have" ] || { echo "SIZE MISMATCH $f want=$want have=$have"; bad=1; }; done
[ $bad = 0 ] || { echo "REFUSE: pieces differ from np5.sum"; exit 2; }
for f in syn/H1.dag syn/H2.dag syn/K2.dag; do echo "$f $(sha256sum $f | cut -c1-16) $(wc -c < $f)B"; done
echo "SCRIPT_SHA256=$(sha256sum "$0" | cut -d' ' -f1)"
[ "${MANIFEST_ONLY:-0}" = 1 ] && exit 0
OUT=~/runs/sp1/np7a; mkdir -p $OUT np7a_out; rm -f $OUT/* np7a_out/*
P=src/v2/test/claim/shape_probe7a.dag
{
echo "module v2.test.claim.shape_probe7a"
echo "import v2.compiler.normalize { normalize_node, bind_retention_free }"
echo "import v2.compiler.parse { parse_module_prepared }"
echo "import v2.compiler.tokenize { tokenize_prepared }"
echo "import v2.compiler.program_assembly { dag_prepared_grammar, dag_prepared_lex }"
echo "import v2.std.node { symbol_intern_lexeme }"
echo "import v2.std.diagnostic { Accepted, Rejected }"
echo "import std.types { String, Bool }"
echo "import extdeps.filesystem.filesystem_io { Filesystem }"
cat <<'EOT'
fn p7_verdict(path: String, stage: Int) -> String {
  let text = Filesystem.Read(path: path).content
  match dag_prepared_grammar() {
    Rejected { diagnostics: _ } => "code=-1 stage=grammar"
    Accepted { value: prepared, diagnostics: _ } =>
      match tokenize_prepared(text: text, file: symbol_intern_lexeme(lexeme: "p.dag"), prepared: dag_prepared_lex()) {
        Rejected { diagnostics: _ } => "code=-2 stage=tokenize"
        Accepted { value: stream, diagnostics: _ } =>
          match parse_module_prepared(tokens: stream, prepared: prepared) {
            Rejected { diagnostics: _ } => "code=-3 stage=parse"
            Accepted { value: artifact, diagnostics: _ } =>
              if stage == 1 { "code=1 stage=accepted-parse" } else {
                match bind_retention_free(o: normalize_node(n: artifact.tree)) {
                  Rejected { diagnostics: _ } => "code=-4 stage=normalize"
                  Accepted { value: _, diagnostics: _ } => "code=1 stage=accepted-normalize"
                }
              }
          }
      }
  }
}
fn p7_run(path: String, stage: Int, out: String) -> Bool {
  Filesystem.Write(path: out, content: p7_verdict(path: path, stage: stage)).success
}
EOT
for f in H1 H2 K2; do for s in 1 2; do echo "test fn ${f}_s$s() -> Bool { p7_run(path: \"syn/$f.dag\", stage: $s, out: \"np7a_out/${f}_s$s.txt\") }"; done; done
} > $P
# order: smallest first so a stop leaves the useful pairs; each process has a hard 40 min cap
for c in H1_s1 H1_s2 H2_s1 H2_s2 K2_s1 K2_s2; do
  GUNBC_RECOMPUTE_TRACE=1 timeout 2400 $B/claim_batch --claim-run --wet --source-root dag --source-root src/v2 --entry $P --functions $c > $OUT/$c.full 2>&1
  echo "$c rc=$? verdict=$(cat np7a_out/$c.txt 2>/dev/null || echo NO-OUTPUT) steps=$(grep -a "witness\] $c:" $OUT/$c.full | sed -n 's/.*eval_steps=\([0-9]*\).*/\1/p')"
done
echo "== CONTROL (np6w, same pins, trace off): H1_s1=4241654 H1_s2=9509435 H2_s1=8641541 H2_s2=36481895 K2_s1=12853364 K2_s2=86375340 ; a differing steps= above means the trace PERTURBS steps and the rows are not attributable to the np6w numbers"
echo "== ROWS per process (fn, calls, inclusive_ms, site); capped top lists, absent = UNKNOWN"
for c in H1_s1 H1_s2 H2_s1 H2_s2 K2_s1 K2_s2; do
  grep -a "^\[recompute-trace\] \(dup\|partial\|keyed_calls\|gap\)" $OUT/$c.full | sed "s/^\[recompute-trace\] /$c /" > $OUT/$c.rows
  echo "$c rows=$(wc -l < $OUT/$c.rows)"
done
echo "== INCREMENT per fn = s2 - s1 on calls, for H1, H2, K2 (only fns present in BOTH tables; '-' = absent in a table)"
for k in H1 H2 K2; do
  echo "-- $k"
  for st in 1 2; do grep -a " \(dup\|partial\) " $OUT/${k}_s$st.rows | sed -n 's/^[A-Z0-9_]* \(dup\|partial\) fn=\([^ ]*\) \(count\|calls\)=\([0-9]*\) total_ms=\([0-9]*\).* @\([^ ]*\).*/\2 \6 \4 \5/p' > $OUT/${k}_s$st.fn; done
  awk 'NR==FNR{c1[$1" "$2]=$3; m1[$1" "$2]=$4; next} {key=$1" "$2; printf "%s s2_calls=%s s1_calls=%s incr_calls=%s s2_ms=%s s1_ms=%s\n",key,$3,(key in c1)?c1[key]:"-",(key in c1)?$3-c1[key]:"-",$4,(key in m1)?m1[key]:"-"}' $OUT/${k}_s1.fn $OUT/${k}_s2.fn | sort -t= -k4 -nr | head -40
done
echo "RAW: every process's full output is in $OUT/ (keep it; the table above is a view)"
