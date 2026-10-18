set -e
cd "$(git rev-parse --show-toplevel)"
python3 - <<'P'
p='src/v1/stage0/src/v1_compiler_infer.rs'
s=open(p).read()
a="""                    } else {
                        Rc::new(vec![inference_error(
                            v1_rt::concat(
                                v1_rt::concat(
                                    v1_rt::concat(
                                        "if branches resolve to incompatible types: ".to_string(),"""
assert s.count(a)==1
b=a.replace("} else {\n","} else {\n                        eprintln!(\"QE_IFJ then={:?}\\n QE_IFJ else={:?}\\n QE_IFJ then_infer={:?} else_infer={:?}\", then_rt, else_rt, then_typed.inferred, else_typed.inferred);\n",1)
open(p,'w').write(s.replace(a,b))
P
cargo build --release --bin claim_executor 2>&1 | tail -1
set +e
target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/regen.out 2>&1
grep -a -A3 QE_IFJ /tmp/regen.out | cut -c1-3000 | head -40
