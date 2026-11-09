use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub file: PathBuf,
    pub line: usize,
    pub reason: String,
    pub fix_hint: String,
}

impl Finding {
    pub fn new(
        rule_id: impl Into<String>,
        severity: Severity,
        file: PathBuf,
        line: usize,
        reason: impl Into<String>,
        fix_hint: impl Into<String>,
    ) -> Self {
        Self {
            rule_id: rule_id.into(),
            severity,
            file,
            line,
            reason: reason.into(),
            fix_hint: fix_hint.into(),
        }
    }
}

pub fn has_high(findings: &[Finding]) -> bool {
    findings.iter().any(|f| f.severity == Severity::High)
}
