cargo build --release --bin claim_batch 2>&1 | tail -1
MODS="callexec/let_match_early_return_eval callexec/named_call_execution callexec/list_introduction_eval body_let_annotation optional_match_denotation manual/match_bool_emit compiler/infer_bind_match_schedule_first compiler/infer_payload_binder bash_program_fold target_model_survivor_witness"
run() { for m in $MODS; do f=src/v2/test/claim/$m.dag; fn=$(grep -ho '^test fn [A-Za-z_0-9]*' $f | sed 's/test fn //' | paste -sd,); ./target/release/claim_batch --source-root dag --source-root src/v2 --entry $f --functions $fn 2>&1 | grep -aE "^PASS|^FAIL|resolve failed|error:" | sed "s|^|$1 $m |"; done; }
run base
cp src/v2/std/node.dag /tmp/node.orig
python3 - <<'P'
p='src/v2/std/node.dag'; s=open(p).read()
s=s.replace("  } else if ordinal == 1 {\n    BindValueSlot\n  } else {\n    BindBodySlot\n  }","  } else if ordinal == 1 {\n    BindBodySlot\n  } else {\n    BindValueSlot\n  }",1)
open(p,'w').write(s)
P
run bindswap
cp /tmp/node.orig src/v2/std/node.dag
python3 - <<'P'
p='src/v2/std/node.dag'; s=open(p).read()
s=s.replace("  if ordinal == 0 {\n    MatchScrutineeSlot\n  } else {\n    MatchArmSlot\n  }","  if ordinal == 0 {\n    MatchArmSlot\n  } else {\n    MatchScrutineeSlot\n  }",1)
open(p,'w').write(s)
P
run matchswap
echo done
