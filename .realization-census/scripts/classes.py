#!/usr/bin/env python3
"""classes.py: one row per ERROR CLASS (schema addendum 2), joined from findings.tsv.

usage: classes.py   (run from .realization-census/ after merge.py)
Each findings row of a hand-read class (production / extdeps / seed) at L3-L5 is assigned to exactly ONE class by the
first matching rule below (dimension + decision site + specimen/note keywords). Unmatched rows fall to
<dimension>_<site>_unclassified, so no instance is silently lost; their count is reported.
Rungs are DESIGN.md section 4b: 0 unmodeled/ambient, 1 mitigatable, 2 mechanically preventable, 3 structurally
guaranteed, 4 structurally impossible. CURRENT rung: this census looked for decision sites, not for gates, so a class
is written 0 unless a wall is named below from the corpus; the README states that limit.
Writes classes.tsv and adds a class_id column to findings.tsv.
"""
import csv, re, collections

R = []  # (class_id, dimension, site set, regex over specimen+note+symbol, fields)
def rule(cid, dim, sites, pat, pattern, authority, wall, red, ceiling, residual, trigger, rung=0):
    R.append(dict(class_id=cid, dimension=dim, sites=set(sites), pat=re.compile(pat, re.I), pattern=pattern,
                  authority=authority, wall=wall, red=red, ceiling=ceiling, residual=residual, trigger=trigger, rung=rung))

DECL = 'derived from one declaration'
# storage
rule('storage_hand_exclusion', 'storage', ['Forked'], r'o_excl|create_new|createnew|flock|\block\b|lease|compare.and.set|\bcas\b|generation|index',
     'own create-once / lock / compare-and-set beside the CAS library', 'std.durable_compare_and_set::CasExpectation; gunbc.durable_cas_file_store::file_compare_and_set',
     'admit_callers roster on the exclusive-create primitive (only the CAS store may call it)', 'a fixture business module calling Filesystem.WriteCreateNew directly is refused',
     '3: the primitive is a modeled effect, so its callers are a decidable set', 'stores whose head semantics the CAS contract does not cover', 'roster of exclusive-create callers declared')
rule('storage_hand_directory_setup', 'storage', ['Forked'], r'mkdir|create_dir|chmod|install -d|owneronlydirectory',
     'module creates its own directories (and modes) beside the owned/managed directory authority', 'gunbc.owned_directory::OwnedDirectoryTarget; gunbc.roadmap.dashboard_instance_directories',
     'admit_callers roster on directory-create effects', 'a fixture module calling shell.Mkdir.Parents on a store root is refused',
     '3: directory creation is a modeled effect', 'one-run scratch trees (scored other)', 'directory-create callers rostered')
rule('storage_hand_retention_sweep', 'storage', ['Forked'], r'rm_recursive|remove\.?recursive|remove_dir_all|removefile|remove\.file|find .*-delete|sweep|prune',
     'module deletes or sweeps kept data by hand', 'gunbc.managed_directory', 'admit_callers roster on recursive-remove effects',
     'a fixture module calling shell.Remove.RecursiveForce on a kept root is refused', '3: removal is a modeled effect', 'teardown of own one-run scratch', 'retention owned by the directory authority')
rule('storage_raw_state_beside_library', 'storage', ['Forked'], r'.',
     'kept record written raw (overwrite, own codec) where a store library exists (mechanism=none-raw)', 'std.artifact_store::ArtifactStore; gunbc.durable_cas_file_store',
     'type: kept records are writable only through a store handle', 'a fixture business module writing Filesystem.Write to a kept path is refused',
     '4: a raw kept-path write has no constructor once stores are handles', 'genuinely scratch writes', 'kept-path writes go through a store handle')
rule('storage_relative_receipt_path', 'storage', ['Literal'], r'"?(target|artifacts|\.gunbc|out)/',
     'repo-relative literal path for a receipt or artifact', 'none exists: receipt/artifact sink', DECL,
     'a fixture module naming "target/x-receipt.txt" is refused', '4 after the sink exists', 'receipts nobody re-reads', 'receipt sink declared')
rule('storage_tmp_literal', 'storage', ['Literal'], r'/tmp',
     'fixed /tmp file or directory name for kept or shared data', 'std.artifact_store_scratch_grant', DECL,
     'a fixture module naming "/tmp/gunbc-x.json" outside the scratch grant is refused', '3', 'mktemp-style unique scratch', 'scratch grant consumed')
rule('storage_root_literal', 'storage', ['Literal'], r'.',
     'store root or kept path hard-coded in business code', 'gunbc.owned_directory; gunbc.roadmap.dashboard_instance_directories', DECL,
     'a fixture module naming "/var/lib/gunbc/x" outside the directory declaration is refused', '4: the path has no spelling outside its declaration', 'upstream-fixed paths (access, ruling d)', 'directory declaration consumed')
