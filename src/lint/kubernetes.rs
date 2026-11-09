use crate::finding::{Finding, Severity};
use crate::lint::dockerfile::is_secret_name;
use crate::scan::line_for_pattern;
use serde::Deserialize;
use serde_yaml::{Mapping, Value};
use std::path::PathBuf;

const DL001: &str = "DL001";
const DL002: &str = "DL002";
const DL003: &str = "DL003";
const DL004: &str = "DL004";
const DL005: &str = "DL005";

#[derive(Clone)]
struct ContainerCtx {
    name: String,
    image: Option<String>,
    has_limits: bool,
    has_liveness: bool,
    has_readiness: bool,
    run_as_root: bool,
    env_secret_names: Vec<String>,
}

pub fn lint(file: &PathBuf, content: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let docs = match parse_documents(content) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut containers = Vec::new();
    for doc in &docs {
        collect_from_value(doc, &mut containers, None);
    }

    for ctx in &containers {
        if let Some(image) = &ctx.image {
            if image_uses_latest_or_untagged(image) {
                let line = line_for_pattern(content, &[&format!("image: {image}"), "image:"]);
                findings.push(Finding::new(
                    DL002,
                    Severity::High,
                    file.clone(),
                    line,
                    format!("Container '{}' image uses :latest or no tag", ctx.name),
                    "Pin container images to a specific tag or digest",
                ));
            }
        }

        if ctx.run_as_root {
            let patterns = if ctx.name.is_empty() {
                vec!["containers:".to_string(), "securityContext:".to_string()]
            } else {
                vec![format!("name: {}", ctx.name)]
            };
            let pat_refs: Vec<&str> = patterns.iter().map(String::as_str).collect();
            let line = line_for_pattern(content, &pat_refs);
            findings.push(Finding::new(
                DL001,
                Severity::High,
                file.clone(),
                line,
                format!(
                    "Container '{}' may run as root (runAsUser 0 or missing runAsNonRoot)",
                    ctx.name
                ),
                "Set securityContext.runAsNonRoot: true and an explicit non-zero runAsUser",
            ));
        }

        if !ctx.has_limits {
            let line = line_for_pattern(
                content,
                &[&format!("name: {}", ctx.name), "resources:"],
            );
            findings.push(Finding::new(
                DL004,
                Severity::Medium,
                file.clone(),
                line,
                format!("Container '{}' has no resource limits", ctx.name),
                "Set resources.limits for cpu and memory",
            ));
        }

        if !ctx.has_liveness && !ctx.has_readiness {
            let line = line_for_pattern(content, &[&format!("name: {}", ctx.name)]);
            findings.push(Finding::new(
                DL005,
                Severity::Medium,
                file.clone(),
                line,
                format!("Container '{}' has no liveness or readiness probe", ctx.name),
                "Add livenessProbe and/or readinessProbe",
            ));
        }

        for env_name in &ctx.env_secret_names {
            let line = line_for_pattern(
                content,
                &[
                    &format!("name: {env_name}"),
                    &format!("- name: {env_name}"),
                ],
            );
            findings.push(Finding::new(
                DL003,
                Severity::High,
                file.clone(),
                line,
                format!("Secret-like env var {env_name} has a literal value (masked)"),
                "Use secretKeyRef, envFrom secretRef, or external secret injection",
            ));
        }
    }

    dedupe_findings(&mut findings);
    findings
}

fn dedupe_findings(findings: &mut Vec<Finding>) {
    findings.sort_by(|a, b| {
        (&a.rule_id, &a.file, a.line, &a.reason).cmp(&(&b.rule_id, &b.file, b.line, &b.reason))
    });
    findings.dedup_by(|a, b| {
        a.rule_id == b.rule_id && a.file == b.file && a.line == b.line && a.reason == b.reason
    });
}

fn parse_documents(content: &str) -> Result<Vec<Value>, serde_yaml::Error> {
    let mut docs = Vec::new();
    for doc in serde_yaml::Deserializer::from_str(content) {
        docs.push(Value::deserialize(doc)?);
    }
    Ok(docs)
}

fn map_get<'a>(map: &'a Mapping, key: &str) -> Option<&'a Value> {
    map.get(&Value::String(key.into()))
}

