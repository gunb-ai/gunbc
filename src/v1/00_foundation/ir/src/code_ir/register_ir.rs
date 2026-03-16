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
    /// Explicit routing to the function's single-exit epilogue.
    JumpEpilogue,

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
/// Each hardware register is an explicit variant — invalid indices are
/// unrepresentable at construction time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    /// `$zero` — always 0.
    Zero,
    /// `$at` — assembler temporary.
    At,
    /// `$v0` — function return value / syscall number.
    V0,
    /// `$v1` — function return value.
    V1,
    /// `$a0` — function argument 0.
    A0,
    /// `$a1` — function argument 1.
    A1,
    /// `$a2` — function argument 2.
    A2,
    /// `$a3` — function argument 3.
    A3,
    /// `$t0` — temporary (caller-saved).
    T0,
    /// `$t1` — temporary (caller-saved).
    T1,
    /// `$t2` — temporary (caller-saved).
    T2,
    /// `$t3` — temporary (caller-saved).
    T3,
    /// `$t4` — temporary (caller-saved).
    T4,
    /// `$t5` — temporary (caller-saved).
    T5,
    /// `$t6` — temporary (caller-saved).
    T6,
    /// `$t7` — temporary (caller-saved).
    T7,
    /// `$t8` — temporary (caller-saved).
    T8,
    /// `$t9` — temporary (caller-saved).
    T9,
    /// `$s0` — saved (callee-saved).
    S0,
    /// `$s1` — saved (callee-saved).
    S1,
    /// `$s2` — saved (callee-saved).
    S2,
    /// `$s3` — saved (callee-saved).
    S3,
    /// `$s4` — saved (callee-saved).
    S4,
    /// `$s5` — saved (callee-saved).
    S5,
    /// `$s6` — saved (callee-saved).
    S6,
    /// `$s7` — saved (callee-saved).
    S7,
    /// `$gp` — global pointer.
    Gp,
    /// `$sp` — stack pointer.
    Sp,
    /// `$fp` — frame pointer.
    Fp,
    /// `$ra` — return address.
    Ra,
}

/// `$a0`–`$a3` indexed by position (for calling-convention loops).
pub const ARG_REGS: [Register; 4] = [Register::A0, Register::A1, Register::A2, Register::A3];

/// `$t0`–`$t9` indexed by position (for the temp-register allocator).
pub const TEMP_REGS: [Register; 10] = [
    Register::T0,
    Register::T1,
    Register::T2,
    Register::T3,
    Register::T4,
    Register::T5,
    Register::T6,
    Register::T7,
    Register::T8,
    Register::T9,
];

impl Register {
    /// Canonical MIPS register name (e.g., `$t0`, `$sp`).
    pub fn name(self) -> &'static str {
        match self {
            Self::Zero => "$zero",
            Self::At => "$at",
            Self::V0 => "$v0",
            Self::V1 => "$v1",
            Self::A0 => "$a0",
            Self::A1 => "$a1",
            Self::A2 => "$a2",
            Self::A3 => "$a3",
            Self::T0 => "$t0",
            Self::T1 => "$t1",
            Self::T2 => "$t2",
            Self::T3 => "$t3",
            Self::T4 => "$t4",
            Self::T5 => "$t5",
            Self::T6 => "$t6",
            Self::T7 => "$t7",
            Self::T8 => "$t8",
            Self::T9 => "$t9",
            Self::S0 => "$s0",
            Self::S1 => "$s1",
            Self::S2 => "$s2",
            Self::S3 => "$s3",
            Self::S4 => "$s4",
            Self::S5 => "$s5",
            Self::S6 => "$s6",
            Self::S7 => "$s7",
            Self::Gp => "$gp",
            Self::Sp => "$sp",
            Self::Fp => "$fp",
            Self::Ra => "$ra",
        }
    }

    /// If this is a temporary register (`$t0`–`$t9`), return its index.
    pub fn temp_index(self) -> Option<u8> {
        match self {
            Self::T0 => Some(0),
            Self::T1 => Some(1),
            Self::T2 => Some(2),
            Self::T3 => Some(3),
            Self::T4 => Some(4),
            Self::T5 => Some(5),
            Self::T6 => Some(6),
            Self::T7 => Some(7),
            Self::T8 => Some(8),
            Self::T9 => Some(9),
            _ => None,
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
