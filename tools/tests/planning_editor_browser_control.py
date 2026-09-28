"""Real loopback DAG editor and Fabric file store, with test-only authentication."""
import json
import os
from pathlib import Path
from urllib.parse import urlparse, parse_qs
from playwright.sync_api import sync_playwright

url = os.environ.get('PLANNING_TEST_URL', 'http://127.0.0.1:18783')
assert url.startswith('http://127.0.0.1:'), 'loopback fixture only'
output = Path('target/planning-validation')
output.mkdir(parents=True, exist_ok=True)
with sync_playwright() as p:
    browser = p.chromium.launch(executable_path=os.environ.get('CHROMIUM_EXECUTABLE', p.chromium.executable_path), headless=True, args=['--no-sandbox'])
    page = browser.new_page(viewport={'width': 1100, 'height': 900})
    errors = []
    page.on('pageerror', lambda e: errors.append(str(e)))
    page.goto(url + '/issue/issue-one/planning', wait_until='networkidle')
    page.locator('#editor').wait_for(state='visible', timeout=60000)
    payload = '<img src=x onerror="window.planningInjected=true"> Browser accepted description'
    page.locator('#fields textarea[name="brief"]').fill(payload)
    page.locator('#fields textarea[name="first_slice"]').fill('Browser accepted plan')
    page.locator('#add-child').click()
    page.locator('#children textarea[name="headline"]').fill('Browser child')
    page.locator('#children textarea[name="brief"]').fill('Child outcome')
    page.locator('#children textarea[name="first_slice"]').fill('Child first step')
    page.locator('#propose').click()
    page.locator('#review').wait_for(state='visible', timeout=60000)
    assert payload in page.locator('#differences').inner_text()
    assert page.locator('#differences img').count() == 0
    assert page.evaluate('window.planningInjected') is None
    page.screenshot(path=str(output / 'planning-review.png'), full_page=True)
    page.locator('#accept').click()
    page.wait_for_function("document.getElementById('status').textContent.startsWith('Accepted revision')", timeout=60000)
    result = page.locator('#status').inner_text()
    context = page.request.get(url + '/issue/issue-one/planning/context').json()
    assert context['snapshot']['ticket']['brief'] == payload, context
    assert context['snapshot']['ticket']['first_slice'] == 'Browser accepted plan', context
    identity = parse_qs(urlparse(page.url).query)['candidate'][0]
    bad = page.request.post(url + '/issue/issue-one/planning/accept', data={'identity': identity, 'csrf_token': 'wrong'})
    assert bad.status == 403, bad.text()
    retry = page.request.post(url + '/issue/issue-one/planning/accept', data={'identity': identity, 'csrf_token': context['csrf_token']})
    assert retry.status == 200, retry.text()
    assert retry.json()['replayed'] is True
    assert retry.json()['revision'] == context['snapshot']['event_revision']
    assert not errors, errors
    receipt = {'result': 'PASS', 'checks': ['real HTML/JS', 'typed edit', 'draft child', 'safe text rendering', 'acceptance', 'stored readback', 'CSRF refusal', 'original-result retry'], 'acceptance': result, 'browser_errors': errors, 'scope': 'fixture authentication; production route helpers and Fabric file store'}
    (output / 'browser-result.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(json.dumps(receipt))
    browser.close()
