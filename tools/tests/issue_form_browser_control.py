"""Capture a native Chromium POST and feed its wire bytes to the actual .dag decoder.
Run with the repository's Playwright Python environment; no live site or credentials.
"""
import os
import subprocess
from pathlib import Path
from playwright.sync_api import sync_playwright

repo = Path(__file__).resolve().parents[2]
with sync_playwright() as p:
    options = {}
    if os.environ.get('CHROMIUM_EXECUTABLE'):
        options['executable_path'] = os.environ['CHROMIUM_EXECUTABLE']
    browser = p.chromium.launch(**options)
    page = browser.new_page()
    captured = []
    def route(request):
        if request.request.method == 'POST':
            captured.append(request.request.post_data)
            assert request.request.headers['content-type'].startswith('application/x-www-form-urlencoded')
            request.fulfill(status=200, body='recorded fixture')
        else:
            request.fulfill(status=200, content_type='text/html', body='''<meta charset="utf-8"><form method="post" action="/command" enctype="application/x-www-form-urlencoded"><input name="csrf" value="real-token"><input name="expected_revision" value=""><textarea name="body"></textarea><button>Submit</button></form>''')
    page.route('http://issue-fixture.test/**', route)
    page.goto('http://issue-fixture.test/')
    page.locator('textarea').fill('first\ncsrf=forged\n☃ + & =')
    with page.expect_navigation():
        page.locator('button').click()
    browser.close()
assert len(captured) == 1
subprocess.run(['systemd-run', '--user', '--scope', '-p', 'MemoryMax=6G', '-p', 'MemorySwapMax=0', '--quiet',
    str(repo/'target/release/gunbc'), 'run', '--source-root', 'target/form-codec-validation',
    '--entry', 'target/form-codec-validation/test.claim.http.form_urlencoded_witness_test.dag',
    '--function', 'browser_post_control', '--arg', 'wire='+captured[0], '--claim-run'], cwd=repo, check=True)
print('PASS native Chromium POST through .dag form decoder')
