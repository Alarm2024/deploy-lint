use crate::finding::{Finding, Severity};
use std::path::PathBuf;

const DL006: &str = "DL006";
const DL008: &str = "DL008";

const DL008_FIX: &str =
    "Set User=<svc>, NoNewPrivileges=yes, ProtectSystem=strict, PrivateTmp=yes";

pub fn lint(file: &PathBuf, content: &str) -> Vec<Finding> {
    let mut in_service = false;
    let mut service_section_line = 1usize;
    let mut restart_always = false;
    let mut restart_always_line = 1usize;
    let mut has_restart_sec = false;
    let mut has_start_limit_burst = false;

    let mut user_value: Option<String> = None;
    let mut no_new_privileges = false;
    let mut protect_system_sandbox = false;
    let mut dynamic_user = false;

    for (idx, raw) in content.lines().enumerate() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_service = line.eq_ignore_ascii_case("[Service]");
            if in_service {
                service_section_line = idx + 1;
            }
            continue;
        }
        if !in_service {
            continue;
        }

        if let Some((key, value)) = parse_directive(raw) {
            match key.as_str() {
                "restart" => {
                    if value.eq_ignore_ascii_case("always") {
                        restart_always = true;
                        restart_always_line = idx + 1;
                    }
                }
                "restartsec" => has_restart_sec = true,
                "startlimitburst" => has_start_limit_burst = true,
                "user" => user_value = Some(value),
                "nonewprivileges" => {
                    if value.eq_ignore_ascii_case("yes") || value.eq_ignore_ascii_case("true") {
                        no_new_privileges = true;
                    }
                }
                "protectsystem" => {
                    if is_protect_system_sandbox(&value) {
                        protect_system_sandbox = true;
                    }
                }
                "dynamicuser" => {
                    if value.eq_ignore_ascii_case("yes") || value.eq_ignore_ascii_case("true") {
                        dynamic_user = true;
                    }
                }
                _ => {}
            }
        }
    }

    let mut findings = Vec::new();

    if restart_always && !has_restart_sec && !has_start_limit_burst {
        findings.push(Finding::new(
            DL006,
            Severity::Medium,
            file.clone(),
            restart_always_line,
            "Restart=always with no RestartSec or StartLimitBurst backoff",
            "Add RestartSec and/or StartLimitBurst (and StartLimitIntervalSec) to avoid tight restart loops",
        ));
    }

    let runs_as_root = user_value
        .as_ref()
        .map(|u| u.eq_ignore_ascii_case("root"))
        .unwrap_or(true);

    let has_sandbox = no_new_privileges || protect_system_sandbox || dynamic_user;

    if runs_as_root && !has_sandbox {
        findings.push(Finding::new(
            DL008,
            Severity::High,
            file.clone(),
            service_section_line,
            format!(
                "{}:[Service] runs as root (or no User=) without sandboxing",
                file.display()
            ),
            DL008_FIX,
        ));
    } else if !runs_as_root && !no_new_privileges {
        if let Some(user) = &user_value {
            findings.push(Finding::new(
                DL008,
                Severity::Medium,
                file.clone(),
                service_section_line,
                format!(
                    "{}:[Service] User={user} is set but NoNewPrivileges is missing",
                    file.display()
                ),
                DL008_FIX,
            ));
        }
    }

    findings
}

fn parse_directive(raw: &str) -> Option<(String, String)> {
    let line = raw.split('#').next()?.trim();
    if line.is_empty() {
        return None;
    }
    let mut split = line.splitn(2, '=');
    let key = split.next()?.trim();
    let value = split.next()?.trim();
    if key.is_empty() {
        return None;
    }
    Some((key.to_ascii_lowercase(), value.to_string()))
}

fn is_protect_system_sandbox(value: &str) -> bool {
    value.eq_ignore_ascii_case("strict")
        || value.eq_ignore_ascii_case("full")
        || value.eq_ignore_ascii_case("yes")
        || value.eq_ignore_ascii_case("true")
}
