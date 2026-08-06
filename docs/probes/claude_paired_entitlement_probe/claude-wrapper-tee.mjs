#!/usr/bin/env node
/**
 * SDK subprocess wrapper: forwards argv to the real Claude executable while
 * teeing stdout to GUNBC_PROBE_SDK_RAW_LOG (audit §4.1 — capture beneath parser).
 */
import { spawn } from 'node:child_process';
import { appendFileSync } from 'node:fs';

const real =
  process.env.GUNBC_PROBE_REAL_CLAUDE ??
  process.env.GUNBC_PROBE_CLAUDE_PATH ??
  'claude';
const logPath = process.env.GUNBC_PROBE_SDK_RAW_LOG;
if (!logPath) {
  console.error('GUNBC_PROBE_SDK_RAW_LOG is required');
  process.exit(2);
}

const child = spawn(real, process.argv.slice(2), {
  env: process.env,
  stdio: ['inherit', 'pipe', 'inherit'],
});

child.stdout.on('data', (chunk) => {
  appendFileSync(logPath, chunk);
  process.stdout.write(chunk);
});

child.on('error', (err) => {
  console.error(err);
  process.exit(1);
});

child.on('close', (code, signal) => {
  if (signal) process.kill(process.pid, signal);
  process.exit(code ?? 1);
});
