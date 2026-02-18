//! Tier 4 — RegisterIR: register-level intermediate representation (MIPS/x86).
//!
//! Lowered from [`super::c_ir`] (Tier 3). Represents machine instructions,
//! register allocation, stack frames, and syscall sequences.
//!
//! The initial target is MIPS32 (suitable for SPIM/MARS simulation and
//! QEMU execution). x86-64 can reuse the same structure with different
//! register sets and calling conventions.

// ---------------------------------------------------------------------------
// Program structure
// ---------------------------------------------------------------------------

/// A complete assembly program.
#[derive(Debug, Clone)]
pub struct AsmProgram {
    /// `.data` section entries.
    pub data: Vec<DataEntry>,
    /// `.text` section: functions as labeled blocks.
    pub functions: Vec<AsmFunction>,
    /// Target architecture (determines register names, syscall numbers, etc.).
    pub target: AsmTarget,
}

/// Target architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsmTarget {
    Mips32,
}

/// A `.data` section entry.
#[derive(Debug, Clone)]
pub enum DataEntry {
    /// `.asciiz "string"` — null-terminated string constant.
    Asciiz { label: String, value: String },
    /// `.word value` — 32-bit integer constant.
    Word { label: String, value: i32 },
    /// `.space n` — reserve n bytes (uninitialized).
    Space { label: String, bytes: usize },
    /// `.byte values...` — byte sequence.
    Bytes { label: String, values: Vec<u8> },
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

/// An assembly function (labeled block with prologue/epilogue).
#[derive(Debug, Clone)]
pub struct AsmFunction {
    pub label: String,
    /// Stack frame layout.
    pub frame: StackFrame,
    /// Instructions (excluding prologue/epilogue — those are generated
    /// from `frame` during rendering).
    pub body: Vec<Instruction>,
    /// Whether this is the program entry point.
    pub is_entry: bool,
}

/// Stack frame layout for a function.
#[derive(Debug, Clone)]
pub struct StackFrame {
    /// Total frame size in bytes (aligned to 4/8).
    pub size: u32,
    /// Local variable slots: (name, offset_from_sp, size_bytes).
    pub locals: Vec<(String, u32, u32)>,
    /// Saved registers: (register, offset_from_sp).
    pub saved_regs: Vec<(Register, u32)>,
    /// Offset where `$ra` is saved (if function makes calls).
    pub ra_offset: Option<u32>,
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

/// A single assembly instruction or pseudo-instruction.
#[derive(Debug, Clone)]
pub enum Instruction {
    // -- Arithmetic --
    /// `add $rd, $rs, $rt`
    Add {
        rd: Register,
        rs: Register,
        rt: Register,
    },
    /// `addi $rt, $rs, imm`
    AddImm {
        rt: Register,
        rs: Register,
        imm: i16,
    },
    /// `sub $rd, $rs, $rt`
    Sub {
        rd: Register,
        rs: Register,
        rt: Register,
    },
    /// `mul $rd, $rs, $rt` (pseudo-instruction)
    Mul {
        rd: Register,
        rs: Register,
        rt: Register,
    },

    // -- Load/Store --
    /// `lw $rt, offset($rs)`
    LoadWord {
        rt: Register,
        offset: i16,
        base: Register,
    },
    /// `sw $rt, offset($rs)`
    StoreWord {
        rt: Register,
        offset: i16,
        base: Register,
    },
    /// `lb $rt, offset($rs)`
    LoadByte {
        rt: Register,
        offset: i16,
        base: Register,
    },
    /// `sb $rt, offset($rs)`
    StoreByte {
        rt: Register,
        offset: i16,
        base: Register,
    },
    /// `li $rt, imm` (pseudo-instruction)
    LoadImm { rt: Register, imm: i32 },
    /// `la $rt, label` (pseudo-instruction: load address)
    LoadAddr { rt: Register, label: String },

    // -- Branch/Jump --
    /// `beq $rs, $rt, label`
    BranchEq {
        rs: Register,
        rt: Register,
        label: String,
    },
    /// `bne $rs, $rt, label`
    BranchNe {
        rs: Register,
        rt: Register,
        label: String,
    },
    /// `bge $rs, $rt, label` (pseudo-instruction)
    BranchGe {
        rs: Register,
        rt: Register,
        label: String,
    },
    /// `blt $rs, $rt, label` (pseudo-instruction)
    BranchLt {
        rs: Register,
        rt: Register,
        label: String,
    },
    /// `j label`
    Jump(String),
    /// `jal label`
    JumpAndLink(String),
    /// `jr $rs`
    JumpReg(Register),

