# Codex device-auth prompt — captured receipt

`device_auth_prompt_2026-07-30.txt` is the **verbatim stdout+stderr** of

```
CODEX_HOME=/home/briansrls/.codex@gunbc-roadmap /usr/bin/codex login --device-auth
```

run on srv2 on 2026-07-30 against codex-cli **0.145.0**, captured before the
15-minute code expired. ANSI SGR sequences are retained exactly as emitted
(`ESC[94m` around the URL and the code, `ESC[90m` around the dim expiry note).

## Why this file exists

`codex login` has **no structured output mode**. `--help` lists only
`-c/--config`, `--with-api-key` and `--device-auth`; there is no `--json` on
either `login` or `login status`. So the URL and one-time code can only be read
from a text surface, and a parser over it is the honest realization — the richer
source does not exist.

That makes this capture load-bearing. A parser written against remembered or
re-typed output is untested against the bytes the tool emits, and the first
thing it gets wrong is the ANSI wrapping: the code is `ESC[94mYWCU-V8ESC ESC[0m`,
not `YWCU-V8ESC`, so a naive line-trim yields a code with escape bytes that
OpenAI rejects when pasted.

## Why `--device-auth` and not plain `codex login`

Plain `codex login` starts a callback server and prints
`redirect_uri=http://localhost:1455/auth/callback`. On a headless host that
redirect **can never resolve from the operator's browser** — localhost is the
server, not their machine. codex's own final line says so:

> On a remote or headless machine? Use `codex login --device-auth` instead.

The device flow needs no inbound port and no browser on the host — a URL and a
short code, entered anywhere — so it is the only flow a remote dashboard can
usefully surface.

## Non-goal

The code in this file is **expired and consumed**; it authenticates nothing.
It is retained as a parse fixture, not a credential, which is why it may live in
the repository.
