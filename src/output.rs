use crate::finding::{Finding, Severity};
use serde::Serialize;
use std::io::{self, Write};
use std::path::Path;

#[derive(Serialize)]
struct JsonReport<'a> {
    findings: &'a [Finding],
    summary: Summary,
}

#[derive(Serialize)]
struct Summary {
    total: usize,
    high: usize,
    medium: usize,
    low: usize,
}

#[derive(Serialize)]
struct SarifLog {
    #[serde(rename = "$schema")]
    schema: &'static str,
    version: &'static str,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResult>,
}

#[derive(Serialize)]
struct SarifTool {
    driver: SarifDriver,
}

#[derive(Serialize)]
struct SarifDriver {
    name: &'static str,
    version: &'static str,
    information_uri: &'static str,
}

#[derive(Serialize)]
struct SarifResult {
    rule_id: String,
    level: &'static str,
    message: SarifMessage,
    locations: Vec<SarifLocation>,
}

#[derive(Serialize)]
struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
struct SarifLocation {
    physical_location: SarifPhysicalLocation,
}

#[derive(Serialize)]
struct SarifPhysicalLocation {
    artifact_location: SarifArtifactLocation,
    region: SarifRegion,
}

#[derive(Serialize)]
struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
struct SarifRegion {
    start_line: usize,
}

pub fn print_json(findings: &[Finding], stdout: &mut dyn Write) -> io::Result<()> {
    let summary = summary_counts(findings);
    let report = JsonReport {
        findings,
        summary,
    };
    writeln!(stdout, "{}", serde_json::to_string_pretty(&report).unwrap())
}

pub fn print_sarif(findings: &[Finding], stdout: &mut dyn Write) -> io::Result<()> {
    let results: Vec<SarifResult> = findings
        .iter()
        .map(|f| SarifResult {
            rule_id: f.rule_id.clone(),
            level: sarif_level(f.severity),
            message: SarifMessage {
                text: f.reason.clone(),
            },
            locations: vec![SarifLocation {
                physical_location: SarifPhysicalLocation {
                    artifact_location: SarifArtifactLocation {
                        uri: path_uri(&f.file),
                    },
                    region: SarifRegion {
                        start_line: f.line,
                    },
                },
            }],
        })
        .collect();

    let log = SarifLog {
        schema: "https://json.schemastore.org/sarif-2.1.0.json",
        version: "2.1.0",
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifDriver {
                    name: "Preflight",
                    version: env!("CARGO_PKG_VERSION"),
                    information_uri: "https://github.com/Alarm2024/deploy-lint",
                },
            },
            results,
        }],
    };

    writeln!(stdout, "{}", serde_json::to_string_pretty(&log).unwrap())
}

pub fn print_ci(findings: &[Finding], stdout: &mut dyn Write) -> io::Result<()> {
    for f in findings {
        let path = path_uri(&f.file);
        let message = ci_message(f);
        writeln!(stdout, "{path}:{} {} {}", f.line, f.rule_id, message)?;
        writeln!(
            stdout,
            "::error file={path},line={}::{}",
            f.line,
            annotation_body(f)
        )?;
    }
    Ok(())
}

fn summary_counts(findings: &[Finding]) -> Summary {
    Summary {
        total: findings.len(),
        high: findings
            .iter()
            .filter(|f| f.severity == Severity::High)
            .count(),
        medium: findings
            .iter()
            .filter(|f| f.severity == Severity::Medium)
            .count(),
        low: findings
            .iter()
            .filter(|f| f.severity == Severity::Low)
            .count(),
    }
}

fn path_uri(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn sarif_level(severity: Severity) -> &'static str {
    match severity {
        Severity::High => "error",
        Severity::Medium => "warning",
        Severity::Low => "note",
    }
}

fn ci_message(f: &Finding) -> String {
    f.reason.clone()
}

fn annotation_body(f: &Finding) -> String {
    let raw = format!("{}: {}", f.rule_id, f.reason);
    escape_annotation(&raw)
}

/// Escape reserved characters for GitHub Actions workflow commands.
fn escape_annotation(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '%' => out.push_str("%25"),
            '\r' => out.push_str("%0D"),
            '\n' => out.push_str("%0A"),
            _ => out.push(ch),
        }
    }
    out
}

pub fn print_table(findings: &[Finding], stdout: &mut dyn Write) -> io::Result<()> {
    if findings.is_empty() {
        writeln!(stdout, "No findings.")?;
        return Ok(());
    }

    let cols = [
        col_width(findings, |f| f.rule_id.len()),
        8usize,
        col_width(findings, |f| f.file.display().to_string().len()),
        6usize,
        48usize,
        48usize,
    ];

    writeln!(
        stdout,
        "{:<cols$} | {:<8} | {:<file$} | {:<6} | {:<48} | {}",
        "RULE",
        "SEVERITY",
        "FILE",
        "LINE",
        "REASON",
        "FIX",
        cols = cols[0],
        file = cols[2]
    )?;
    writeln!(stdout, "{}", "-".repeat(cols.iter().sum::<usize>() + 20))?;

    for f in findings {
        writeln!(
            stdout,
            "{:<cols$} | {:<8} | {:<file$} | {:<6} | {:<48} | {}",
            f.rule_id,
            f.severity.as_str(),
            f.file.display(),
            f.line,
            truncate(&f.reason, 48),
            truncate(&f.fix_hint, 60),
            cols = cols[0],
            file = cols[2]
        )?;
    }

    Ok(())
}

fn col_width(findings: &[Finding], len: impl Fn(&Finding) -> usize) -> usize {
    findings
        .iter()
        .map(len)
        .max()
        .unwrap_or(8)
        .max(4)
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        return s.to_string();
    }
    let mut t: String = s.chars().take(max.saturating_sub(1)).collect();
    t.push('…');
    t
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::finding::Finding;
    use std::path::PathBuf;

    #[test]
    fn sarif_never_includes_fix_hint_secrets() {
        let findings = vec![Finding::new(
            "DL003",
            Severity::High,
            PathBuf::from("Dockerfile"),
            2,
            "Secret-like environment variable API_KEY has a literal value (masked)",
            "never-print-this-secret-value-xyz",
        )];
        let mut buf = Vec::new();
        print_sarif(&findings, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("DL003"));
        assert!(!s.contains("never-print-this-secret-value-xyz"));
        assert!(!s.contains("API_KEY=supersecret"));
    }

    #[test]
    fn ci_lines_and_annotations() {
        let findings = vec![Finding::new(
            "DL001",
            Severity::High,
            PathBuf::from("demo/bad/Dockerfile"),
            1,
            "Container is configured to run as root",
            "Add USER",
        )];
        let mut buf = Vec::new();
        print_ci(&findings, &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(s.contains("demo/bad/Dockerfile:1 DL001 Container is configured to run as root"));
        assert!(s.contains("::error file=demo/bad/Dockerfile,line=1::DL001: Container is configured to run as root"));
    }
}
