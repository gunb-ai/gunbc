#!/usr/bin/env bash
# qemu-m1-serial-capture.sh — M1 serial liveness demo
#
# Emits the serial writer ELF via gunbc, wraps it in a minimal x86 boot sector,
# boots the image in qemu-system-i386, and captures the 'H' character on COM1.
#
# Requirements:
#   gunbc             (on PATH — from this repo)
#   qemu-system-i386  (macOS: brew install qemu)
#
# Substrate:
#   dsl/extdeps/boot/serial_emit.dag  — emit_serial_writer_elf64()
#   dsl/extdeps/boot/uart.dag         — NS16550A COM1 constants
#
# x86 boot sequence:
#   BIOS loads 512-byte boot sector at 0x7C00 in 16-bit real mode.
#   The sector sets up a minimal GDT, switches to 32-bit protected mode,
#   initialises flat data segments, then falls through to the serial writer code.
#   The serial writer polls LSR (0x3FD) until THRE=1, writes 'H' (0x48)
#   to THR (0x3F8), then halts.
#
# Boot sector layout (512 bytes at 0x7C00):
#   0x00–0x15  16-bit: cli + lgdt + CR0.PE + far-jmp 0x0008:0x7C16
#   0x16–0x20  32-bit: mov eax,0x10; mov ds/es/ss,ax
#   0x21–0x33  serial writer code (19 bytes from gunbc)
#   0x34–0x3B  GDT null descriptor
#   0x3C–0x43  GDT code descriptor (4 GB, 32-bit, exec/read)
#   0x44–0x4B  GDT data descriptor (4 GB, 32-bit, read/write)
#   0x4C–0x51  GDT descriptor (limit=0x17, base=0x00007C34)
#   0x52–0x1FD padding
#   0x1FE–0x1FF  0x55 0xAA

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

cd "$REPO_ROOT"

# --- 1. Emit ELF via gunbc ---
echo "[1/3] gunbc run → emit_serial_writer_elf64"
# gunbc exits 2 for non-ProcessExit return types; bytes still printed to stdout.
ELF_STDOUT=$(gunbc run --source-root dsl \
  --entry dsl/extdeps/boot/serial_emit.dag --function emit_serial_writer_elf64 2>/dev/null) || true

# --- 2. Build 512-byte boot sector (Python) ---
echo "[2/3] Building boot sector"
python3 - "$WORK/boot.img" "$ELF_STDOUT" <<'PYEOF'
import sys, re

out_path = sys.argv[1]
elf_str  = sys.argv[2]

# Parse [127, 69, 76, ...] → list[int]
elf_bytes = list(map(int, re.findall(r'\d+', elf_str)))
assert len(elf_bytes) == 139, f"expected 139 ELF bytes, got {len(elf_bytes)}"

# Verify ELF magic
assert elf_bytes[:4] == [0x7F, 0x45, 0x4C, 0x46], "bad ELF magic"

# Extract serial writer code at freestanding_code_file_offset = 120, length 19
code = elf_bytes[120:139]
assert len(code) == 19, f"expected 19 code bytes, got {len(code)}"
EXPECTED_CODE = [
    # mov edx,0x3FD (LSR)
    186, 253, 3, 0, 0,
    # in al,dx
    236,
    # test al,0x20 (THRE bit)
    168, 32,
    # jz -5 (loop back to in al,dx)
    116, 251,
    # mov edx,0x3F8 (THR/data port)
    186, 248, 3, 0, 0,
    # mov al,0x48 ('H')
    176, 72,
    # out dx,al
    238,
    # hlt
    244,
]
assert code == EXPECTED_CODE, (
    f"code bytes mismatch\n  got:      {code}\n  expected: {EXPECTED_CODE}"
)
print(f"    ELF={len(elf_bytes)}B, code bytes verified ({len(code)}B at offset 120)")

sector = bytearray(512)

# 0x00: cli
sector[0x00] = 0xFA
# 0x01: lgdt [0x004C]  — address of gdt_desc in this sector
sector[0x01:0x06] = bytes([0x0F, 0x01, 0x16, 0x4C, 0x00])
# 0x06: mov eax, cr0
sector[0x06:0x09] = bytes([0x0F, 0x20, 0xC0])
# 0x09: or al, 1  — set PE bit
sector[0x09:0x0B] = bytes([0x0C, 0x01])
# 0x0B: mov cr0, eax
sector[0x0B:0x0E] = bytes([0x0F, 0x22, 0xC0])
# 0x0E: jmp far 0x0008:0x00007C16  (66 EA imm32 imm16 — operand-size-prefixed far jmp)
sector[0x0E:0x16] = bytes([0x66, 0xEA, 0x16, 0x7C, 0x00, 0x00, 0x08, 0x00])

# 0x16: mov eax, 0x10  (data selector)
sector[0x16:0x1B] = bytes([0xB8, 0x10, 0x00, 0x00, 0x00])
# 0x1B: mov ds, ax
sector[0x1B:0x1D] = bytes([0x8E, 0xD8])
# 0x1D: mov es, ax
sector[0x1D:0x1F] = bytes([0x8E, 0xC0])
# 0x1F: mov ss, ax
sector[0x1F:0x21] = bytes([0x8E, 0xD0])

# 0x21..0x33: serial writer code (falls through from segment init above)
sector[0x21:0x34] = bytes(code)

# GDT at 0x34
sector[0x34:0x3C] = bytes(8)                                                  # null
sector[0x3C:0x44] = bytes([0xFF, 0xFF, 0x00, 0x00, 0x00, 0x9A, 0xCF, 0x00])  # code
sector[0x44:0x4C] = bytes([0xFF, 0xFF, 0x00, 0x00, 0x00, 0x92, 0xCF, 0x00])  # data

# GDT descriptor at 0x4C: limit=0x17 (3*8-1), base=0x00007C34
sector[0x4C:0x52] = bytes([0x17, 0x00, 0x34, 0x7C, 0x00, 0x00])

# Boot signature
sector[0x1FE] = 0x55
sector[0x1FF] = 0xAA

with open(out_path, 'wb') as f:
    f.write(sector)
print(f"    boot.img written: {len(sector)}B  (code@0x7C21, GDT@0x7C34)")
PYEOF

# --- 3. Boot in QEMU, capture COM1 ---
echo "[3/3] QEMU: booting x86 (floppy), COM1 → serial.log"
SERIAL_LOG="$WORK/serial.log"

# Run QEMU (macOS lacks GNU timeout; use background + sleep + kill)
qemu-system-i386 \
  -drive file="$WORK/boot.img",format=raw,if=floppy \
  -serial file:"$SERIAL_LOG" \
  -machine accel=tcg \
  -nographic \
  -no-reboot \
  -m 16M \
  2>/dev/null &
QEMU_PID=$!
sleep 4
kill "$QEMU_PID" 2>/dev/null || true
wait "$QEMU_PID" 2>/dev/null || true

echo ""
echo "=== Serial transcript ==="
if [[ -s "$SERIAL_LOG" ]]; then
  python3 -c "
import sys
data = open('$SERIAL_LOG','rb').read()
hex_view = ' '.join(f'{b:02X}' for b in data)
print(f'  hex: {hex_view}')
print(f'  str: {data!r}')
found = 0x48 in data
print()
if found:
    print('PASS: 0x48 (H) captured on COM1 — M1 serial liveness probe green')
else:
    print('FAIL: 0x48 (H) not found in serial output')
    sys.exit(1)
"
else
  echo "  (empty — QEMU may have failed to write COM1)"
  echo "  Verify: qemu-system-i386 --version"
  exit 1
fi
