# Linux can typecheck and run the approve-ios wire tests — with one declared transport gap

**Found:** 2026-10-19, as the L1 probe for the Swift generation migration (Charter 5).
**Class:** capability probe, not a defect. It establishes what a Linux swiftc rung can and cannot claim,
and records the one place where it cannot claim what the iOS build claims.
**Head it was established at:** `6ae9b3b9591e89752ad40486955910610e26f0be` (main), pinned toolchain
`swift-6.4.1-RELEASE` / debian13 — see `extdeps.swift_toolchain`.

## What was asked, and what the answer is

Can a pinned swift.org Linux toolchain, on the BuildBuddy amd64 runner, (a) `-typecheck` the
Foundation-only approve-ios sources and (b) build and run the two ApproveTests vector suites against
`dag/test/fixture/approval_device_redemption/vectors.json` through swift-corelibs-xctest?

Yes to both, at that head, in one dispatch — **provided** one external fact is stated rather than
assumed: on swift-corelibs-foundation the HTTP transport types live in a separate `FoundationNetworking`
module, and `Approve/Wire.swift` imports only `Foundation`. So the three named files are **not** Linux
-clean as shipped; they become clean when a separate, harness-owned file re-exports that module. Nothing
shipped is edited, and the iOS build is untouched.

The interesting part is not "it works". It is that the three files are not interchangeable, that the
harness found its fixture in a place the test file never names, and that a GREEN which is really 15
fixture-load errors can look exactly like a pass until you count.

## The rows, in the order that makes them informative

All rows below are `swiftc -typecheck -module-name Approve …` against files checked out fresh at the head,
with the toolchain's own `usr/lib` on `LD_LIBRARY_PATH`.

| Row | Inputs | Result |
|---|---|---|
| A1 | `Protocol.swift` + `EnrolmentMachine.swift`, shipped bytes | **rc=1** — `Protocol.swift:117:45: cannot find 'Route' in scope` |
| A2 | the three named files, shipped bytes, nothing added | **rc=1**, 8 errors, all `Wire.swift` transport uses |
| A3 | the same three files **+** a separate `FoundationNetworking` re-export file | **rc=0, 0 errors, 0 warnings** |
| B | SwiftPM harness over those sources + the two shipped test files + the shipped fixture | **GREEN: 25 tests, 0 failures** |
| C | B, with **one byte** of `vectors.json` flipped | **rc=1, exactly 1 failing test** |
| D | B, with **one byte** of generated `Protocol.swift` flipped | **rc=1, 3 failing tests, same root assertion** |
| E | B again, sources restored byte-for-byte from the checkout | **GREEN: 25 tests, 0 failures** |

A2's eight errors are the whole gap, and they are all one sentence repeated:

```
Wire.swift:482:19: error: 'URLSession' (aka 'AnyObject') cannot be constructed because it has no accessible initializers
Wire.swift:485:19: error: cannot find 'URLRequest' in scope
Wire.swift:492:19: error: 'URLResponse' is unavailable: This type has moved to the FoundationNetworking module. Import that module to use it.
Wire.swift:498:56: error: value of type 'HTTPURLResponse' (aka 'AnyObject') has no member 'statusCode'
```

`aka AnyObject` is the tell. The types are not missing from the platform; they are present in a module
this translation unit never imports, so each resolves to an opaque `AnyObject`.

## A1 is the finding that changes how the rung must be stated

`Protocol.swift` does not typecheck alone, and the reason is not a Linux gap: `Protocol.swift:117` reads
`Route.push`, and `Route` is declared in `Wire.swift`. Both files import only `Foundation`. So "the
Foundation-only files" are not N independently-checkable files — **Protocol.swift,
Wire.swift and EnrolmentMachine.swift are one typechecking unit**, and any rung that typechecks them
separately gets a spurious red on `Protocol.swift` and a misleading green on a file whose counterpart it
never looked at. The `-module-name Approve` and the whole-file set are load-bearing, not decoration.

## The fixture loads from somewhere the test file never says

`ProtocolVectorTests.loadVectors()` resolves the fixture as
`Bundle(for: ProtocolVectorTests.self).url(forResource: "vectors", withExtension: "json")`, failing hard if
absent. On Xcode that is the test bundle. Under SwiftPM on Linux there is no test bundle at all, and the
diagnostic run had to answer "what does `Bundle(for:)` even return here":

```
main_bundle_path          /root/l1/harness/.build/out/Products/Debug-linux-x86_64/ApproveTestsRun-test-runner
main_resource_url         …/ApproveTestsRun-test-runner/Contents/Resources
bundle_for_class_path     …/ApproveTestsRun-test-runner          (same)
bundle_for_class_resource …/ApproveTestsRun-test-runner/Contents/Resources
url_for_vectors           …/ApproveHarness_ApproveTestsRun.bundle/vectors.json
candidate vectors.json beside the exe   false
candidate ApproveHarness_ApproveTestsRun.bundle/vectors.json   true
```

Two things worth keeping. The resource lands in a **side-car bundle directory** next to the executable, in
a `.bundle` named `<Package>_<Target>.bundle` — so a harness that copies the fixture beside the binary and
nothing else will silently not be found. And `Bundle(for:)` on Linux is reported as the executable itself
(the `.xctest` bundle is absent, and `swift-corelibs-foundation` answers with the main bundle): so any
runbook that reasons about "the test bundle" on Linux is reasoning about a thing that does not exist, and
the resource search falls through to the side-car bundle. The names come from SwiftPM's own layout, which
is why a harness should ask `--show-bin-path` and read the directory rather than hard-code it: the
executable is `ApproveTestsRun-test-runner`, **not** `*.xctest`, and a discovery rule looking for `.xctest`
finds nothing and reports a green-looking nothing.

