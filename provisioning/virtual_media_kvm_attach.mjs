#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { chromium } from 'playwright';
import fs from 'node:fs';
import path from 'node:path';
import https from 'node:https';
import http from 'node:http';

function fail(msg) { console.error('[virtual-media-kvm] FATAL: ' + msg); process.exit(2); }
function log(msg) { console.log('[virtual-media-kvm] ' + msg); }
function sleep(ms) { return new Promise(function (r) { setTimeout(r, ms); }); }

function parseArgs(argv) {
  const a = {
    bmcHost: 'https://192.168.1.192',
    bmcUser: 'root',
    bmcPass: process.env.BMC_PASS || '0penBmc',
    isoUrl: 'http://192.168.1.188/ubuntu-24.04/ubuntu.iso',
    isoFile: null,
    bootHookCommand: 'gunbc run --source-root dsl --entry dsl/gunbc/srv3_boot_once_cd.dag --function srv3_boot_once_cd',
    attach: false,
    waitSeconds: 2400,
    holdOnFailure: true,
    isoCache: '/tmp/gunbc-virtual-media',
  };
  for (let i = 2; i < argv.length; i++) {
    const k = argv[i];
    const next = function () { i = i + 1; return argv[i]; };
    if (k === '--bmc-host') { a.bmcHost = next(); }
    else if (k === '--bmc-user') { a.bmcUser = next(); }
    else if (k === '--bmc-pass') { a.bmcPass = next(); }
    else if (k === '--iso-url') { a.isoUrl = next(); }
    else if (k === '--iso-file') { a.isoFile = next(); }
    else if (k === '--boot-hook-command') { a.bootHookCommand = next(); }
    else if (k === '--no-boot-hook') { a.bootHookCommand = ''; }
    else if (k === '--attach') { a.attach = true; }
    else if (k === '--wait-seconds') { a.waitSeconds = parseInt(next(), 10); }
    else if (k === '--no-hold-on-failure') { a.holdOnFailure = false; }
    else if (k === '--iso-cache') { a.isoCache = next(); }
    else { fail('unknown argument: ' + k); }
  }
  return a;
}

function download(url, dest) {
  return new Promise(function (resolve, reject) {
    const mod = url.startsWith('https') ? https : http;
    const file = fs.createWriteStream(dest);
    mod.get(url, function (res) {
      if (res.statusCode !== 200) { reject(new Error('GET ' + url + ' -> ' + res.statusCode)); return; }
      const total = parseInt(res.headers['content-length'] || '0', 10);
      let got = 0, lastPct = -1;
      res.on('data', function (c) {
        got += c.length;
        const pct = total ? Math.floor((got / total) * 100) : -1;
        if (pct !== lastPct && pct % 10 === 0) { log('  download ' + pct + '% (' + got + '/' + total + ')'); lastPct = pct; }
      });
      res.pipe(file);
      file.on('finish', function () { file.close(function () { resolve({ bytes: got, total: total }); }); });
    }).on('error', reject);
  });
}

async function resolveIso(a) {
  if (a.isoFile) {
    if (!fs.existsSync(a.isoFile)) fail('--iso-file does not exist: ' + a.isoFile);
    log('using local ISO: ' + a.isoFile + ' (' + fs.statSync(a.isoFile).size + ' bytes)');
    return a.isoFile;
  }
  fs.mkdirSync(a.isoCache, { recursive: true });
  const dest = path.join(a.isoCache, path.basename(new URL(a.isoUrl).pathname) || 'image.iso');
  if (fs.existsSync(dest) && fs.statSync(dest).size > 0) {
    log('ISO already cached: ' + dest + ' (' + fs.statSync(dest).size + ' bytes)');
    return dest;
  }
  log('fetching ISO ' + a.isoUrl + ' -> ' + dest);
  const r = await download(a.isoUrl, dest);
  if (r.total && r.bytes !== r.total) fail('ISO download truncated: ' + r.bytes + '/' + r.total);
  log('ISO fetched: ' + r.bytes + ' bytes');
  return dest;
}

function fireBootHook(cmd) {
  log('boot hook (synchronous): ' + cmd);
  const r = spawnSync('bash', ['-lc', cmd], { stdio: 'inherit' });
  if (r.status === 0) { log('boot hook exit 0 -- boot-once-CD issued'); return true; }
  console.error('[virtual-media-kvm] boot hook FAILED (exit ' + r.status + '). Aborting per fail-closed contract.');
  return false;
}

const a = parseArgs(process.argv);
log('mode: ' + (a.attach ? 'ATTACH (live BMC writes)' : 'DRY-RUN (read-only; nothing fired)'));
const browser = await chromium.launch({ headless: true });
const ctx = await browser.newContext({ ignoreHTTPSErrors: true });
const page = await ctx.newPage();
let exitCode = 0;
try {
  await page.goto(a.bmcHost + '/#/login', { waitUntil: 'networkidle', timeout: 30000 });
  await page.fill('#username', a.bmcUser);
  await page.fill('#password', a.bmcPass);
  await page.click('button[type="submit"]');
  await page.waitForLoadState('networkidle', { timeout: 30000 });
  log('logged in');
  await page.goto(a.bmcHost + '/#/operations/virtual-media', { waitUntil: 'networkidle', timeout: 30000 });
  await page.getByRole('heading', { name: 'Load image from web browser' }).waitFor({ timeout: 30000 });
  await page.waitForSelector('#virtual_media_device', { state: 'attached', timeout: 30000 });
  log('virtual-media page mapped');
  if (!a.attach) {
    log('DRY-RUN: would fetch ISO, set the file input, confirm Start enables, click Start.');
    log('DRY-RUN: would then run boot hook: ' + (a.bootHookCommand || '(no boot command configured)'));
    log('DRY-RUN complete -- no ISO downloaded, no BMC write performed.');
  } else {
    const isoPath = await resolveIso(a);
  await page.setInputFiles('#virtual_media_device', isoPath);
  await page.waitForTimeout(800);
  await page.click('button.btn-primary:has-text("Start")');
  await page.locator('button.btn-primary', { hasText: 'Stop' }).first().waitFor({ state: 'visible', timeout: 30000 });
    log('virtual media MOUNTED (NBD session live; browser must stay alive while mounted)');
    if (a.bootHookCommand) {
      const ok = fireBootHook(a.bootHookCommand);
      if (!ok) {
        exitCode = 3;
        if (a.holdOnFailure) { log('boot failed: HOLDING NBD session for inspection. SIGINT to release.'); await new Promise(function (resolve) { process.on('SIGINT', resolve); }); }
        throw new Error('boot hook failed (fail-closed abort)');
      }
    } else { log('no boot hook configured: media attached, boot NOT issued (attach-only).'); }
    log('holding NBD session alive for up to ' + a.waitSeconds + 's (install reads the media). SIGINT to stop early.');
    let interrupted = false;
    process.on('SIGINT', function () { interrupted = true; });
    const deadline = Date.now() + a.waitSeconds * 1000;
    while (!interrupted && Date.now() < deadline) { await sleep(15000); log('  ...still mounted, ' + Math.round((deadline - Date.now()) / 1000) + 's remaining'); }
    log(interrupted ? 'SIGINT -- stopping media' : 'wait elapsed -- stopping media');
    try { await page.click('button.btn-primary:has-text("Stop")'); await page.waitForTimeout(1500); } catch (e) { }
    log('virtual media stopped/detached');
  }
} catch (e) {
  console.error('[virtual-media-kvm] ERROR: ' + e.message);
  if (exitCode === 0) exitCode = 1;
} finally {
  await browser.close();
}
process.exit(exitCode);
