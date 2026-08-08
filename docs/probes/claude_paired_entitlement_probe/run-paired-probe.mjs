#!/usr/bin/env node
/**
 * SCAFFOLD — audit §4 live paired entitlement probe runner (docs/probes/).
 * dissolve-on: gunbc.claude_agent_sdk_press enrolls the paired probe as a typed
 * WorkerTurn witness green-by-execution on an authenticated host — delete this Node
 * orchestration when that path replaces the hand runner.
 *
 * Section 4 paired entitlement probe (provider-control-interface-audit.md).
 *
 * Same Claude account state root · same model · same prompt:
 *   direct claude stream-json  versus  Claude Agent SDK query()
 *
 * Writes a machine-readable receipt JSON beside raw captures.
 * Does NOT assume either arm preserves subscription behaviour.
 */
import { createRequire } from 'node:module';
import { spawn } from 'node:child_process';
import {
  mkdirSync,
  writeFileSync,
  readFileSync,
  cpSync,
  existsSync,
} from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, '..', '..', '..');

const PROMPT =
  process.env.GUNBC_PROBE_PROMPT ??
  'Reply with exactly the single word ACK and nothing else.';
const MODEL =
  process.env.GUNBC_PROBE_MODEL ?? 'claude-haiku-4-5-20251001';
const SOURCE_HOME = process.env.GUNBC_PROBE_SOURCE_HOME ?? process.env.HOME;
const SDK_VERSION =
  process.env.GUNBC_PROBE_SDK_VERSION ?? '0.3.220';

function sha256(text) {
  return createHash('sha256').update(text, 'utf8').digest('hex');
}

function nowIso() {
  return new Date().toISOString();
}

function parseJsonl(text) {
  const events = [];
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    try {
      events.push(JSON.parse(trimmed));
    } catch (err) {
      events.push({ _parse_error: String(err), _raw: trimmed.slice(0, 500) });
    }
  }
  return events;
}

function summarizeEvents(events, label) {
  const typeCounts = {};
  const rateLimitEvents = [];
  const sessionIds = new Set();
  let terminal = null;
  let authFailure = false;
  let fableMention = false;
  let opusWarning = false;

  for (const ev of events) {
    const t = ev?.type ?? '_unknown';
    typeCounts[t] = (typeCounts[t] ?? 0) + 1;
    if (ev?.session_id) sessionIds.add(ev.session_id);
    if (t === 'rate_limit_event') rateLimitEvents.push(ev);
    if (t === 'result') terminal = ev;
    if (ev?.error === 'authentication_failed') authFailure = true;
    if (t === 'assistant' && ev?.error === 'authentication_failed') {
      authFailure = true;
    }
    if (
      t === 'result' &&
      typeof ev?.result === 'string' &&
      /not logged in/i.test(ev.result)
    ) {
      authFailure = true;
    }
  }

  const blob = JSON.stringify(events);
  if (/fable/i.test(blob)) fableMention = true;
  if (/opus/i.test(blob) && /warn|warning|allowed_warning/i.test(blob)) {
    opusWarning = true;
  }

  return {
    arm: label,
    event_count: events.length,
    type_counts: typeCounts,
    session_ids: [...sessionIds],
    rate_limit_events: rateLimitEvents,
    terminal_result: terminal,
    auth_failure_observed: authFailure,
    fable_mention_observed: fableMention,
    opus_warning_observed: opusWarning,
    raw_event_types_preserved: Object.keys(typeCounts),
  };
}

function buildStreamJsonUserInput(prompt) {
  return (
    JSON.stringify({
      type: 'user',
      message: { role: 'user', content: prompt },
    }) + '\n'
  );
}

function collectChildOutput(child, { stdinPayload } = {}) {
  return new Promise((resolve, reject) => {
    const stdoutChunks = [];
    const stderrChunks = [];
    child.stdout.on('data', (d) => stdoutChunks.push(d));
    child.stderr.on('data', (d) => stderrChunks.push(d));
    child.on('error', reject);
    if (stdinPayload !== undefined && child.stdin) {
      child.stdin.write(stdinPayload);
      child.stdin.end();
    }
    child.on('close', (code) => {
      resolve({
        exit_code: code,
        stdout: Buffer.concat(stdoutChunks).toString('utf8'),
        stderr: Buffer.concat(stderrChunks).toString('utf8'),
      });
    });
  });
}

