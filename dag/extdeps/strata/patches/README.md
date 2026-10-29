`request-bound-save-8183c6b6.patch` applies to Strata
[`8183c6b67162340c09b67360860177c577a65f2e`](https://github.com/Niko1221/Strata/commit/8183c6b67162340c09b67360860177c577a65f2e).
It adds conditional saves using the completed response's `id`; it is a local patch, not an upstream feature.

SHA-256: `56f20cb522f1534bc30f170febbd9b61d0ef7ae60242495acd98a090d1c12287`.
Patched `serve/server.py`: `c387b3bcda8df95e5341af9ed57c698dd6d307740d45db9b3b18e8fc9bad3673`.
The interface is modeled by `extdeps.strata.session_file`; the patch includes HTTP/FIFO tests.
Apply with `git apply --check` then `git apply`; run `python -m unittest serve.test_request_bound_save`.
