set -u
export CARGO_TARGET_DIR=$PWD/target RUSTC_WRAPPER=
echo "HEAD $(git rev-parse HEAD)"
cargo build -q --release -p v1-compiler --bin gunbc --bin claim_executor || { echo BUILD0-FAIL; exit 2; }
rm -rf target/stage0-regen-candidate
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > regen1.log 2>&1; echo "regen1 exit=$?"; grep -E 'FAIL|first_generation_equal|refused' regen1.log | cut -c1-220 | head -5
[ -d target/stage0-regen-candidate/src ] || { echo NO-CANDIDATE; tail -30 regen1.log; exit 2; }
n=0; while IFS= read -r f; do rel=${f#target/stage0-regen-candidate/src/}; if ! cmp -s "$f" "src/v1/stage0/src/$rel"; then cp "$f" "src/v1/stage0/src/$rel"; echo "  installed $rel"; n=$((n+1)); fi; done < <(find target/stage0-regen-candidate/src -type f | sort)
echo "installed $n; census text in mirror: $(grep -c TOSTRCENSUS src/v1/stage0/src/v1_compiler_infer.rs)"
cargo build -q --release -p v1-compiler --bin gunbc || { echo BUILD1-FAIL; exit 2; }
ok_all=1
ctl() { c=$1; exp=$2; body=$3; d=target/tsc_$c; rm -rf $d; mkdir -p $d
  printf 'module tsc_%s\nimport std.types { String, Int, Bool, List, Float }\nimport std.nat { Nat }\n%s\nfn main() -> String { "x" }\n' $c "$body" > $d/p.dag
  ./target/release/gunbc run --source-root dag --source-root src/v2 --source-root $d --entry $d/p.dag > $d/out.log 2>&1
  got=$(grep -oE 'TOSTRCENSUS [^(]*' $d/out.log | sed 's/ *$//' | sort -u | tr '\n' ';')
  if [ "$exp" = NONE ]; then [ -z "$got" ] && r=ok || { r=BAD; ok_all=0; }; elif echo "$got" | grep -qF -- "$exp"; then r=ok; else r=BAD; ok_all=0; fi
  echo "control $c expect='$exp' got='$got' $r"; [ $r = BAD ] && grep -E 'error' $d/out.log | head -3 | cut -c1-200
}
ctl int 'kernel:Int' 'fn f(n: Int) -> String { to_string(value: n) }'
ctl pos 'kernel:Int' 'fn f(n: Int) -> String { to_string(n) }'
ctl nat 'std.nat.Nat' 'fn f(n: Nat) -> String { to_string(value: n) }'
ctl bool 'kernel:Bool' 'fn f(b: Bool) -> String { to_string(value: b) }'
ctl str 'kernel:String' 'fn f(s: String) -> String { to_string(value: s) }'
ctl flt 'kernel:Float' 'fn f(x: Float) -> String { to_string(value: x) }'
ctl rec 'decl:tsc_rec.P' 'type P { value: Int }
fn f(p: P) -> String { to_string(value: p) }'
ctl opt 'card=opt' 'fn f(o: Int?) -> String { to_string(value: o) }'
ctl lam 'binder=lambda-param' 'fn f(xs: List<Int>) -> List<String> { xs |> map(n => to_string(value: n)) }'
ctl meth 'TOSTRCENSUS method' 'fn f(n: Int) -> String { n.to_string() }'
ctl match 'binder=match-binder' 'type W = A { n: Int } | B
fn f(w: W) -> String { match w { A { n: k } => to_string(value: k)  B => "b" } }'
ctl lit 'binder=expr' 'fn f() -> String { to_string(value: 42) }'
ctl decl 'NONE' 'fn to_string(value: Int) -> String { "x" }
fn f(n: Int) -> String { to_string(value: n) }'
echo "controls all_ok=$ok_all"
