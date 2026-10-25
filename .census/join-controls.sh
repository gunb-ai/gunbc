set -u
export CARGO_TARGET_DIR=$PWD/target RUSTC_WRAPPER=
echo "JOIN-CTL instrument"
cargo build -q --release -p v1-compiler --bin gunbc --bin claim_executor || { echo BUILD0-FAIL; exit 2; }
rm -rf target/stage0-regen-candidate
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > regen1.log 2>&1; echo "regen1 exit=$?"; grep -E 'FAIL generated|refused' regen1.log | cut -c1-200 | head -3
[ -d target/stage0-regen-candidate/src ] || { echo NO-CANDIDATE; tail -20 regen1.log; exit 2; }
n=0; while IFS= read -r f; do rel=${f#target/stage0-regen-candidate/src/}; if ! cmp -s "$f" "src/v1/stage0/src/$rel"; then cp "$f" "src/v1/stage0/src/$rel"; echo "  installed $rel"; n=$((n+1)); fi; done < <(find target/stage0-regen-candidate/src -type f | sort)
echo "installed $n; census text in mirror: $(grep -c JOINCENSUS src/v1/stage0/src/v1_compiler_infer.rs)"
cargo build -q --release -p v1-compiler --bin gunbc || { echo BUILD1-FAIL; exit 2; }
ok_all=1
ctl() { c=$1; exp=$2; body=$3; d=target/jc_$c; rm -rf $d; mkdir -p $d
  printf 'module jc_%s\nimport std.types { String, Int, Bool, List, NonEmptyStr }\n%s\nfn main() -> String { "x" }\n' $c "$body" > $d/p.dag
  ./target/release/gunbc run --source-root dag --source-root src/v2 --source-root $d --entry $d/p.dag > $d/out.log 2>&1
  got=$(grep -oE 'JOINCENSUS [^(]*' $d/out.log | sed 's/ *$//' | sort -u | tr '\n' ';')
  if [ "$exp" = NONE ]; then [ -z "$got" ] && r=ok || { r=BAD; ok_all=0; }; elif echo "$got" | grep -qF -- "$exp"; then r=ok; else r=BAD; ok_all=0; fi
  echo "control $c expect='$exp' got='$got' $r"; [ $r = BAD ] && grep -E 'error' $d/out.log | head -3 | cut -c1-200
}
ctl str 'JOINCENSUS bare elem=kernel:String:mint/req sep=kernel:String:mint/req' 'fn f(xs: List<String>) -> String { join(xs, ",") }'
ctl lit 'elem=kernel:String' 'fn f() -> String { join(["a", "b"], ",") }'
ctl pipe 'JOINCENSUS method elem=kernel:String' 'fn f(xs: List<String>) -> String { xs |> join(",") }'
ctl int 'elem=kernel:Int:mint' 'fn f(ns: List<Int>) -> String { join(ns, ",") }'
ctl nes 'elem=decl:std.types.NonEmptyStr/req sep=decl:std.types.NonEmptyStr' 'fn f(xs: List<NonEmptyStr>, s: NonEmptyStr) -> String { join(xs, s) }'
ctl rec 'elem=decl:jc_rec.P' 'type P { value: Int }
fn f(ps: List<P>) -> String { join(ps, ",") }'
ctl opt '/opt' 'fn f(xs: List<String?>) -> String { join(xs, ",") }'
ctl lam 'binder=lambda-param' 'fn f(xss: List<List<String>>) -> List<String> { xss |> map(xs => join(xs, ",")) }'
ctl local 'NONE' 'fn combine(join: fn(String, String) -> String, a: String) -> String { join(a, a) }'
echo "controls all_ok=$ok_all"
