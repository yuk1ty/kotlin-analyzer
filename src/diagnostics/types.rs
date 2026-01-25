#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub struct TextRange {
    pub start: usize,
    pub end: usize,
}

impl TextRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub range: TextRange,
    pub severity: Severity,
    pub message: String,
}

impl Diagnostic {
    pub fn new(range: TextRange, severity: Severity, message: impl Into<String>) -> Self {
        Self {
            range,
            severity,
            message: message.into(),
        }
    }
}