fn collect_from_value(value: &Value, out: &mut Vec<ContainerCtx>, pod_security: Option<&Value>) {
    match value {
        Value::Sequence(seq) => {
            for v in seq {
                collect_from_value(v, out, pod_security);
            }
        }
        Value::Mapping(map) => {
            let pod_sc = map_get(map, "securityContext").or_else(|| {
                map_get(map, "spec")
                    .and_then(|spec| spec.as_mapping())
                    .and_then(|spec_map| map_get(spec_map, "securityContext"))
            });

            let effective_pod_sc = pod_sc.or(pod_security);

            if let Some(containers) = find_containers_array(map) {
                for c in containers {
                    if let Some(ctx) = container_from_value(c, effective_pod_sc) {
                        out.push(ctx);
                    }
                }
            }

            if let Some(template) = map_get(map, "template") {
                collect_from_value(template, out, effective_pod_sc);
            }

            for (k, v) in map {
                if k.as_str() == Some("containers") {
                    continue;
                }
                collect_from_value(v, out, effective_pod_sc);
            }
        }
        _ => {}
    }
}

fn find_containers_array(map: &Mapping) -> Option<&serde_yaml::Sequence> {
    if let Some(c) = map_get(map, "containers") {
        return c.as_sequence();
    }
    if let Some(spec) = map_get(map, "spec") {
        if let Some(spec_map) = spec.as_mapping() {
            if let Some(c) = map_get(spec_map, "containers") {
                return c.as_sequence();
            }
        }
    }
    None
}

fn container_from_value(container: &Value, pod_security: Option<&Value>) -> Option<ContainerCtx> {
    let cmap = container.as_mapping()?;
    let name = map_get(cmap, "name")
        .and_then(|v| v.as_str())
        .unwrap_or("container")
        .to_string();

    let image = map_get(cmap, "image")
        .and_then(|v| v.as_str())
        .map(String::from);

    let has_limits = map_get(cmap, "resources")
        .and_then(|r| map_get(r.as_mapping()?, "limits"))
        .map(|limits| {
            limits
                .as_mapping()
                .map(|m| !m.is_empty())
                .unwrap_or(false)
        })
        .unwrap_or(false);

    let has_liveness = map_get(cmap, "livenessProbe").is_some();
    let has_readiness = map_get(cmap, "readinessProbe").is_some();

    let container_sc = map_get(cmap, "securityContext");
    let run_as_root = container_runs_as_root(container_sc, pod_security);

    let mut env_secret_names = Vec::new();
    if let Some(env) = map_get(cmap, "env").and_then(|e| e.as_sequence()) {
        for entry in env {
            if let Some(em) = entry.as_mapping() {
                let ename = entry_get_str(em, "name").unwrap_or_default();
                let has_literal = entry_get_str(em, "value")
                    .map(|v| !v.is_empty())
                    .unwrap_or(false)
                    || em.contains_key(&Value::String("value".into()));
                let value_from = em.contains_key(&Value::String("valueFrom".into()));
                if is_secret_name(&ename) && has_literal && !value_from {
                    env_secret_names.push(ename);
                }
            }
        }
    }

    Some(ContainerCtx {
        name,
        image,
        has_limits,
        has_liveness,
        has_readiness,
        run_as_root,
        env_secret_names,
    })
}

fn entry_get_str(map: &Mapping, key: &str) -> Option<String> {
    map_get(map, key)
        .and_then(|v| v.as_str())
        .map(String::from)
}

fn container_runs_as_root(container_sc: Option<&Value>, pod_sc: Option<&Value>) -> bool {
    if let Some(sc) = container_sc.and_then(|v| v.as_mapping()) {
        if map_get(sc, "runAsNonRoot")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return false;
        }
        if let Some(uid) = map_get(sc, "runAsUser").and_then(|v| v.as_i64()) {
            return uid == 0;
        }
    }

    if let Some(sc) = pod_sc.and_then(|v| v.as_mapping()) {
        if map_get(sc, "runAsNonRoot")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            return false;
        }
        if let Some(uid) = map_get(sc, "runAsUser").and_then(|v| v.as_i64()) {
            return uid == 0;
        }
    }

    true
}

fn image_uses_latest_or_untagged(image: &str) -> bool {
    let image = image.split('@').next().unwrap_or(image);
    if !image.contains(':') {
        return true;
    }
    image.ends_with(":latest")
}