    // -- Data movement --
    /// `move $rd, $rs` (pseudo-instruction)
    Move { rd: Register, rs: Register },
    /// `slt $rd, $rs, $rt` — set on less than
    SetLt {
        rd: Register,
        rs: Register,
        rt: Register,
    },

    // -- Syscall --
    /// `syscall` (service selected by `$v0`)
    Syscall,

    // -- Pseudo/Structural --
    /// A label (not an instruction, but interleaved in the instruction stream).
    Label(String),
    /// Assembly comment: `# text`
    Comment(String),
    /// Blank line.
    Blank,
    /// `nop`
    Nop,
}

// ---------------------------------------------------------------------------
// Registers
// ---------------------------------------------------------------------------

/// MIPS32 register set.
///
/// Named registers following standard MIPS conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    /// `$zero` — always 0.
    Zero,
    /// `$at` — assembler temporary.
    At,
    /// `$v0`-`$v1` — function return values / syscall number.
    V(u8),
    /// `$a0`-`$a3` — function arguments.
    A(u8),
    /// `$t0`-`$t9` — temporaries (caller-saved).
    T(u8),
    /// `$s0`-`$s7` — saved (callee-saved).
    S(u8),
    /// `$gp` — global pointer.
    Gp,
    /// `$sp` — stack pointer.
    Sp,
    /// `$fp` — frame pointer.
    Fp,
    /// `$ra` — return address.
    Ra,
}

impl Register {
    /// Canonical MIPS register name (e.g., `$t0`, `$sp`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Zero => "$zero",
            Self::At => "$at",
            Self::V(0) => "$v0",
            Self::V(1) => "$v1",
            Self::V(_) => "$v?",
            Self::A(0) => "$a0",
            Self::A(1) => "$a1",
            Self::A(2) => "$a2",
            Self::A(3) => "$a3",
            Self::A(_) => "$a?",
            Self::T(n) => match n {
                0 => "$t0",
                1 => "$t1",
                2 => "$t2",
                3 => "$t3",
                4 => "$t4",
                5 => "$t5",
                6 => "$t6",
                7 => "$t7",
                8 => "$t8",
                9 => "$t9",
                _ => "$t?",
            },
            Self::S(n) => match n {
                0 => "$s0",
                1 => "$s1",
                2 => "$s2",
                3 => "$s3",
                4 => "$s4",
                5 => "$s5",
                6 => "$s6",
                7 => "$s7",
                _ => "$s?",
            },
            Self::Gp => "$gp",
            Self::Sp => "$sp",
            Self::Fp => "$fp",
            Self::Ra => "$ra",
        }
    }
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

// ---------------------------------------------------------------------------
// MIPS syscall constants
// ---------------------------------------------------------------------------

/// Standard MIPS syscall service numbers (SPIM/MARS compatible).
pub mod syscall {
    /// `print_int` — prints integer in `$a0`.
    pub const PRINT_INT: i32 = 1;
    /// `print_string` — prints null-terminated string at address in `$a0`.
    pub const PRINT_STRING: i32 = 4;
    /// `read_int` — reads integer into `$v0`.
    pub const READ_INT: i32 = 5;
    /// `read_string` — reads string into buffer at `$a0`, max length `$a1`.
    pub const READ_STRING: i32 = 8;
    /// `sbrk` — allocate `$a0` bytes on heap, return address in `$v0`.
    pub const SBRK: i32 = 9;
    /// `exit` — terminate program.
    pub const EXIT: i32 = 10;
    /// `open` — open file. `$a0` = filename, `$a1` = flags, `$a2` = mode. Returns fd in `$v0`.
    pub const OPEN: i32 = 13;
    /// `read` — read from file. `$a0` = fd, `$a1` = buffer, `$a2` = count. Returns bytes read in `$v0`.
    pub const READ: i32 = 14;
    /// `write` — write to file. `$a0` = fd, `$a1` = buffer, `$a2` = count. Returns bytes written in `$v0`.
    pub const WRITE: i32 = 15;
    /// `close` — close file. `$a0` = fd.
    pub const CLOSE: i32 = 16;
}
