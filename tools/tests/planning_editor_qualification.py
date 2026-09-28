"""Run each planning/storage witness directly over unchanged whole-module sources."""
import json
import os
from pathlib import Path
import re
import subprocess
import sys

modules = [
    'test.claim.roadmap.roadmap_planning_candidate_witness_test',
    'test.claim.roadmap.roadmap_planning_candidate_storage_witness_test',
    'test.claim.roadmap.roadmap_planning_apply_witness_test',
    'test.claim.roadmap.roadmap_planning_http_witness_test',
    'test.claim.roadmap.roadmap_planning_projection_witness_test',
    'test.claim.roadmap.roadmap_task_projection_witness_test',
    'test.claim.roadmap_task_wire_witness_test',
    'test.claim.roadmap.roadmap_event_log_witness_test',
    'test.claim.roadmap.roadmap_event_carrier_witness_test',
]
output = Path('target/planning-validation')
source = output / 'qualification-source'
subprocess.run([sys.executable, 'tools/tests/dag_validation_closure.py', *modules,
                '--output', str(source)], check=True)
functions = []
imports = []
for module in modules:
    names = re.findall(r'^test fn (\w+)\(', (source / (module + '.dag')).read_text(), re.M)
    functions.extend(module + '.' + name for name in names)
    imports.append('import ' + module + ' { ' + ', '.join(names) + ' }')
entry = source / 'test.manual.planning_qualification.dag'
entry.write_text('module test.manual.planning_qualification\n\n' + '\n'.join(imports) + '\n')
command = ['systemd-run', '--user', '--scope', '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0',
           '--quiet', os.environ.get('GUNBC_TEST_BINARY', 'gunbc'), 'run', '--source-root',
           str(source), '--entry', str(entry), '--claim-run']
for function in functions:
    command.extend(['--function', function])
with (output / 'qualification.log').open('w') as log:
    result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT)
(output / 'qualification-exit.txt').write_text(str(result.returncode) + '\n')
(output / 'qualification-functions.json').write_text(json.dumps(functions, indent=2) + '\n')
print(f'{len(functions)} controls, exit {result.returncode}; {output}/qualification.log')
raise SystemExit(result.returncode)
