# Codex provider-runtime selection

Exact `@openai/codex@0.146.0` manifest + lock for `gunbc.package_delivery`.

## Regenerate (do not hand-edit `packages`)

```bash
cd provider-runtime/codex
npm install --package-lock-only --ignore-scripts
```

`node_modules/` is gitignored and must not appear here. Materialization runs `npm ci --ignore-scripts` in an isolated build root, then observes archive digests against lock integrity before unpack is trusted.