async function runDirectCli({ probeDir, env, cwd, prompt }) {
  const outPath = join(probeDir, 'direct_cli.stdout.jsonl');
  const errPath = join(probeDir, 'direct_cli.stderr.txt');
  const stdinPayload = buildStreamJsonUserInput(prompt);
  const args = [
    '-p',
    '--output-format',
    'stream-json',
    '--verbose',
    '--input-format',
    'stream-json',
    '--permission-mode',
    'plan',
    '--model',
    MODEL,
  ];

  const child = spawn('claude', args, {
    env,
    cwd,
    stdio: ['pipe', 'pipe', 'pipe'],
  });
  const { exit_code, stdout, stderr } = await collectChildOutput(child, {
    stdinPayload,
  });
  writeFileSync(outPath, stdout, 'utf8');
  writeFileSync(errPath, stderr, 'utf8');
  return {
    arm: 'direct_claude_stream_json',
    exit_code,
    stdout_path: outPath,
    stderr_path: errPath,
    stdout_sha256: sha256(stdout),
    stdout_empty: stdout.length === 0,
    events: parseJsonl(stdout),
    argv: ['claude', ...args],
    stdin_format: 'stream-json user frame',
  };
}

async function runSdkQuery({ probeDir, env, cwd, sdkRoot, prompt, wrapperPath }) {
  const outPath = join(probeDir, 'sdk_parsed.stdout.jsonl');
  const errPath = join(probeDir, 'sdk_parsed.stderr.txt');
  const rawSubprocessPath = join(probeDir, 'sdk_subprocess_raw.stdout.jsonl');
  writeFileSync(rawSubprocessPath, '', 'utf8');

  const sdkEnv = {
    ...env,
    GUNBC_PROBE_SDK_RAW_LOG: rawSubprocessPath,
    GUNBC_PROBE_REAL_CLAUDE: 'claude',
  };

  const require = createRequire(join(sdkRoot, 'package.json'));
  const { query } = require('@anthropic-ai/claude-agent-sdk');

  const lines = [];
  let stderr = '';

  const origStderrWrite = process.stderr.write.bind(process.stderr);
  process.stderr.write = (chunk, ...rest) => {
    stderr += chunk.toString();
    return origStderrWrite(chunk, ...rest);
  };

  let exitCode = 0;
  try {
    for await (const message of query({
      prompt,
      options: {
        cwd,
        env: sdkEnv,
        model: MODEL,
        maxTurns: 1,
        permissionMode: 'plan',
        allowDangerouslySkipPermissions: false,
        pathToClaudeCodeExecutable: wrapperPath,
      },
    })) {
      lines.push(JSON.stringify(message));
    }
  } catch (err) {
    lines.push(
      JSON.stringify({
        type: 'gunbc_probe_adapter_error',
        message: err instanceof Error ? err.message : String(err),
      }),
    );
    exitCode = 2;
  } finally {
    process.stderr.write = origStderrWrite;
  }

  const stdout = lines.join('\n') + (lines.length ? '\n' : '');
  const rawSubprocess = readFileSync(rawSubprocessPath, 'utf8');
  writeFileSync(outPath, stdout, 'utf8');
  writeFileSync(errPath, stderr, 'utf8');

  return {
    arm: 'claude_agent_sdk_query',
    exit_code: exitCode,
    stdout_path: outPath,
    stderr_path: errPath,
    raw_subprocess_path: rawSubprocessPath,
    raw_subprocess_sha256: sha256(rawSubprocess),
    stdout_sha256: sha256(stdout),
    events: parseJsonl(stdout),
    raw_subprocess_events: parseJsonl(rawSubprocess),
    sdk_version: SDK_VERSION,
    sdk_module_root: sdkRoot,
  };
}

function extractNativeRateLimitTokens(events) {
  const tokens = [];
  for (const ev of events) {
    if (ev?.type !== 'rate_limit_event') continue;
    const info = ev.rate_limit_info ?? ev.rateLimitInfo ?? {};
    const token =
      info.rateLimitType ??
      info.rate_limit_type ??
      info.native_token ??
      null;
    tokens.push({
      status: info.status ?? null,
      rate_limit_type_native: token,
      resets_at: info.resetsAt ?? info.resets_at ?? null,
      raw_rate_limit_info: info,
    });
  }
  return tokens;
}

