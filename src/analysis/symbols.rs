use std::collections::HashMap;

use tower_lsp::lsp_types::Range;

use crate::parser::{Decl, File};
use crate::text::char_idx_to_position;

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub range: Range,
}

#[derive(Debug, Default, Clone)]
pub struct SymbolIndex {
    symbols_by_name: HashMap<String, Vec<Symbol>>,
}

impl SymbolIndex {
    pub fn from_file(file: &File, text: &ropey::Rope) -> Self {
        let mut index = SymbolIndex::default();

        for decl in &file.declarations {
            let (name, span) = match decl {
                Decl::Class(class_decl) => (&class_decl.name.name, class_decl.name.span),
                Decl::Fun(fun_decl) => (&fun_decl.name.name, fun_decl.name.span),
                Decl::Property(prop_decl) => (&prop_decl.name.name, prop_decl.name.span),
            };

            let start = char_idx_to_position(text, byte_to_char_idx(text, span.start));
            let end = char_idx_to_position(text, byte_to_char_idx(text, span.end));

            let symbol = Symbol {
                name: name.clone(),
                range: Range { start, end },
            };

            index
                .symbols_by_name
                .entry(name.clone())
                .or_default()
                .push(symbol);
        }

        index
    }

    pub fn find(&self, name: &str) -> Option<&Symbol> {
        self.symbols_by_name.get(name).and_then(|list| list.first())
    }

    pub fn find_all(&self, name: &str) -> Vec<Symbol> {
        self.symbols_by_name
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn all_symbols(&self) -> Vec<Symbol> {
        self.symbols_by_name
            .values()
            .flat_map(|list| list.iter().cloned())
            .collect()
    }
}

fn byte_to_char_idx(text: &ropey::Rope, byte: usize) -> usize {
    let clamped = byte.min(text.len_bytes());
    text.byte_slice(..clamped).chars().count()
}
