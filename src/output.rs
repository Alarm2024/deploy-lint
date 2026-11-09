use crate::finding::{Finding, Severity};
use serde::Serialize;
use std::io::{self, Write};

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

pub fn print_json(findings: &[Finding], stdout: &mut dyn Write) -> io::Result<()> {
    let summary = Summary {
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
    };
    let report = JsonReport {
        findings,
        summary,
    };
    writeln!(stdout, "{}", serde_json::to_string_pretty(&report).unwrap())
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