function compareArms(direct, sdk) {
  const d = summarizeEvents(direct.events, 'direct');
  const s = summarizeEvents(sdk.events, 'sdk');
  const raw = summarizeEvents(sdk.raw_subprocess_events ?? [], 'sdk_subprocess_raw');
  d.stdout_empty = direct.stdout_empty ?? false;
  s.stdout_empty = false;
  raw.stdout_empty = (sdk.raw_subprocess_events ?? []).length === 0;

  const directNativeTokens = extractNativeRateLimitTokens(direct.events);
  const sdkParsedTokens = extractNativeRateLimitTokens(sdk.events);
  const sdkRawTokens = extractNativeRateLimitTokens(sdk.raw_subprocess_events ?? []);

  const sameSession =
    d.session_ids.length > 0 &&
    raw.session_ids.length > 0 &&
    d.session_ids[0] === raw.session_ids[0];

  const directTypes = new Set(d.raw_event_types_preserved);
  const sdkParsedTypes = new Set(s.raw_event_types_preserved);
  const sdkRawTypes = new Set(raw.raw_event_types_preserved);
  const onlyDirect = [...directTypes].filter((t) => !sdkRawTypes.has(t));
  const onlySdkRaw = [...sdkRawTypes].filter((t) => !directTypes.has(t));
  const droppedBySdkParser = [...sdkRawTypes].filter(
    (t) => !sdkParsedTypes.has(t),
  );

  const directRateCount = d.rate_limit_events.length;
  const sdkRateCount = s.rate_limit_events.length;
  const sdkRawRateCount = raw.rate_limit_events.length;

  let entitlement_probe_verdict = 'inconclusive_auth_or_missing_entitlement_signal';
  if (direct.stdout_empty && !raw.stdout_empty && s.auth_failure_observed) {
    entitlement_probe_verdict = 'direct_arm_empty_sdk_subprocess_nonempty';
  } else if (d.auth_failure_observed && s.auth_failure_observed) {
    entitlement_probe_verdict =
      'both_arms_auth_failed_before_entitlement_surface';
  } else if (directRateCount > 0 || sdkRateCount > 0 || sdkRawRateCount > 0) {
    entitlement_probe_verdict =
      directRateCount === sdkRawRateCount
        ? 'rate_limit_event_count_matches_direct_vs_sdk_raw'
        : 'rate_limit_event_count_differs';
  } else if (
    !d.auth_failure_observed &&
    !s.auth_failure_observed &&
    d.terminal_result?.subtype === 'success' &&
    !d.terminal_result?.is_error &&
    s.terminal_result?.subtype === 'success' &&
    !s.terminal_result?.is_error
  ) {
    entitlement_probe_verdict = 'both_arms_terminal_success';
  }

  let direct_cli_dissolves = null;
  if (entitlement_probe_verdict === 'both_arms_terminal_success') {
    const typesMatch =
      onlyDirect.length === 0 &&
      onlySdkRaw.length === 0 &&
      directRateCount === sdkRawRateCount;
    direct_cli_dissolves = typesMatch;
  } else if (entitlement_probe_verdict.startsWith('both_arms_auth_failed')) {
    direct_cli_dissolves = null;
  }

  const failure_path_symmetry_note =
    entitlement_probe_verdict === 'both_arms_auth_failed_before_entitlement_surface'
      ? 'Symmetric auth failure only. Not entitlement-path parity: rate_limit_event emission never ran on either arm.'
      : null;

  return {
    direct_summary: d,
    sdk_parsed_summary: s,
    sdk_subprocess_raw_summary: raw,
    native_rate_limit_tokens: {
      direct_cli: directNativeTokens,
      sdk_parsed: sdkParsedTokens,
      sdk_subprocess_raw: sdkRawTokens,
      modeling_note:
        'ClaudeNativeRateLimitType should be NonEmptyStr at the carrier layer; recognised projections layer over the open token (audit §4.1).',
    },
    session_id_match_direct_vs_sdk_raw: sameSession,
    event_types_only_on_direct: onlyDirect,
    event_types_only_on_sdk_subprocess_raw: onlySdkRaw,
    event_types_dropped_by_sdk_parser: droppedBySdkParser,
    entitlement_probe_verdict,
    direct_cli_dissolves_for_realm:
      direct_cli_dissolves === null
        ? 'undecided_probe_did_not_reach_entitlement_surface'
        : direct_cli_dissolves,
    failure_path_symmetry_note,
    audit_section_4_note:
      'Outcome is execution evidence only. Neither subscription preservation nor CLI dissolution is assumed.',
  };
}

