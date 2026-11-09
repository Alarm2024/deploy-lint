use crate::finding::Finding;
use crate::lint::{dockerfile, kubernetes, systemd};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const DOCKERFILE_NAMES: &[&str] = &["Dockerfile", "dockerfile", "Containerfile", "containerfile"];
const K8S_EXTENSIONS: &[&str] = &["yaml", "yml"];
const SERVICE_EXTENSION: &str = "service";

pub fn lint_path(path: &Path) -> Result<Vec<Finding>, String> {
    if path.is_file() {
        return lint_file(path);
    }
    if path.is_dir() {
        let mut all = Vec::new();
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let p = entry.path();
            if is_supported(p) {
                all.extend(lint_file(p)?);
            }
        }
        return Ok(all);
    }
    Err(format!("path not found: {}", path.display()))
}

fn is_supported(path: &Path) -> bool {
    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if DOCKERFILE_NAMES.contains(&name) {
        return true;
    }
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| K8S_EXTENSIONS.contains(&e))
        .unwrap_or(false)
    {
        return true;
    }
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| e == SERVICE_EXTENSION)
        .unwrap_or(false)
}

fn lint_file(path: &Path) -> Result<Vec<Finding>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    let path_buf = PathBuf::from(path);

    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if DOCKERFILE_NAMES.contains(&name) {
        return Ok(dockerfile::lint(&path_buf, &content));
    }
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| K8S_EXTENSIONS.contains(&e))
        .unwrap_or(false)
    {
        return Ok(kubernetes::lint(&path_buf, &content));
    }
    if path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e == SERVICE_EXTENSION)
        .unwrap_or(false)
    {
        return Ok(systemd::lint(&path_buf, &content));
    }
    Ok(Vec::new())
}

pub fn line_for_pattern(content: &str, patterns: &[&str]) -> usize {
    for (idx, line) in content.lines().enumerate() {
        for pat in patterns {
            if line.contains(pat) {
                return idx + 1;
            }
        }
    }
    1
}
