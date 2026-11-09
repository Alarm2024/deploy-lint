use crate::finding::{Finding, Severity};
use crate::scan::line_for_pattern;
use std::path::PathBuf;

const DL001: &str = "DL001";
const DL002: &str = "DL002";
const DL003: &str = "DL003";
const DL005: &str = "DL005";

pub fn lint(file: &PathBuf, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(check_root(file, content));
    findings.extend(check_image_tags(file, content));
    findings.extend(check_env_secrets(file, content));
    findings.extend(check_healthcheck(file, content));
    findings
}

fn check_root(file: &PathBuf, content: &str) -> Vec<Finding> {
    let mut has_user_non_root = false;
    let mut user_root_line = None;

    for (idx, raw) in content.lines().enumerate() {
        let line = raw.trim();
        let upper = line.to_uppercase();
        if upper.starts_with("USER ") {
            let user = line[5..].trim();
            if user.eq_ignore_ascii_case("root") || user == "0" {
                user_root_line = Some(idx + 1);
            } else {
                has_user_non_root = true;
            }
        }
    }

    if has_user_non_root {
        return Vec::new();
    }

    let line = user_root_line.unwrap_or(1);
    vec![Finding::new(
        DL001,
        Severity::High,
        file.clone(),
        line,
        "Container is configured to run as root",
        "Add a non-root USER (and use USER directive before CMD/ENTRYPOINT)",
    )]
}

fn check_image_tags(file: &PathBuf, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (idx, raw) in content.lines().enumerate() {
        let line = raw.trim();
        let upper = line.to_uppercase();
        if !upper.starts_with("FROM ") {
            continue;
        }
        let rest = line[5..].trim();
        let image = rest
            .split_whitespace()
            .next()
            .unwrap_or(rest)
            .trim_end_matches('\r');
        if image_uses_latest_or_untagged(image) {
            findings.push(Finding::new(
                DL002,
                Severity::High,
                file.clone(),
                idx + 1,
                format!("Base image uses a floating tag or no tag: {image}"),
                "Pin the image to an explicit version tag or digest (avoid :latest)",
            ));
        }
    }

    findings
}

fn image_uses_latest_or_untagged(image: &str) -> bool {
    let image = image.split('@').next().unwrap_or(image);
    if !image.contains(':') {
        return true;
    }
    image.ends_with(":latest")
}

fn check_env_secrets(file: &PathBuf, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    for (idx, raw) in content.lines().enumerate() {
        let line = raw.trim();
        let upper = line.to_uppercase();
        if !upper.starts_with("ENV ") {
            continue;
        }
        let rest = line[4..].trim();
        if let Some((name, value)) = parse_env_assignment(rest) {
            if is_secret_name(&name) && !value.is_empty() && !is_env_ref(&value) {
                findings.push(Finding::new(
                    DL003,
                    Severity::High,
                    file.clone(),
                    idx + 1,
                    format!("Secret-like environment variable {} has a literal value (masked)", name),
                    "Load secrets from a secret manager, mount, or runtime env — do not bake literals into the image",
                ));
            }
        }
    }

    findings
}

fn parse_env_assignment(rest: &str) -> Option<(String, String)> {
    if let Some((name, value)) = rest.split_once('=') {
        let name = name.trim().to_string();
        let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
        return Some((name, value));
    }
    let mut parts = rest.split_whitespace();
    let name = parts.next()?.to_string();
    let value = parts.collect::<Vec<_>>().join(" ");
    Some((name, value))
}

pub(crate) fn is_secret_name(name: &str) -> bool {
    let upper = name.to_uppercase();
    upper.contains("KEY")
        || upper.contains("TOKEN")
        || upper.contains("SECRET")
        || upper.contains("PASSWORD")
}

fn is_env_ref(value: &str) -> bool {
    value.starts_with('$') || value.contains("${")
}

fn check_healthcheck(file: &PathBuf, content: &str) -> Vec<Finding> {
    let has_health = content.lines().any(|l| {
        let t = l.trim();
        t.to_uppercase().starts_with("HEALTHCHECK ")
            || t.to_uppercase().starts_with("HEALTHCHECK\t")
    });

    if has_health {
        return Vec::new();
    }

    let line = line_for_pattern(content, &["CMD ", "ENTRYPOINT "]);
    vec![Finding::new(
        DL005,
        Severity::Medium,
        file.clone(),
        line,
        "Dockerfile has no HEALTHCHECK instruction",
        "Add HEALTHCHECK so orchestrators can detect a failed container",
    )]
}
