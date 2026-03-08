//! MIPS assembly renderer — renders `AsmProgram` (RegisterIR) to `.s` text.
//!
//! Standalone renderer for daglang-emit: takes an `AsmProgram` (after
//! `lower_to_mips`) and produces valid MIPS assembly text suitable for
//! SPIM, MARS, or `mips-linux-gnu-as` + QEMU.
//!
//! MIPS assembly conventions:
//! - `.data` section for string literals, constants, buffers
//! - `.text` section for code with `.globl main`
//! - Stack frame prologue/epilogue generated from `StackFrame`
//! - `#` for comments
//! - Tabs for instruction indentation
//!
//! **Owned by**: Task 16 (dsl-codegen-tasks.md)

use gunbc_ir::code_ir::register_ir::*;
use std::fmt::Write;

// ===========================================================================
// Public API
// ===========================================================================

/// Render an `AsmProgram` to MIPS assembly text.
pub fn render_mips_source(program: &AsmProgram) -> String {
    let mut out = String::new();

    // Data section.
    if !program.data.is_empty() {
        writeln!(out, ".data").unwrap();
        for entry in &program.data {
            out.push_str(&render_data_entry(entry));
        }
        out.push('\n');
    }

    // Text section.
    writeln!(out, ".text").unwrap();

    // Emit .globl for entry point(s).
    for func in &program.functions {
        if func.is_entry {
            writeln!(out, ".globl {}", func.label).unwrap();
        }
    }
    out.push('\n');

    // Functions.
    for func in &program.functions {
        out.push_str(&render_function(func));
        out.push('\n');
    }

    out
}

// ===========================================================================
// C4.1: .data section rendering
// ===========================================================================

fn render_data_entry(entry: &DataEntry) -> String {
    match entry {
        DataEntry::Asciiz { label, value } => {
            format!("{}:\t.asciiz \"{}\"\n", label, escape_asm_str(value))
        }
        DataEntry::Word { label, value } => {
            format!("{}:\t.word {}\n", label, value)
        }
        DataEntry::Space { label, bytes } => {
            format!("{}:\t.space {}\n", label, bytes)
        }
        DataEntry::Bytes { label, values } => {
            let vals: Vec<String> = values.iter().map(|b| b.to_string()).collect();
            format!("{}:\t.byte {}\n", label, vals.join(", "))
        }
    }
}

fn escape_asm_str(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
        .replace('\0', "\\0")
}

// ===========================================================================
// C4.2 + C4.4: Function rendering with prologue/epilogue
// ===========================================================================

fn render_function(func: &AsmFunction) -> String {
    let mut out = String::new();

    // Label.
    writeln!(out, "{}:", func.label).unwrap();

    // Prologue (C4.4).
    if func.frame.size > 0 {
        out.push_str(&render_prologue(&func.frame));
    }

    // Body instructions.
    for instr in &func.body {
        out.push_str(&render_instruction_impl(instr, &func.label));
    }

    // Epilogue label for JumpEpilogue to target.
    writeln!(out, "{}_epilogue:", func.label).unwrap();

    // Epilogue (C4.4).
    if func.frame.size > 0 {
        out.push_str(&render_epilogue(&func.frame));
    }

    // Return if not explicitly done in body.
    // (Only add jr $ra if the body doesn't already end with one.)
    let ends_with_return = func
        .body
        .last()
        .is_some_and(|i| matches!(i, Instruction::JumpReg(Register::Ra) | Instruction::Syscall));
    if !ends_with_return && func.frame.size == 0 {
        writeln!(out, "\tjr $ra").unwrap();
    }

    out
}

/// Render stack frame prologue: allocate stack, save registers.
fn render_prologue(frame: &StackFrame) -> String {
    let mut out = String::new();

    // Allocate stack frame.
    writeln!(out, "\t# prologue").unwrap();
    writeln!(out, "\taddi $sp, $sp, -{}", frame.size).unwrap();

    // Save $ra if needed.
    if let Some(ra_off) = frame.ra_offset {
        writeln!(out, "\tsw $ra, {}($sp)", ra_off).unwrap();
    }

    // Save callee-saved registers.
    for (reg, offset) in &frame.saved_regs {
        writeln!(out, "\tsw {}, {}($sp)", reg.name(), offset).unwrap();
    }

    out
}