async function diagnoseCredentialBinding({ sourceHome, stateRoot, repoRoot }) {
  const stdinPayload = buildStreamJsonUserInput('ACK');
  const args = [
    '-p',
    '--output-format',
    'stream-json',
    '--verbose',
    '--input-format',
    'stream-json',
    '--permission-mode',
    'plan',
    '--model',
    MODEL,
  ];

  async function probeInit(home) {
    const env = { ...process.env, HOME: home };
    const child = spawn('claude', args, {
      env,
      cwd: repoRoot,
      stdio: ['pipe', 'pipe', 'pipe'],
    });
    const { stdout } = await collectChildOutput(child, { stdinPayload });
    const init = parseJsonl(stdout).find((ev) => ev?.type === 'system');
    const result = parseJsonl(stdout).find((ev) => ev?.type === 'result');
    return {
      api_key_source: init?.apiKeySource ?? null,
      auth_failure: stdout.includes('authentication_failed'),
      terminal_result: result?.result ?? null,
      stdout_sha256: sha256(stdout),
    };
  }

  const ambient = await probeInit(sourceHome);
  const copied = await probeInit(stateRoot);

  const sourceClaudeJson = join(sourceHome, '.claude.json');
  const copiedClaudeJson = join(stateRoot, '.claude.json');
  const claudeJsonShaMatch =
    existsSync(sourceClaudeJson) &&
    existsSync(copiedClaudeJson) &&
    sha256(readFileSync(sourceClaudeJson, 'utf8')) ===
      sha256(readFileSync(copiedClaudeJson, 'utf8'));

  let diagnosis = 'inconclusive';
  let diagnosis_detail =
    'Could not distinguish genuine unauthenticated host from lossy credential copy.';
  if (
    ambient.api_key_source === copied.api_key_source &&
    ambient.auth_failure === copied.auth_failure &&
    ambient.terminal_result === copied.terminal_result &&
    claudeJsonShaMatch
  ) {
    if (ambient.api_key_source === 'none' && ambient.auth_failure) {
      diagnosis = 'genuinely_unauthenticated_host';
      diagnosis_detail =
        'Ambient HOME and isolated file-tree copy produce identical apiKeySource=none and authentication_failed. .claude.json is bit-identical. Subscription profile metadata may be cached in .claude.json, but no active OAuth token is present in either binding — not a lossy-copy artifact.';
    } else if (ambient.api_key_source && ambient.api_key_source !== 'none') {
      diagnosis = 'authenticated_binding_reproduced_by_copy';
      diagnosis_detail =
        'Ambient and copied bindings agree on a non-none apiKeySource; file-tree copy preserved auth signals.';
    }
  } else if (claudeJsonShaMatch && ambient.api_key_source !== copied.api_key_source) {
    diagnosis = 'lossy_copy_suspected';
    diagnosis_detail =
      'Identical .claude.json but divergent apiKeySource between ambient HOME and copied state — credential likely has out-of-tree storage (keychain or other).';
  } else if (!claudeJsonShaMatch) {
    diagnosis = 'copy_incomplete_or_divergent';
    diagnosis_detail =
      'Copied credential_state .claude.json does not match source; probe copy step is lossy or stale.';
  }

  return {
    diagnosis,
    diagnosis_detail,
    ambient_home_probe: ambient,
    copied_state_probe: copied,
    claude_json_sha_match: claudeJsonShaMatch,
    claude_state_root_ref_denotes_whole_credential:
      diagnosis === 'lossy_copy_suspected'
        ? false
        : diagnosis === 'genuinely_unauthenticated_host'
          ? 'profile_metadata_file_resident_token_absent_in_observed_binding'
          : 'undecided_pending_authenticated_observation',
  };
}

function writeParserDropReceipt({ probeDir, comparison, probeReceiptPath, hostToolchain }) {
  const receipt = {
    receipt: 'claude_sdk_parser_drop',
    date: nowIso().slice(0, 10),
    authority: 'docs/plans/provider-control-interface-audit.md section 4.1',
    purpose:
      'Execution evidence that control_response is present in subprocess stream-json beneath SDK 0.3.220 and dropped by the SDK parser.',
    probe_parameters: {
      sdk_version: SDK_VERSION,
      claude_version: hostToolchain.claude_version,
      capture_method: 'pathToClaudeCodeExecutable wrapper teeing subprocess stdout',
      paired_probe_receipt: probeReceiptPath.split('/').pop(),
    },
    comparison: {
      sdk_subprocess_raw_event_types:
        comparison.sdk_subprocess_raw_summary.raw_event_types_preserved,
      sdk_parsed_event_types:
        comparison.sdk_parsed_summary.raw_event_types_preserved,
      event_types_dropped_by_sdk_parser:
        comparison.event_types_dropped_by_sdk_parser,
      event_types_only_on_sdk_subprocess_raw:
        comparison.event_types_only_on_sdk_subprocess_raw,
      parser_drop_verdict:
        comparison.event_types_dropped_by_sdk_parser.includes('control_response')
          ? 'control_response_present_in_subprocess_raw_absent_from_sdk_parsed'
          : 'control_response_not_observed',
      modeling_note: comparison.native_rate_limit_tokens.modeling_note,
    },
    captures_relative_to: probeDir,
    redaction_note: 'Derived from wet probe capture; no secrets included.',
  };
  const stablePath = join(here, 'claude_sdk_parser_drop_receipt.json');
  writeFileSync(stablePath, JSON.stringify(receipt, null, 2), 'utf8');
  return stablePath;
}

