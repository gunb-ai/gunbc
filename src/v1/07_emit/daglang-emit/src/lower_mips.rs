//! CStyleIR → MIPS (RegisterIR) lowering.
//!
//! Lowers C-level constructs to MIPS register-level instructions: register allocation,
//! stack frame layout, calling conventions, syscall sequences.
//!
//! Provides [`lower_to_mips`] which transforms a `CSourceFile` into an `AsmProgram`:
//!
//! - Variables → stack slots (all locals on stack, -O0 style)
//! - Expression evaluation → `$t0`-`$t9` temporaries (bitset allocator)
//! - Function calls → `$a0`-`$a3` args + `jal label` + `$v0` return
//! - Control flow (if/while/for) → branch/jump + labels
//! - String literals → `.data` section `.asciiz` entries
//! - I/O operations → syscall sequences (SPIM/MARS compatible)
//! - `malloc` → `sbrk` syscall; `free` → no-op
//!
//! **Owned by**: Task 15 (dsl-codegen-tasks.md)

use std::collections::HashMap;

use gunbc_ir::code_ir::c_ir::*;
use gunbc_ir::code_ir::lower::LowerError;
use gunbc_ir::code_ir::register_ir::*;

// ===========================================================================
// Configuration
// ===========================================================================

/// Configuration for MIPS lowering.
#[derive(Debug, Clone)]
pub struct MipsConfig {
    /// Whether to emit SPIM/MARS compatible syscalls (vs Linux MIPS).
    pub spim_compat: bool,
}

impl Default for MipsConfig {
    fn default() -> Self {
        Self { spim_compat: true }
    }
}

// ===========================================================================
// Public API
// ===========================================================================

/// Lower a `CSourceFile` to an `AsmProgram`.
pub fn lower_to_mips(source: &CSourceFile, config: &MipsConfig) -> Result<AsmProgram, LowerError> {
    let mut ctx = LowerCtx::new(config);

    for item in &source.items {
        ctx.lower_item(item)?;
    }

    Ok(AsmProgram {
        data: ctx.data,
        functions: ctx.functions,
        target: AsmTarget::Mips32,
    })
}

// ===========================================================================
// Global lowering context
// ===========================================================================

struct LowerCtx {
    data: Vec<DataEntry>,
    functions: Vec<AsmFunction>,
    /// Interned string literals: value → data label.
    strings: HashMap<String, String>,
    str_count: usize,
    label_count: usize,
}

impl LowerCtx {
    fn new(_config: &MipsConfig) -> Self {
        Self {
            data: Vec::new(),
            functions: Vec::new(),
            strings: HashMap::new(),
            str_count: 0,
            label_count: 0,
        }
    }

    /// Intern a string literal: returns its `.data` label, adding to `.data` if new.
    fn intern_string(&mut self, value: &str) -> String {
        if let Some(label) = self.strings.get(value) {
            return label.clone();
        }
        let label = format!("_str_{}", self.str_count);
        self.str_count += 1;
        self.data.push(DataEntry::Asciiz {
            label: label.clone(),
            value: value.to_string(),
        });
        self.strings.insert(value.to_string(), label.clone());
        label
    }

