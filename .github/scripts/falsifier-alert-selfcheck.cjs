// EXECUTING CONTROL for .github/workflows/falsifier-alert.yml.
//
// It runs that workflow's github-script body against stubbed GitHub APIs and
// asserts the one property the alert exists to deliver: every artifact that
// NOTIFIES carries the owner mention. That property is invisible to YAML linting
// and to the workflow's own green conclusion — the alert ran successfully for two
// days while notifying nobody — so it needs a consumer that executes.
//
// Receipt for each scenario, against the workflow as it stood at 70b422f3c:
//   1 open-issue body mentions owner ................. PASS (already correct)
//   2 failing-set-changed comment mentions owner ..... FAIL
//   3 heartbeat comment mentions owner ............... FAIL
//   4 green-close comment mentions owner ............. FAIL
//   5 throttle holds when newest comment is fresh .... FAIL
// Scenarios 2-5 are therefore discriminating REDs, not decoration: reverting any
// one of the four fixes turns this file red. Scenario 1 is the positive control —
// the path that always worked — so a harness that broke wholesale is
// distinguishable from one reporting a real regression.
//
// It reads the SHIPPED YAML rather than a copy of the script, so the two cannot
// drift apart.
//
// 🟡 dissolve-on: gunbc.falsifier_workflow models workflow_run triggers + issue
// effects (the same trigger falsifier-alert.yml's own header carries). The alert
// becomes a typed effect row, these scenarios become ordinary .dag witnesses under
// dag/test/claim, and this file and its job delete together.
//
// Usage: node .github/scripts/falsifier-alert-selfcheck.cjs [<workflow-path>] [<git-ref>]
//   git-ref, when given, reads the workflow out of that ref instead of the
//   worktree — how the RED controls above were measured.
const fs = require('fs');
const path = require('path');
const cp = require('child_process');

const REPO = process.env.REPO || process.cwd();
const wfPath = process.argv[2] || '.github/workflows/falsifier-alert.yml';
const gitRef = process.argv[3] || null;

let raw;
if (gitRef) {
  raw = cp.execSync(`git show ${gitRef}:${wfPath}`, { cwd: REPO, encoding: 'utf8' });
} else {
  raw = fs.readFileSync(path.join(REPO, wfPath), 'utf8');
}
const lines = raw.split('\n');
const start = lines.findIndex((l) => l.includes('script: |'));
// Refuse rather than proceed on an empty script. A harness that silently tests
// nothing is the coverage-by-illusion this file exists to prevent, one level up.
if (start === -1) {
  console.error(`REFUSED: no 'script: |' block found in ${wfPath} — nothing to check.`);
  process.exit(2);
}
const scriptSrc = lines.slice(start + 1).map((l) => l.replace(/^ {12}/, '')).join('\n');
if (!/createComment/.test(scriptSrc)) {
  console.error(`REFUSED: extracted script from ${wfPath} contains no createComment call — extraction is wrong.`);
  process.exit(2);
}
const runScript = new Function('github', 'context', 'core', 'require', `return (async () => {\n${scriptSrc}\n})();`);

const RECEIPT = {
  schema: 'floor-component-receipt/v1',
  component_count: 8,
  affected_set_cold_control: {
    state: 'present',
    verified: false,
    component: { outcome: 'failed', error: 'boom', witnesses: 1 },
  },
  unsuccessful_components: [
    { index: 1, label: 'witness discovery', outcome: 'failed', error: 'boom' },
  ],
};

function makeEnv({ conclusion, existing, comments, receipt, signatureInBody }) {
  const calls = { create: [], update: [], comment: [] };
  // listComments is PAGE-ACCURATE: the real endpoint returns comments oldest-first
  // and honours per_page/page, so a bare call yields only the first page. Modelling
  // that is the whole point — a stub that hands back every comment regardless of
  // per_page cannot tell a paginated read from an unpaginated one, and scenario 5
  // would pass against both.
  const listComments = async (p) => {
    const per = p.per_page || 30;
    const page = p.page || 1;
    return { data: comments.slice((page - 1) * per, page * per) };
  };
  const github = {
    paginate: async (fn, params) => {
      const out = [];
      for (let page = 1; ; page++) {
        const r = await fn({ ...params, page });
        out.push(...r.data);
        if (r.data.length < (params.per_page || 30)) break;
      }
      return out;
    },
    rest: {
      issues: {
        listForRepo: async () => ({ data: existing ? [existing] : [] }),
        create: async (p) => { calls.create.push(p); return { data: { number: 1 } }; },
        update: async (p) => { calls.update.push(p); },
        listComments,
        createComment: async (p) => { calls.comment.push(p); },
      },
    },
  };
  const context = { repo: { owner: 'gunb-ai', repo: 'gunbc' }, payload: { workflow_run: {
    id: 999, conclusion, html_url: 'https://x/999', run_started_at: '2026-08-06T00:00:00Z',
  } } };
  const core = { info: () => {}, notice: () => {} };
  // The script reads the receipt off disk relative to cwd.
  const dir = fs.mkdtempSync('/tmp/falsifier-alert-');
  if (receipt) {
    fs.mkdirSync(path.join(dir, 'receipt'));
    fs.writeFileSync(path.join(dir, 'receipt/floor-component-receipt.json'), JSON.stringify(receipt));
  }
  process.chdir(dir);
  return { github, context, core, calls };
}

