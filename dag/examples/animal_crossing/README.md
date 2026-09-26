# Animal Crossing dialogue patching

`dialogue.dag` builds a GameCube Animal Crossing disc image with rewritten
villager dialogue from a patch set: a directory holding `BASE` (the sha1 of the
base image) and `.acmsg` files of message replacements.

## How it's split

The formats are modeled in `extdeps`:
- `formats.gamecube.disc`
- `formats.jsystem.rarc`
- `games.animal_crossing.message`
- `games.animal_crossing_deluxe.control_codes`

The only work outside `.dag` is literal byte I/O, through
`extdeps.tools.{xxd,dd,shasum,wc}`. The interpreter never holds the image or
the message text. It reads only the headers, the message table and the
messages being edited. It then plans the output as byte ranges copied from the
base plus octets written.

## Entry points

```bash
BASE=~/Library/Application\ Support/Dolphin/Games/Animal\ Crossing\ Deluxe.iso
run() { ./target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry dag/examples/animal_crossing/dialogue.dag "$@"; }

run --function show  --arg "base=$BASE" --arg ids=0x311D,12593
run --function check --arg "base=$BASE" --arg patches=PATCHES
run --function build --arg "base=$BASE" --arg patches=PATCHES --arg out=OUT.iso
```

`build` reads the output back, confirms every edit, and writes
`OUT.iso.manifest`.

## Claims

- `test.claim.gamecube_formats_witness` covers octets, hex, xxd lines, dd
  blocks, the file system table, placement and the RARC repack.
- `test.claim.animal_crossing_dialogue_witness` covers the text form, control
  codes, steering, the message table and `.acmsg` parsing.

Both take literal inputs. A real-image run needs a disc image the repository
does not carry.
