# `make gist-recent` credential diagnostics baseline (E1.0)

Date: 2026-02-18

## Command run

```bash
RUSTUP_TOOLCHAIN=nightly GUNBC_FRESHNESS_ACTIVE=1 make gist-recent REPO=.
```

Notes:
- `RUSTUP_TOOLCHAIN=nightly` is required in this repo because `core/test` currently uses the unstable `unsigned_is_multiple_of` API.
- `GUNBC_FRESHNESS_ACTIVE=1` avoids recursive freshness preflight loops while collecting runtime credential diagnostics.

## Observed credential-resolution execution path

From the DAG execution trace, `gist-recent` credential flow is:

1. `cloud_env`
2. `resolve_auth`
3. `bind_secret`
4. `scope_preflight`
5. `resolve_config`
6. `map_gcp_inputs`
7. `should_impersonate`
8. `prepare_read_adc`

Failure occurs at `prepare_read_adc` with:

> ADC file not found at `/home/ubuntu/.config/gcloud/application_default_credentials.json`.  
> Run `gcloud auth application-default login` and retry.

## Current credential resolution precedence (from code)

`lib/cloud-ops/src/config_loader.rs::resolve_graph_cloud_config()` resolves in this order:

1. `GUNBC_CLOUD_CONFIG_JSON`
2. `GUNBC_CLOUD_CONFIG_TOML` (+ namespace/profile resolution)
3. legacy GCP env (`GCP_SECRETS_PROJECT`, `GCP_SECRETS_PREFIX`, optional SA/impersonation)

When no source is configured:

- `graph_cloud_config()` falls back to `default_local_dev_config()` unless
  `GUNBC_CLOUD_CONFIG_REQUIRED=1|true` is set.

## Hidden/default behaviors identified

1. **Implicit local-dev fallback config**  
   If no config source is set, runtime silently uses `default_local_dev_config()` (unless required-mode is enabled), which can mask missing environment/config setup.

2. **Implicit ADC path fallback**  
   `lib/gcp-ops/src/ops.rs::adc_file_path()` resolves ADC path in this order:
   - `GOOGLE_APPLICATION_CREDENTIALS` env var
   - `$HOME/.config/gcloud/application_default_credentials.json`
   - `/root/.config/gcloud/application_default_credentials.json` when `$HOME` is unset

3. **Legacy audience default**  
   Legacy env config path defaults audience to `"local-dev"` when `GCP_WIF_PROVIDER` is absent.

4. **Build-system default dependency**  
   `make gist-recent` routes through `ensure-codegen` first; this can obscure whether failures are credential/runtime failures vs generation/bootstrap failures unless toolchain/freshness conditions are pinned.

## Immediate operator guidance (baseline)

- Prefer explicit config sources (`GUNBC_CLOUD_CONFIG_JSON` or `GUNBC_CLOUD_CONFIG_TOML`) and set `GUNBC_CLOUD_CONFIG_REQUIRED=1` in CI.
- Set `GOOGLE_APPLICATION_CREDENTIALS` explicitly when using nonstandard ADC location.
- Treat fallback defaults above as transitional compatibility behavior; they should be made explicit and diagnosable in later E1.x phases.
