"""Exercise the real planner file entry point with synthetic, offline inputs."""
import datetime
import hashlib
import json
import pathlib
import subprocess
import sys
import tempfile

from support import stage_import_closure

root = pathlib.Path.cwd()
binary = sys.argv[1]
scratch_root = root / 'target/namecheap-tests'
scratch_root.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='planner-', dir=scratch_root) as temp:
    work = pathlib.Path(temp)
    closure = work / 'closure'
    entry = stage_import_closure(root / 'dag/gunbc/namecheap/planning/run.dag', closure)
    observation = work / 'observation.json'
    intent_file = work / 'intent.json'
    output = work / 'review.json'
    revision = '1' * 40
    rows = [
        {'HostId': '1', 'Name': '@', 'Type': 'A', 'Address': '185.199.108.153', 'TTL': '1800', 'Future': 'retained'},
        {'HostId': '2', 'Name': '@', 'Type': 'MX', 'Address': 'SMTP.GOOGLE.COM.', 'MXPref': '1', 'TTL': '1800'},
        {'HostId': '3', 'Name': '@', 'Type': 'TXT', 'Address': 'v=DMARC1; p=none', 'TTL': '1800'},
    ]
    receipt = {'schema': 'gunbc-namecheap-observation/v1', 'standing': 'getHosts-observed',
               'run_id': '123', 'run_attempt': '1', 'revision': revision,
               'started_at': datetime.datetime.now(datetime.timezone.utc).strftime('%Y-%m-%dT%H:%M:%SZ'),
               'exact_credential_version': 'projects/582015116396/secrets/namecheap-api-key/versions/1',
               'account': 'briansrls', 'public_egress_ipv4': '96.224.201.7', 'domain': 'gunb.ai',
               'result_fields': {'Domain': 'gunb.ai', 'IsUsingOurDNS': 'true', 'FutureResult': 'retained'},
               'hosts': rows, 'mail_mode': 'unobserved', 'write_authority': 'withheld'}
    intent = {'schema': 'gunbc-dev-dns-intent/v1', 'services': [
        {'service': 'svc:tracker-dev', 'ipv4': '100.100.1.1', 'observation_ref': 'fixture-only/tracker'},
        {'service': 'svc:approvals-dev', 'ipv4': '100.100.1.2', 'observation_ref': 'fixture-only/approvals'}],
        'challenges': [{'service': 'svc:tracker-dev', 'action': 'publish', 'value': 'A' * 43, 'ownership_ref': 'fixture-only/order'}]}
    observation.write_text(json.dumps(receipt))
    intent_file.write_text(json.dumps(intent))
    command = [binary, 'run', '--source-root', str(closure), '--entry', str(entry)]
    for key, value in dict(observation_path=observation, intent_path=intent_file, output_path=output,
                           expected_run='123', expected_attempt='1', expected_revision=revision).items():
        command += ['--arg', f'{key}={value}']
    result = subprocess.run(command, capture_output=True, text=True, timeout=180)
    assert result.returncode == 0, result.stdout + result.stderr
    saved = json.loads(output.read_text())
    plan = saved['plan']
    assert plan['observation'] == receipt and plan['intent'] == intent
    assert plan['proposed_hosts'][:3] == rows and len(plan['proposed_hosts']) == 6
    assert plan['preserved_result_fields'] == receipt['result_fields']
    assert plan['write_authority'] == 'withheld' and plan['mail_mode'] == 'unobserved'
    assert [s['origin'] for s in plan['service_origins']] == ['https://tracker-dev.gunb.ai', 'https://approvals-dev.gunb.ai']
    assert all(s['required_access'] == 'tailnet-only' for s in plan['service_origins'])
    assert plan['zone_writer_key'] == 'namecheap/zone/gunb.ai'
    # Independent digest oracle for the actual emitted content, using its framing.
    content = json.dumps(plan, ensure_ascii=False, separators=(', ', ': '))
    preimage = f'namecheap-dns-review/v1:{len(content)}:{content};'
    assert saved['sha256'] == hashlib.sha256(preimage.encode()).hexdigest()
    # Same file entry refuses an observation with mismatched independently supplied provenance.
    command[command.index('expected_run=123')] = 'expected_run=999'
    refused = subprocess.run(command, capture_output=True, text=True, timeout=180)
    assert refused.returncode != 0
    assert json.loads(output.read_text()) == saved  # refusal does not publish a new plan
print('Planner file path passed: preserved snapshot, combined plan, SHA-256 oracle, wrong-run refusal')
