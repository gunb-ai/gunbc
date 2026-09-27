# Namecheap transport control

Run from the repository root with a current gunbc binary and an enforced memory
limit (the parser/compiler refuses an unbounded host budget):

```sh
systemd-run --user --scope -p MemoryMax=6G --quiet \
  python3 test/namecheap/query_transport.py /absolute/path/to/gunbc
```

The harness stages its own temporary import closure, starts a loopback HTTP
server, and sends a fixture-only query through the actual
modeled `Http.Client.GetQueryStdinWithin` operation, verifies the received query,
and inspects that curl process's argv to ensure the fixture key is absent. It
uses no real credential and reaches neither GCP nor Namecheap.

After regenerating the workflow, run `python3 test/namecheap/workflow_success.py`
(requires PyYAML). It executes the emitted shell from a clean temporary root with
a successful observer stand-in producing only the declared JSON artifact, checks
the exact upload path, and confirms a planted unmatched receipt read fails. It
does not substitute for a live GCP or Namecheap observation.