/// Render stack frame epilogue: restore registers, deallocate stack, return.
fn render_epilogue(frame: &StackFrame) -> String {
    let mut out = String::new();
    writeln!(out, "\t# epilogue").unwrap();

    // Restore callee-saved registers (reverse order).
    for (reg, offset) in frame.saved_regs.iter().rev() {
        writeln!(out, "\tlw {}, {}($sp)", reg.name(), offset).unwrap();
    }

    // Restore $ra.
    if let Some(ra_off) = frame.ra_offset {
        writeln!(out, "\tlw $ra, {}($sp)", ra_off).unwrap();
    }

    // Deallocate frame.
    writeln!(out, "\taddi $sp, $sp, {}", frame.size).unwrap();
    writeln!(out, "\tjr $ra").unwrap();

    out
}

// ===========================================================================
// C4.3: Instruction rendering
// ===========================================================================

fn render_instruction_impl(instr: &Instruction, func_name: &str) -> String {
    match instr {
        // Arithmetic.
        Instruction::Add { rd, rs, rt } => {
            format!("\tadd {}, {}, {}\n", rd.name(), rs.name(), rt.name())
        }
        Instruction::AddImm { rt, rs, imm } => {
            format!("\taddi {}, {}, {}\n", rt.name(), rs.name(), imm)
        }
        Instruction::Sub { rd, rs, rt } => {
            format!("\tsub {}, {}, {}\n", rd.name(), rs.name(), rt.name())
        }
        Instruction::Mul { rd, rs, rt } => {
            format!("\tmul {}, {}, {}\n", rd.name(), rs.name(), rt.name())
        }

        // Load/Store.
        Instruction::LoadWord { rt, offset, base } => {
            format!("\tlw {}, {}({})\n", rt.name(), offset, base.name())
        }
        Instruction::StoreWord { rt, offset, base } => {
            format!("\tsw {}, {}({})\n", rt.name(), offset, base.name())
        }
        Instruction::LoadByte { rt, offset, base } => {
            format!("\tlb {}, {}({})\n", rt.name(), offset, base.name())
        }
        Instruction::StoreByte { rt, offset, base } => {
            format!("\tsb {}, {}({})\n", rt.name(), offset, base.name())
        }
        Instruction::LoadImm { rt, imm } => {
            format!("\tli {}, {}\n", rt.name(), imm)
        }
        Instruction::LoadAddr { rt, label } => {
            format!("\tla {}, {}\n", rt.name(), label)
        }

        // Branch/Jump.
        Instruction::BranchEq { rs, rt, label } => {
            format!("\tbeq {}, {}, {}\n", rs.name(), rt.name(), label)
        }
        Instruction::BranchNe { rs, rt, label } => {
            format!("\tbne {}, {}, {}\n", rs.name(), rt.name(), label)
        }
        Instruction::BranchGe { rs, rt, label } => {
            format!("\tbge {}, {}, {}\n", rs.name(), rt.name(), label)
        }
        Instruction::BranchLt { rs, rt, label } => {
            format!("\tblt {}, {}, {}\n", rs.name(), rt.name(), label)
        }
        Instruction::Jump(label) => format!("\tj {}\n", label),
        Instruction::JumpAndLink(label) => format!("\tjal {}\n", label),
        Instruction::JumpReg(rs) => format!("\tjr {}\n", rs.name()),

        // Data movement.
        Instruction::Move { rd, rs } => {
            format!("\tmove {}, {}\n", rd.name(), rs.name())
        }
        Instruction::SetLt { rd, rs, rt } => {
            format!("\tslt {}, {}, {}\n", rd.name(), rs.name(), rt.name())
        }

        // Syscall.
        Instruction::Syscall => "\tsyscall\n".to_string(),

        // Structural.
        Instruction::Label(label) => format!("{}:\n", label),
        Instruction::Comment(text) => format!("\t# {}\n", text),
        Instruction::Blank => "\n".to_string(),
        Instruction::Nop => "\tnop\n".to_string(),
        Instruction::JumpEpilogue => format!("\tj {}_epilogue\n", func_name),
    }
}

