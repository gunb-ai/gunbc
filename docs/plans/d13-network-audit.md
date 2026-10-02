# D13 Network audit — the selection rule and its citations

Input to D13 step (b) (`gunbc.plans.demand_engine_program`; rung drop `gunbc.rung_drop` `network_requirement_unrepresented_after_uses_cut`). This page holds the reasoning behind the `requires` clauses on `dag/extdeps` operations: the rule, the rulings, and a citation for each program or API class. **It holds no per-operation verdicts.** Each verdict is the operation's own `requires` clause, which is the single authority (DESIGN §3, §6). Read the population from the clauses themselves, e.g. `grep -rn 'requires ' dag/extdeps`. The per-row table used for review was posted on gunbc#12965 rather than committed.

## Selection rule (ruled by quiet-seal-543, 2026-10-01)

A `requires` clause states what a sandbox must GRANT, so the fail-closed direction is to over-declare. An operation is **Network** if executing it Wet **may** open a connection to a non-local endpoint. This replaces the first draft's "necessarily". The judgement is made from the operation's own declaration (transport kind + argv/endpoint) against the cited upstream behaviour of the program or API it invokes.

- **rest**: Network unless the declared endpoint is a unix socket or a fixed loopback address. Docker Engine binds `extdeps.docker.endpoint` `docker_default_endpoint`, a unix socket, so it is NotNetwork. The GCP metadata server (169.254.169.254) is link-local, not loopback, so it is Network.
- **file**: NotNetwork.
- **shell**: decided by program **and** authored argv:
  - A fixed network protocol to a host is Network: ssh/scp/sshpass, `ipmitool -I lanplus`, `curl https://…`, `gh`, `gcloud auth`, hosted-model CLIs.
  - A url/remote parameter is Network: `curl {url}`, `git fetch|push|ls-remote {remote}`, Playwright `goto {url}`, and `github.OIDC` `GetToken` with its runtime `request_url`. An operation name is not a type, so `http.Client` `GetLocalhostBounded` is Network until its `{url}` is typed loopback. That typing is the trigger that would move it.
  - A fetch that depends on cache state is Network: cargo without `--offline`, `npm ci`, `npm cache add`, `npx -y`, `apt-get install`, `gcloud auth print-access-token`.
  - `arping` is Network: it uses the network interface.
  - A runtime-program parameter is **OpaqueDemand**: `shell.exec` Run*, `systemd-run` with `command_argv`, the sudo probe `Check`, `gunbc.WitnessBin` `Run`, and the node/python/go `RunFile` runners. The argv is runtime data, so step (b) yields DemandUndecided for their callers instead of an empty demand.
  - **Runtime program in an argument** (ruling 2026-10-02): an argument that carries a program the invoked tool executes is **OpaqueDemand** -- scripts, expressions, inner argv, executing templates, and a caller-supplied program path in argv[0] (`codex_app_server` `GenerateJsonSchema`). This includes an unrefined string in option position that the tool accepts as a program-executing option (`git grep -O<pager>`). A runtime program whose language has no network or exec primitive (a jq filter; Rust source that rustc only compiles) does not raise the demand above its host program's.
  - **Remote-selecting option in an unrefined string** (ruling 2026-10-02): if an unrefined string sits where the tool parses options, and the tool has an option that selects a remote host or URL (`systemctl -H/--host=`, `hostnamectl -H`, curl `-K<config>`/`-x<proxy>`), the operation is **Network**. That holds unless the argv structurally prevents it, e.g. the string follows `--` or is the value of a preceding option.
  - Everything else is a local program on local paths: NotNetwork.
  - **Live browser page** (ruling 2026-10-02): an interaction with a running browser is **Network**. A click can navigate, page scripts run during any call, and browser startup can fetch. A program whose option surface cannot be verified (`playwright-runner` is not in this repository) is **OpaqueDemand** for every runtime positional it receives; it is never assumed none.
  - **Partial clone** (ruling 2026-10-02): a git read that needs blob or tree objects is **Network**, because in a partial clone git lazily fetches missing objects from the promisor remote, and whether a repository is a partial clone belongs to the repository the operation is pointed at, i.e. its input. Reads of commits, refs, the index or config only are not affected.
