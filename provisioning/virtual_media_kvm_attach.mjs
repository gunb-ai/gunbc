#!/usr/bin/env node
// Headless WebUI-KVM virtual-media attach + boot-once-CD driver for OpenBMC boards
// whose Redfish VirtualMedia surface is absent (e.g. ASRockRack ALTRAD8UD-1L2T,
// srv3 @ 192.168.1.192). See provisioning/README-virtual-media-kvm.md.
//
// The grounded surface (selectors, route, NBD transport) is NOT hardcoded here:
// it is read from the drift-gated descriptor emitted from extdeps.bmc.webui_kvm
// (default provisioning/srv3/virtual-media-kvm.yaml). This file is the transport
// handler; the descriptor is the single authority.
//
// SAFETY: default is DRY-RUN. It logs in, maps the DOM read-only, and prints the
// attach + boot it WOULD perform, but never clicks Start and never fires boot.
// The real attach + boot only happen under --attach (operator-gated; the operator
// is lockout-sensitive about live BMC writes).

import { spawnSync } from 'node:child_process';
import { chromium } from 'playwright';
import fs from 'node:fs';
import path from 'node:path';
import https from 'node:https';
import http from 'node:http';

// ---- arg parsing ----------------------------------------------------------
function parseArgs(argv) {
  const a = {
    bmcHost: 'https://192.168.1.192',
    bmcUser: 'root',
    bmcPass: process.env.BMC_PASS || '0penBmc',
    isoUrl: 'http://192.168.1.188/ubuntu-24.04/ubuntu.iso',
    isoFile: null,
    descriptor: path.join(path.dirname(new URL(import.meta.url).pathname), 'srv3', 'virtual-media-kvm.yaml'),
    // default boot hook = neat-boar-71's srv3 Redfish boot-once-CD entry (PR #5750).
    // Fires ONLY under --attach; resolves on a checkout that has that entry on main.
    bootHookCommand: 'gunbc run --source-root dsl --entry dsl/gunbc/srv3_boot_once_cd.dag --function srv3_boot_once_cd',
    attach: false,
    waitSeconds: 2400,
    holdOnFailure: true,
    isoCache: '/tmp/gunbc-virtual-media',
  };
  for (let i = 2; i < argv.length; i++) {
    const k = argv[i];
    const next = () => argv[++i];
    switch (k) {
      case '--bmc-host': a.bmcHost = next(); break;
      case '--bmc-user': a.bmcUser = next(); break;
      case '--bmc-pass': a.bmcPass = next(); break;
      case '--iso-url': a.isoUrl = next(); break;
      case '--iso-file': a.isoFile = next(); break;
      case '--descriptor': a.descriptor = next(); break;
      case '--boot-hook-command': a.bootHookCommand = next(); break;
      case '--no-boot-hook': a.bootHookCommand = ''; break;
      case '--attach': a.attach = true; break;
      case '--wait-seconds': a.waitSeconds = parseInt(next(), 10); break;
      case '--no-hold-on-failure': a.holdOnFailure = false; break;
      case '--iso-cache': a.isoCache = next(); break;
      case '-h': case '--help': printHelp(); process.exit(0); break;
      default: fail(`unknown argument: ${k}`);
    }
  }
  return a;
}

function printHelp() {
  console.log(`virtual_media_kvm_attach.mjs — headless OpenBMC WebUI-KVM virtual-media attach

  Default = DRY-RUN (maps DOM, prints planned attach+boot, fires nothing).
  --attach                 actually attach the ISO and fire the boot hook (operator-gated)
  --bmc-host <url>         default https://192.168.1.192
  --bmc-user <u>           default root
  --bmc-pass <p>           default $BMC_PASS or 0penBmc
  --iso-url <url>          default http://192.168.1.188/ubuntu-24.04/ubuntu.iso
  --iso-file <path>        use an already-local ISO (skip download)
  --descriptor <path>      grounded surface YAML (default provisioning/srv3/virtual-media-kvm.yaml)
  --boot-hook-command <s>  command run synchronously after attach; exit 0 = boot issued
  --no-boot-hook           attach only; do not issue boot
  --wait-seconds <n>       after boot, keep NBD session alive this long (default 2400); SIGINT ends early
  --no-hold-on-failure     on boot-hook failure, tear down instead of holding media for inspection`);
}