rule('storage_root_rederived', 'storage', ['Rederived', 'Passed'], r'.',
     'store root re-derived from an authority the module was (or could be) handed', 'gunbc.roadmap.dashboard_instance_directories', 'type: the store handle is the parameter, not the instance',
     'a fixture that receives stores and re-derives a root from the instance is refused', '3', '', 'store handles passed')
# caching
rule('caching_generated_constant_memo', 'caching', ['Forked'], r'emit_data_def|generated|thread_local cached',
     'emitter template wraps every nullary constant in a thread_local memo', 'std.materialization_ladder', 'one template edit (05_emit_rust emit_data_def)',
     'an emitted constant with a CACHED wrapper is refused by the emit census', '4 for the template', 'none', 'template change')
rule('caching_process_memo_table', 'caching', ['Forked'], r'.',
     'process-wide memo table with a hand-authored key and no invalidation (OnceLock / thread_local / static map)', 'std.materialization_ladder; std.materialization_provider',
     'declared requires: a memo needs an admitted provider and key', 'a seed fn adding a static OnceLock memo without a provider row is refused',
     '2 in the seed (validation over Rust); 4 in v2 (memo is a provider binding)', 'the seed is frozen-maintenance', 'v2 self-host of the module')
rule('caching_root_literal', 'caching', ['Literal', 'Rederived'], r'.',
     'cache directory chosen by literal in business code', 'std.cache_interface::StorageSurface', DECL,
     'a fixture naming a cache root literal is refused', '4', '', 'cache root declared')
# access
rule('access_kernel_abi_path', 'access', ['Literal', 'Forked'], r'/proc/|/sys/|os-release',
     'kernel ABI or upstream-constant path read directly (ruling d)', 'extdeps.linux.*', 'derived from the extdeps interface model',
     'a fixture reading "/proc/self/cgroup" outside extdeps is refused', '4', 'none', 'extdeps interface rows consumed')
rule('access_credential_location', 'access', ['Literal', 'Forked'], r'credential|secret|token|\.key|mac-key|credstore|netrc|bearer|private',
     'credential or secret location hard-coded (or re-implemented) in business code', 'gunbc.auth.gcp_secret_access::SecretAccessGrant; gunbc.auth.access_token_source::select_access_token_source',
     'type: a secret is reachable only through a SecretAccessGrant', 'a fixture module opening a key path literal is refused', '4', 'bootstrap secrets before any grant exists', 'secret grant consumed')
rule('access_ambient_env', 'access', ['Literal', 'Forked'], r'env|home',
     'resource location or identity reached through an ambient environment variable', 'std.resources::ResourceHandle', 'declared requires: an entry declares the env it reads',
     'a fixture fn calling shell.Env.Get outside an entry declaration is refused', '3', 'upstream-contract env (CI variables)', 'entry-environment declaration')
rule('access_identity_literal', 'access', ['Literal'], r'.',
     'endpoint, user or identity to call as, hard-coded', 'gunbc.fleet.fleet_posix_accounts; gunbc.auth.access_token_source', DECL,
     'a fixture naming a login or endpoint literal outside the roster is refused', '4', '', 'roster consumed')
rule('access_rederived', 'access', ['Rederived'], r'.',
     'access path re-selected per host or per caller instead of received', 'gunbc.fleet.fleet_host_identity', 'type: access is a parameter of the host row',
     'a fixture branching on host identity to pick access is refused', '3', '', 'host access passed')
# authz
rule('authz_ambient_sudo', 'authz', ['Literal', 'Rederived', 'Forked'], r'sudo',
     'privileged effect performed by ambient sudo rather than a received capability', 'gunbc.auth.authorization_pattern_selection::select_authorization_pattern; gunbc.auth.privileged_effect_census',
     'declared requires: the effect names its authorization pattern', 'a fixture sudo_elevate site absent from the selection is refused',
     '3', 'bootstrap of the grant machinery itself', 'every site executes the selection', rung=1)
rule('authz_other', 'authz', ['Literal', 'Rederived', 'Forked'], r'.',
     'authorization decided locally (token read, standing composed in place)', 'std.scoped_authorization::ScopedAuthorization; gunbc.auth.approval_capability::ApprovalCapability',
     'type: the capability is the parameter', 'a fixture composing a standing from a read token is refused', '3', '', 'capability passed')
# placement
rule('placement_instance_repick', 'placement', ['Rederived'], r'.',
     'host or instance re-selected downstream of an entry (srv1_live_dashboard_instance() wrappers, per-host fns, host == srvN guards)',
     'gunbc.roadmap.roadmap_dashboard_instance::HostDashboardInstance', 'admit_callers roster: only entry points may call the instance selectors',
     'a fixture non-entry fn calling srv1_live_dashboard_instance() is refused', '3', 'operator-facing CLIs that are entries by design', 'selector callers rostered')
rule('placement_device_node', 'placement', ['Literal'], r'/dev/',
     'device node named in business code (ruling d)', 'host or instance declaration naming the device', DECL, 'a fixture naming "/dev/vdb" outside the host row is refused', '4', '', 'device rows declared')