async function main() {
  const stamp = nowIso().replace(/[:.]/g, '-');
  const probeDir = join(here, `capture_${stamp}`);
  mkdirSync(probeDir, { recursive: true });

  const stateRoot = join(probeDir, 'credential_state');
  mkdirSync(stateRoot, { recursive: true });

  const sourceClaudeJson = join(SOURCE_HOME, '.claude.json');
  if (existsSync(sourceClaudeJson)) {
    cpSync(sourceClaudeJson, join(stateRoot, '.claude.json'));
  }
  if (existsSync(join(SOURCE_HOME, '.claude'))) {
    cpSync(join(SOURCE_HOME, '.claude'), join(stateRoot, '.claude'), {
      recursive: true,
    });
  }

  const credential_binding_diagnosis = await diagnoseCredentialBinding({
    sourceHome: SOURCE_HOME,
    stateRoot,
    repoRoot,
  });

  const env = {
    ...process.env,
    HOME: stateRoot,
    GUNBC_PROBE_EXPLICIT_CREDENTIAL_HOME: stateRoot,
  };

  const sdkInstallDir = join(probeDir, 'sdk-install');
  mkdirSync(sdkInstallDir, { recursive: true });
  writeFileSync(
    join(sdkInstallDir, 'package.json'),
    JSON.stringify(
      {
        name: 'gunbc-claude-paired-probe-sdk',
        private: true,
        dependencies: {
          '@anthropic-ai/claude-agent-sdk': SDK_VERSION,
        },
      },
      null,
      2,
    ),
    'utf8',
  );

  const npmCi = spawn('npm', ['install', '--no-audit', '--no-fund'], {
    cwd: sdkInstallDir,
    stdio: 'inherit',
    env,
  });
  await new Promise((resolve, reject) => {
    npmCi.on('close', (code) =>
      code === 0 ? resolve() : reject(new Error(`npm install exit ${code}`)),
    );
  });

  const cwd = repoRoot;
  const wrapperPath = join(here, 'claude-wrapper-tee.mjs');
  const direct = await runDirectCli({ probeDir, env, cwd, prompt: PROMPT });
  const sdk = await runSdkQuery({
    probeDir,
    env,
    cwd,
    sdkRoot: sdkInstallDir,
    prompt: PROMPT,
    wrapperPath,
  });
  const comparison = compareArms(direct, sdk);

  const host_toolchain = {
    node_version: process.version,
    claude_version: await new Promise((resolve) => {
      const p = spawn('claude', ['--version'], { env });
      let out = '';
      p.stdout.on('data', (d) => {
        out += d.toString();
      });
      p.on('close', () => resolve(out.trim()));
    }),
  };

  const receipt = {
    receipt: 'claude_paired_entitlement_probe',
    date: nowIso().slice(0, 10),
    authority: 'docs/plans/provider-control-interface-audit.md section 4',
    purpose:
      'Live paired probe: direct claude stream-json versus Claude Agent SDK on identical credential binding, model, and prompt.',
    probe_parameters: {
      prompt: PROMPT,
      model: MODEL,
      sdk_version: SDK_VERSION,
      explicit_credential_home: stateRoot,
      source_home_observed: SOURCE_HOME,
      cwd,
    },
    host_toolchain,
    credential_binding_diagnosis,
    arms: {
      direct_cli: {
        exit_code: direct.exit_code,
        stdout_sha256: direct.stdout_sha256,
        stdout_empty: direct.stdout_empty,
        argv: direct.argv,
        stdin_format: direct.stdin_format,
      },
      sdk_query: {
        exit_code: sdk.exit_code,
        stdout_sha256: sdk.stdout_sha256,
        raw_subprocess_sha256: sdk.raw_subprocess_sha256,
        sdk_module_root: sdk.sdk_module_root,
        subprocess_wrapper: wrapperPath,
      },
    },
    comparison,
    captures_relative_to: probeDir,
    redaction_note:
      'Raw captures may contain session identifiers. Secrets are not copied into this receipt.',
  };

  const receiptPath = join(here, `claude_paired_entitlement_probe_${stamp}.json`);
  writeFileSync(receiptPath, JSON.stringify(receipt, null, 2), 'utf8');
  writeParserDropReceipt({
    probeDir,
    comparison,
    probeReceiptPath: receiptPath,
    hostToolchain: host_toolchain,
  });
  console.log(receiptPath);
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