// ---- tiny YAML reader for the fixed 2-level descriptor ---------------------
// The descriptor is drift-gated to a known flat/2-level shape (extdeps.bmc.webui_kvm
// -> yaml format authority); a minimal indent-based reader avoids a second npm dep.
function readDescriptor(file) {
  const text = fs.readFileSync(file, 'utf8');
  const root = {};
  let cur = root;
  for (const raw of text.split('\n')) {
    if (!raw.trim() || raw.trim().startsWith('#')) continue;
    const indent = raw.length - raw.trimStart().length;
    const line = raw.trim();
    const ci = line.indexOf(': ');
    if (ci === -1 && line.endsWith(':')) {
      const key = line.slice(0, -1);
      cur = (indent === 0) ? (root[key] = {}) : (root[Object.keys(root).pop()][key] = {});
      continue;
    }
    const key = line.slice(0, ci);
    let val = line.slice(ci + 2);
    if (val === 'true') val = true; else if (val === 'false') val = false;
    cur[key] = val;
  }
  return root;
}

function fail(msg) { console.error(`[virtual-media-kvm] FATAL: ${msg}`); process.exit(2); }
function log(msg) { console.log(`[virtual-media-kvm] ${msg}`); }

// ---- ISO fetch (streamed to local file; NBD needs a LOCAL file) ------------
function download(url, dest) {
  return new Promise((resolve, reject) => {
    const mod = url.startsWith('https') ? https : http;
    const file = fs.createWriteStream(dest);
    mod.get(url, (res) => {
      if (res.statusCode !== 200) { reject(new Error(`GET ${url} -> ${res.statusCode}`)); return; }
      const total = parseInt(res.headers['content-length'] || '0', 10);
      let got = 0, lastPct = -1;
      res.on('data', (c) => {
        got += c.length;
        const pct = total ? Math.floor((got / total) * 100) : -1;
        if (pct !== lastPct && pct % 10 === 0) { log(`  download ${pct}% (${got}/${total})`); lastPct = pct; }
      });
      res.pipe(file);
      file.on('finish', () => file.close(() => resolve({ bytes: got, total })));
    }).on('error', reject);
  });
}

async function resolveIso(a) {
  if (a.isoFile) {
    if (!fs.existsSync(a.isoFile)) fail(`--iso-file does not exist: ${a.isoFile}`);
    log(`using local ISO: ${a.isoFile} (${fs.statSync(a.isoFile).size} bytes)`);
    return a.isoFile;
  }
  fs.mkdirSync(a.isoCache, { recursive: true });
  const dest = path.join(a.isoCache, path.basename(new URL(a.isoUrl).pathname) || 'image.iso');
  if (fs.existsSync(dest) && fs.statSync(dest).size > 0) {
    log(`ISO already cached: ${dest} (${fs.statSync(dest).size} bytes)`);
    return dest;
  }
  log(`fetching ISO ${a.isoUrl} -> ${dest}`);
  const { bytes, total } = await download(a.isoUrl, dest);
  if (total && bytes !== total) fail(`ISO download truncated: ${bytes}/${total}`);
  log(`ISO fetched: ${bytes} bytes`);
  return dest;
}

// ---- boot hook (synchronous; exit code is the sole boot-success oracle) ----
function fireBootHook(cmd) {
  log(`boot hook (synchronous): ${cmd}`);
  const r = spawnSync('bash', ['-lc', cmd], { stdio: 'inherit' });
  if (r.status === 0) { log('boot hook exit 0 — boot-once-CD issued'); return true; }
  console.error(`[virtual-media-kvm] boot hook FAILED (exit ${r.status}). Aborting per fail-closed contract.`);
  return false;
}

function sleep(ms) { return new Promise(r => setTimeout(r, ms)); }

// ---- main ------------------------------------------------------------------
const a = parseArgs(process.argv);
if (!fs.existsSync(a.descriptor)) fail(`descriptor not found: ${a.descriptor}`);
const d = readDescriptor(a.descriptor).bmc_webui_virtual_media;
if (!d || !d.dom) fail(`descriptor missing bmc_webui_virtual_media.dom: ${a.descriptor}`);

log(`mode: ${a.attach ? 'ATTACH (live BMC writes)' : 'DRY-RUN (read-only; nothing fired)'}`);
log(`bmc=${a.bmcHost} route=${d.spa_route} transport=${d.transport} ws=${d.nbd_websocket_path}`);