    /// Generate a fresh unique label with the given prefix.
    fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("_{}_{}", prefix, self.label_count);
        self.label_count += 1;
        label
    }

    // -----------------------------------------------------------------------
    // Item lowering
    // -----------------------------------------------------------------------

    fn lower_item(&mut self, item: &CItem) -> Result<(), LowerError> {
        match item {
            CItem::FnDef(f) => {
                let func = self.lower_fn(f)?;
                self.functions.push(func);
            }
            CItem::Define { name, value } => {
                if let Ok(int_val) = value.parse::<i32>() {
                    self.data.push(DataEntry::Word {
                        label: name.clone(),
                        value: int_val,
                    });
                }
            }
            // Structs, typedefs, forward decls, includes, comments don't produce
            // MIPS directly — they inform layout when variables are declared.
            CItem::StructDef { .. }
            | CItem::TaggedUnion { .. }
            | CItem::Typedef { .. }
            | CItem::FnDecl(_)
            | CItem::Include { .. }
            | CItem::Comment(_) => {}
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // B5.4: Function lowering (calling convention)
    // -----------------------------------------------------------------------

    fn lower_fn(&mut self, f: &CFnDef) -> Result<AsmFunction, LowerError> {
        let mut state = FnState::new();
        let is_entry = f.name == "main";

        // Pre-scan: does function body contain any calls?
        state.has_calls = body_has_calls(&f.body);

        // Allocate stack slots for parameters and copy from $a registers.
        for (i, (name, ty)) in f.params.iter().enumerate() {
            let size = type_size_aligned(ty);
            let offset = state.alloc_local(name, size);
            if i < 4 {
                state.emit(Instruction::StoreWord {
                    rt: ARG_REGS[i],
                    offset: offset as i16,
                    base: Register::Sp,
                });
            }
            // Args 5+ are already on the caller's frame above $sp.
        }

        // Lower body statements.
        for stmt in &f.body {
            self.lower_stmt(&mut state, stmt)?;
        }

        // Entry point: add exit syscall if no explicit return at end.
        if is_entry {
            let ends_with_return = f.body.last().is_some_and(|s| matches!(s, CStmt::Return(_)));
            if !ends_with_return {
                state.emit(Instruction::Comment("exit program".to_string()));
                state.emit(Instruction::LoadImm {
                    rt: Register::V0,
                    imm: syscall::EXIT,
                });
                state.emit(Instruction::Syscall);
            }
        }

        // Build the stack frame from collected locals and call info.
        let frame = state.build_frame();

        Ok(AsmFunction {
            label: f.name.clone(),
            frame,
            body: state.body,
            is_entry,
        })
    }

    // -----------------------------------------------------------------------
    // Statement lowering
    // -----------------------------------------------------------------------

    fn lower_stmt(&mut self, state: &mut FnState, stmt: &CStmt) -> Result<(), LowerError> {
        match stmt {
            CStmt::Decl { name, ty, init } => {
                let size = type_size_aligned(ty);
                let offset = state.alloc_local(name, size);
                if let Some(expr) = init {
                    let reg = self.lower_expr(state, expr)?;
                    state.emit(Instruction::StoreWord {
                        rt: reg,
                        offset: offset as i16,
                        base: Register::Sp,
                    });
                    state.free_temp(reg);
                }
            }

            CStmt::Assign { lhs, rhs } => {
                let rhs_reg = self.lower_expr(state, rhs)?;
                self.store_to_lvalue(state, lhs, rhs_reg)?;
                state.free_temp(rhs_reg);
            }

            CStmt::Expr(expr) => {
                let reg = self.lower_expr(state, expr)?;
                state.free_temp(reg);
            }

            CStmt::If {
                cond,
                then_body,
                else_body,
            } => {
                let cond_reg = self.lower_expr(state, cond)?;
                let has_else = else_body.is_some();
                let else_label = self.fresh_label("else");
                let end_label = self.fresh_label("endif");

                state.emit(Instruction::BranchEq {
                    rs: cond_reg,
                    rt: Register::Zero,
                    label: if has_else {
                        else_label.clone()
                    } else {
                        end_label.clone()
                    },
                });
                state.free_temp(cond_reg);

                for s in then_body {
                    self.lower_stmt(state, s)?;
                }

                if let Some(else_stmts) = else_body {
                    state.emit(Instruction::Jump(end_label.clone()));
                    state.emit(Instruction::Label(else_label));
                    for s in else_stmts {
                        self.lower_stmt(state, s)?;
                    }
                }

                state.emit(Instruction::Label(end_label));
            }

            CStmt::While { cond, body } => {
                let loop_label = self.fresh_label("while");
                let end_label = self.fresh_label("endwhile");

                state.emit(Instruction::Label(loop_label.clone()));
                let cond_reg = self.lower_expr(state, cond)?;
                state.emit(Instruction::BranchEq {
                    rs: cond_reg,
                    rt: Register::Zero,
                    label: end_label.clone(),
                });
                state.free_temp(cond_reg);

                for s in body {
                    self.lower_stmt(state, s)?;
                }
                state.emit(Instruction::Jump(loop_label));
                state.emit(Instruction::Label(end_label));
            }

            CStmt::For {
                init,
                cond,
                step,
                body,
            } => {
                self.lower_stmt(state, init)?;
                let loop_label = self.fresh_label("for");
                let end_label = self.fresh_label("endfor");

                state.emit(Instruction::Label(loop_label.clone()));
                let cond_reg = self.lower_expr(state, cond)?;
                state.emit(Instruction::BranchEq {
                    rs: cond_reg,
                    rt: Register::Zero,
                    label: end_label.clone(),
                });
                state.free_temp(cond_reg);

                for s in body {
                    self.lower_stmt(state, s)?;
                }
                self.lower_stmt(state, step)?;
                state.emit(Instruction::Jump(loop_label));
                state.emit(Instruction::Label(end_label));
            }

            CStmt::Return(expr) => {
                if let Some(e) = expr {
                    let reg = self.lower_expr(state, e)?;
                    state.emit(Instruction::Move {
                        rd: Register::V0,
                        rs: reg,
                    });
                    state.free_temp(reg);
                }
                state.emit(Instruction::JumpEpilogue);
            }

            CStmt::Goto(label) => {
                state.emit(Instruction::Jump(label.clone()));
            }
            CStmt::Label(label) => {
                state.emit(Instruction::Label(label.clone()));
            }
            CStmt::BlockScope(body) => {
                state.enter_scope();
                for s in body {
                    self.lower_stmt(state, s)?;
                }
                state.exit_scope();
            }
            CStmt::Free(_) => {
                state.emit(Instruction::Comment("free (no-op in MIPS)".to_string()));
            }
            CStmt::Comment(text) => {
                state.emit(Instruction::Comment(text.clone()));
            }
            CStmt::Blank => {
                state.emit(Instruction::Blank);
            }
        }
        Ok(())
    }

    /// Store a value (in `src_reg`) into an l-value expression.
    fn store_to_lvalue(
        &mut self,
        state: &mut FnState,
        lhs: &CExpr,
        src_reg: Register,
    ) -> Result<(), LowerError> {
        match lhs {
            CExpr::Var(name) => {
                if let Some(offset) = state.find_var(name) {
                    state.emit(Instruction::StoreWord {
                        rt: src_reg,
                        offset: offset as i16,
                        base: Register::Sp,
                    });
                }
            }
            CExpr::Deref(inner) => {
                let addr_reg = self.lower_expr(state, inner)?;
                state.emit(Instruction::StoreWord {
                    rt: src_reg,
                    offset: 0,
                    base: addr_reg,
                });
                state.free_temp(addr_reg);
            }
            CExpr::Index { expr, index } => {
                let base_reg = self.lower_expr(state, expr)?;
                let idx_reg = self.lower_expr(state, index)?;
                let addr_reg = self.emit_array_addr(state, base_reg, idx_reg)?;
                state.emit(Instruction::StoreWord {
                    rt: src_reg,
                    offset: 0,
                    base: addr_reg,
                });
                state.free_temp(addr_reg);
                state.free_temp(idx_reg);
                state.free_temp(base_reg);
            }
            _ => {
                state.emit(Instruction::Comment(
                    "unsupported lvalue in assignment".to_string(),
                ));
            }
        }
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Expression lowering — returns register holding the result
    // -----------------------------------------------------------------------

    fn lower_expr(&mut self, state: &mut FnState, expr: &CExpr) -> Result<Register, LowerError> {
        match expr {
            CExpr::IntLit(n) => {
                let reg = state.alloc_temp()?;
                state.emit(Instruction::LoadImm {
                    rt: reg,
                    imm: *n as i32,
                });
                Ok(reg)
            }
            CExpr::BoolLit(b) => {
                let reg = state.alloc_temp()?;
                state.emit(Instruction::LoadImm {
                    rt: reg,
                    imm: i32::from(*b),
                });
                Ok(reg)
            }
            CExpr::StrLit(s) => {
                let label = self.intern_string(s);
                let reg = state.alloc_temp()?;
                state.emit(Instruction::LoadAddr { rt: reg, label });
                Ok(reg)
            }
            CExpr::CharLit(c) => {
                let reg = state.alloc_temp()?;
                state.emit(Instruction::LoadImm {
                    rt: reg,
                    imm: *c as i32,
                });
                Ok(reg)
            }
            CExpr::Null => {
                let reg = state.alloc_temp()?;
                state.emit(Instruction::Move {
                    rd: reg,
                    rs: Register::Zero,
                });
                Ok(reg)
            }
            CExpr::Var(name) => {
                let reg = state.alloc_temp()?;
                if let Some(offset) = state.find_var(name) {
                    state.emit(Instruction::LoadWord {
                        rt: reg,
                        offset: offset as i16,
                        base: Register::Sp,
                    });
                } else {
                    // Global label.
                    state.emit(Instruction::LoadAddr {
                        rt: reg,
                        label: name.clone(),
                    });
                }
                Ok(reg)
            }

            CExpr::BinOp { left, op, right } => self.lower_binop(state, left, op, right),

            CExpr::UnaryOp { op, expr } => self.lower_unaryop(state, op, expr),

            CExpr::Call { func, args } => self.lower_call(state, func, args),

            CExpr::Field(base_expr, field) => {
                let base_reg = self.lower_expr(state, base_expr)?;
                state.emit(Instruction::Comment(format!("field .{field}")));
                // Without struct layout info, we return the base pointer.
                Ok(base_reg)
            }
            CExpr::Arrow(base_expr, field) => {
                let base_reg = self.lower_expr(state, base_expr)?;
                state.emit(Instruction::Comment(format!("arrow ->{field}")));
                let result = state.alloc_temp()?;
                state.emit(Instruction::LoadWord {
                    rt: result,
                    offset: 0,
                    base: base_reg,
                });
                state.free_temp(base_reg);
                Ok(result)
            }
            CExpr::Index { expr, index } => {
                let base_reg = self.lower_expr(state, expr)?;
                let idx_reg = self.lower_expr(state, index)?;
                let addr_reg = self.emit_array_addr(state, base_reg, idx_reg)?;
                let result = state.alloc_temp()?;
                state.emit(Instruction::LoadWord {
                    rt: result,
                    offset: 0,
                    base: addr_reg,
                });
                state.free_temp(addr_reg);
                state.free_temp(idx_reg);
                state.free_temp(base_reg);
                Ok(result)
            }
            CExpr::AddressOf(inner) => {
                if let CExpr::Var(name) = inner.as_ref() {
                    if let Some(offset) = state.find_var(name) {
                        let reg = state.alloc_temp()?;
                        state.emit(Instruction::AddImm {
                            rt: reg,
                            rs: Register::Sp,
                            imm: offset as i16,
                        });
                        return Ok(reg);
                    }
                }
                self.lower_expr(state, inner)
            }
            CExpr::Deref(inner) => {
                let addr_reg = self.lower_expr(state, inner)?;
                let result = state.alloc_temp()?;
                state.emit(Instruction::LoadWord {
                    rt: result,
                    offset: 0,
                    base: addr_reg,
                });
                state.free_temp(addr_reg);
                Ok(result)
            }
            CExpr::Cast { expr, .. } => {
                // At MIPS32 level, most casts are no-ops (same 32-bit word).
                self.lower_expr(state, expr)
            }
            CExpr::SizeOf(ty) => {
                let reg = state.alloc_temp()?;
                state.emit(Instruction::LoadImm {
                    rt: reg,
                    imm: type_size_aligned(ty) as i32,
                });
                Ok(reg)
            }
            CExpr::Malloc(size_expr) => {
                // SPIM/MARS sbrk syscall: $a0 = size, returns address in $v0.
                let size_reg = self.lower_expr(state, size_expr)?;
                state.has_calls = true;
                state.emit(Instruction::Move {
                    rd: Register::A0,
                    rs: size_reg,
                });
                state.free_temp(size_reg);
                state.emit(Instruction::LoadImm {
                    rt: Register::V0,
                    imm: syscall::SBRK,
                });
                state.emit(Instruction::Syscall);
                let result = state.alloc_temp()?;
                state.emit(Instruction::Move {
                    rd: result,
                    rs: Register::V0,
                });
                Ok(result)
            }
            CExpr::Ternary {
                cond,
                then_expr,
                else_expr,
            } => {
                let cond_reg = self.lower_expr(state, cond)?;
                let result = state.alloc_temp()?;
                let else_label = self.fresh_label("tern_else");
                let end_label = self.fresh_label("tern_end");

                state.emit(Instruction::BranchEq {
                    rs: cond_reg,
                    rt: Register::Zero,
                    label: else_label.clone(),
                });
                state.free_temp(cond_reg);

                let then_reg = self.lower_expr(state, then_expr)?;
                state.emit(Instruction::Move {
                    rd: result,
                    rs: then_reg,
                });
                state.free_temp(then_reg);
                state.emit(Instruction::Jump(end_label.clone()));

                state.emit(Instruction::Label(else_label));
                let else_reg = self.lower_expr(state, else_expr)?;
                state.emit(Instruction::Move {
                    rd: result,
                    rs: else_reg,
                });
                state.free_temp(else_reg);

                state.emit(Instruction::Label(end_label));
                Ok(result)
            }
        }
    }

    // -----------------------------------------------------------------------
    // B5.2: Binary/unary operation lowering
    // -----------------------------------------------------------------------

    fn lower_binop(
        &mut self,
        state: &mut FnState,
        left: &CExpr,
        op: &str,
        right: &CExpr,
    ) -> Result<Register, LowerError> {
        let left_reg = self.lower_expr(state, left)?;
        let right_reg = self.lower_expr(state, right)?;
        let result = state.alloc_temp()?;

        match op {
            "+" => {
                state.emit(Instruction::Add {
                    rd: result,
                    rs: left_reg,
                    rt: right_reg,
                });
            }
            "-" => {
                state.emit(Instruction::Sub {
                    rd: result,
                    rs: left_reg,
                    rt: right_reg,
                });
            }
            "*" => {
                state.emit(Instruction::Mul {
                    rd: result,
                    rs: left_reg,
                    rt: right_reg,
                });
            }
            "<" => {
                state.emit(Instruction::SetLt {
                    rd: result,
                    rs: left_reg,
                    rt: right_reg,
                });
            }
            ">" => {
                // a > b ≡ b < a
                state.emit(Instruction::SetLt {
                    rd: result,
                    rs: right_reg,
                    rt: left_reg,
                });
            }
            ">=" => {
                // a >= b ≡ !(a < b)
                state.emit(Instruction::SetLt {
                    rd: result,
                    rs: left_reg,
                    rt: right_reg,
                });
                self.emit_logical_not(state, result);
            }
            "<=" => {
                // a <= b ≡ !(b < a)
                state.emit(Instruction::SetLt {
                    rd: result,
                    rs: right_reg,
                    rt: left_reg,
                });
                self.emit_logical_not(state, result);
            }
            "==" => {
                self.emit_equality(state, left_reg, right_reg, result, true);
            }
            "!=" => {
                self.emit_equality(state, left_reg, right_reg, result, false);
            }
            "&&" => {
                // Short-circuit: if left == 0, result = 0, else result = (right != 0)
                let short_label = self.fresh_label("and_short");
                let end_label = self.fresh_label("and_end");
                state.emit(Instruction::BranchEq {
                    rs: left_reg,
                    rt: Register::Zero,
                    label: short_label.clone(),
                });
                // Left is true — result = (right != 0)
                self.emit_equality(state, right_reg, Register::Zero, result, false);
                state.emit(Instruction::Jump(end_label.clone()));
                state.emit(Instruction::Label(short_label));
                state.emit(Instruction::LoadImm { rt: result, imm: 0 });
                state.emit(Instruction::Label(end_label));
            }
            "||" => {
                let short_label = self.fresh_label("or_short");
                let end_label = self.fresh_label("or_end");
                state.emit(Instruction::BranchNe {
                    rs: left_reg,
                    rt: Register::Zero,
                    label: short_label.clone(),
                });
                self.emit_equality(state, right_reg, Register::Zero, result, false);
                state.emit(Instruction::Jump(end_label.clone()));
                state.emit(Instruction::Label(short_label));
                state.emit(Instruction::LoadImm { rt: result, imm: 1 });
                state.emit(Instruction::Label(end_label));
            }
            _ => {
                state.emit(Instruction::Comment(format!("unsupported binop: {op}")));
                state.emit(Instruction::Move {
                    rd: result,
                    rs: Register::Zero,
                });
            }
        }

        state.free_temp(left_reg);
        state.free_temp(right_reg);
        Ok(result)
    }

    fn lower_unaryop(
        &mut self,
        state: &mut FnState,
        op: &str,
        expr: &CExpr,
    ) -> Result<Register, LowerError> {
        let inner = self.lower_expr(state, expr)?;
        match op {
            "-" => {
                let result = state.alloc_temp()?;
                state.emit(Instruction::Sub {
                    rd: result,
                    rs: Register::Zero,
                    rt: inner,
                });
                state.free_temp(inner);
                Ok(result)
            }
            "!" => {
                let result = state.alloc_temp()?;
                let set_one = self.fresh_label("not_true");
                let end = self.fresh_label("not_end");
                state.emit(Instruction::BranchEq {
                    rs: inner,
                    rt: Register::Zero,
                    label: set_one.clone(),
                });
                state.emit(Instruction::LoadImm { rt: result, imm: 0 });
                state.emit(Instruction::Jump(end.clone()));
                state.emit(Instruction::Label(set_one));
                state.emit(Instruction::LoadImm { rt: result, imm: 1 });
                state.emit(Instruction::Label(end));
                state.free_temp(inner);
                Ok(result)
            }
            "++" => {
                state.emit(Instruction::AddImm {
                    rt: inner,
                    rs: inner,
                    imm: 1,
                });
                // Write-back to variable if this came from a Var.
                // The caller (typically For step) handles the store.
                Ok(inner)
            }
            _ => {
                state.emit(Instruction::Comment(format!("unsupported unaryop: {op}")));
                Ok(inner)
            }
        }
    }

    // -----------------------------------------------------------------------
    // B5.4: Call lowering (calling convention)
    // -----------------------------------------------------------------------

    fn lower_call(
        &mut self,
        state: &mut FnState,
        func: &str,
        args: &[CExpr],
    ) -> Result<Register, LowerError> {
        state.has_calls = true;

        // B5.6: Check for well-known syscall wrappers.
        if let Some(syscall_num) = syscall_for_func(func) {
            return self.lower_syscall(state, syscall_num, args);
        }

        // Standard calling convention: first 4 args in $a0-$a3.
        for (i, arg) in args.iter().enumerate().take(4) {
            let reg = self.lower_expr(state, arg)?;
            state.emit(Instruction::Move {
                rd: ARG_REGS[i],
                rs: reg,
            });
            state.free_temp(reg);
        }

        // Args 5+ go on the stack (push in reverse order).
        if args.len() > 4 {
            let overflow = args.len() - 4;
            state.emit(Instruction::AddImm {
                rt: Register::Sp,
                rs: Register::Sp,
                imm: -(overflow as i16 * 4),
            });
            for (i, arg) in args.iter().enumerate().skip(4) {
                let reg = self.lower_expr(state, arg)?;
                let slot = (i - 4) as i16 * 4;
                state.emit(Instruction::StoreWord {
                    rt: reg,
                    offset: slot,
                    base: Register::Sp,
                });
                state.free_temp(reg);
            }
        }

        state.emit(Instruction::JumpAndLink(func.to_string()));

        // Restore stack if overflow args were pushed.
        if args.len() > 4 {
            let overflow = args.len() - 4;
            state.emit(Instruction::AddImm {
                rt: Register::Sp,
                rs: Register::Sp,
                imm: overflow as i16 * 4,
            });
        }

        // Result is in $v0.
        let result = state.alloc_temp()?;
        state.emit(Instruction::Move {
            rd: result,
            rs: Register::V0,
        });
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // B5.6: Syscall emission
    // -----------------------------------------------------------------------

    fn lower_syscall(
        &mut self,
        state: &mut FnState,
        syscall_num: i32,
        args: &[CExpr],
    ) -> Result<Register, LowerError> {
        // Load arguments into $a0-$a2 (syscalls use at most 3 args).
        for (i, arg) in args.iter().enumerate().take(3) {
            let reg = self.lower_expr(state, arg)?;
            state.emit(Instruction::Move {
                rd: ARG_REGS[i],
                rs: reg,
            });
            state.free_temp(reg);
        }

        state.emit(Instruction::LoadImm {
            rt: Register::V0,
            imm: syscall_num,
        });
        state.emit(Instruction::Syscall);

        // Syscall result (if any) is in $v0.
        let result = state.alloc_temp()?;
        state.emit(Instruction::Move {
            rd: result,
            rs: Register::V0,
        });
        Ok(result)
    }

    // -----------------------------------------------------------------------
    // B5.5: String operation helpers
    // -----------------------------------------------------------------------

    /// Emit a byte-by-byte string comparison loop.
    /// Returns register with 1 (equal) or 0 (not equal).
    #[cfg(test)]
    fn emit_strcmp(&mut self, state: &mut FnState, a_reg: Register, b_reg: Register) -> Register {
        let result = state
            .alloc_temp()
            .expect("test helper should not exhaust temporary registers");
        let loop_label = self.fresh_label("strcmp_loop");
        let ne_label = self.fresh_label("strcmp_ne");
        let eq_label = self.fresh_label("strcmp_eq");
        let end_label = self.fresh_label("strcmp_end");

        let byte_a = state
            .alloc_temp()
            .expect("test helper should not exhaust temporary registers");
        let byte_b = state
            .alloc_temp()
            .expect("test helper should not exhaust temporary registers");

        state.emit(Instruction::Label(loop_label.clone()));

        // Load current bytes.
        state.emit(Instruction::LoadByte {
            rt: byte_a,
            offset: 0,
            base: a_reg,
        });
        state.emit(Instruction::LoadByte {
            rt: byte_b,
            offset: 0,
            base: b_reg,
        });

        // If bytes differ → not equal.
        state.emit(Instruction::BranchNe {
            rs: byte_a,
            rt: byte_b,
            label: ne_label.clone(),
        });

        // If both are null terminator → equal.
        state.emit(Instruction::BranchEq {
            rs: byte_a,
            rt: Register::Zero,
            label: eq_label.clone(),
        });

        // Advance pointers.
        state.emit(Instruction::AddImm {
            rt: a_reg,
            rs: a_reg,
            imm: 1,
        });
        state.emit(Instruction::AddImm {
            rt: b_reg,
            rs: b_reg,
            imm: 1,
        });
        state.emit(Instruction::Jump(loop_label));

        // Not equal.
        state.emit(Instruction::Label(ne_label));
        state.emit(Instruction::LoadImm { rt: result, imm: 0 });
        state.emit(Instruction::Jump(end_label.clone()));

        // Equal.
        state.emit(Instruction::Label(eq_label));
        state.emit(Instruction::LoadImm { rt: result, imm: 1 });

        state.emit(Instruction::Label(end_label));
        state.free_temp(byte_a);
        state.free_temp(byte_b);
        result
    }

    /// Emit a byte-by-byte string copy loop (until null terminator).
    #[cfg(test)]
    fn emit_strcpy(&mut self, state: &mut FnState, dst_reg: Register, src_reg: Register) {
        let loop_label = self.fresh_label("strcpy_loop");
        let end_label = self.fresh_label("strcpy_end");
        let byte = state
            .alloc_temp()
            .expect("test helper should not exhaust temporary registers");

        state.emit(Instruction::Label(loop_label.clone()));
        state.emit(Instruction::LoadByte {
            rt: byte,
            offset: 0,
            base: src_reg,
        });
        state.emit(Instruction::StoreByte {
            rt: byte,
            offset: 0,
            base: dst_reg,
        });
        state.emit(Instruction::BranchEq {
            rs: byte,
            rt: Register::Zero,
            label: end_label.clone(),
        });
        state.emit(Instruction::AddImm {
            rt: src_reg,
            rs: src_reg,
            imm: 1,
        });
        state.emit(Instruction::AddImm {
            rt: dst_reg,
            rs: dst_reg,
            imm: 1,
        });
        state.emit(Instruction::Jump(loop_label));
        state.emit(Instruction::Label(end_label));

        state.free_temp(byte);
    }

    // -----------------------------------------------------------------------
    // Instruction helpers
    // -----------------------------------------------------------------------

    /// Compute `base + index * 4` (word-aligned array address).
    fn emit_array_addr(
        &mut self,
        state: &mut FnState,
        base_reg: Register,
        idx_reg: Register,
    ) -> Result<Register, LowerError> {
        let four_reg = state.alloc_temp()?;
        state.emit(Instruction::LoadImm {
            rt: four_reg,
            imm: 4,
        });
        let scaled = state.alloc_temp()?;
        state.emit(Instruction::Mul {
            rd: scaled,
            rs: idx_reg,
            rt: four_reg,
        });
        state.free_temp(four_reg);
        let addr = state.alloc_temp()?;
        state.emit(Instruction::Add {
            rd: addr,
            rs: base_reg,
            rt: scaled,
        });
        state.free_temp(scaled);
        Ok(addr)
    }

    /// Emit `result = (a == b)` or `result = (a != b)` using branch sequence.
    fn emit_equality(
        &mut self,
        state: &mut FnState,
        a: Register,
        b: Register,
        result: Register,
        is_eq: bool,
    ) {
        let set_label = self.fresh_label(if is_eq { "eq_true" } else { "ne_true" });
        let end_label = self.fresh_label(if is_eq { "eq_end" } else { "ne_end" });

        if is_eq {
            state.emit(Instruction::BranchEq {
                rs: a,
                rt: b,
                label: set_label.clone(),
            });
        } else {
            state.emit(Instruction::BranchNe {
                rs: a,
                rt: b,
                label: set_label.clone(),
            });
        }
        state.emit(Instruction::LoadImm { rt: result, imm: 0 });
        state.emit(Instruction::Jump(end_label.clone()));
        state.emit(Instruction::Label(set_label));
        state.emit(Instruction::LoadImm { rt: result, imm: 1 });
        state.emit(Instruction::Label(end_label));
    }

    /// Flip a register value: `reg = (reg == 0) ? 1 : 0`.
    fn emit_logical_not(&mut self, state: &mut FnState, reg: Register) {
        let set_one = self.fresh_label("flip_true");
        let end = self.fresh_label("flip_end");
        state.emit(Instruction::BranchEq {
            rs: reg,
            rt: Register::Zero,
            label: set_one.clone(),
        });
        state.emit(Instruction::LoadImm { rt: reg, imm: 0 });
        state.emit(Instruction::Jump(end.clone()));
        state.emit(Instruction::Label(set_one));
        state.emit(Instruction::LoadImm { rt: reg, imm: 1 });
        state.emit(Instruction::Label(end));
    }
}

// ===========================================================================
// B5.3: Per-function state (stack frame, register allocation)
// ===========================================================================

struct FnState {
    /// All local variables seen during lowering (for frame metadata).
    locals: Vec<(String, u32, u32)>,
    /// Lexically-visible local variables (for name lookup).
    visible_locals: Vec<(String, u32, u32)>,
    /// Visibility stack markers into `visible_locals`.
    scope_markers: Vec<usize>,
    /// Next available stack offset for locals.
    next_offset: u32,
    /// Whether function makes any calls (needs $ra save).
    has_calls: bool,
    /// Instruction body.
    body: Vec<Instruction>,
    /// Bitset tracking which $t registers are in use.
    temps_in_use: u16,
}

impl FnState {
    fn new() -> Self {
        Self {
            locals: Vec::new(),
            visible_locals: Vec::new(),
            scope_markers: Vec::new(),
            next_offset: 0,
            has_calls: false,
            body: Vec::new(),
            temps_in_use: 0,
        }
    }

    /// Allocate a local variable on the stack. Returns offset from $sp.
    fn alloc_local(&mut self, name: &str, size: u32) -> u32 {
        let offset = self.next_offset;
        let entry = (name.to_string(), offset, size);
        self.locals.push(entry.clone());
        self.visible_locals.push(entry);
        self.next_offset += size;
        offset
    }

    /// Find a variable's stack offset by name (most recent binding wins).
    fn find_var(&self, name: &str) -> Option<u32> {
        self.visible_locals
            .iter()
            .rev()
            .find(|(n, _, _)| n == name)
            .map(|(_, off, _)| *off)
    }

    /// Enter a lexical scope for local-visibility tracking.
    fn enter_scope(&mut self) {
        self.scope_markers.push(self.visible_locals.len());
    }

    /// Exit the innermost lexical scope.
    fn exit_scope(&mut self) {
        if let Some(marker) = self.scope_markers.pop() {
            self.visible_locals.truncate(marker);
        }
    }

    /// Allocate a temporary register ($t0-$t9).
    fn alloc_temp(&mut self) -> Result<Register, LowerError> {
        for (i, &reg) in TEMP_REGS.iter().enumerate() {
            if self.temps_in_use & (1 << i) == 0 {
                self.temps_in_use |= 1 << i;
                return Ok(reg);
            }
        }
        Err(LowerError::InternalError(
            "MIPS temporary register exhaustion ($t0-$t9)".to_string(),
        ))
    }

    /// Release a temporary register.
    fn free_temp(&mut self, reg: Register) {
        if let Some(n) = reg.temp_index() {
            self.temps_in_use &= !(1u16 << n);
        }
    }

    fn emit(&mut self, inst: Instruction) {
        self.body.push(inst);
    }

    /// Build the `StackFrame` from collected locals and call info.
    fn build_frame(&self) -> StackFrame {
        let locals_size = self.next_offset;
        let ra_size = if self.has_calls { 4 } else { 0 };
        let total = align_to(locals_size + ra_size, 8);

        let ra_offset = if self.has_calls {
            Some(total - 4)
        } else {
            None
        };

        StackFrame {
            size: total,
            locals: self.locals.clone(),
            saved_regs: Vec::new(), // No callee-saved regs used in this lowering.
            ra_offset,
        }
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Align `value` up to `align` boundary.
fn align_to(value: u32, align: u32) -> u32 {
    (value + align - 1) & !(align - 1)
}

/// Compute stack slot size for a CType (minimum 4 bytes, word-aligned).
fn type_size_aligned(ty: &CType) -> u32 {
    match ty {
        CType::Void => 4,
        CType::Int(_) => 4,
        CType::Char => 4, // 1-byte type but 4-byte stack slot for alignment.
        CType::Float(CFloatKind::Float) => 4,
        CType::Float(CFloatKind::Double) => 8,
        CType::Ptr(_) => 4,
        CType::Const(inner) => type_size_aligned(inner),
        CType::Array { element, size } => {
            let elem_size = type_size_aligned(element);
            let count = size.unwrap_or(1) as u32;
            align_to(elem_size * count, 4)
        }
        CType::Named(_) => 4, // Assume pointer-sized for named types.
        CType::FnPtr { .. } => 4,
    }
}

/// Check whether a C statement body contains any function calls.
fn body_has_calls(stmts: &[CStmt]) -> bool {
    stmts.iter().any(stmt_has_calls)
}

fn stmt_has_calls(stmt: &CStmt) -> bool {
    match stmt {
        CStmt::Decl {
            init: Some(expr), ..
        } => expr_has_calls(expr),
        CStmt::Assign { rhs, .. } => expr_has_calls(rhs),
        CStmt::Expr(expr) => expr_has_calls(expr),
        CStmt::Return(Some(expr)) => expr_has_calls(expr),
        CStmt::If {
            cond,
            then_body,
            else_body,
        } => {
            expr_has_calls(cond)
                || body_has_calls(then_body)
                || else_body.as_ref().is_some_and(|b| body_has_calls(b))
        }
        CStmt::While { cond, body } => expr_has_calls(cond) || body_has_calls(body),
        CStmt::For {
            init,
            cond,
            step,
            body,
        } => {
            stmt_has_calls(init)
                || expr_has_calls(cond)
                || stmt_has_calls(step)
                || body_has_calls(body)
        }
        _ => false,
    }
}

fn expr_has_calls(expr: &CExpr) -> bool {
    match expr {
        CExpr::Call { .. } => true,
        CExpr::BinOp { left, right, .. } => expr_has_calls(left) || expr_has_calls(right),
        CExpr::UnaryOp { expr, .. } => expr_has_calls(expr),
        CExpr::Field(inner, _) | CExpr::Arrow(inner, _) | CExpr::Deref(inner) => {
            expr_has_calls(inner)
        }
        CExpr::Index { expr, index } => expr_has_calls(expr) || expr_has_calls(index),
        CExpr::Ternary {
            cond,
            then_expr,
            else_expr,
        } => expr_has_calls(cond) || expr_has_calls(then_expr) || expr_has_calls(else_expr),
        CExpr::Malloc(inner) | CExpr::AddressOf(inner) | CExpr::Cast { expr: inner, .. } => {
            expr_has_calls(inner)
        }
        _ => false,
    }
}

/// Map well-known C library/runtime function names to MIPS syscall numbers.
fn syscall_for_func(name: &str) -> Option<i32> {
    match name {
        "printf" | "print_string" | "puts" => Some(syscall::PRINT_STRING),
        "print_int" => Some(syscall::PRINT_INT),
        "scanf" | "read_string" | "gets" => Some(syscall::READ_STRING),
        "read_int" => Some(syscall::READ_INT),
        "exit" => Some(syscall::EXIT),
        "open" | "fopen" => Some(syscall::OPEN),
        "read" | "fread" => Some(syscall::READ),
        "write" | "fwrite" => Some(syscall::WRITE),
        "close" | "fclose" => Some(syscall::CLOSE),
        _ => None,
    }
}

// ===========================================================================
// B5.7: Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_c_main(params: Vec<(&str, CType)>, body: Vec<CStmt>) -> CSourceFile {
        CSourceFile {
            includes: vec![],
            items: vec![CItem::FnDef(CFnDef {
                name: "main".to_string(),
                return_type: CType::Int(CIntKind::Int),
                params: params
                    .into_iter()
                    .map(|(n, t)| (n.to_string(), t))
                    .collect(),
                body,
                is_static: false,
            })],
        }
    }

    fn simple_main(body: Vec<CStmt>) -> CSourceFile {
        make_c_main(vec![], body)
    }

    fn nested_add_expr(depth: usize) -> CExpr {
        let mut expr = CExpr::IntLit(depth as i64);
        for i in (0..depth).rev() {
            expr = CExpr::BinOp {
                left: Box::new(CExpr::IntLit(i as i64)),
                op: "+".to_string(),
                right: Box::new(expr),
            };
        }
        expr
    }

    // -- B5.1: AsmProgram structure --

    #[test]
    fn lower_empty_main_produces_asm_program() {
        let source = simple_main(vec![CStmt::Return(Some(CExpr::IntLit(0)))]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        assert_eq!(program.target, AsmTarget::Mips32);
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].label, "main");
        assert!(program.functions[0].is_entry);
    }

    // -- B5.2: Register allocation --

    #[test]
    fn temp_registers_allocated_and_freed() {
        let mut state = FnState::new();

        let r0 = state.alloc_temp().expect("should allocate $t0");
        let r1 = state.alloc_temp().expect("should allocate $t1");
        assert_eq!(r0, Register::T0);
        assert_eq!(r1, Register::T1);

        state.free_temp(r0);
        let r2 = state.alloc_temp().expect("should reuse freed $t0");
        assert_eq!(r2, Register::T0, "freed $t0 should be reused");

        state.free_temp(r1);
        state.free_temp(r2);
    }

    #[test]
    fn temp_register_exhaustion_fails_closed() {
        let mut state = FnState::new();
        let mut regs = Vec::new();
        for _ in 0..10 {
            regs.push(state.alloc_temp().expect("should allocate temp register"));
        }

        let err = state
            .alloc_temp()
            .expect_err("11th temporary register should fail closed");
        assert!(
            matches!(err, LowerError::InternalError(ref msg) if msg.contains("temporary register exhaustion")),
            "expected explicit register exhaustion error, got: {err:?}"
        );

        for reg in regs {
            state.free_temp(reg);
        }
    }

    #[test]
    fn scope_exit_restores_previous_binding() {
        let mut state = FnState::new();
        let outer = state.alloc_local("x", 4);
        state.enter_scope();
        let inner = state.alloc_local("x", 4);
        assert_eq!(state.find_var("x"), Some(inner));
        state.exit_scope();
        assert_eq!(state.find_var("x"), Some(outer));
    }

    #[test]
    fn variable_declaration_allocates_stack_slot() {
        let source = simple_main(vec![CStmt::Decl {
            name: "x".to_string(),
            ty: CType::Int(CIntKind::Int),
            init: Some(CExpr::IntLit(42)),
        }]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        assert!(
            main.frame.locals.iter().any(|(n, _, _)| n == "x"),
            "should have local 'x' in frame"
        );
        // Body should contain: li $t0, 42; sw $t0, offset($sp)
        let has_load = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::LoadImm {
                    rt: Register::T0,
                    imm: 42
                }
            )
        });
        let has_store = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::StoreWord {
                    rt: Register::T0,
                    base: Register::Sp,
                    ..
                }
            )
        });
        assert!(has_load, "should have li for init value");
        assert!(has_store, "should have sw to stack slot");
    }

    #[test]
    fn return_lowers_to_epilogue_jump_not_direct_jr_ra() {
        let source = simple_main(vec![CStmt::Return(Some(CExpr::IntLit(7)))]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).expect("lowering should succeed");
        let main = &program.functions[0];

        assert!(
            main.body
                .iter()
                .any(|i| matches!(i, Instruction::JumpEpilogue)),
            "return should route to epilogue"
        );
        assert!(
            !main
                .body
                .iter()
                .any(|i| matches!(i, Instruction::JumpReg(Register::Ra))),
            "lowerer should not emit direct jr $ra for returns"
        );
    }

    #[test]
    fn deep_expression_register_pressure_fails_closed() {
        let source = simple_main(vec![CStmt::Return(Some(nested_add_expr(16)))]);
        let config = MipsConfig::default();
        let err = lower_to_mips(&source, &config).expect_err("should fail on temp exhaustion");
        assert!(
            matches!(err, LowerError::InternalError(ref msg) if msg.contains("temporary register exhaustion")),
            "expected explicit register exhaustion error, got: {err:?}"
        );
    }

    // -- B5.3: Stack frame layout --

    #[test]
    fn frame_size_includes_locals_and_ra() {
        let source = simple_main(vec![
            CStmt::Decl {
                name: "a".to_string(),
                ty: CType::Int(CIntKind::Int),
                init: Some(CExpr::IntLit(1)),
            },
            CStmt::Decl {
                name: "b".to_string(),
                ty: CType::Int(CIntKind::Int),
                init: Some(CExpr::IntLit(2)),
            },
            CStmt::Expr(CExpr::Call {
                func: "some_func".to_string(),
                args: vec![],
            }),
        ]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        // 2 locals × 4 bytes + 4 bytes $ra = 12, aligned to 8 → 16.
        assert_eq!(main.frame.size, 16);
        assert!(
            main.frame.ra_offset.is_some(),
            "should save $ra (has calls)"
        );
    }

    #[test]
    fn frame_no_ra_when_no_calls() {
        let source = simple_main(vec![
            CStmt::Decl {
                name: "x".to_string(),
                ty: CType::Int(CIntKind::Int),
                init: Some(CExpr::IntLit(10)),
            },
            CStmt::Return(Some(CExpr::Var("x".to_string()))),
        ]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        assert!(
            main.frame.ra_offset.is_none(),
            "no calls → no $ra save needed"
        );
    }

    // -- B5.4: Calling convention --

    #[test]
    fn function_params_stored_from_a_registers() {
        let source = make_c_main(
            vec![
                ("argc", CType::Int(CIntKind::Int)),
                (
                    "argv",
                    CType::Ptr(Box::new(CType::Ptr(Box::new(CType::Char)))),
                ),
            ],
            vec![CStmt::Return(Some(CExpr::IntLit(0)))],
        );
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        // Should have sw $a0 and sw $a1 at the start.
        let stores_a0 = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::StoreWord {
                    rt: Register::A0,
                    ..
                }
            )
        });
        let stores_a1 = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::StoreWord {
                    rt: Register::A1,
                    ..
                }
            )
        });
        assert!(stores_a0, "should store $a0 (argc) to stack");
        assert!(stores_a1, "should store $a1 (argv) to stack");
    }

    #[test]
    fn call_loads_args_into_a_registers() {
        let source = simple_main(vec![CStmt::Expr(CExpr::Call {
            func: "add".to_string(),
            args: vec![CExpr::IntLit(1), CExpr::IntLit(2)],
        })]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        // Should have: move $a0, ...; move $a1, ...; jal add
        let has_jal = main
            .body
            .iter()
            .any(|i| matches!(i, Instruction::JumpAndLink(name) if name == "add"));
        assert!(has_jal, "should have jal add");

        let moves_to_a: Vec<_> = main
            .body
            .iter()
            .filter(|i| {
                matches!(
                    i,
                    Instruction::Move {
                        rd: Register::A0
                            | Register::A1
                            | Register::A2
                            | Register::A3,
                        ..
                    }
                )
            })
            .collect();
        assert_eq!(moves_to_a.len(), 2, "should move 2 args to $a registers");
    }

    #[test]
    fn return_value_in_v0() {
        let source = simple_main(vec![CStmt::Return(Some(CExpr::IntLit(42)))]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        let has_move_v0 = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::Move {
                    rd: Register::V0,
                    ..
                }
            )
        });
        assert!(has_move_v0, "return should move value to $v0");
    }

    // -- B5.5: String operations --

    #[test]
    fn string_literal_interned_to_data_section() {
        let source = simple_main(vec![CStmt::Decl {
            name: "msg".to_string(),
            ty: CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
            init: Some(CExpr::StrLit("hello".to_string())),
        }]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let has_asciiz = program
            .data
            .iter()
            .any(|d| matches!(d, DataEntry::Asciiz { value, .. } if value == "hello"));
        assert!(has_asciiz, "should have .asciiz 'hello' in data section");

        let main = &program.functions[0];
        let has_la = main
            .body
            .iter()
            .any(|i| matches!(i, Instruction::LoadAddr { label, .. } if label == "_str_0"));
        assert!(has_la, "should have la to string label");
    }

    #[test]
    fn duplicate_strings_share_label() {
        let source = simple_main(vec![
            CStmt::Decl {
                name: "a".to_string(),
                ty: CType::Ptr(Box::new(CType::Char)),
                init: Some(CExpr::StrLit("dup".to_string())),
            },
            CStmt::Decl {
                name: "b".to_string(),
                ty: CType::Ptr(Box::new(CType::Char)),
                init: Some(CExpr::StrLit("dup".to_string())),
            },
        ]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let dup_entries: Vec<_> = program
            .data
            .iter()
            .filter(|d| matches!(d, DataEntry::Asciiz { value, .. } if value == "dup"))
            .collect();
        assert_eq!(
            dup_entries.len(),
            1,
            "duplicate strings should share one label"
        );
    }

    #[test]
    fn strcmp_loop_structure() {
        let config = MipsConfig::default();
        let mut ctx = LowerCtx::new(&config);
        let mut state = FnState::new();

        let a = state.alloc_temp().expect("should allocate temp");
        let b = state.alloc_temp().expect("should allocate temp");
        let result = ctx.emit_strcmp(&mut state, a, b);

        assert!(result.temp_index().is_some());
        // Should contain LoadByte, BranchNe, BranchEq, AddImm, Jump.
        let has_lb = state
            .body
            .iter()
            .any(|i| matches!(i, Instruction::LoadByte { .. }));
        let has_bne = state
            .body
            .iter()
            .any(|i| matches!(i, Instruction::BranchNe { .. }));
        let has_jump = state.body.iter().any(|i| matches!(i, Instruction::Jump(_)));
        assert!(has_lb, "strcmp should have lb instructions");
        assert!(has_bne, "strcmp should have bne for byte comparison");
        assert!(has_jump, "strcmp should have loop jump");
    }

    #[test]
    fn strcpy_loop_structure() {
        let config = MipsConfig::default();
        let mut ctx = LowerCtx::new(&config);
        let mut state = FnState::new();

        let dst = state.alloc_temp().expect("should allocate temp");
        let src = state.alloc_temp().expect("should allocate temp");
        ctx.emit_strcpy(&mut state, dst, src);

        let has_lb = state
            .body
            .iter()
            .any(|i| matches!(i, Instruction::LoadByte { .. }));
        let has_sb = state
            .body
            .iter()
            .any(|i| matches!(i, Instruction::StoreByte { .. }));
        let has_beq = state
            .body
            .iter()
            .any(|i| matches!(i, Instruction::BranchEq { .. }));
        assert!(has_lb, "strcpy should have lb");
        assert!(has_sb, "strcpy should have sb");
        assert!(has_beq, "strcpy should have beq (null terminator check)");
    }

    // -- B5.6: Syscall emission --

    #[test]
    fn exit_call_becomes_syscall_10() {
        let source = simple_main(vec![CStmt::Expr(CExpr::Call {
            func: "exit".to_string(),
            args: vec![],
        })]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        let has_li_v0_10 = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::LoadImm {
                    rt: Register::V0,
                    imm: 10
                }
            )
        });
        let has_syscall = main.body.iter().any(|i| matches!(i, Instruction::Syscall));
        assert!(has_li_v0_10, "exit should load syscall 10 into $v0");
        assert!(has_syscall, "exit should emit syscall");
    }

    #[test]
    fn printf_becomes_print_string_syscall() {
        let source = simple_main(vec![CStmt::Expr(CExpr::Call {
            func: "printf".to_string(),
            args: vec![CExpr::StrLit("hello".to_string())],
        })]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        let has_li_v0_4 = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::LoadImm {
                    rt: Register::V0,
                    imm: 4
                }
            )
        });
        assert!(
            has_li_v0_4,
            "printf should become print_string syscall (v0=4)"
        );
    }

    #[test]
    fn malloc_becomes_sbrk_syscall() {
        let source = simple_main(vec![CStmt::Decl {
            name: "ptr".to_string(),
            ty: CType::Ptr(Box::new(CType::Void)),
            init: Some(CExpr::Malloc(Box::new(CExpr::IntLit(100)))),
        }]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        let has_sbrk = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::LoadImm {
                    rt: Register::V0,
                    imm: 9
                }
            )
        });
        assert!(has_sbrk, "malloc should become sbrk syscall (v0=9)");
    }

    #[test]
    fn open_becomes_syscall_13() {
        let source = simple_main(vec![CStmt::Expr(CExpr::Call {
            func: "open".to_string(),
            args: vec![CExpr::StrLit("test.txt".to_string()), CExpr::IntLit(0)],
        })]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        let has_li_v0_13 = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::LoadImm {
                    rt: Register::V0,
                    imm: 13
                }
            )
        });
        assert!(has_li_v0_13, "open should become syscall 13");
    }

    // -- B5.7: Integration tests --

    #[test]
    fn lower_c_if_to_branch_sequence() {
        let source = simple_main(vec![
            CStmt::Decl {
                name: "x".to_string(),
                ty: CType::Int(CIntKind::Int),
                init: Some(CExpr::IntLit(5)),
            },
            CStmt::If {
                cond: CExpr::BinOp {
                    left: Box::new(CExpr::Var("x".to_string())),
                    op: ">".to_string(),
                    right: Box::new(CExpr::IntLit(0)),
                },
                then_body: vec![CStmt::Expr(CExpr::Call {
                    func: "print_int".to_string(),
                    args: vec![CExpr::Var("x".to_string())],
                })],
                else_body: None,
            },
        ]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        let has_beq = main
            .body
            .iter()
            .any(|i| matches!(i, Instruction::BranchEq { .. }));
        let has_print_int = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::LoadImm {
                    rt: Register::V0,
                    imm: 1
                }
            )
        });
        assert!(has_beq, "if condition should produce beq");
        assert!(has_print_int, "then-body should emit print_int syscall");
    }

    #[test]
    fn lower_c_for_loop_to_branch_sequence() {
        let source = simple_main(vec![
            CStmt::Decl {
                name: "sum".to_string(),
                ty: CType::Int(CIntKind::Int),
                init: Some(CExpr::IntLit(0)),
            },
            CStmt::For {
                init: Box::new(CStmt::Decl {
                    name: "i".to_string(),
                    ty: CType::Int(CIntKind::Int),
                    init: Some(CExpr::IntLit(0)),
                }),
                cond: CExpr::BinOp {
                    left: Box::new(CExpr::Var("i".to_string())),
                    op: "<".to_string(),
                    right: Box::new(CExpr::IntLit(10)),
                },
                step: Box::new(CStmt::Expr(CExpr::UnaryOp {
                    op: "++".to_string(),
                    expr: Box::new(CExpr::Var("i".to_string())),
                })),
                body: vec![CStmt::Assign {
                    lhs: CExpr::Var("sum".to_string()),
                    rhs: CExpr::BinOp {
                        left: Box::new(CExpr::Var("sum".to_string())),
                        op: "+".to_string(),
                        right: Box::new(CExpr::Var("i".to_string())),
                    },
                }],
            },
        ]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        // Should have: for_label, beq (exit), loop body, addi (step), j loop, endfor.
        let labels: Vec<_> = main
            .body
            .iter()
            .filter(|i| matches!(i, Instruction::Label(_)))
            .collect();
        assert!(labels.len() >= 2, "for loop should have loop + end labels");

        let has_slt = main
            .body
            .iter()
            .any(|i| matches!(i, Instruction::SetLt { .. }));
        assert!(has_slt, "i < 10 should produce slt");

        let has_addi = main
            .body
            .iter()
            .any(|i| matches!(i, Instruction::AddImm { imm: 1, .. }));
        assert!(has_addi, "i++ should produce addi with imm=1");
    }

    #[test]
    fn lower_multiple_functions() {
        let source = CSourceFile {
            includes: vec![],
            items: vec![
                CItem::FnDef(CFnDef {
                    name: "helper".to_string(),
                    return_type: CType::Int(CIntKind::Int),
                    params: vec![("n".to_string(), CType::Int(CIntKind::Int))],
                    body: vec![CStmt::Return(Some(CExpr::BinOp {
                        left: Box::new(CExpr::Var("n".to_string())),
                        op: "+".to_string(),
                        right: Box::new(CExpr::IntLit(1)),
                    }))],
                    is_static: false,
                }),
                CItem::FnDef(CFnDef {
                    name: "main".to_string(),
                    return_type: CType::Int(CIntKind::Int),
                    params: vec![],
                    body: vec![CStmt::Return(Some(CExpr::Call {
                        func: "helper".to_string(),
                        args: vec![CExpr::IntLit(5)],
                    }))],
                    is_static: false,
                }),
            ],
        };
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        assert_eq!(program.functions.len(), 2);
        assert_eq!(program.functions[0].label, "helper");
        assert!(!program.functions[0].is_entry);
        assert_eq!(program.functions[1].label, "main");
        assert!(program.functions[1].is_entry);
    }

    #[test]
    fn define_becomes_data_word() {
        let source = CSourceFile {
            includes: vec![],
            items: vec![
                CItem::Define {
                    name: "MAX_SIZE".to_string(),
                    value: "256".to_string(),
                },
                CItem::FnDef(CFnDef {
                    name: "main".to_string(),
                    return_type: CType::Int(CIntKind::Int),
                    params: vec![],
                    body: vec![CStmt::Return(Some(CExpr::IntLit(0)))],
                    is_static: false,
                }),
            ],
        };
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let has_word = program
            .data
            .iter()
            .any(|d| matches!(d, DataEntry::Word { label, value } if label == "MAX_SIZE" && *value == 256));
        assert!(has_word, "define should produce .word in data section");
    }

    #[test]
    fn entry_point_gets_exit_syscall() {
        let source = simple_main(vec![CStmt::Decl {
            name: "x".to_string(),
            ty: CType::Int(CIntKind::Int),
            init: Some(CExpr::IntLit(0)),
        }]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        // Main without explicit return should end with exit syscall.
        let last_two: Vec<_> = main.body.iter().rev().take(2).collect();
        assert!(
            last_two.iter().any(|i| matches!(i, Instruction::Syscall)),
            "main without return should end with exit syscall"
        );
    }

    #[test]
    fn while_loop_structure() {
        let source = simple_main(vec![
            CStmt::Decl {
                name: "n".to_string(),
                ty: CType::Int(CIntKind::Int),
                init: Some(CExpr::IntLit(10)),
            },
            CStmt::While {
                cond: CExpr::BinOp {
                    left: Box::new(CExpr::Var("n".to_string())),
                    op: ">".to_string(),
                    right: Box::new(CExpr::IntLit(0)),
                },
                body: vec![CStmt::Assign {
                    lhs: CExpr::Var("n".to_string()),
                    rhs: CExpr::BinOp {
                        left: Box::new(CExpr::Var("n".to_string())),
                        op: "-".to_string(),
                        right: Box::new(CExpr::IntLit(1)),
                    },
                }],
            },
        ]);
        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        let main = &program.functions[0];
        let labels: Vec<String> = main
            .body
            .iter()
            .filter_map(|i| match i {
                Instruction::Label(l) => Some(l.clone()),
                _ => None,
            })
            .collect();
        assert!(
            labels.iter().any(|l| l.contains("while")),
            "should have while loop label"
        );
        assert!(
            labels.iter().any(|l| l.contains("endwhile")),
            "should have endwhile label"
        );
    }

    // -- Integration: C makegen IR → expected MIPS program structure --

    #[test]
    fn lower_makegen_c_ir_to_mips_program() {
        let source = CSourceFile {
            includes: vec![CItem::Include {
                path: "stdio.h".to_string(),
                system: true,
            }],
            items: vec![
                CItem::Define {
                    name: "OP_READ".to_string(),
                    value: "0".to_string(),
                },
                CItem::FnDef(CFnDef {
                    name: "main".to_string(),
                    return_type: CType::Int(CIntKind::Int),
                    params: vec![(
                        "path".to_string(),
                        CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
                    )],
                    body: vec![
                        CStmt::Comment("step 0: load_registry".to_string()),
                        CStmt::Decl {
                            name: "registry".to_string(),
                            ty: CType::Ptr(Box::new(CType::Void)),
                            init: Some(CExpr::Null),
                        },
                        CStmt::Blank,
                        CStmt::Comment("step 1: prepare_read".to_string()),
                        CStmt::Decl {
                            name: "read_request".to_string(),
                            ty: CType::Ptr(Box::new(CType::Void)),
                            init: None,
                        },
                        CStmt::Decl {
                            name: "read_request_rc".to_string(),
                            ty: CType::Int(CIntKind::Int),
                            init: Some(CExpr::Call {
                                func: "gunbc_file_read_request".to_string(),
                                args: vec![CExpr::Var("path".to_string())],
                            }),
                        },
                        CStmt::If {
                            cond: CExpr::BinOp {
                                left: Box::new(CExpr::Var("read_request_rc".to_string())),
                                op: "!=".to_string(),
                                right: Box::new(CExpr::IntLit(0)),
                            },
                            then_body: vec![CStmt::Return(Some(CExpr::IntLit(-1)))],
                            else_body: None,
                        },
                        CStmt::Comment("step 2: render".to_string()),
                        CStmt::Decl {
                            name: "content".to_string(),
                            ty: CType::Ptr(Box::new(CType::Const(Box::new(CType::Char)))),
                            init: Some(CExpr::StrLit("# Generated Makefile\n".to_string())),
                        },
                        CStmt::Return(Some(CExpr::IntLit(0))),
                    ],
                    is_static: false,
                }),
            ],
        };

        let config = MipsConfig::default();
        let program = lower_to_mips(&source, &config).unwrap();

        // Program structure.
        assert_eq!(program.target, AsmTarget::Mips32);
        assert!(!program.data.is_empty(), "should have data entries");
        assert_eq!(program.functions.len(), 1);

        // Data section: define + string literal.
        let has_op_read = program.data.iter().any(
            |d| matches!(d, DataEntry::Word { label, value } if label == "OP_READ" && *value == 0),
        );
        assert!(has_op_read, "should have OP_READ word");

        let has_makefile_str = program.data.iter().any(
            |d| matches!(d, DataEntry::Asciiz { value, .. } if value.contains("Generated Makefile")),
        );
        assert!(has_makefile_str, "should have makefile string");

        // Main function.
        let main = &program.functions[0];
        assert!(main.is_entry);
        assert!(main.frame.ra_offset.is_some(), "has calls → needs $ra");

        // Should have: param store, jal, branch (error check), jr $ra.
        let has_param_store = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::StoreWord {
                    rt: Register::A0,
                    ..
                }
            )
        });
        assert!(has_param_store, "should store path param from $a0");

        let has_jal = main.body.iter().any(
            |i| matches!(i, Instruction::JumpAndLink(name) if name == "gunbc_file_read_request"),
        );
        assert!(has_jal, "should have jal to gunbc_file_read_request");

        let has_branch = main.body.iter().any(|i| {
            matches!(
                i,
                Instruction::BranchEq { .. } | Instruction::BranchNe { .. }
            )
        });
        assert!(has_branch, "should have branch for error check");
    }
}
