mod ast;
mod grammar;
mod lexer;

use chumsky::error::Simple;
use chumsky::span::SimpleSpan;

use crate::diagnostics::{Diagnostic, Severity, TextRange};

pub use ast::{ClassDecl, Decl, File, FunDecl, Ident, Mutability, PropertyDecl};
pub fn parse<'src>(text: &'src str) -> Vec<Diagnostic> {
    parse_with_ast(text).1
}

pub fn parse_with_ast<'src>(text: &'src str) -> (Option<File>, Vec<Diagnostic>) {
    let (tokens, lex_errors) = lexer::lex(text);
    let mut diagnostics = lex_errors
        .into_iter()
        .map(|err| diag_from_lex_error(text, err))
        .collect::<Vec<_>>();

    let (ast, parse_diagnostics) = grammar::parse(&tokens, text);
    diagnostics.extend(parse_diagnostics);
    (ast, diagnostics)
}

fn diag_from_lex_error(text: &str, err: Simple<'_, char>) -> Diagnostic {
    let span = err.span().clone();
    let range = span_to_range(text, span);
    let message = match err.found() {
        Some(found) => format!("Unexpected character: {found:?}"),
        None => "Unexpected end of input".to_string(),
    };

    Diagnostic::new(range, Severity::Error, message)
}

fn span_to_range(text: &str, span: SimpleSpan<usize>) -> TextRange {
    let start = byte_to_char_idx(text, span.start);
    let end = byte_to_char_idx(text, span.end);
    TextRange::new(start, end)
}

fn byte_to_char_idx(text: &str, byte: usize) -> usize {
    let clamped = byte.min(text.len());
    text[..clamped].chars().count()
}