// ===========================================================================
// Tests (C4.5)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn render_instruction(instr: &Instruction) -> String {
        super::render_instruction_impl(instr, "test_func")
    }

    fn empty_frame() -> StackFrame {
        StackFrame {
            size: 0,
            locals: vec![],
            saved_regs: vec![],
            ra_offset: None,
        }
    }

    // -- C4.1: .data section --

    #[test]
    fn render_data_section() {
        let program = AsmProgram {
            data: vec![
                DataEntry::Asciiz {
                    label: "msg_hello".to_string(),
                    value: "Hello, world!\\n".to_string(),
                },
                DataEntry::Word {
                    label: "buffer_size".to_string(),
                    value: 1024,
                },
                DataEntry::Space {
                    label: "buffer".to_string(),
                    bytes: 256,
                },
                DataEntry::Bytes {
                    label: "magic".to_string(),
                    values: vec![0x7f, 0x45, 0x4c, 0x46],
                },
            ],
            functions: vec![],
            target: AsmTarget::Mips32,
        };
        let rendered = render_mips_source(&program);
        assert!(rendered.contains(".data"), ".data directive");
        assert!(
            rendered.contains("msg_hello:\t.asciiz \"Hello, world!\\\\n\""),
            "asciiz: got {rendered}"
        );
        assert!(
            rendered.contains("buffer_size:\t.word 1024"),
            "word: got {rendered}"
        );
        assert!(
            rendered.contains("buffer:\t.space 256"),
            "space: got {rendered}"
        );
        assert!(
            rendered.contains("magic:\t.byte 127, 69, 76, 70"),
            "bytes: got {rendered}"
        );
    }

    // -- C4.2: .text section (functions as labeled blocks) --

    #[test]
    fn render_simple_function() {
        let program = AsmProgram {
            data: vec![],
            functions: vec![AsmFunction {
                label: "add_nums".to_string(),
                frame: empty_frame(),
                body: vec![
                    Instruction::Add {
                        rd: Register::V(0),
                        rs: Register::A(0),
                        rt: Register::A(1),
                    },
                    Instruction::JumpReg(Register::Ra),
                ],
                is_entry: false,
            }],
            target: AsmTarget::Mips32,
        };
        let rendered = render_mips_source(&program);
        assert!(rendered.contains(".text"), ".text directive");
        assert!(rendered.contains("add_nums:"), "label");
        assert!(
            rendered.contains("\tadd $v0, $a0, $a1"),
            "add instruction: got {rendered}"
        );
        assert!(rendered.contains("\tjr $ra"), "jr $ra");
        // No .globl for non-entry function.
        assert!(!rendered.contains(".globl add_nums"), "should not be globl");
    }

    #[test]
    fn render_entry_function() {
        let program = AsmProgram {
            data: vec![],
            functions: vec![AsmFunction {
                label: "main".to_string(),
                frame: empty_frame(),
                body: vec![
                    Instruction::LoadImm {
                        rt: Register::V(0),
                        imm: 10,
                    },
                    Instruction::Syscall,
                ],
                is_entry: true,
            }],
            target: AsmTarget::Mips32,
        };
        let rendered = render_mips_source(&program);
        assert!(rendered.contains(".globl main"), ".globl main");
        assert!(rendered.contains("main:"), "main label");
        assert!(rendered.contains("\tli $v0, 10"), "li: got {rendered}");
        assert!(rendered.contains("\tsyscall"), "syscall");
    }

    // -- C4.3: Instructions --

    #[test]
    fn render_arithmetic_instructions() {
        assert_eq!(
            render_instruction(&Instruction::Add {
                rd: Register::T(0),
                rs: Register::T(1),
                rt: Register::T(2),
            }),
            "\tadd $t0, $t1, $t2\n"
        );
        assert_eq!(
            render_instruction(&Instruction::AddImm {
                rt: Register::T(0),
                rs: Register::T(1),
                imm: -4,
            }),
            "\taddi $t0, $t1, -4\n"
        );
        assert_eq!(
            render_instruction(&Instruction::Sub {
                rd: Register::T(0),
                rs: Register::T(1),
                rt: Register::T(2),
            }),
            "\tsub $t0, $t1, $t2\n"
        );
        assert_eq!(
            render_instruction(&Instruction::Mul {
                rd: Register::T(0),
                rs: Register::T(1),
                rt: Register::T(2),
            }),
            "\tmul $t0, $t1, $t2\n"
        );
    }

    #[test]
    fn render_load_store_instructions() {
        assert_eq!(
            render_instruction(&Instruction::LoadWord {
                rt: Register::T(0),
                offset: 8,
                base: Register::Sp,
            }),
            "\tlw $t0, 8($sp)\n"
        );
        assert_eq!(
            render_instruction(&Instruction::StoreWord {
                rt: Register::T(0),
                offset: 0,
                base: Register::Sp,
            }),
            "\tsw $t0, 0($sp)\n"
        );
        assert_eq!(
            render_instruction(&Instruction::LoadByte {
                rt: Register::T(0),
                offset: 0,
                base: Register::T(1),
            }),
            "\tlb $t0, 0($t1)\n"
        );
        assert_eq!(
            render_instruction(&Instruction::StoreByte {
                rt: Register::T(0),
                offset: 0,
                base: Register::T(1),
            }),
            "\tsb $t0, 0($t1)\n"
        );
        assert_eq!(
            render_instruction(&Instruction::LoadImm {
                rt: Register::V(0),
                imm: 42,
            }),
            "\tli $v0, 42\n"
        );
        assert_eq!(
            render_instruction(&Instruction::LoadAddr {
                rt: Register::A(0),
                label: "msg".to_string(),
            }),
            "\tla $a0, msg\n"
        );
    }

    #[test]
    fn render_branch_jump_instructions() {
        assert_eq!(
            render_instruction(&Instruction::BranchEq {
                rs: Register::T(0),
                rt: Register::Zero,
                label: "L_end".to_string(),
            }),
            "\tbeq $t0, $zero, L_end\n"
        );
        assert_eq!(
            render_instruction(&Instruction::BranchNe {
                rs: Register::T(0),
                rt: Register::T(1),
                label: "L_loop".to_string(),
            }),
            "\tbne $t0, $t1, L_loop\n"
        );
        assert_eq!(
            render_instruction(&Instruction::BranchGe {
                rs: Register::T(0),
                rt: Register::T(1),
                label: "L_done".to_string(),
            }),
            "\tbge $t0, $t1, L_done\n"
        );
        assert_eq!(
            render_instruction(&Instruction::BranchLt {
                rs: Register::T(0),
                rt: Register::T(1),
                label: "L_body".to_string(),
            }),
            "\tblt $t0, $t1, L_body\n"
        );
        assert_eq!(
            render_instruction(&Instruction::Jump("L_exit".to_string())),
            "\tj L_exit\n"
        );
        assert_eq!(
            render_instruction(&Instruction::JumpAndLink("process_file".to_string())),
            "\tjal process_file\n"
        );
        assert_eq!(
            render_instruction(&Instruction::JumpReg(Register::Ra)),
            "\tjr $ra\n"
        );
    }

    #[test]
    fn render_data_movement_instructions() {
        assert_eq!(
            render_instruction(&Instruction::Move {
                rd: Register::A(0),
                rs: Register::T(0),
            }),
            "\tmove $a0, $t0\n"
        );
        assert_eq!(
            render_instruction(&Instruction::SetLt {
                rd: Register::T(0),
                rs: Register::T(1),
                rt: Register::T(2),
            }),
            "\tslt $t0, $t1, $t2\n"
        );
    }

    #[test]
    fn render_structural_instructions() {
        assert_eq!(render_instruction(&Instruction::Syscall), "\tsyscall\n");
        assert_eq!(
            render_instruction(&Instruction::Label("L_loop".to_string())),
            "L_loop:\n"
        );
        assert_eq!(
            render_instruction(&Instruction::Comment("load string address".to_string())),
            "\t# load string address\n"
        );
        assert_eq!(render_instruction(&Instruction::Blank), "\n");
        assert_eq!(render_instruction(&Instruction::Nop), "\tnop\n");
    }

    // -- C4.4: Stack frame prologue/epilogue --

    #[test]
    fn render_prologue_and_epilogue() {
        let frame = StackFrame {
            size: 32,
            locals: vec![("x".to_string(), 0, 4), ("y".to_string(), 4, 4)],
            saved_regs: vec![(Register::S(0), 8), (Register::S(1), 12)],
            ra_offset: Some(28),
        };

        let prologue = render_prologue(&frame);
        assert!(
            prologue.contains("addi $sp, $sp, -32"),
            "allocate: got {prologue}"
        );
        assert!(
            prologue.contains("sw $ra, 28($sp)"),
            "save ra: got {prologue}"
        );
        assert!(
            prologue.contains("sw $s0, 8($sp)"),
            "save s0: got {prologue}"
        );
        assert!(
            prologue.contains("sw $s1, 12($sp)"),
            "save s1: got {prologue}"
        );

        let epilogue = render_epilogue(&frame);
        // Restored in reverse order.
        assert!(
            epilogue.contains("lw $s1, 12($sp)"),
            "restore s1: got {epilogue}"
        );
        assert!(
            epilogue.contains("lw $s0, 8($sp)"),
            "restore s0: got {epilogue}"
        );
        assert!(
            epilogue.contains("lw $ra, 28($sp)"),
            "restore ra: got {epilogue}"
        );
        assert!(
            epilogue.contains("addi $sp, $sp, 32"),
            "deallocate: got {epilogue}"
        );
        assert!(epilogue.contains("jr $ra"), "return: got {epilogue}");
    }

    // -- C4.5: Full integration test --

    #[test]
    fn render_hello_world_program() {
        let program = AsmProgram {
            data: vec![DataEntry::Asciiz {
                label: "msg".to_string(),
                value: "Hello, world!".to_string(),
            }],
            functions: vec![AsmFunction {
                label: "main".to_string(),
                frame: empty_frame(),
                body: vec![
                    Instruction::Comment("print hello world".to_string()),
                    Instruction::LoadImm {
                        rt: Register::V(0),
                        imm: 4,
                    },
                    Instruction::LoadAddr {
                        rt: Register::A(0),
                        label: "msg".to_string(),
                    },
                    Instruction::Syscall,
                    Instruction::Blank,
                    Instruction::Comment("exit".to_string()),
                    Instruction::LoadImm {
                        rt: Register::V(0),
                        imm: 10,
                    },
                    Instruction::Syscall,
                ],
                is_entry: true,
            }],
            target: AsmTarget::Mips32,
        };

        let rendered = render_mips_source(&program);

        // Data section.
        assert!(rendered.contains(".data"));
        assert!(rendered.contains("msg:\t.asciiz \"Hello, world!\""));

        // Text section.
        assert!(rendered.contains(".text"));
        assert!(rendered.contains(".globl main"));

        // Body.
        assert!(rendered.contains("main:"));
        assert!(rendered.contains("\t# print hello world"));
        assert!(rendered.contains("\tli $v0, 4"));
        assert!(rendered.contains("\tla $a0, msg"));
        assert!(rendered.contains("\tsyscall"));
        assert!(rendered.contains("\t# exit"));
        assert!(rendered.contains("\tli $v0, 10"));
    }

    #[test]
    fn render_function_with_stack_frame() {
        let program = AsmProgram {
            data: vec![],
            functions: vec![AsmFunction {
                label: "process_file".to_string(),
                frame: StackFrame {
                    size: 24,
                    locals: vec![("result".to_string(), 0, 4)],
                    saved_regs: vec![(Register::S(0), 4)],
                    ra_offset: Some(20),
                },
                body: vec![
                    Instruction::Comment("save arg".to_string()),
                    Instruction::Move {
                        rd: Register::S(0),
                        rs: Register::A(0),
                    },
                    Instruction::JumpAndLink("read_file".to_string()),
                    Instruction::StoreWord {
                        rt: Register::V(0),
                        offset: 0,
                        base: Register::Sp,
                    },
                    Instruction::LoadWord {
                        rt: Register::V(0),
                        offset: 0,
                        base: Register::Sp,
                    },
                ],
                is_entry: false,
            }],
            target: AsmTarget::Mips32,
        };

        let rendered = render_mips_source(&program);

        // Prologue.
        assert!(
            rendered.contains("addi $sp, $sp, -24"),
            "allocate: got {rendered}"
        );
        assert!(
            rendered.contains("sw $ra, 20($sp)"),
            "save ra: got {rendered}"
        );
        assert!(
            rendered.contains("sw $s0, 4($sp)"),
            "save s0: got {rendered}"
        );

        // Body.
        assert!(rendered.contains("\tmove $s0, $a0"), "body: got {rendered}");
        assert!(rendered.contains("\tjal read_file"), "jal: got {rendered}");

        // Epilogue.
        assert!(
            rendered.contains("lw $s0, 4($sp)"),
            "restore s0: got {rendered}"
        );
        assert!(
            rendered.contains("lw $ra, 20($sp)"),
            "restore ra: got {rendered}"
        );
        assert!(
            rendered.contains("addi $sp, $sp, 24"),
            "dealloc: got {rendered}"
        );
    }
}
