# #12512 receipt tooling (royal-newt-820)

These are the scripts and roster lists behind the receipts recorded in gunbc#12512's PR body and comments. They are kept so that a later session can re-derive them. They are NOT for merging.
- `scripts/`:
  - `census_run.sh` / `census_corpus.sh` / `n3s_run.sh`: BuildBuddy dispatch scripts. Note the 1h free-tier cap.
  - `nat_tmpl.sh`: warm-wolf-234's native per-file census template; needs GITHUB_SHA.
  - `mregen.sh` / `m*.sh`: the local merge→regen→claims loops.
  - `patch_witness.py`: rewrites `bin/infer_semantics_witness.rs` call sites for `TextJudgment`.
- `rosters/`: the claim rosters and pre-registered samples (seed 12512). Their sha256s are posted on the PR.
