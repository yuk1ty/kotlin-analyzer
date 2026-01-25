mod convert;
mod types;

pub use convert::to_lsp_diagnostic;
pub use types::{Diagnostic, Severity, TextRange};