## GREEN is not self-evident here, and the near-miss is the lesson

An earlier cut of the harness reported 24 tests, 15 failing. Those were not wire failures: every one of
them threw at `loadVectors()` because the fixture had not been seeded where `Bundle(for:)` looks, and the
9 that "passed" were the rows that never touch the fixture. Read carelessly — "tests ran, some failed,
Linux gap" — that is a false negative about the platform, produced by a harness bug. The fix was seeding,
not patching: after it, 25/25 green (the 24 shipped tests plus one diagnostic), zero failures.

The discipline that catches it is the pair, not either half: a **GREEN** is meaningless next to a RED that
does not discriminate narrowly, and a RED is meaningless unless the same harness goes green again after the
mutation is reverted. Hence rows C, D and **E** above.

## Both mutations discriminate, and each hits a different layer

**C — one byte of the fixture.** Offset 819 is the `1` of `.v1` inside the first redemption vector's
`expected` frame. Flipping it to `2` makes the server's expected bytes disagree with the client builder.
Exactly one test fails, and no other:

```
ProtocolVectorTests.testRedemptionSigningInputMatchesEveryVector:221: XCTAssertNotNil failed
```

One byte in, one test out — which also proves the test is genuinely reading the fixture rather than
recomputing it.

**D — one byte of the generated builder.** `Protocol.swift`'s `static let redemptionProtocol` changed from
`…v1` to `…v2` (the generated constant, `gunbc.approval_ios_swift_protocol`; the fixture untouched). Three
tests fail, all rooted in the same assertion plus the two preconditions that carry the protocol string:

```
ProtocolVectorTests.testRedemptionSigningInputMatchesEveryVector:221: XCTAssertNotNil failed
ProtocolVectorTests.testRedemptionRequestEnvelopeMatchesVector:243: XCTAssertNotNil failed
ProtocolVectorTests.testWireDecodeStrictnessRejectsEveryCorruption:282: XCTAssertNotNil failed
```

C and D failing on the *same* line 221 while C leaves 243 and 282 green is the useful shape: the fixture
byte and the builder byte meet at one comparison, and each mutation moves only its own side of it.

## The gap that must stay declared, not patched

The only file in scope needing anything beyond Foundation is `Wire.swift`, and only for its four transport
types. The probe closes it with a **separate, harness-owned file**:

```swift
#if canImport(FoundationNetworking)
@_exported import FoundationNetworking
#endif
```

That is legitimate *as a harness shim for a probe*, and it is the thing to watch: it is a file in which a
future author could quietly patch Linux behavior while claiming to be running the shipped sources. The
guard is that the harness copies shipped bytes and compares them (`cmp` after every mutation, row E), and
that the added file re-exports a module and contains no logic. The gap is recorded as data —
`extdeps.swift_toolchain.linux_transport_module_gap_files` — so a rung names it rather than restating it
in prose.

`CryptoKit` is **not** a Linux gap for this scope: `Protocol.swift:58` mentions it only in a comment about
what verifies the signature (verification is server-side); no file in scope imports it, so swift-crypto was
not needed and nothing was stubbed. `DeviceKeys.swift`, `AppState.swift`, `Views.swift`, `ApproveApp.swift`
and `EnrolmentState.swift` are the ones that would need Apple-only stubs (CryptoKit / Security / local
authentication / SwiftUI) and are outside the probe's scope, untested, and stay that way.

## How to re-run it

One `ctrl-build --remote` dispatch (the toolchain download and the compile must not be split across
dispatches; the runner's `target/` and `/root` do not survive between them for this purpose):

1. Fetch `https://download.swift.org/swift-6.4.1-release/debian13/swift-6.4.1-RELEASE/swift-6.4.1-RELEASE-debian13.tar.gz`
   (1,122,730,813 bytes, sha256 `350260329794a6009360c232d2f9303229581d8bd15187136cc11553df6cd313`) into a
   cache dir, extract with `--strip-components=1`.
2. `swift --version` fails until the Debian runtime closures are installed: `libncurses6 libcurl4 libxml2
   libedit2 libsqlite3-0 libz3-4` (the first is what is actually missing on the runner image; the rest are
   swift.org's declared set). Symptom of skipping this is a missing-library error at `swift --version` that
   reads exactly like a corrupt download.
3. `git init` + `git fetch --depth 1 origin <printed-head-sha>` + `git checkout FETCH_HEAD -- apps/approve-ios
   dag/test/fixture/approval_device_redemption/vectors.json`. Fresh clone at a printed SHA, so the receipt
   names the tree it saw.
4. Rows A1–A3 with `swiftc -typecheck -module-name Approve` over the **whole** file set.
5. Build the SwiftPM harness (`--build-tests`), locate the runner via `swift build --show-bin-path`, seed
   `vectors.json` into the `*.bundle` beside it, run it. Then C, D, and E in the same dispatch.

## What this is NOT

No CI job is registered, and none is implied: the CI roster is closed to growth. The runner-side toolchain
cache is warm but must not be relied on — a dispatch that assumes `/root/toolchains` exists will pass on a
warm runner and fail on a cold one, which is the same class of lie as trusting `uname -m` to name the host
that runs the command.