- **Repository-selected executable** (ruling 2026-10-02): if the invoked command executes a program selected by the input repository or its configuration, the operation is **OpaqueDemand**, unless the operation structurally disables that surface. This covers hooks, clean/smudge/process filters, diff/merge drivers, textconv, the credential helper, the fsmonitor hook, `core.sshCommand`, `remote.*.uploadpack`, a cargo build script or proc macro, `.cargo/config` aliases and runners, the `.npmrc` `git` executable, and agent hooks in a settings argument. Upstream surfaces, all git per git-scm.com/docs:
  - githooks(5): `pre-commit`, `prepare-commit-msg` (not suppressed by `--no-verify`), `commit-msg`, `post-commit`, `post-checkout` (checkout, worktree add), `pre-merge-commit`/`post-merge`, `pre-push`, `reference-transaction` (every ref update, including `update-ref`), `post-index-change` (every index write: add, read-tree, update-index, write-tree).
  - gitattributes(5): clean on add/status/diff-against-worktree/`hash-object` with a path; smudge on checkout/restore/reset/checkout-index/merge; diff drivers and textconv on patch output; merge drivers on merge and merge-tree.
  - git-config(1): `core.fsmonitor` on index refresh (status, add, commit, untracked scans); `credential.helper`, `core.sshCommand`, `remote.<name>.uploadpack`/`receivepack` on every transport (fetch, push, ls-remote).
  - Cargo: build scripts and proc macros (doc.rust-lang.org/cargo/reference/build-scripts.html), and user aliases that shadow external subcommands such as `fmt` (doc.rust-lang.org/cargo/reference/config.html#alias).
  - npm: the `git` config key (docs.npmjs.com/cli/using-npm/config#git) for git dependencies.
  - Claude Code: hooks in `--settings` (docs.anthropic.com/claude-code/hooks).
  - Agent CLIs (`codex exec`, and `claude -p` with a runtime permission mode) execute model-chosen commands, which makes them runtime programs.

  - Index reads (ruling 2026-10-02, no special case): any git command that reads the index can run the `core.fsmonitor` hook on the first index load (read-cache `tweak_fsmonitor`). That covers plain `ls-files`, and also every unrefined revision argument, because a revision may be spelled `:<path>` or `:<stage>:<path>`, which reads the index (gitrevisions(7)). So rev-parse/show/cat-file/ls-tree/merge-base/reflog/rev-list/commit-tree/diff with a runtime revision are opaque.
  - Toolchain proxies in an input directory (ruling 2026-10-02): rustup selects the toolchain from `rust-toolchain.toml` in the working directory (rust-lang.github.io/rustup/overrides.html). npx reads the project `.npmrc` (e.g. `node-options`). So an operation that runs these in a cwd or repository it takes as input is opaque. The same programs run with an ambient cwd (`cargo --version`, `rustc --version`) fall under the host-environment boundary.

  Git operations that reach none of these surfaces keep their transport verdict. They have fixed revisions and no index read: rev-parse of `HEAD`/`--show-toplevel`, `rev-list --before=… HEAD`, `for-each-ref`, `worktree list`, `branch -r`, `config`, `init`, `hash-object --stdin`, and `log -- {path}` (Network: partial clone). Paging is not reached because stdout is captured, not a TTY.
- **Boundary — host environment is not demand** (ruling 2026-10-02): host-wide configuration (global/system git config, `~/.ssh/config`, rustup toolchain selection from an ambient cwd) and host-wide resolver configuration (NSS, which can route `id`/`whoami`/`stat %U` to LDAP; DNS) is the sandbox's environment, not an operation's demand, and does not raise a clause.

## Citations by class

| verdict → clause | citation | modules |
|---|---|---|
| Network → `requires Network` | Cloudflare API v4 (developers.cloudflare.com/api) | `extdeps.cloudflare.account_api_tokens`, `extdeps.cloudflare.r2_buckets` |
| Network → `requires Network` | GCP metadata server (cloud.google.com/compute/docs/metadata/overview): metadata.google.internal = 169.254.169.254, link-local, not loopback | `extdeps.cloud.gcp.sts` |
| Network → `requires Network` | GitHub REST API docs (docs.github.com/rest), base https://api.github.com | `extdeps.github.actions_jit_runner`, `extdeps.github.app`, `extdeps.github.checks`, `extdeps.github.code_search`, `extdeps.github.commits`, `extdeps.github.gists`, `extdeps.github.git_database`, `extdeps.github.issues`, `extdeps.github.org_actions`, `extdeps.github.pulls`, `extdeps.github.repository_contents`, `extdeps.github.rulesets`, `extdeps.github.users`, `extdeps.github.workflow_runs`, `extdeps.github.workflows` |
| Network → `requires Network` | Google Drive API v3 reference (developers.google.com/drive/api/reference/rest/v3) | `extdeps.google.drive` |
| Network → `requires Network` | Google Sheets API v4 reference (developers.google.com/sheets/api/reference/rest) | `extdeps.google.sheets` |
| Network → `requires Network` | PARAM remote: git-fetch(1)/git-push(1)/git-ls-remote(1) use the remote's transport (gitprotocol-v2(5)); a {remote} may be a local path (file transport) or a URL; operations run in a repository are now opaque (repository-selected credential helper/sshCommand/uploadpack/pre-push/reference-transaction); the Network clause remains only where none of those surfaces is reached | `extdeps.git`, `extdeps.git.publication_transport` |
| Network → `requires Network` | PARAM request_url: endpoint is the runtime ACTIONS_ID_TOKEN_REQUEST_URL (docs.github.com/actions/reference/security/oidc); declaration fixes no host | `extdeps.cloud.gcp.sts` |
| Network → `requires Network` | PARAM tarball: npm cache add (docs.npmjs.com/cli/commands/npm-cache) accepts a path or a URL | `extdeps.tools.npm` |
| Network → `requires Network` | PARAM url: Playwright page.goto (playwright.dev/docs/api/class-page#page-goto) navigates to a runtime URL | `extdeps.browser` |
| Network → `requires Network` | PARAM url: curl(1) URL scheme+host both runtime (could be file:// or loopback) | `extdeps.http.client` |
| Network → `requires Network` | RULING link-layer: arping(8) broadcasts ARP on {device} to {target}; L2 traffic, no connection to an endpoint | `extdeps.iputils.arping` |
| Network → `requires Network` | SEC EDGAR APIs (sec.gov/search-filings/edgar-application-programming-interfaces) | `extdeps.sec.edgar_rest` |
| Network → `requires Network` | STATE credential cache: gcloud auth print-access-token (cloud.google.com/sdk/gcloud/reference/auth/print-access-token) refreshes over oauth2.googleapis.com only when the cached token is expired | `extdeps.shell` |
| Network → `requires Network` | STATE package cache: apt-get(8) install downloads .debs from sources.list unless already installed/cached; PARAM package | `extdeps.apt` |
| Network → `requires Network` | STATE package cache: npm ci (docs.npmjs.com/cli/commands/npm-ci) fetches from the registry unless every tarball is cached; superseded for `npm.Ci` by the repository-selected executable rule (`.npmrc` `git`): opaque | `extdeps.tools.npm` |
| Network → `requires Network` | STATE package cache: npx -y -p (docs.npmjs.com/cli/commands/npx) installs the package from the registry unless already cached | `extdeps.typescript` |
| Network → `requires Network` | STATE registry cache: cargo build/check/test (doc.rust-lang.org/cargo/commands/cargo-build.html, --offline/--frozen) fetch the index/crates only when Cargo.lock deps are not already downloaded; argv does not pass --offline; superseded for `cargo.Build` by the repository-selected executable rule (build scripts, proc macros): opaque | `extdeps.cargo_build` |
| Network → `requires Network` | TCGplayer API (docs.tcgplayer.com), base https://api.tcgplayer.com | `extdeps.tcgplayer.catalog`, `extdeps.tcgplayer.pricing`, `extdeps.tcgplayer.store`, `extdeps.tcgplayer.tcgplayer` |
| Network → `requires Network` | curl(1) to fixed https://api.github.com (GitHub Apps REST docs.github.com/rest/apps) | `extdeps.github.app` |
| Network → `requires Network` | curl(1) to https://{bmc_host}: DMTF Redfish DSP0266 / MegaRAC web API over HTTPS — argv fixes the https scheme to a BMC host | `extdeps.bmc.http`, `extdeps.bmc.megarac` |
| Network → `requires Network` | eBay REST APIs (developer.ebay.com/api-docs), base https://api.ebay.com | `extdeps.ebay.browse`, `extdeps.ebay.inventory`, `extdeps.ebay.oauth` |
| Network → `requires Network` | fixed HTTPS endpoint https://api.anthropic.com (upstream API reference for that host) | `extdeps.llm.anthropic_rest` |
| Network → `requires Network` | fixed HTTPS endpoint https://api.openai.com (upstream API reference for that host) | `extdeps.llm.openai_rest` |
| Network → `requires Network` | fixed HTTPS endpoint https://api.tailscale.com (upstream API reference for that host) | `extdeps.tailscale.acl_api` |
| Network → `requires Network` | fixed HTTPS endpoint https://cloudresourcemanager.googleapis.com (upstream API reference for that host) | `extdeps.cloud.gcp.iam` |
| Network → `requires Network` | fixed HTTPS endpoint https://iam.googleapis.com (upstream API reference for that host) | `extdeps.cloud.gcp.iam`, `extdeps.cloud.gcp.iam_admin` |
| Network → `requires Network` | fixed HTTPS endpoint https://iamcredentials.googleapis.com (upstream API reference for that host) | `extdeps.cloud.gcp.iam` |
| Network → `requires Network` | fixed HTTPS endpoint https://jsonplaceholder.typicode.com (upstream API reference for that host) | `extdeps.test.http_pilot` |
| Network → `requires Network` | fixed HTTPS endpoint https://oauth2.googleapis.com (upstream API reference for that host) | `extdeps.cloud.gcp.gcp` |
| Network → `requires Network` | fixed HTTPS endpoint https://secretmanager.googleapis.com (upstream API reference for that host) | `extdeps.cloud.gcp.secret_manager` |
| Network → `requires Network` | fixed HTTPS endpoint https://serviceusage.googleapis.com (upstream API reference for that host) | `extdeps.cloud.gcp.serviceusage` |
| Network → `requires Network` | fixed HTTPS endpoint https://sts.googleapis.com (upstream API reference for that host) | `extdeps.cloud.gcp.sts` |
| Network → `requires Network` | gcloud auth login (cloud.google.com/sdk/gcloud/reference/auth/login): OAuth flow against accounts.google.com | `extdeps.cloud.gcp.gcp` |
| Network → `requires Network` | gh(1) manual (cli.github.com/manual): `gh api`/`gh pr`/`gh run` call api.github.com | `extdeps.github.actions_runs`, `extdeps.github.ci_runner`, `extdeps.github.org_actions`, `extdeps.github.organizations`, `extdeps.github.pulls` |
| Network → `requires Network` | ipmitool(1) INTERFACES: -I lanplus = IPMI v2.0 RMCP+ over UDP/623 to -H {bmc_host} | `extdeps.bmc.ipmi` |
| Network → `requires Network` | ssh(1)/scp(1) (+sshpass(1)): opens TCP/22 session to the host named in argv | `extdeps.bmc.openbmc_password_ssh_transport`, `extdeps.ssh.password_session`, `extdeps.ssh.session` |
| Network → `requires Network` | vendor CLI prompt mode calls the hosted model API (Claude Code docs.anthropic.com/claude-code/cli-reference; Codex CLI `exec` github.com/openai/codex; Gemini CLI github.com/google-gemini/gemini-cli); `claude.Invoke.Run` (hooks in `--settings`), `llm.Anthropic.CliPrompt` (runtime permission mode) and `llm.Codex.Review` (agent exec in `-C {cwd}`) are opaque | `extdeps.llm.anthropic_rest`, `extdeps.llm.cli` |
| NotNetwork → `requires none` | /usr/bin/stat: local coreutils/POSIX utility (man stat(1)); argv names only local paths/values | `extdeps.tools.stat` |
| NotNetwork → `requires none` | Docker Engine API (docs.docker.com/reference/api/engine): default endpoint unix:///var/run/docker.sock (extdeps.docker.endpoint docker_default_endpoint), a unix socket | `extdeps.docker.container_inspect`, `extdeps.docker.container_stats` |
| Network → `requires Network` | RULING live page: Playwright Page/BrowserContext API (playwright.dev/docs/api/class-page); a running page and browser startup may fetch (`Launch`, `CurrentUrl`, `Title`) | `extdeps.browser` |
| OpaqueDemand → `requires opaque` | RULING unverifiable surface: `playwright-runner` is not in this repository, so a runtime positional (selector, text, path, ms, context) may reach an unknown option | `extdeps.browser` |
| OpaqueDemand → `requires opaque` | RULING runtime program: Playwright page.evaluate / locator.evaluate (playwright.dev/docs/evaluating) run runtime JavaScript in the page, which can call fetch | `extdeps.browser` |
| NotNetwork → `requires none` | cargo(1) --version / cargo-fmt: no registry access (doc.rust-lang.org/cargo/commands); `cargo fmt` is now opaque (a workspace alias can shadow the external subcommand); `--version` stays none | `extdeps.cargo_build` |
| NotNetwork → `requires none` | cat: local coreutils/POSIX utility (man cat(1)); argv names only local paths/values | `extdeps.linux.cgroup_v2`, `extdeps.linux.procfs` |
| NotNetwork → `requires none` | chmod: local coreutils/POSIX utility (man chmod(1)); argv names only local paths/values | `extdeps.shell` |
| OpaqueDemand → `requires opaque` | RULING runtime program: argv[0] is the caller-supplied `CodexAppServerExecutable.path`, so the executed program is runtime-selected (codex app-server generate-json-schema itself writes schema files locally, per the github.com/openai/codex app-server README) | `extdeps.llm.codex_app_server` |
| NotNetwork → `requires none` | cp: local coreutils/POSIX utility (man cp(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | crontab(1): local spool | `extdeps.cron` |
| Network → `requires Network` | RULING remote option: curl(1) --unix-socket, but the `{url}` positional is unrefined and not after `--`, so it can carry curl options (`-K<config>`, `-x<proxy>`) that retarget the connection | `extdeps.http.client` |
| NotNetwork → `requires none` | date: local coreutils/POSIX utility (man date(1)); argv names only local paths/values | `extdeps.clock` |
| NotNetwork → `requires none` | diff: local coreutils/POSIX utility (man diff(1)); argv names only local paths/values | `extdeps.tools.diffutils` |
| NotNetwork → `requires none` | dpkg(1): local status database | `extdeps.dpkg` |
| NotNetwork → `requires none` | find: local coreutils/POSIX utility (man find(1)); argv names only local paths/values | `extdeps.linux.cgroup_v2`, `extdeps.shell` |
| NotNetwork → `requires none` | getconf: local coreutils/POSIX utility (man getconf(1)); argv names only local paths/values | `extdeps.posix.getconf` |
| NotNetwork → `requires none` | git(1); local subcommand per git-scm.com/docs reading only commits, refs, the index or config (no transport; only fetch/push/ls-remote/clone/pull use git transfer protocols, gitprotocol-v2(5)), and running no hook, filter, driver or fsmonitor surface | `extdeps.git`, `extdeps.git.inspect`, `extdeps.git.plumbing`, `extdeps.git.publication_transport` |
| Network → `requires Network` | RULING partial clone: git-partial-clone (git-scm.com/docs/partial-clone) lazily fetches missing blobs/trees from the promisor remote; diff, show, cat-file, ls-tree, log -- path, read-tree, checkout-index, unpack-file, merge, merge-tree, checkout, restore, reset --hard, worktree add; any of these that also runs a repository-selected executable is opaque instead | `extdeps.git`, `extdeps.git.inspect`, `extdeps.git.plumbing` |
| OpaqueDemand → `requires opaque` | RULING runtime program: git-grep(1) `-O<pager>` opens matches with a runtime program; `{pattern}` and `{ref}` (GitRef is only non-empty) sit in option position | `extdeps.git.inspect` |
| NotNetwork → `requires none` | grep: local coreutils/POSIX utility (man grep(1)); argv names only local paths/values | `extdeps.tools.grep` |
| NotNetwork → `requires none` | gunbc file transport: local filesystem read/write (POSIX open(2), read(2)) | `extdeps.filesystem.filesystem_io`, `extdeps.linux.procfs` |
| NotNetwork → `requires none` | gzip: local coreutils/POSIX utility (man gzip(1)); argv names only local paths/values | `extdeps.tools.gzip` |
| NotNetwork → `requires none` | hostname: local coreutils/POSIX utility (man hostname(1)); argv names only local paths/values | `extdeps.tools.hostname` |
| Network → `requires Network` | RULING remote option: hostnamectl(1) `-H/--host=` runs over SSH; `Process.Run` forwards an unrestricted ProcessArgvExpansion | `extdeps.tools.hostname` |
| NotNetwork → `requires none` | id: local coreutils/POSIX utility (man id(1)); argv names only local paths/values | `extdeps.shell`, `extdeps.tools.id` |
| NotNetwork → `requires none` | ip-address(8): rtnetlink to the local kernel | `extdeps.iproute2.ip_address` |
| NotNetwork → `requires none` | journalctl(1): reads the local journal | `extdeps.systemd.journalctl` |
| NotNetwork → `requires none` | jq(1) manual: filter over stdin/args | `extdeps.bmc.openbmc_fan_control`, `extdeps.tools.jq` |
| NotNetwork → `requires none` | kill: local coreutils/POSIX utility (man kill(1)); argv names only local paths/values | `extdeps.posix.signal` |
| NotNetwork → `requires none` | ln: local coreutils/POSIX utility (man ln(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | mkdir: local coreutils/POSIX utility (man mkdir(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | mktemp: local coreutils/POSIX utility (man mktemp(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | mv: local coreutils/POSIX utility (man mv(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | node: local coreutils/POSIX utility (man node(1)); argv names only local paths/values | `extdeps.tools.node` |
| NotNetwork → `requires none` | npm --version (docs.npmjs.com/cli/commands/npm) | `extdeps.tools.npm` |
| NotNetwork → `requires none` | npm ci --offline (docs.npmjs.com/cli/using-npm/config#offline): forces cache-only, no network; superseded by the repository-selected executable rule: opaque | `extdeps.tools.npm` |
| NotNetwork → `requires none` | nvidia-smi(1): local NVML | `extdeps.nvidia.system_management_interface` |
| NotNetwork → `requires none` | oomctl(1): local systemd-oomd | `extdeps.systemd.oomd` |
| NotNetwork → `requires none` | openssl-dgst(1): local signing | `extdeps.tools.openssl` |
| NotNetwork → `requires none` | printenv: local coreutils/POSIX utility (man printenv(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | ras-mc-ctl(8): local EDAC sysfs/rasdaemon DB | `extdeps.linux.edac` |
| NotNetwork → `requires none` | realpath: local coreutils/POSIX utility (man realpath(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | rm: local coreutils/POSIX utility (man rm(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | rmdir: local coreutils/POSIX utility (man rmdir(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | rustc --version / local compilation (doc.rust-lang.org/rustc) | `extdeps.rustc` |
| NotNetwork → `requires none` | rustfmt: local coreutils/POSIX utility (man rustfmt(1)); argv names only local paths/values | `extdeps.tools.rustfmt` |
| OpaqueDemand → `requires opaque` | RULING runtime program: GNU sed's `e` command and `s///e` execute shell commands; the operations take a runtime sed program without `--sandbox` | `extdeps.tools.sed` |
| NotNetwork → `requires none` | sh -c script authored in the declaration uses only local programs (mktemp(1), find(1), sort(1), head(1)/tr(1) over /dev/urandom, rustc --emit=metadata, command -v) | `extdeps.entropy`, `extdeps.rustc`, `extdeps.shell` |
| NotNetwork → `requires none` | sha256sum: local coreutils/POSIX utility (man sha256sum(1)); argv names only local paths/values | `extdeps.crypto.hash`, `extdeps.tools.sha256sum` |
| NotNetwork → `requires none` | sha512sum: local coreutils/POSIX utility (man sha512sum(1)); argv names only local paths/values | `extdeps.tools.sha512sum` |
| NotNetwork → `requires none` | sleep: local coreutils/POSIX utility (man sleep(1)); argv names only local paths/values | `extdeps.tools.sleep` |
| NotNetwork → `requires none` | stat: local coreutils/POSIX utility (man stat(1)); argv names only local paths/values | `extdeps.shell`, `extdeps.tools.coreutils_stat` |
| NotNetwork → `requires none` | sudo(8) -l: local policy | `extdeps.sudo.nopasswd_execute_probe_check_op` |
| NotNetwork → `requires none` | systemctl(1): talks to the local systemd manager over D-Bus/private socket; only operations with no unrefined positional string (`DaemonReload`) | `extdeps.systemd.systemctl` |
| Network → `requires Network` | RULING remote option: systemctl(1) `-H/--host=` runs over SSH; the unit/pattern positional is an unrefined NonEmptyStr with no `--` before it | `extdeps.systemd.systemctl` |
| NotNetwork → `requires none` | tailscale CLI `serve status` reads the local tailscaled LocalAPI socket (tailscale.com/kb/1242/tailscale-serve) | `extdeps.tailscale.serve` |
| NotNetwork → `requires none` | test: local coreutils/POSIX utility (man test(1)); argv names only local paths/values | `extdeps.linux.cgroup_v2`, `extdeps.shell` |
| NotNetwork → `requires none` | tmux(1): local server socket (`List`, `Kill`) | `extdeps.tmux` |
| OpaqueDemand → `requires opaque` | RULING runtime program: tmux new-session runs `inner_argv`, a runtime command | `extdeps.tmux` |
| NotNetwork → `requires none` | uname: local coreutils/POSIX utility (man uname(1)); argv names only local paths/values | `extdeps.shell` |
| NotNetwork → `requires none` | wc: local coreutils/POSIX utility (man wc(1)); argv names only local paths/values | `extdeps.tools.wc` |
| NotNetwork → `requires none` | whoami: local coreutils/POSIX utility (man whoami(1)); argv names only local paths/values | `extdeps.access.posix_effective_principal_read_op` |
| NotNetwork → `requires none` | xorriso: local coreutils/POSIX utility (man xorriso(1)); argv names only local paths/values | `extdeps.tools.xorriso` |
| OpaqueDemand → `requires opaque` | PARAM bin_path: program is runtime | `extdeps.gunbc` |
| OpaqueDemand → `requires opaque` | PARAM command_argv: systemd-run(1) itself is local; the transient unit runs a runtime command | `extdeps.systemd.systemd_run` |
| OpaqueDemand → `requires opaque` | PARAM probe.command_path: sudo(8) runs a runtime program | `extdeps.sudo.nopasswd_execute_probe_check_op` |
| OpaqueDemand → `requires opaque` | PARAM program/command body: argv supplied at runtime | `extdeps.shell.exec` |
| OpaqueDemand → `requires opaque` | PARAM script_path: behaviour is the runtime script | `extdeps.go`, `extdeps.node`, `extdeps.python` |

## Follow-ups (better modeling than the clauses above; not done here)

- Repository-selected executables in git, by structural disabling. Each item has its citation, and each would move the affected operations back to their transport verdict:
  - `-c core.hooksPath=/dev/null` disables every hook, including `prepare-commit-msg` and `reference-transaction` (git-config `core.hooksPath`; githooks(5)). `--no-verify` alone is insufficient.
  - `-c core.fsmonitor=false` (git-config `core.fsmonitor`) disables the fsmonitor hook on every index read. `--no-optional-locks` does not: it only skips the opportunistic index write (git(1)).
  - Revision arguments: a refined revision type that excludes the `:<path>` index forms, or `--end-of-options` plus a full object id, removes the index read from rev-parse/show/cat-file/ls-tree/merge-base/rev-list.
  - Toolchain proxies: pin the toolchain explicitly (`cargo +<toolchain>`, `RUSTUP_TOOLCHAIN`) and run npx with `--userconfig`/an isolated cwd, or the operation stays opaque.
  - `--no-textconv --no-ext-diff` on diff (git-diff(1)).
  - `hash-object --no-filters` (git-hash-object(1)).
  - Overriding the filter and merge-driver surfaces needs per-driver `-c filter.<driver>.*`/`merge.<driver>.driver` overrides, which are unknown in advance. A typed attribute-free checkout realization is the structural route; otherwise those operations stay opaque.
  - On transports, `-c credential.helper=` (an empty value resets the helper list; gitcredentials(7)), `-c core.sshCommand=ssh`, and dropping `--upload-pack`.
- Cargo: no flag disables build scripts or proc macros, so those operations stay opaque until the workspace's build-time programs are modeled.
- npm ci: `--git=false`-style suppression is not documented. Pin the lockfile to registry-only sources and refuse git dependencies.

- `systemd.Systemctl` and `hostnamectl.Process`: terminate options before the unit/pattern positional (`--`), or refine the input to a unit-name type. Either move those operations back to `requires none`.
- `sed.Sed`: a typed non-executing sed subset, or `--sandbox` in the argv, would move both operations to `requires none`.
- `tmux.Session.New`: an inner-command carrier whose program demand is declared.
- `browser.Page.Evaluate` / `browser.Element.EvaluateOn`: a structurally non-network expression vocabulary.
- `git.Inspect` grep operations: `-e {pattern}` plus `--` before the revision, or a refined GitRef, would remove the `-O` route.
- git object reads: add `git --no-lazy-fetch` (git 2.44+) to the read operations, then move them to `requires none`.
- `extdeps.browser`: bring the runner's CLI into the repository (or cite its option surface) and terminate options before positionals; the selector operations then fall to the live-page Network rule.
- `http.Client.PostStdinWithinUnixSocket`: put `--` before `{url}` and type the url to the socket's authority.

## Product-layer operations outside `dag/extdeps` (same rule)

These 14 services in `dag/gunbc` had no clause; an absent clause is Undecided, so every caller would refuse. Verdicts:

| verdict → clause | reason | operations |
|---|---|---|
| Network → `requires Network` | rest to `approval_ntfy_endpoint` (`https://ntfy.sh` or the tailnet HTTPS base), not a unix socket or loopback | `gunbc.auth.approval_ntfy_deployment` `ntfy.Publish.PublishMessage` |
| OpaqueDemand → `requires opaque` | runtime program: argv[0] is the caller-supplied `bin_path` | `gunbc.cli_services` `gunbc.Cli.Run`, `claim_executor.Executor.VerifyBuildArtifacts` |
| OpaqueDemand → `requires opaque` | runtime program: the script runs the caller-supplied `{ipmitool}` path (lanplus to `{bmc_host}`) | `gunbc.machine_intake.sol_hold` `ActivateHeld` |
| OpaqueDemand → `requires opaque` | runtime program: launches the caller-supplied `command` argv | `gunbc.owned_process` `launch.LaunchOwned` |
| NotNetwork → `requires none` | local only: reads the pid record, checks `/proc`, `kill`s the recorded pid | `gunbc.machine_intake.sol_hold` `ReleaseHeld` |
| NotNetwork → `requires none` | no transport: a pure fold over its inputs | `gunbc.code_change_workflow` `ClassifyGithubPrTerminalAnchor`, `DecideTransition`; `gunbc.pr_digests` `ExtractAttachedUrls`, `RenderPrSummaryLine`, `JudgeMergeReadiness`, `ClassifyRestFallback`; `gunbc.review_verdict` `Parse`, `Tally` |

## Deliberately undeclared test fixtures

These test and fixture operations carry no `requires` clause on purpose. Each exists to exercise a transport, argv or governance mechanism, and no test of theirs reads a demand, so none needs a clause today. A fixture gains one when a test first needs it. The list is the population at this writing; read the current one by scanning for `operation` blocks without `requires` outside `dag/extdeps` and `dag/gunbc`.

- `dag/test/claim/m4_governed_service_witness.dag`: `GovernedProbe` `Allowed`, `Forbidden`
- `dag/test/claim/reconstruction_door_fixture_probe.dag`: `DoorProbe.Fetch`
- `dag/test/claim/reconstruction_door_rest_probe.dag`: `DoorProbeRest.Fetch`
- `dag/test/claim/shell_spawn_refused_real_execution_witness_test.dag`: `test.ShellSpawnProbe.Observed`
- `dag/test/fixture/argv_executable_position_probe.dag`: `ArgvExecutablePositionProbe` `SmuggleExecutable`, `LiteralExecutable`
- `dag/test/fixture/m4_universal_governed_probe.dag`: `GovernedUniversalProbe` `Allowed`, `Forbidden`
- `dag/test/fixture/rest_exchange_replay_probe.dag`: `test.RestReplay` `Observe`, `RootArray`, `RootFile`, `RootPage`, `WorkflowRunsPage`, `WifProvidersPage`, `WifPool`, `WorkflowRunsPageLegacy`, `RootUncontracted`, `Sibling`, `OptionalObserve`, `Legacy`
- `fixtures/atomic_materialization/subject.dag`: `AtomicFixture.Read`
- `fixtures/bare_service_provider/provider.dag`: `fixture.BareServiceEcho.Say`
- `fixtures/fixture_closure_rustc/argv_word_list_splice_probe.dag`: `fixture.WordListTransport` `InteriorList`, `TrailingList`, `TwoListsOneArgv`, `SingleWordOnly`
- `fixtures/fixture_closure_rustc/shell_multi_field_projection_probe.dag`: `fixture.MultiFieldProjection.TwoFieldExitSuccess`
- `fixtures/fixture_closure_rustc/shell_single_field_projection_probe.dag`: `fixture.SingleFieldProjection.OneFieldExitSuccess`
