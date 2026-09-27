# Namecheap transport control

Run from the repository root with a current gunbc binary and an enforced memory
limit (the parser/compiler refuses an unbounded host budget):

```sh
systemd-run --user --scope -p MemoryMax=6G --quiet \
  python3 test/namecheap/query_transport.py /absolute/path/to/gunbc
```

This harness currently uses the import-closure helper at `target/a7/closure.py`.
It starts a loopback HTTP server, sends a fixture-only query through the actual
modeled `Http.Client.GetQueryStdinWithin` operation, verifies the received query,
and inspects that curl process's argv to ensure the fixture key is absent. It
uses no real credential and reaches neither GCP nor Namecheap.
