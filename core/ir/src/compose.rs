//! Macro for composing operation enums.
//!
//! This macro generates operation enum definitions along with their
//! `Executable` implementation, reducing boilerplate when combining
//! multiple operation types.
//!
//! # Example
//!
//! ```ignore
//! use gunbc_ir::compose_ops;
//! use gunbc_exec::Executable;
//! use gunbc_primitives::PrimitiveOp;
//! use gunbc_lib_markdown::MarkdownOp;
//!
//! compose_ops! {
//!     /// Combined operation type for example graphs.
//!     pub enum ExampleGraphOp {
//!         Primitive(PrimitiveOp),
//!         Markdown(MarkdownOp),
//!     }
//! }
//!
//! // The macro generates:
//! // 1. The enum definition with the doc comment
//! // 2. An Executable impl that delegates to each variant
//! ```

/// Macro to generate operation enum + Executable impl.
///
/// This macro reduces boilerplate when creating composite operation types
/// that combine multiple operation libraries.
///
/// # Usage
///
/// ```ignore
/// compose_ops! {
///     $(#[$meta:meta])*
///     $vis:vis enum $name:ident {
///         $($variant:ident($ty:ty)),* $(,)?
///     }
/// }
/// ```
///
/// # Generated Code
///
/// For each call, the macro generates:
/// 1. The enum with all specified variants
/// 2. An implementation of `gunbc_exec::Executable` that matches on each
///    variant and delegates to the inner type's `execute` method
///
/// # Example
///
/// ```ignore
/// compose_ops! {
///     #[derive(Debug, Clone)]
///     pub enum MyGraphOp {
///         Primitive(PrimitiveOp),
///         Custom(CustomOp),
///     }
/// }
/// ```
///
/// Expands to:
///
/// ```ignore
/// #[derive(Debug, Clone)]
/// pub enum MyGraphOp {
///     Primitive(PrimitiveOp),
///     Custom(CustomOp),
/// }
///
/// impl gunbc_exec::Executable for MyGraphOp {
///     fn execute(
///         &self,
///         inputs: std::collections::HashMap<String, gunbc_ir::Value>,
///     ) -> Result<std::collections::HashMap<String, gunbc_ir::Value>, gunbc_exec::ExecError> {
///         match self {
///             Self::Primitive(op) => op.execute(inputs),
///             Self::Custom(op) => op.execute(inputs),
///         }
///     }
/// }
/// ```
/// Macro to generate operation enum + Executable impl.
///
/// Note: This macro requires `gunbc_exec` to be in scope. The caller must
/// import both `gunbc_ir` and `gunbc_exec` for the macro to work.
#[macro_export]
macro_rules! compose_ops {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($variant:ident($ty:ty)),* $(,)?
        }
    ) => {
        $(#[$meta])*
        $vis enum $name {
            $($variant($ty)),*
        }

        impl gunbc_exec::Executable for $name {
            fn execute(
                &self,
                inputs: std::collections::HashMap<String, gunbc_ir::Value>,
            ) -> Result<std::collections::HashMap<String, gunbc_ir::Value>, gunbc_exec::ExecError> {
                match self {
                    $(Self::$variant(op) => op.execute(inputs)),*
                }
            }
        }
    };
}

// Tests are in a separate integration test file that has access to both
// gunbc_ir and gunbc_exec crates. The macro is tested there.
