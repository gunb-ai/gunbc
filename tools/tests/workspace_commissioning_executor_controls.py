#!/usr/bin/env python3
"""Exercise the exact DAG-exported transport against controlled process boundaries.

These doubles are not commissioning evidence or production admission helpers.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

parser = argparse.ArgumentParser()
parser.add_argument('--script', type=Path, required=True)
parser.add_argument('--user-manager', action='store_true')
args = parser.parse_args()
script = args.script.resolve(strict=True)
OLD = '1' * 32
NEW = '2' * 32
UNIT = 'gunbc-microvm-slot@srv1-13.service'
DOUBLE = r'''#!/usr/bin/env python3
import json, os, sys
from pathlib import Path
root=Path(os.environ['EXECUTOR_TEST_ROOT'])
config=json.loads((root/'config').read_text())
with (root/'calls').open('a') as f: f.write(json.dumps(sys.argv[1:])+'\n')
a=sys.argv[1]
if a=='prepare':
    if config.get('prepare_refuse'): sys.exit(1)
    print(config.get('decision','start:'+('1'*32)))
elif a=='stop':
    if config.get('stop_refuse'): sys.exit(1)
    (root/'stopped').write_text('yes')
elif a=='rearm':
    if config.get('rearm_refuse'): sys.exit(1)
    print(config.get('rearm_decision','start:'+('1'*32)))
elif a=='bind':
    if config.get('bind_refuse'): sys.exit(1)
    (root/'bound').write_text(sys.argv[3])
elif a=='finish':
    if config.get('finish_refuse'): sys.exit(1)
    print(config.get('terminal','commissioning-fleet-generation-committed'))
elif a=='start':
    (root/'started').write_text('yes')
    if config.get('start_refuse'): sys.exit(1)
elif a=='show':
    n=int((root/'observations').read_text()) if (root/'observations').exists() else 0
    (root/'observations').write_text(str(n+1))
    started=(root/'started').exists()
    inv='2'*32 if started or config.get('existing') else '1'*32
    if config.get('change_at')==n: inv='3'*32
    fields={'Id':'gunbc-microvm-slot@srv1-13.service','LoadState':'loaded',
            'ActiveState':'active' if config.get('running') and started else 'inactive',
            'MainPID':'123' if config.get('running') and started else '0',
            'InvocationID':inv,'Job':''}
    fields.update(config.get('fields',{}))
    for k,v in fields.items():
        if k!=config.get('omit'): print(k+'='+v)
    if config.get('duplicate'): print('Job=')
else: sys.exit(99)
'''
results = []
with tempfile.TemporaryDirectory(prefix='workspace-executor-controls-') as tmp:
    base = Path(tmp)
    helper = base/'helper'
    helper.write_text(DOUBLE)
    helper.chmod(0o700)
    def setup(name, config):
        root=base/name
        root.mkdir()
        (root/'config').write_text(json.dumps(config))
        if not config.get('missing_lock'): (root/'lock').touch()
        env=dict(os.environ, EXECUTOR_TEST_ROOT=str(root), INVOCATION_ID='a'*32)
        cmd=['bash',str(script),str(helper),UNIT,str(root/'lock'),'2',str(helper)]
        return root,env,cmd
    def calls(root):
        return [json.loads(x) for x in (root/'calls').read_text().splitlines()] if (root/'calls').exists() else []
    for name,config,code,starts,finish in [
        ('start-and-settle',{},0,1,True),
        ('fence-drain-rearm',{'decision':'drain:none'},0,1,True),
        ('drain-stop-refused',{'decision':'drain:none','stop_refuse':True},1,0,False),
        ('drain-job-remains',{'decision':'drain:none','fields':{'Job':'123'}},67,0,False),
        ('drain-rearm-refused',{'decision':'drain:none','rearm_refuse':True},1,0,False),
        ('drain-invalid-rearm',{'decision':'drain:none','rearm_decision':'complete:'+NEW},65,0,False),
        ('missing-lock-refuses',{'missing_lock':True},1,0,False),
        ('reattach-no-start',{'decision':'wait:'+NEW,'existing':True},0,0,True),
        ('recover-completed',{'decision':'complete:'+NEW,'existing':True},0,0,True),
        ('settle-after-reuse',{'decision':'settle:'+NEW,'fields':{'InvocationID':'3'*32}},0,0,True),
        ('complete-after-reuse',{'decision':'complete:'+NEW,'fields':{'InvocationID':'3'*32}},0,0,True),
        ('missing-job',{'omit':'Job'},66,0,False),
        ('duplicate-job',{'duplicate':True},66,0,False),
        ('queued-job',{'fields':{'Job':'123'}},67,0,False),
        ('different-unit',{'fields':{'Id':'other.service'}},66,0,False),
        ('changed-invocation',{'change_at':2},67,1,False),
        ('change-at-final-read',{'change_at':3},67,1,False),
        ('settlement-refused',{'finish_refuse':True},1,1,True),
        ('readable-refusal',{'terminal':'refused: CellNotBound x'},69,1,True),
        ('controller-only-success',{'terminal':'commissioned: readiness and provenance read back'},69,1,True),
        ('bind-failed',{'bind_refuse':True},1,1,False),
        ('start-submit-failed',{'start_refuse':True},1,1,False),
        ('prepare-uncertain',{'prepare_refuse':True},1,0,False),
        ('bad-decision',{'decision':'start:$(touch not-permitted)'},65,0,False),
        ('timeout',{'running':True},68,1,False),
    ]:
        root,env,cmd=setup(name,config)
        before=(root/'lock').stat() if (root/'lock').exists() else None
        run=subprocess.run(cmd,env=env,capture_output=True,text=True,timeout=10)
        if before is not None:
            after=(root/'lock').stat()
            assert (before.st_ino,before.st_uid,before.st_gid,before.st_mode)==(after.st_ino,after.st_uid,after.st_gid,after.st_mode),name
        seen=calls(root)
        assert run.returncode==code,(name,run.returncode,run.stderr)
        assert sum(c[0]=='start' for c in seen)==starts,name
        assert any(c[0]=='finish' for c in seen)==finish,name
        if name=='fence-drain-rearm':
            order=[c[0] for c in seen]
            assert order.index('prepare') < order.index('stop') < order.index('rearm') < order.index('start') < order.index('bind'),order
        if name in ('settle-after-reuse','complete-after-reuse'):
            assert not any(c[0]=='show' for c in seen),name
        assert ('commissioning-fleet-generation-committed' in run.stdout)==(code==0),name
        results.append({'case':name,'passed':True})
    # A second actor cannot even prepare while the native waiter owns the fleet lock.
    root,env,cmd=setup('lock-and-cancel',{'running':True})
    cmd[-2]='30'
    first=subprocess.Popen(cmd,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    deadline=time.monotonic()+5
    while not (root/'bound').exists() and time.monotonic()<deadline: time.sleep(.02)
    assert (root/'bound').exists()
    second=subprocess.Popen(cmd,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
    try:
        time.sleep(.25)
        assert sum(c[0]=='prepare' for c in calls(root))==1
        assert not any(c[0]=='finish' for c in calls(root))
    finally:
        # Simulate systemd KillMode=control-group; neither process may publish success.
        import signal
        os.killpg(second.pid,signal.SIGTERM)
        second.wait(timeout=5)
        os.killpg(first.pid,signal.SIGTERM)
        first.wait(timeout=5)
    assert not any(c[0]=='finish' for c in calls(root))
    assert (root/'bound').read_text()==NEW
    results.append({'case':'exclusion-and-cancellation','passed':True})
    if args.user_manager:
        import uuid
        root,env,cmd=setup('host-owned-lifetime',{'running':True})
        cmd[-2]='30'
        unit='workspace-executor-control-'+uuid.uuid4().hex+'.service'
        launched=False
        try:
            subprocess.run(['systemd-run','--user','--quiet','--unit='+unit,
                '--property=Type=exec','--property=RemainAfterExit=yes',
                '--property=KillMode=control-group','--property=MemoryMax=64M',
                '--property=MemorySwapMax=0',
                '--setenv=EXECUTOR_TEST_ROOT='+str(root),*cmd],check=True)
            launched=True
            deadline=time.monotonic()+10
            while not (root/'bound').exists() and time.monotonic()<deadline: time.sleep(.05)
            assert (root/'bound').exists()
            # systemd-run has exited. The executor, with systemd's own InvocationID, persists.
            current=subprocess.check_output(['systemctl','--user','show',unit,
                '--property=MainPID','--value'],text=True).strip()
            assert int(current)>0
            assert not any(c[0]=='finish' for c in calls(root))
            assert sum(c[0]=='prepare' for c in calls(root))==1
            (root/'config-next').write_text('{}')
            (root/'config-next').replace(root/'config')
            deadline=time.monotonic()+10
            while time.monotonic()<deadline:
                state=subprocess.check_output(['systemctl','--user','show',unit,
                    '--property=SubState','--value'],text=True).strip()
                if state=='exited': break
                time.sleep(.05)
            assert state=='exited',state
            assert subprocess.check_output(['systemctl','--user','show',unit,
                '--property=ExecMainStatus','--value'],text=True).strip()=='0'
            assert sum(c[0]=='start' for c in calls(root))==1
            assert sum(c[0]=='finish' for c in calls(root))==1
            peak=subprocess.check_output(['systemctl','--user','show',unit,
                '--property=MemoryPeak','--value'],text=True).strip()
            results.append({'case':'survives-dispatcher-exit','passed':True,
                'test_unit_memory_peak_bytes':peak})
        finally:
            if launched:
                subprocess.run(['systemctl','--user','stop',unit],check=True)
                subprocess.run(['systemctl','--user','reset-failed',unit],
                    stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
print(json.dumps(results,indent=2))
