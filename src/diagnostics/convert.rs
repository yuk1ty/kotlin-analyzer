use ropey::Rope;
use tower_lsp::lsp_types::{Diagnostic as LspDiagnostic, DiagnosticSeverity, Range};

use crate::diagnostics::{Diagnostic, Severity};
use crate::text::char_idx_to_position;

pub fn to_lsp_diagnostic(diag: &Diagnostic, text: &Rope) -> LspDiagnostic {
    let start = char_idx_to_position(text, diag.range.start);
    let end = char_idx_to_position(text, diag.range.end);
    let severity = match diag.severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
    };

    LspDiagnostic {
        range: Range { start, end },
        severity: Some(severity),
        source: Some("kotlin-analyzer".to_string()),
        message: diag.message.clone(),
        ..Default::default()
    }
}
