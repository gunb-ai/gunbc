LIVE-body re-verification of the posted bytes (run from this repo, gh api against the comment bodies, stated extraction command verbatim):

```
$ gh api repos/gunb-ai/gunbc/issues/comments/6692850236 -q .body | awk '/^~~~$/{f=!f;next} f' > p.b64
$ base64 -d p.b64 | gunzip > packet-evidence-v15.sh
packet-evidence-v15.sh: decode rc=0 sha=86ec8c12ca4a976512fd2c11ed11a7adca6264b600a7b7c1bb28914a833fb83a want=86ec8c12ca4a976512fd2c11ed11a7adca6264b600a7b7c1bb28914a833fb83a size=101008 want=101008 decode_err=0
cmp vs owner copy: BYTE-IDENTICAL-PACKET
$ gh api repos/gunb-ai/gunbc/issues/comments/6692850661 -q .body | awk '/^~~~$/{f=!f;next} f' > p.b64
$ base64 -d p.b64 | gunzip > read_v15.sh
read_v15.sh: decode rc=0 sha=589ef2005e7a1050ad0d2d028a1db92ae390a2f1b583c9f23c843d5332628f5d want=589ef2005e7a1050ad0d2d028a1db92ae390a2f1b583c9f23c843d5332628f5d size=5089 want=5089 decode_err=0
cmp vs owner copy: BYTE-IDENTICAL-READER
```

Citation check: zero occurrences of the superseded head d37ac27e in packet-evidence-v15.sh, PRE_REGISTRATION_v15.md, or the packet description; PARENT_A = a4a21da46284e64a31efff08585c42aaf589089a (current #15999 head) is cited in the packet PIN TABLE and header (2x), the PRE_REGISTRATION (1x), and the packet description (1x). Seeds unchanged: SEED_A d92fd54d… (parent a4a21da4, tree fcba0d8d), SEED_B a2b334f8… (parent 3c14fdab, tree 072b7d2b).