rule('placement_unit_name_or_dir', 'placement', ['Literal'], r'systemd|\.service|\.slice|\.timer|\.socket|\bunit',
     'systemd unit name or unit directory hard-coded', 'none exists: unit-directory choice (extdeps.systemd lists legal locations)', DECL,
     'a fixture naming "/etc/systemd/system/" outside the deployment declaration is refused', '4 after the home exists', '', 'deployment unit home declared')
rule('placement_upstream_binary', 'placement', ['Literal'], r'/usr/|/s?bin/|homebrew|/opt/',
     'upstream program location named in business code (ruling b)', 'gunbc.roadmap.roadmap_dashboard_instance execution_environment; dispatch_capability_program', DECL,
     'a fixture naming "/usr/bin/systemctl" outside the environment declaration is refused', '4', '', 'environment declaration consumed')
rule('placement_port_address', 'placement', ['Literal'], r'port|127\.0\.0\.1|0\.0\.0\.0|localhost|:[0-9]{2,5}|listen',
     'port or listen address hard-coded', 'none exists: served-endpoint declaration', DECL, 'a fixture binding a literal port is refused', '4 after the home exists', '', 'endpoint home declared')
rule('placement_env_branch', 'placement', ['Literal'], r'env|github_|ci\b',
     'behaviour switches on the environment (CI variables, env-selected instance)', 'gunbc.ci.* (no CI-environment authority consumed by the seed)', 'declared requires on the entry',
     'a fixture branching on GITHUB_ACTIONS outside an entry is refused', '3', '', 'entry-environment declaration')
rule('placement_host_literal', 'placement', ['Literal'], r'.',
     'host, instance id or address literal in business code', 'gunbc.fleet.*; gunbc.roadmap.roadmap_dashboard_instance', DECL,
     'a fixture naming "srv1" outside the fleet declaration is refused', '4', 'observed host names in records (L0)', 'fleet rows consumed')

def assign(r):
    t = ' '.join([r['specimen'], r['note'], r['symbol']])
    for c in R:
        if c['dimension'] == r['dimension'] and r['site'] in c['sites'] and c['pat'].search(t):
            return c['class_id']
    return f"{r['dimension']}_{r['site'].lower()}_unclassified"

F = list(csv.DictReader(open('findings.tsv'), delimiter='\t'))
hdr = list(F[0].keys())
if 'class_id' not in hdr:
    hdr.insert(1, 'class_id')
pop = collections.defaultdict(list)
for r in F:
    r['class_id'] = assign(r) if r['class'] in ('production', 'extdeps', 'seed') and r['site'] in ('Rederived', 'Literal', 'Forked') else ''
    if r['class_id']:
        pop[r['class_id']].append(r)
with open('findings.tsv', 'w') as o:
    o.write('\t'.join(hdr) + '\n')
    for r in F:
        o.write('\t'.join(r[h] for h in hdr) + '\n')

meta = {c['class_id']: c for c in R}
cols = ['class_id', 'dimension', 'decision_site', 'pattern', 'instances', 'modules', 'production_modules', 'extdeps_modules', 'seed_modules',
        'module_list', 'current_rung', 'attainable_ceiling', 'owning_authority', 'wall', 'red_control', 'residual', 'next_trigger']
def short(x, n=200):
    return x if len(x) <= n else x[:n - 1] + '…'
rows = []
for cid, rs in pop.items():
    c = meta.get(cid)
    mods = sorted({r['module'] for r in rs})
    by = collections.Counter(r['class'] for r in {r['module']: r for r in rs}.values())
    sites = '/'.join(sorted({r['site'] for r in rs}))
    rows.append({'class_id': cid, 'dimension': rs[0]['dimension'], 'decision_site': sites,
                 'pattern': c['pattern'] if c else 'not matched by any class rule: read these rows',
                 'instances': str(len(rs)), 'modules': str(len(mods)), 'production_modules': str(by['production']), 'extdeps_modules': str(by['extdeps']), 'seed_modules': str(by['seed']),
                 'module_list': 'findings.tsv where class_id = ' + cid,
                 'current_rung': str(c['rung'] if c else 0), 'attainable_ceiling': c['ceiling'] if c else '', 'owning_authority': c['authority'] if c else '',
                 'wall': c['wall'] if c else '', 'red_control': c['red'] if c else '', 'residual': c['residual'] if c else '', 'next_trigger': c['trigger'] if c else ''})
rows.sort(key=lambda r: (-int(r['modules']), r['class_id']))
with open('classes.tsv', 'w') as o:
    o.write('\t'.join(cols) + '\n')
    for r in rows:
        o.write('\t'.join(short(r[c]) for c in cols) + '\n')
un = sum(len(v) for k, v in pop.items() if k.endswith('_unclassified'))
print('classes', len(rows), 'instances', sum(len(v) for v in pop.values()), 'unclassified instances', un)
