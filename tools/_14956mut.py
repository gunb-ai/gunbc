#!/usr/bin/env python3
# #14956 plant: repoint Blocking emitted_binary (tree M) and AuditOnly vacuity (tree V) consumer
# witnesses at a file that does not exist anywhere in-tree. Fails loudly if anchors are off.
import sys, os
gone_entry = 'entry: "src/v2/test/claim/gone_14956_probe.dag",'
gone_fn    = 'function: "gone_14956_probe_holds",'
def repoint(path, block, old_pair, gone_pair):
    src = open(path).read()
    if gone_entry in src:
        print("ALREADY_MUTATED %s (idempotent no-op)" % path); return
    lines = src.split("\n")
    i = next(k for k,l in enumerate(lines) if l.startswith(block))
    j = next(k for k in range(i,len(lines)) if lines[k]=="}")
    blk = "\n".join(lines[i:j+1])
    assert blk.count(old_pair)==1, "anchor count %d in %s -- STOP" % (blk.count(old_pair), path)
    blk2 = blk.replace(old_pair, gone_pair)
    lines[i:j+1] = blk2.split("\n")
    open(path,"w").write("\n".join(lines))
    print("MUTATED %s" % path)
    print(blk2)
EB = "data lens_contract_emitted_binary_effect_confinement: LensContract"
EBP = 'entry: "src/v2/test/claim/emitted_binary_effect_confinement_red_control.dag",\n      function: "emitted_binary_effect_confinement_authority_points_at_host_transport",'
VC = "data lens_contract_vacuity: LensContract"
VCP = 'entry: "src/v2/test/claim/long/vacuity_consumer_witness_long.dag",\n      function: "vacuity_consumer_witness_real_test_fn_gap_stays_unknown_by_execution",'
M, V = sys.argv[1], sys.argv[2]
repoint(os.path.join(M,"src/v2/lens/enforcement/contract.dag"), EB, EBP, gone_entry+"\n      "+gone_fn)
repoint(os.path.join(V,"src/v2/lens/enforcement/contract.dag"), VC, VCP, gone_entry+"\n      "+gone_fn)
for T in (M,V):
    assert not os.path.exists(os.path.join(T,"src/v2/test/claim/gone_14956_probe.dag")), "plant target exists "+T
print("PLANT_TARGET_TRULY_ABSENT both")
