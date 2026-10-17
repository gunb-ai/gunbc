import subprocess, shutil
M='dag/gunbc/provisioning/altra_autoinstall_seed.dag'
W='dag/test/claim/altra_autoinstall_seed_witness.dag'
RUN=['./target/release/gunbc','run','--claim-run','--source-root','dag','--source-root','src/v2','--entry',W]
shutil.copy(M,'/tmp/m.bak'); shutil.copy(W,'/tmp/w.bak')
def run():
    p=subprocess.run(RUN,capture_output=True,text=True)
    lines=p.stdout.splitlines()+p.stderr.splitlines()
    passes=[l for l in lines if l.startswith('PASS ')]
    fails=sorted(l[len('FAIL '):].strip() for l in lines if l.startswith('FAIL '))
    return (len(passes), fails) if passes else (0,['<NO-TESTS-RAN>'])
MUTS=[
 # ---- review 81772 finding 1: the fail-open audit ----
 ('R1-unclassified-arm-fails-open', [(M,
'''    UnclassifiedPackage { name: un, debconf_seed: _ } =>
      Present { value: PackagePromptingUnclassified { name: un } }''',
'''    UnclassifiedPackage { name: _, debconf_seed: _ } => Absent''')]),
 ('R2-a-debconf-seed-can-classify', [(M,
'''    UnclassifiedPackage { name: un, debconf_seed: _ } =>
      Present { value: PackagePromptingUnclassified { name: un } }''',
'''    UnclassifiedPackage { name: un, debconf_seed: seed } =>
      match seed {
        Present { value: _ } => Absent
        Absent => Present { value: PackagePromptingUnclassified { name: un } }
      }''')]),
 ('R3-question-unanswered-arm-fails-open', [(M,
'''    QuestionUnanswered { package: named } => Present {''',
'''    QuestionUnanswered { package: _ } => Absent
    QuestionAnswered { package: named, debconf_seed: _ } => Present {''')]),
 # ---- review 81772 finding 2: one spelling per fact ----
 ('R4-prompting-name-drifts', [(M, '    TzdataPackage => "tzdata"', '    TzdataPackage => "timezone-data"')]),
 # ---- review 81772 finding 4: two axes, each with its own warrant ----
 ('R5-package-axis-prefix-dropped', [(M,
'      .map(driver => concat("pkg:", altra_install_driver_package(driver: driver)) as String),',
'      .map(driver => concat("driver-", altra_install_driver_package(driver: driver)) as String),')]),
 ('R6-module-token-drops-symbol', [(M,
'          concat("=", loadable_module_kconfig_symbol(driver: module)),',
'          "",')]),
 ('R7-module-axis-dropped-from-render', [(M,
'    concat("modules=", loadable_module_list_diagnostic(modules: seed.loadable_modules)),',
'    concat("modules=", "fixed"),')]),
 ('R8-module-spelling-dropped', [(M,
'          concat("mod:", altra_loadable_module(driver: module)),',
'          concat("mod:", "unnamed"),')]),
 # ---- review 81772 finding 3: predicates dissolved into total matches ----
 ('R9-console-arm-collapsed', [(M,
'    PxeStrategy { boot_mode: LegacyBiosPxe { tftp_root: _ } } => ConsoleSuppressed',
'    PxeStrategy { boot_mode: LegacyBiosPxe { tftp_root: _ } } =>\n      ConsoleAnsweredByUnattended { reason: "collapsed" as NonEmptyStr }')]),
 ('R10-boot-mode-token-collapsed', [(M, '    UefiHttpBoot => "uefi-http"', '    UefiHttpBoot => "uefi-pxe"')]),
 ('R11-entrypoint-console-token-collapsed', [(M,
'        ConsoleEntryUnlocked { reason: _ } => "keys=0,console=unlocked"',
'        ConsoleEntryUnlocked { reason: _ } => "keys=0,console=locked"')]),
 ('R12-answered-arm-not-counted', [(M,
'    QuestionAnswered { package: _, debconf_seed: _ } => true',
'    QuestionAnswered { package: _, debconf_seed: _ } => false')]),
 ('R13-proof-count-drops-a-term', [(M,
'  closed + addressing_closed + mirror_closed + 1',
'  closed + addressing_closed + mirror_closed')]),
 ('R14-unexamined-filter-inverted', [(M,
'''  match unexamined_candidate(selector: selector, device: device) {
    Present { value: _ } => true
    Absent => false
  }''',
'''  match unexamined_candidate(selector: selector, device: device) {
    Present { value: _ } => false
    Absent => true
  }''')]),
 # ---- the operator-facing disk pair: manager ask (a) ----
 ('R15-unread-serial-becomes-zero-match', [(M,
'            Present { value: CandidateIdentityUnobserved { part_number: c.part_number } }',
'            Absent')]),
 ('R16-zero-match-becomes-multi-match', [(M,
'      0 => SelectionRefused { cause: NoDiskMatches { selector: selector } }',
'      0 => SelectionRefused { cause: MultipleDisksMatch { selector: selector, candidates: ["phantom" as NonEmptyStr] } }')]),
 ('R17-multi-match-arm-bypassed', [(M,
'    Absent => match candidate_count(selector: selector, observed: observed) {',
'    Absent => match 1 {')]),
 # ---- the two rules the manager named ----
 ('R18-ephemeral-admitted', [(M, '    Ephemeral => false', '    Ephemeral => true')]),
 ('R19-cached-mirror-admitted', [(M, '    CachedMirror => false', '    CachedMirror => true')]),
 ('R20-credential-guard-removed', [(M,
'  match list_length(items: authorised_keys) == 0 {',
'  match list_length(items: authorised_keys) < 0 {')]),
 ('R21-receipt-accepts-plaintext', [(M,
'  candidate.starts_with(prefix: "$6$") && candidate.length() >= 20',
'  candidate.length() >= 20')]),
 ('R22-catalog-capacity-check-dropped', [(M,
'              c.part_number == wanted_part && measure_le(a: c.capacity_bytes, b: wanted_capacity) && measure_le(a: wanted_capacity, b: c.capacity_bytes)',
'              c.part_number == wanted_part')]),
 # ---- digest / binding integrity ----
 ('R23-transport-token-constant', [(M,
'      concat("pxe-", client_network_boot_mode_token(mode: mode))',
'      "pxe"')]),
 ('R24-package-order-affects-digest', [(M,
'    False => names.sort_by(n => n).join(",") as NonEmptyStr',
'    False => names.join(",") as NonEmptyStr')]),
 ('R25-credential-version-dropped-from-digest', [(M, '        h.secret_version,', '        "0",')]),
 ('R26-binding-digest-not-computed', [(M,
'              seed_digest: altra_autoinstall_seed_digest(seed: seed),',
'              seed_digest: content_hash_atom(value: "constant" as NonEmptyStr),')]),
 ('R27-binding-ownership-check-removed', [(M,
'  match binding.unit_key == unit_key && binding.attempt == attempt {',
'  match true {')]),
 ('R28-bound-serial-not-recorded', [(M,
'    Present { value: serial } => BoundDisk { serial: serial }',
'    Present { value: _ } => BoundDiskUnread { note: "unrecorded" as NonEmptyStr }')]),
 ('R29-binding-digest-drops-unit', [(M,
'        left: content_hash_atom(value: to_string(binding.unit_key) as NonEmptyStr),',
'        left: content_hash_atom(value: "unit" as NonEmptyStr),')]),
 ('R30-binding-over-refused-selection', [(M,
'        SelectionRefused { cause: _ } => Absent',
'        SelectionRefused { cause: _ } => Present { value: AltraAutoinstallSeedBinding { unit_key: unit_key, attempt: attempt, seed_digest: altra_autoinstall_seed_digest(seed: seed), seed_rendering: render_altra_autoinstall_seed(seed: seed), delivery: seed.delivery, disk: seed.disk, disk_selection: BoundDiskUnread { note: "refused" as NonEmptyStr } } }')]),
 ('R31-mirror-audit-removed', [(M,
'        MirrorAbsent => SeedRefused { cause: ArchiveMirrorAbsent }',
'        MirrorAbsent => SeedRefused { cause: AddressingIsInstallerManaged }')]),
]
n0,f0=run()
print('BASELINE PASS=%d FAIL=%d RED=%s' % (n0,len(f0),f0), flush=True)
print('BASELINE_TESTFN=%s' % subprocess.run(['grep','-c','^test fn',W],capture_output=True,text=True).stdout.strip(), flush=True)
for label,edits in MUTS:
    shutil.copy('/tmp/m.bak',M); shutil.copy('/tmp/w.bak',W)
    ok=True
    for path,frm,to in edits:
        src=open(path).read()
        if frm not in src:
            print('RESULT %s SKIP-PATTERN-NOT-FOUND %r' % (label,frm[:60]), flush=True); ok=False; break
        open(path,'w').write(src.replace(frm,to,1))
    if not ok: continue
    n,fails=run()
    print('RESULT %s PASS=%d RED(%d): %s' % (label,n,len(fails),';'.join(fails)), flush=True)
    shutil.copy('/tmp/m.bak',M); shutil.copy('/tmp/w.bak',W)
shutil.copy('/tmp/m.bak',M); shutil.copy('/tmp/w.bak',W)
print('RESTORED', flush=True)
print('POST_HASHES:'); print(subprocess.run(['sha256sum',M,W],capture_output=True,text=True).stdout)
