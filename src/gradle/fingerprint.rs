use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use crate::gradle::GradleClasspath;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GradleFingerprint(pub u64);

#[derive(Debug, Clone)]
pub struct GradleCacheEntry {
    pub classpath: GradleClasspath,
    pub fingerprint: GradleFingerprint,
}

pub fn compute_gradle_fingerprint(root: &Path) -> Option<GradleFingerprint> {
    let mut files = HashSet::new();
    collect_gradle_files(root, &mut files);
    if files.is_empty() {
        return None;
    }

    let mut files = files.into_iter().collect::<Vec<_>>();
    files.sort();

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for path in files {
        path.to_string_lossy().hash(&mut hasher);
        if let Ok(meta) = fs::metadata(&path) {
            meta.len().hash(&mut hasher);
            if let Ok(modified) = meta.modified() {
                if let Ok(duration) = modified.duration_since(UNIX_EPOCH) {
                    duration.as_nanos().hash(&mut hasher);
                }
            }
        }
    }

    Some(GradleFingerprint(hasher.finish()))
}

fn collect_gradle_files(root: &Path, out: &mut HashSet<PathBuf>) {
    let top_level = [
        "build.gradle",
        "build.gradle.kts",
        "settings.gradle",
        "settings.gradle.kts",
        "gradle.properties",
        "gradle.lockfile",
    ];

    for name in top_level {
        let path = root.join(name);
        if path.is_file() {
            out.insert(path);
        }
    }

    let wrapper = root
        .join("gradle")
        .join("wrapper")
        .join("gradle-wrapper.properties");
    if wrapper.is_file() {
        out.insert(wrapper);
    }

    let versions = root.join("gradle").join("libs.versions.toml");
    if versions.is_file() {
        out.insert(versions);
    }

    collect_gradle_files_recursive(root, out);
}

fn collect_gradle_files_recursive(root: &Path, out: &mut HashSet<PathBuf>) {
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
            collect_gradle_files_recursive(&path, out);
            continue;
        }

        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let is_gradle = matches!(
            name,
            "build.gradle"
                | "build.gradle.kts"
                | "settings.gradle"
                | "settings.gradle.kts"
                | "gradle.properties"
                | "gradle.lockfile"
                | "libs.versions.toml"
        );
        let is_wrapper = name == "gradle-wrapper.properties";

        if is_gradle || is_wrapper {
            out.insert(path);
        }
    }
}

fn is_ignored_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".gradle" | ".idea" | "build" | "out" | "target" | "node_modules"
    )
}
