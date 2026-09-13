# Preregistered prediction — Mt. Collins unit 1, 2DRx4 socket swap

**Committed BEFORE the hardware is touched.** Round 2 of the spare bisection, and
simultaneously the second controlled test of the OEM socket-selector hypothesis.

## What round 1 established

12 originals + 4 x 2DRx4, with the 2DRx4 group installed ENTIRELY IN SOCKET 1
(operator-confirmed placement; socket 0 held 8 originals). Restart at 4m50s, OEM payload
`0fde703b1102`.

That was the first `3b` of the session; every prior failure was `33`. Under the candidate
schema (stage = byte>>4, socket = (byte>>3)&1, status = byte&7), `33` is socket 0 and `3b`
socket 1, both stage 3 / status 3. The socket bit therefore tracked a variable we controlled.

## Configuration to be tested

The same sixteen modules, SIDES EXCHANGED:

- 4 x `2DRx4` moved to **socket 0** connectors (odd: J1, J3, J5, J7)
- originals filling **socket 1** connectors (odd: J17 .. J31)
- 8 per socket, one per channel, unchanged otherwise

## The prediction

**It will fail, and the OEM payload will report `33` (socket 0) rather than `3b`.**

This is a conjunction and both halves can fail independently:
1. that it fails at all
2. that the socket field follows the modules

## What each outcome earns

- **Fails with `33`** -> the fault FOLLOWS THE MODULES, and the socket-selector reading has a
  second controlled confirmation. Strongest single result available from this test.
- **Fails with `3b`** -> the fault is tied to the socket-1 LOCATION -- channel path, connector,
  memory controller -- and NOT to the 2DRx4 modules. This would falsify the "modules
  implicated" reading that round 1 suggested, and it would also break the module-following
  interpretation of the socket bit.
- **Trains** -> those modules are functional in socket 0 but not socket 1: an interaction
  between module and location. Both simple readings would be wrong.
- **Fails with a payload in neither form** -> the schema is worse than we think; preserve the
  bytes and stop interpreting.

## What NO outcome earns

None of these identifies WHICH of the four 2DRx4 modules contributes, and none establishes a
physical defect in any module. Four change together. A module that is merely untolerated in
this platform's population is indistinguishable here from one that is broken.

The socket-selector schema remains a HYPOTHESIS with two controlled confirmations at best. It
is not a decoder, and it must not be used to name a DIMM. The vendor mapping from payload
bytes to physical slots is still unobtained.

## Standing disjunction, carried forward

Round 1's falsification (2026-09-11) established that population shape alone is NOT
SUFFICIENT. It did not establish that supported shape is NECESSARY. Both remain open.
