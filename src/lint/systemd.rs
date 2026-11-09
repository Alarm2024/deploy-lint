use crate::finding::{Finding, Severity};
use std::path::PathBuf;

const DL006: &str = "DL006";

pub fn lint(file: &PathBuf, content: &str) -> Vec<Finding> {
    let mut in_service = false;
    let mut restart_always = false;
    let mut restart_always_line = 1usize;
    let mut has_restart_sec = false;
    let mut has_start_limit_burst = false;

    for (idx, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_service = line.eq_ignore_ascii_case("[Service]");
            continue;
        }
        if !in_service {
            continue;
        }

        if line.to_ascii_lowercase().starts_with("restart=") {
            let value = line.split('=').nth(1).unwrap_or("").trim();
            if value.eq_ignore_ascii_case("always") {
                restart_always = true;
                restart_always_line = idx + 1;
            }
        }
        if line.to_ascii_lowercase().starts_with("restartsec=") {
            has_restart_sec = true;
        }
        if line.to_ascii_lowercase().starts_with("startlimitburst=") {
            has_start_limit_burst = true;
        }
    }

    if restart_always && !has_restart_sec && !has_start_limit_burst {
        return vec![Finding::new(
            DL006,
            Severity::Medium,
            file.clone(),
            restart_always_line,
            "Restart=always with no RestartSec or StartLimitBurst backoff",
            "Add RestartSec and/or StartLimitBurst (and StartLimitIntervalSec) to avoid tight restart loops",
        )];
    }

    Vec::new()
}
