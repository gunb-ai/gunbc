#!/usr/bin/env python3
"""slice.py: partition hand-read-class modules with a Phase 1 signal into namespace slices (one reader each)."""
import csv,re,collections
rows=[x for x in csv.DictReader(open('mechanical.tsv'),delimiter='\t') if x['signal']=='1' and x['class'] in('production','extdeps','seed')]
rules=[('01-seed',r'^src/v1/'),
('02-extdeps',r'^extdeps\.'),
('03-runner-ci',r'^gunbc\.(runner|ci|build_cache|floor|witness|heal_|required_ci|emitted_subject|generated_artifact|tier_o|instrument_targets|target_binding|parse_verdict|seed_growth|[a-z_]*seed_growth|census_|materialization_|typed_module_cache|native_|observation_|toolchain|v1\.|namespace_cut|source_|doc_graph|design|annotation|class_b|extdeps_scope|compiler_frontend|live_rust|decl_facts|root_demand|review_sheet|repo)'),
('04-fleet-host',r'^gunbc\.(fleet|host|hostname|oomd|memory_provisioning|systemd_|owned_process|process_hold|command_runner|host_|network_identity|package_delivery|os_install|provisioning|secret_provision|srv1_|operator_fan)'),
('05-machine-intake-sites',r'^gunbc\.(machine_intake|bmc|busybox|srv3|srv4|site|oracle_oci|cloudflare|namecheap|boot_|firmware|guest_image|revision_job|board_|mt_collins|assimilate|devboot|demo|url_file|tools|econ|model|container|frontend_eval|served_surface|node_http|public_workload|readback|deploy_transition|deployment_risk|apply)'),
('06-roadmap-auth-dispatch',r'^gunbc\.(auth|roadmap|live_deploy|dispatch_|harness|scm|codex_|claude_|cursor_|flex_|github_|tailscale_acl|session_|worker_turn|action_use|actions_run|cli_dispatch|provider_|decoder|durable_|executor_|cli_run|git_diff|filesystem_|heal|persistent_|plan)'),
('07-fabric-compute-workspace',r'^gunbc\.(fabric|spark|compute|serving|workspace_|compile_pool|typed_remote)|^product\.'),
]
sl=collections.defaultdict(list)
for x in rows:
  for n,p in rules:
    if re.search(p,x['module']): sl[n].append(x); break
  else: sl['08-std-and-rest'].append(x)
for n in sorted(sl):
  with open(f'slices/{n}.tsv','w') as o:
    o.write('\t'.join(['module','path','mech_storage','mech_caching','mech_access','mech_authz','mech_placement','ambient','leads'])+'\n')
    for x in sl[n]: o.write('\t'.join([x['module'],x['path'],x['storage'],x['caching'],x['access'],x['authz'],x['placement'],x['ambient'],x['leads']])+'\n')
  print(n,len(sl[n]))
