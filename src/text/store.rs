use std::collections::HashMap;

use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::text::TextDocument;

#[derive(Default)]
pub struct DocumentStore {
    documents: HashMap<Url, TextDocument>,
}

impl DocumentStore {
    pub fn open(&mut self, uri: Url, version: i32, text: String) {
        let document = TextDocument::new(uri.clone(), version, text);
        self.documents.insert(uri, document);
    }

    pub fn apply_changes(
        &mut self,
        uri: &Url,
        version: i32,
        changes: &[TextDocumentContentChangeEvent],
    ) {
        if let Some(document) = self.documents.get_mut(uri) {
            for change in changes {
                document.apply_change(change);
            }
            document.set_version(version);
        }
    }

    pub fn close(&mut self, uri: &Url) {
        self.documents.remove(uri);
    }

    pub fn get(&self, uri: &Url) -> Option<&TextDocument> {
        self.documents.get(uri)
    }

    pub fn uris(&self) -> Vec<Url> {
        self.documents.keys().cloned().collect()
    }
}
