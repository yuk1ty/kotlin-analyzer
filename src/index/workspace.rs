use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use ropey::Rope;
use tower_lsp::lsp_types::{Location, Url};

use crate::analysis::SymbolIndex;
use crate::parser::{self, File};

#[derive(Debug, Default, Clone)]
pub struct WorkspaceIndex {
    symbols_by_name: HashMap<String, Vec<Location>>,
    symbols_by_uri: HashMap<Url, Vec<(String, Location)>>,
    files: HashMap<PathBuf, SystemTime>,
}

impl WorkspaceIndex {
    pub fn build(roots: &[Url]) -> Self {
        let mut index = WorkspaceIndex::default();
        let skip = HashSet::new();
        index.refresh_roots(roots, &skip);
        index
    }

    pub fn find(&self, name: &str) -> Option<Location> {
        self.symbols_by_name
            .get(name)
            .and_then(|items| items.first())
            .cloned()
    }

    pub fn find_all(&self, name: &str) -> Vec<Location> {
        self.symbols_by_name
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn update_from_ast(&mut self, uri: Url, file: Option<&File>, text: &str) {
        self.remove_uri(&uri);
        if let Some(file) = file {
            self.add_symbols_from_file(uri, file, text);
        }
    }

    pub fn refresh_from_disk(&mut self, uri: &Url) {
        if let Ok(path) = uri.to_file_path() {
            if let Ok(text) = fs::read_to_string(&path) {
                let (ast, _) = parser::parse_with_ast(&text);
                self.update_from_ast(uri.clone(), ast.as_ref(), &text);
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        self.files.insert(path, modified);
                    }
                }
                return;
            }
        }

        self.remove_uri(uri);
    }

    pub fn refresh_roots(&mut self, roots: &[Url], skip_uris: &HashSet<Url>) {
        let current_files = scan_roots(roots);

        for (path, modified) in &current_files {
            let uri = match Url::from_file_path(path) {
                Ok(uri) => uri,
                Err(_) => continue,
            };

            if skip_uris.contains(&uri) {
                continue;
            }

            let needs_update = self
                .files
                .get(path)
                .map(|prev| *prev != *modified)
                .unwrap_or(true);

            if needs_update {
                self.index_file_path(path);
                self.files.insert(path.clone(), *modified);
            }
        }

        let removed = self
            .files
            .keys()
            .filter(|path| !current_files.contains_key(*path))
            .cloned()
            .collect::<Vec<_>>();

        for path in removed {
            if let Ok(uri) = Url::from_file_path(&path) {
                self.remove_uri(&uri);
            }
            self.files.remove(&path);
        }
    }

    fn index_file_path(&mut self, path: &Path) {
        let Ok(text) = fs::read_to_string(path) else {
            return;
        };

        let Ok(uri) = Url::from_file_path(path) else {
            return;
        };

        let (ast, _) = parser::parse_with_ast(&text);
        self.update_from_ast(uri, ast.as_ref(), &text);
    }

    fn add_symbols_from_file(&mut self, uri: Url, file: &File, text: &str) {
        let rope = Rope::from_str(text);
        let index = SymbolIndex::from_file(file, &rope);

        let mut entries = Vec::new();
        for symbol in index.all_symbols() {
            let location = Location {
                uri: uri.clone(),
                range: symbol.range,
            };

            self.symbols_by_name
                .entry(symbol.name.clone())
                .or_default()
                .push(location.clone());

            entries.push((symbol.name, location));
        }

        if !entries.is_empty() {
            self.symbols_by_uri.insert(uri, entries);
        }
    }

    fn remove_uri(&mut self, uri: &Url) {
        let Some(entries) = self.symbols_by_uri.remove(uri) else {
            return;
        };

        for (name, location) in entries {
            if let Some(list) = self.symbols_by_name.get_mut(&name) {
                list.retain(|item| !(item.uri == location.uri && item.range == location.range));
                if list.is_empty() {
                    self.symbols_by_name.remove(&name);
                }
            }
        }
    }
}

fn scan_roots(roots: &[Url]) -> HashMap<PathBuf, SystemTime> {
    let mut files = HashMap::new();
    for root in roots {
        let Ok(path) = root.to_file_path() else {
            continue;
        };
        collect_kotlin_files_with_mtime(&path, &mut files);
    }
    files
}

fn collect_kotlin_files_with_mtime(root: &Path, out: &mut HashMap<PathBuf, SystemTime>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if is_ignored_dir(name) {
                    continue;
                }
            }
            collect_kotlin_files_with_mtime(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("kt") {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    out.insert(path, modified);
                }
            }
        }
    }
}

fn is_ignored_dir(name: &str) -> bool {
    matches!(name, ".git" | ".gradle" | ".idea" | "build" | "out" | "target")
}