const browser = await chromium.launch({ headless: true });
const ctx = await browser.newContext({ ignoreHTTPSErrors: true });
const page = await ctx.newPage();
let exitCode = 0;
try {
  // login
  await page.goto(`${a.bmcHost}/#/login`, { waitUntil: 'networkidle', timeout: 30000 });
  await page.fill(`#${d.dom.login_username_input_id}`, a.bmcUser);
  await page.fill(`#${d.dom.login_password_input_id}`, a.bmcPass);
  await page.click(d.dom.login_submit_selector);
  await page.waitForLoadState('networkidle', { timeout: 30000 });
  log(`logged in (post-login url ${page.url()})`);

  // navigate to virtual media
  await page.goto(`${a.bmcHost}${d.spa_route}`, { waitUntil: 'networkidle', timeout: 30000 });
  await page.getByRole('heading', { name: d.dom.attach_section_heading }).waitFor({ timeout: 30000 });
  await page.waitForSelector(`#${d.dom.attach_file_input_id}`, { state: 'attached', timeout: 30000 });
  const startBtn = page.locator('button.btn-primary', { hasText: d.dom.attach_start_button_label }).first();
  log(`virtual-media page mapped: file-input #${d.dom.attach_file_input_id}, Start button present (disabled=${await startBtn.isDisabled()})`);

  if (!a.attach) {
    // dry-run stays cheap: do NOT download the 2.85GB ISO; just probe reachability.
    const reach = a.isoFile ? `local ${a.isoFile} (${fs.existsSync(a.isoFile) ? 'present' : 'MISSING'})` : a.isoUrl;
    log(`DRY-RUN: would fetch ISO (${reach}), set file input #${d.dom.attach_file_input_id}, confirm Start enables, click Start.`);
    log(`DRY-RUN: would then run boot hook: ${a.bootHookCommand || '(no boot command configured)'}`);
    log('DRY-RUN complete — no ISO downloaded, no BMC write performed.');
  } else {
    const isoPath = await resolveIso(a);
    // ATTACH: select the local ISO and start the NBD stream
    await page.setInputFiles(`#${d.dom.attach_file_input_id}`, isoPath);
    await page.waitForTimeout(800);
    if (await startBtn.isDisabled()) fail('Start did not enable after selecting the ISO');
    log('clicking Start (begins NBD stream of the local ISO to the BMC)…');
    await startBtn.click();
    // confirm mounted: Start flips to Stop (label changes); poll up to 30s
    let mounted = false;
    for (let i = 0; i < 30; i++) {
      const stopVisible = await page.locator('button.btn-primary', { hasText: 'Stop' }).count();
      if (stopVisible > 0) { mounted = true; break; }
      await page.waitForTimeout(1000);
    }
    if (!mounted) fail('virtual media did not reach mounted state (Stop button never appeared)');
    log('virtual media MOUNTED (NBD session live; browser must stay alive while mounted)');

    // boot hook — synchronous, fail-closed
    if (a.bootHookCommand) {
      const ok = fireBootHook(a.bootHookCommand);
      if (!ok) {
        exitCode = 3;
        if (a.holdOnFailure) {
          log('boot failed: HOLDING NBD session (media stays attached) for inspection. SIGINT to release.');
          await new Promise((resolve) => process.on('SIGINT', resolve));
        }
        throw new Error('boot hook failed (fail-closed abort)');
      }
    } else {
      log('no boot hook configured: media attached, boot NOT issued (attach-only).');
    }

    // wait phase — keep NBD/browser alive while the host installs from the media
    log(`holding NBD session alive for up to ${a.waitSeconds}s (install reads the media). SIGINT to stop early.`);
    let interrupted = false;
    const onSig = () => { interrupted = true; };
    process.on('SIGINT', onSig);
    const deadline = Date.now() + a.waitSeconds * 1000;
    while (!interrupted && Date.now() < deadline) {
      await sleep(15000);
      log(`  …still mounted, ${Math.round((deadline - Date.now()) / 1000)}s remaining`);
    }
    log(interrupted ? 'SIGINT — stopping media' : 'wait elapsed — stopping media');
    const stopBtn = page.locator('button.btn-primary', { hasText: 'Stop' }).first();
    if (await stopBtn.count()) { await stopBtn.click(); await page.waitForTimeout(1500); }
    log('virtual media stopped/detached');
  }
} catch (e) {
  console.error(`[virtual-media-kvm] ERROR: ${e.message}`);
  if (exitCode === 0) exitCode = 1;
} finally {
  await browser.close();
}
process.exit(exitCode);
