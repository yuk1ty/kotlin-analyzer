use ropey::Rope;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::text::positions::position_to_char_idx;

#[derive(Debug, Clone)]
pub struct TextDocument {
    uri: Url,
    version: i32,
    text: Rope,
}

impl TextDocument {
    pub fn new(uri: Url, version: i32, text: String) -> Self {
        Self {
            uri,
            version,
            text: Rope::from_str(&text),
        }
    }

    pub fn uri(&self) -> &Url {
        &self.uri
    }

    pub fn version(&self) -> i32 {
        self.version
    }

    pub fn set_version(&mut self, version: i32) {
        self.version = version;
    }

    pub fn rope(&self) -> &Rope {
        &self.text
    }

    pub fn slice(&self, start: usize, end: usize) -> String {
        if start >= end {
            return String::new();
        }
        let end = end.min(self.text.len_chars());
        let start = start.min(end);
        self.text.slice(start..end).to_string()
    }

    pub fn text(&self) -> String {
        self.text.to_string()
    }

    pub fn apply_change(&mut self, change: &TextDocumentContentChangeEvent) {
        match change.range {
            None => {
                self.text = Rope::from_str(&change.text);
            }
            Some(range) => {
                let start = position_to_char_idx(&self.text, range.start);
                let end = position_to_char_idx(&self.text, range.end);
                if start <= end {
                    self.text.remove(start..end);
                    self.text.insert(start, &change.text);
                }
            }
        }
    }
}
