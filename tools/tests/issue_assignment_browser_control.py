"""Execute the emitted assignment client against lost-response fixture commands."""
import json
import os
from pathlib import Path
from playwright.sync_api import sync_playwright

script = Path('/tmp/issue-assignment-script.js').read_text()
with sync_playwright() as p:
    browser = p.chromium.launch(executable_path=os.environ['CHROMIUM_EXECUTABLE'], headless=True)
    page = browser.new_page()
    page.set_content('''<li class="node"><span><button class="issue-inline-assign" data-assign-node="fixture">--</button></span></li>
      <form class="issue-assign-form" action="/issue/fixture/assign">
      <input name="key" value="native-stable-key"><input name="expected_revision" value="original-head">
      <input name="assignee" value="person@example.invalid"><button type="submit">Assign</button></form>''')
    page.evaluate('''() => {
      window.gunbcSessionRead = Promise.resolve({authenticated: true});
      window.posts = []; window.contexts = 0; window.loseResponse = true;
      window.fetch = async (path, init) => {
        if (!init.method) return {ok: true, json: async () => ({key: 'new-' + (++contexts), csrf: 'fixture-csrf', expected_revision: 'new-head', options: [{value: 'person@example.invalid', label: 'Person'}, {value: 'other@example.invalid', label: 'Other'}]})};
        posts.push(Object.fromEntries(new URLSearchParams(init.body)));
        if (loseResponse) { loseResponse = false; throw new Error('fixture lost response'); }
        return {ok: true, text: async () => 'assigned: fixture'};
      };
    }''')
    page.add_script_tag(content=script)
    page.locator('form button').click()
    page.wait_for_function('posts.length === 1 && !document.querySelector("form button").disabled')
    page.locator('form button').click()
    page.wait_for_function('posts.length === 2 && !document.querySelector("form button").disabled')
    posts = page.evaluate('posts')
    assert posts[0] == posts[1], posts
    assert posts[0]['key'] == 'native-stable-key' and posts[0]['expected_revision'] == 'original-head'
    page.evaluate('loseResponse = true')
    page.locator('.issue-inline-assign').click()
    page.locator('select').select_option('person@example.invalid')
    page.wait_for_function('posts.length === 3 && !document.querySelector(".issue-inline-assign").disabled')
    page.locator('.issue-inline-assign').click()
    assert page.locator('select option[value="other@example.invalid"]').is_disabled()
    page.locator('select').select_option('person@example.invalid')
    page.wait_for_function('posts.length === 4 && !document.querySelector(".issue-inline-assign").disabled')
    posts = page.evaluate('posts')
    assert posts[2] == posts[3], posts
    assert page.locator('.issue-inline-assign').inner_text() == 'Person'
    browser.close()
print('PASS emitted client: native-form key retained and inline lost-response retry reuses exact command')
