"""Exercise the modeled GET query transport against loopback, using a fake key."""
import http.server
import json
import pathlib
import re
import shutil
import tempfile
import subprocess
import sys
import threading
import urllib.parse

root = pathlib.Path.cwd()
binary = sys.argv[1]
fixture = 'fixture-only-not-a-key&value'
observed = {}


def stage_import_closure(entry, destination):
    index = {}
    for tree in ('dag', 'src/v2'):
        for source in (root / tree).rglob('*.dag'):
            content = source.read_text()
            module = re.search(r'^module\s+([\w.]+)', content, re.M)
            if module:
                index[module[1]] = (source, content)
    pending, seen = [entry], set()
    while pending:
        source = pending.pop()
        if source in seen:
            continue
        seen.add(source)
        for module in re.findall(r'^import\s+([\w.]+)', source.read_text(), re.M):
            if module not in index:
                raise RuntimeError('Unresolved fixture import: ' + module)
            pending.append(index[module][0])
    for source in seen:
        copied = destination / source.relative_to(root)
        copied.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, copied)
    return destination / entry.relative_to(root)

class Handler(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        observed['query'] = urllib.parse.parse_qs(urllib.parse.urlsplit(self.path).query)
        observed['argv_exposed'] = False
        for proc in pathlib.Path('/proc').iterdir():
            if not proc.name.isdigit():
                continue
            try:
                args = (proc / 'cmdline').read_bytes().split(b'\0')
                if args and pathlib.Path(args[0].decode()).name == 'curl' and any(url.encode() == a for a in args):
                    observed['curl_seen'] = True
                    observed['argv_exposed'] |= any(b'fixture-only-not-a-key' in a for a in args)
            except (OSError, UnicodeError):
                pass
        self.send_response(200)
        self.end_headers()
        self.wfile.write(b'fixture-response')
    def log_message(self, *args):
        pass

server = http.server.HTTPServer(('127.0.0.1', 0), Handler)
url = f'http://127.0.0.1:{server.server_port}/probe'
thread = threading.Thread(target=server.serve_forever, daemon=True)
thread.start()
scratch_root = root / 'target/namecheap-tests'
scratch_root.mkdir(parents=True, exist_ok=True)
scratch = tempfile.TemporaryDirectory(prefix='query-', dir=scratch_root)
entry = pathlib.Path(scratch.name) / 'query_transport_test.dag'
entry.write_text('''module test.namecheap.query_transport_probe
import std.types { Bool, String, NonEmptyStr }
import extdeps.http.client

test fn query_reaches_server(url: String) -> Bool {
  let result = http.Client.GetQueryStdinWithin(url: url as NonEmptyStr,
    query: "ApiKey=fixture-only-not-a-key%26value&Command=namecheap.domains.dns.getHosts",
    connect_seconds: "2" as NonEmptyStr, max_seconds: "5" as NonEmptyStr)
  result.success && result.body == "fixture-response\\n200"
}
''')
try:
    closure = pathlib.Path(scratch.name) / 'closure'
    staged_entry = stage_import_closure(entry, closure)
    result = subprocess.run([binary, 'run', '--source-root', str(closure),
                             '--entry', str(staged_entry),
                             '--claim-run', '--arg', 'url=' + url], capture_output=True, text=True, timeout=120)
    if result.returncode:
        raise RuntimeError(result.stdout + result.stderr)
    assert observed.get('query') == {'ApiKey': [fixture], 'Command': ['namecheap.domains.dns.getHosts']}, observed
    assert observed.get('curl_seen') and not observed['argv_exposed'], observed
    print(json.dumps({'get_query_transport': 'passed', 'query_received': True, 'key_in_argv': False}))
finally:
    server.shutdown()
    server.server_close()
    scratch.cleanup()
