import re
p='/home/briansrls/.worktrees/gunbc/royal-newt-820/src/v1/stage0/src/bin/infer_semantics_witness.rs'
s=open(p).read()
s=s.replace("node_type_compatible(user_id.clone(), account_id, result.source_indices.clone())","node_type_compatible(user_id.clone(), account_id, result.source_indices.clone(), Rc::new(v1_compiler::v1_compiler_infer_types::TextJudgment::TextJudgedIn { env: module.type_env.clone() }))")
s=s.replace("node_type_compatible(user_id.clone(), user_id, result.source_indices.clone())","node_type_compatible(user_id.clone(), user_id, result.source_indices.clone(), Rc::new(v1_compiler::v1_compiler_infer_types::TextJudgment::TextJudgedIn { env: module.type_env.clone() }))")
s=re.sub(r"(check_(?:index|slice)_access_node\((?:[^;]*?)empty_source_indices\(\),)(\s*)\)", r"\1\2    Rc::new(v1_compiler::v1_compiler_infer_types::TextJudgment::TextNotAsked { reason: v1_compiler::v1_compiler_infer_types::TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes }),\2)", s)
s=re.sub(r"node_type_compatible\((list_sym, fm_sym|list_int, fm_string|fm_sym, list_sym), empty_source_indices\(\)\)", r"node_type_compatible(\1, empty_source_indices(), Rc::new(v1_compiler::v1_compiler_infer_types::TextJudgment::TextNotAsked { reason: v1_compiler::v1_compiler_infer_types::TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes }))", s)
open(p,'w').write(s)
s=open(p).read()
J='Rc::new(v1_compiler::v1_compiler_infer_types::TextJudgment::TextJudgedIn { env: module.type_env.clone() })'
N='Rc::new(v1_compiler::v1_compiler_infer_types::TextJudgment::TextNotAsked { reason: v1_compiler::v1_compiler_infer_types::TextNotAskedReason::TextNotAskedForSyntheticWitnessNodes })'
s=s.replace('Some(module.type_env.clone())',J)
s=re.sub(r"(empty_source_indices\(\),\s*)None(,?\s*\))", lambda m: m.group(1)+N+m.group(2), s)
open(p,'w').write(s)
import subprocess; subprocess.run(['rustfmt','--edition','2021',p])