const MENTION = '@briansrls';
const results = [];
function check(name, pass, detail) {
  results.push({ name, pass, detail });
}

const bodyWithState = (sig) =>
  `some text\n<!-- falsifier-alert-state: ${JSON.stringify({ first_red: 'run 1', signature: sig })} -->`;

(async () => {
  // 1 — first red opens the issue.
  {
    const env = makeEnv({ conclusion: 'failure', existing: null, comments: [], receipt: RECEIPT });
    await runScript(env.github, env.context, env.core, require);
    const b = env.calls.create[0]?.body || '';
    check('1 open-issue body mentions owner', b.includes(MENTION), b.slice(0, 60));
  }

  // 2 — sustained red, signature CHANGED -> comment must mention.
  {
    const existing = { number: 7737, body: bodyWithState('OLD-SIGNATURE'), created_at: '2026-08-03T00:00:00Z' };
    const comments = [{ created_at: new Date(Date.now() - 3600 * 1000).toISOString() }];
    const env = makeEnv({ conclusion: 'failure', existing, comments, receipt: RECEIPT });
    await runScript(env.github, env.context, env.core, require);
    const b = env.calls.comment[0]?.body || '';
    check('2 failing-set-changed comment mentions owner', b.includes(MENTION), b.slice(0, 60));
  }

  // 3 — sustained red, signature UNCHANGED, last comment > 20h old -> heartbeat must mention.
  {
    const sig = '1:witness discovery:failed';
    const existing = { number: 7737, body: bodyWithState(sig), created_at: '2026-08-03T00:00:00Z' };
    const comments = [{ created_at: new Date(Date.now() - 25 * 3600 * 1000).toISOString() }];
    const env = makeEnv({ conclusion: 'failure', existing, comments, receipt: RECEIPT });
    await runScript(env.github, env.context, env.core, require);
    const b = env.calls.comment[0]?.body || '';
    check('3 heartbeat comment mentions owner', b.includes(MENTION), b.slice(0, 60));
  }

  // 4 — green close comment must mention.
  {
    const existing = { number: 7737, body: bodyWithState('x'), created_at: '2026-08-03T00:00:00Z' };
    const env = makeEnv({ conclusion: 'success', existing, comments: [], receipt: RECEIPT });
    await runScript(env.github, env.context, env.core, require);
    const b = env.calls.comment[0]?.body || '';
    check('4 green-close comment mentions owner', b.includes(MENTION), b.slice(0, 60));
  }

  // 5 — throttle across >100 comments: the NEWEST comment is 1h old, so a
  //     correct throttle stays silent. A single-page read sees the 100th
  //     (old) comment and heartbeats.
  {
    const sig = '1:witness discovery:failed';
    const existing = { number: 7737, body: bodyWithState(sig), created_at: '2026-08-03T00:00:00Z' };
    const comments = [];
    for (let i = 0; i < 130; i++) {
      const ageH = i < 100 ? 200 - i : 1; // first 100 are ancient, last 30 are fresh
      comments.push({ created_at: new Date(Date.now() - ageH * 3600 * 1000).toISOString() });
    }
    const env = makeEnv({ conclusion: 'failure', existing, comments, receipt: RECEIPT });
    await runScript(env.github, env.context, env.core, require);
    check('5 throttle holds when newest comment is fresh (>100 comments)',
      env.calls.comment.length === 0, `comments posted: ${env.calls.comment.length}`);
  }

  let bad = 0;
  for (const r of results) {
    if (!r.pass) bad++;
    console.log(`${r.pass ? 'PASS' : 'FAIL'}  ${r.name}${r.pass ? '' : `   [${r.detail}]`}`);
  }
  console.log(bad === 0 ? '\nALL GREEN' : `\n${bad} RED`);
  process.exit(bad === 0 ? 0 : 1);
})();
