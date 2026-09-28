"""Run emitted DAG sort/focus behavior in Chromium; progress values are explicit fixtures."""
import os
from pathlib import Path
from playwright.sync_api import sync_playwright
script = Path('/tmp/issue-sort-script.js').read_text()
project = 'shell-typed-invocation'
names = ['shell-dag-cron-entry-line-builder', 'shell-dag-floor-peak-calibration-rows', 'shell-dag-ci-fmt-gate-line', 'shell-dag-ci-deploy-invoke-prelude', 'shell-dag-ci-heal-regen-invoke', 'shell-dag-compile-pool-slice-install']
def row(node, dependency='', complete=False, owner=project):
    return f'<li class="node" tabindex="0" data-node="{node}" data-project="{owner}" data-complete="{str(complete).lower()}" data-dependencies="{dependency}" data-search="{node}"><span data-col="title" data-sort-value="{node}"><a class="issue-title-link" href="/issue/{node}">{node}</a></span></li>'
fixture = '<section class="roadmap-frontier"><input class="issue-search"><button class="issue-topological">Topo</button><button class="issue-sort-undo">Undo</button><span class="issue-sort-status" role="status"></span><div class="issue-table"><span><button data-sort-key="title">Title</button></span><ul>'
fixture += row(names[5], names[4]) + row('unrelated', owner='unrelated')
fixture += ''.join(row(name, names[i-1] if i else '', complete=True) for i,name in enumerate(names[:5]))
fixture += row(project, names[5]) + '</ul></div></section>'
with sync_playwright() as p:
    browser = p.chromium.launch(executable_path=os.environ['CHROMIUM_EXECUTABLE'], headless=True)
    page = browser.new_page(viewport={'width': 1100, 'height': 850})
    page.set_content(fixture)
    style = Path('/tmp/issue-sort-style.css')
    if style.exists(): page.add_style_tag(content=style.read_text())
    errors = []
    page.on('pageerror', lambda err: errors.append(str(err)))
    page.add_script_tag(content=script)
    order = lambda: page.locator('li').evaluate_all('(rows) => rows.map(r => r.dataset.node)')
    visible = lambda: page.locator('.issue-title-link:visible').evaluate_all('(links) => links.map(a => a.closest("li").dataset.node)')
    original = order()
    page.locator(f'[data-node="{names[5]}"] a').click()
    assert visible() == names[2:] + [project], visible()
    assert page.locator('.issue-focus-controls button').nth(1).inner_text() == '… 2 earlier'
    assert page.locator('.issue-focus-controls a').get_attribute('href') == '/issue/' + names[5]
    page.locator('.issue-focus-controls button').nth(1).click()
    assert visible() == names + [project]
    page.locator('.issue-sort-undo').click()
    assert visible() == names[2:] + [project]
    page.locator('[data-sort-key]').click()
    assert page.locator('.issue-topological').get_attribute('aria-pressed') == 'false'
    assert len(visible()) == 8
    page.locator('.issue-sort-undo').click()
    assert visible() == names[2:] + [project]
    page.locator('.issue-search').fill('compile-pool')
    assert visible() == [names[5]]
    page.locator('.issue-search').fill('')
    page.locator('.issue-focus-controls button').first.click()
    assert order() == original and len(visible()) == 8
    page.locator(f'[data-node="{names[5]}"] a').focus()
    page.keyboard.press('Enter')
    assert visible() == names[2:] + [project]
    page.locator('.issue-focus-controls button').first.click()
    page.locator(f'[data-node="{names[0]}"]').evaluate('(row, dependency) => row.dataset.dependencies = dependency', names[5])
    before = order()
    page.locator('.issue-topological').click()
    assert order() == before and 'cyclic' in page.locator('.issue-sort-status').inner_text()
    page.locator(f'[data-node="{names[0]}"]').evaluate('(row) => row.dataset.dependencies = ""')
    page.locator('[data-node="unrelated"]').evaluate('(row) => row.dataset.dependencies = "unrelated"')
    page.locator(f'[data-node="{names[5]}"] a').click()
    assert visible() == names[2:] + [project]
    page.locator('.issue-topological').click()
    assert page.locator('.issue-topological').get_attribute('aria-pressed') == 'false'
    assert len(visible()) == 8
    page.locator('.issue-sort-undo').click()
    assert visible() == names[2:] + [project]
    assert not errors, errors
    browser.close()
print('PASS emitted focus/sort: recursive project identity, recent three, expansion, upcoming, undo, search, keyboard, cycle refusal')
