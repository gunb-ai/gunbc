## wc-probe-v1 classifier fixture output (local run of the frozen fixtures block)

Run form (keeper-style, no execution of anything gunbc/claim): the fixtures block from the frozen bytes, with the live cgroup reads shimmed to the booked values and the single live scope-gate line omitted; everything else verbatim.

```
$ bash fx-harness.sh
fixtures: all held
FXRUN_OK
$ echo $?
0
```

Probe-classifier cases inside the block (frozen in the bytes above, section (7)): valid EXIT3 (reason captured verbatim: WCVERIFY ctx=OK arm=R field=R binder=R body=X{r=0,a=?,s=T,al=1}); duplicate -> NONE; PASS; bare FAIL; code 1 -> OTHER; code 2 -> UNOBSERVED; unclosed paren -> OTHER; code 33 boundary -> OTHER (the prefix needs exactly 'code 3: '); NotBool -> OTHER; code 4 -> EXIT4. Section (8) checks the stated bound arithmetic 41 + 4*38 = 193 <= 193.

The probe entry file itself: blob 084b051209fdf262691c3b1899f2f98aa2e2f9a3 on branch session/sleek-seal-226-wc-probe (commit 5fb8ed7e913c5b7a2fd3e100ab24dd40c7b49baf, tree d917abc098c0ba5bd3593f181f939a6ecbe056d8): `git -c protocol.version=2 fetch origin session/sleek-seal-226-wc-probe && git show 5fb8ed7e...:src/v2/test/claim/namespace_xl0/wc_verdict_probe.dag`.
