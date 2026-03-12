//! Lowering trait for IR tier transformations.
//!
//! Each tier transition (AbstractIR → SystemsIR, AbstractIR → CStyleIR,
//! CStyleIR → RegisterIR) is expressed as a `LowerIR` implementation.
//! This makes the compilation pipeline composable: the same AbstractIR
//! feeds into multiple target paths.
//!
//! ```text
//! AbstractIR (Tier 0)
//!     ├──LowerIR──→ SystemsIR (Tier 1, Rust)    ──→ CodeRenderer → .rs
//!     ├──LowerIR──→ ManagedIR (Tier 2, Go)      ──→ CodeRenderer → .go
//!     └──LowerIR──→ CStyleIR  (Tier 3, C)       ──→ CodeRenderer → .c
//!                       └──LowerIR──→ RegisterIR (Tier 4, MIPS) ──→ .s
//! ```

use super::c_ir::CSourceFile;
use super::register_ir::AsmProgram;
use super::SourceFile;

/// Lower one IR tier to another.
///
/// `From` is the source IR type, `To` is the target IR type.
/// The `Context` associated type carries any configuration needed
/// for the lowering (e.g., target-specific type mappings, naming
/// conventions, optimization level).
///
/// # Examples
///
/// ```text
/// struct RustLowering;
/// impl LowerIR<SourceFile, SourceFile> for RustLowering {
///     type Context = RustConfig;
///     type Error = LowerError;
///     fn lower(source: &SourceFile, ctx: &Self::Context) -> Result<SourceFile, Self::Error> {
///         // Add Result wrapping, derives, use statements, etc.
///     }
/// }
/// ```
pub trait LowerIR<From, To> {
    /// Configuration/context for the lowering pass.
    type Context;
    /// Error type for lowering failures.
    type Error: std::fmt::Debug;

    /// Transform source IR into target IR.
    fn lower(source: &From, ctx: &Self::Context) -> Result<To, Self::Error>;
}

/// Errors that can occur during IR lowering.
#[derive(Debug, Clone)]
pub enum LowerError {
    /// A source construct has no equivalent in the target tier.
    UnsupportedConstruct {
        tier_from: &'static str,
        tier_to: &'static str,
        construct: String,
    },
    /// A type could not be mapped to the target tier's type system.
    UnmappedType { ty: String, target: &'static str },
    /// Internal consistency error.
    InternalError(String),
}

impl std::fmt::Display for LowerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedConstruct {
                tier_from,
                tier_to,
                construct,
            } => write!(
                f,
                "cannot lower `{construct}` from {tier_from} to {tier_to}"
            ),
            Self::UnmappedType { ty, target } => {
                write!(f, "type `{ty}` has no mapping for target {target}")
            }
            Self::InternalError(msg) => write!(f, "internal lowering error: {msg}"),
        }
    }
}

// ---------------------------------------------------------------------------
// Marker types for the standard lowering paths
// ---------------------------------------------------------------------------

/// AbstractIR (Tier 0) → SystemsIR (Tier 1, Rust).
///
/// Adds: Result wrapping, derive macros, `use` statements, ownership
/// (`&`, `&mut`, `*`), implicit return (`TailExpr`).
pub struct ToRust;

/// AbstractIR (Tier 0) → ManagedIR (Tier 2, Go).
///
/// Adds: multi-return `(result, error)`, `if err != nil` patterns,
/// package declaration, Go imports.
pub struct ToGo;

/// AbstractIR (Tier 0) → CStyleIR (Tier 3, C).
///
/// Converts: `String` → `char*`, `Vec` → `(ptr, len)`, closures → function
/// pointers, let-bindings → explicit declarations with types.
pub struct ToC;

/// CStyleIR (Tier 3) → RegisterIR (Tier 4, MIPS).
///
/// Converts: variables → stack offsets/registers, function calls →
/// `$a0`-`$a3` + `jal`, control flow → branch/jump + labels,
/// I/O → syscall sequences.
pub struct ToMips;

// ---------------------------------------------------------------------------
// Composable lowering
// ---------------------------------------------------------------------------

/// Lower through two tiers in sequence.
///
/// If `A` can lower `From → Mid` and `B` can lower `Mid → To`, then
/// `Compose<A, B, Mid>` can lower `From → To`.
pub struct Compose<A, B, Mid>(std::marker::PhantomData<(A, B, Mid)>);

impl<From, Mid, To, A, B> LowerIR<From, To> for Compose<A, B, Mid>
where
    A: LowerIR<From, Mid>,
    B: LowerIR<Mid, To, Context = A::Context, Error = A::Error>,
{
    type Context = A::Context;
    type Error = A::Error;

    fn lower(source: &From, ctx: &Self::Context) -> Result<To, Self::Error> {
        let mid = A::lower(source, ctx)?;
        B::lower(&mid, ctx)
    }
}

// ---------------------------------------------------------------------------
// Tier marker trait (for compile-time tier checking)
// ---------------------------------------------------------------------------

/// Marker trait identifying which IR tier a type belongs to.
pub trait IrTier {
    /// Human-readable tier name (e.g., "AbstractIR", "CStyleIR").
    const TIER_NAME: &'static str;
    /// Numeric tier level (0 = most abstract, 4 = most concrete).
    const TIER_LEVEL: u8;
}

impl IrTier for SourceFile {
    const TIER_NAME: &'static str = "AbstractIR";
    const TIER_LEVEL: u8 = 0;
}

impl IrTier for CSourceFile {
    const TIER_NAME: &'static str = "CStyleIR";
    const TIER_LEVEL: u8 = 3;
}

impl IrTier for AsmProgram {
    const TIER_NAME: &'static str = "RegisterIR";
    const TIER_LEVEL: u8 = 4;
}
