import sys
p=sys.argv[1]; s=open(p).read()
anchors=['data canonical_apply_source: String = "module probe_canonical_apply\\n',
         'data canonical_approval_routes_source: String = "module probe_canonical_approval_routes\\n',
         'data call_duplicate_green_source: String = "module probe_call_duplicate_green\\n']
for anchor in anchors:
    if s.count(anchor)==1:
        open(p,'w').write(s.replace(anchor, anchor+'fn broken() -> Int { let s = \\"abc\\n'))
        print("  plant unterm applied at", anchor[:56]); sys.exit(0)
print("  ANCHOR_NOT_FOUND -- STOP"); sys.exit(1)
