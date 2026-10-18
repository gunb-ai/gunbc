cd "$(git rev-parse --show-toplevel)"
M=src/v1/stage0/src/v1_compiler_infer.rs
export CARGO_TARGET_DIR=$PWD/target
python3 - <<'P'
p='src/v1/stage0/src/v1_compiler_infer.rs'
s=open(p).read()
a="""                                Some(fi) => Some(crate::v1_compiler_infer_types::resolved_type(
                                    crate::v1_std_core::arg_value(fi.typed_arg.clone()),
                                )),
                                std::option::Option::None => std::option::Option::None,
                            };
                            let declared_formal_authority_failed"""
assert s.count(a)==1
s=s.replace(a,a.replace("Some(fi) => Some(crate::v1_compiler_infer_types::resolved_type(\n                                    crate::v1_std_core::arg_value(fi.typed_arg.clone()),\n                                )),","Some(_fi) => std::option::Option::None,"))
open(p,'w').write(s)
P
git diff --stat | tail -1
cargo build --release --bin gunbc 2>&1 | tail -1
timeout 2700 target/release/gunbc test //gunbc/instruments:self-host > /tmp/sh_mut.out 2>&1; echo "rc_mut=$?"
grep -o 'closure=[0-9a-f]* binary=[0-9a-f]*' /tmp/sh_mut.out
tail -6 /tmp/sh_mut.out | cut -c1-300
