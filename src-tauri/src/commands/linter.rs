// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::{Api, api::ListParams, ResourceExt};
use k8s_openapi::api::core::v1::{Pod, Service, Endpoints};
use k8s_openapi::api::apps::v1::Deployment;
use serde::{Serialize, Deserialize};

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct LinterFinding {
    pub resource_kind: String,
    pub resource_name: String,
    pub namespace: String,
    pub severity: String, // "Critical", "Warning", "Info"
    pub message: String,
    pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LinterReport {
    pub findings: Vec<LinterFinding>,
    pub health_score: i32, // 0-100
}

#[tauri::command]
pub async fn kuboard_run_linter(
    namespace: Option<String>,
    state: State<'_, AppState>
) -> Result<LinterReport, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context".to_string())?;

    let mut findings = Vec::new();
    
    // Fetch resources
    let pods_api: Api<Pod> = if let Some(ref ns) = namespace { Api::namespaced(client.clone(), ns) } else { Api::all(client.clone()) };
    let svcs_api: Api<Service> = if let Some(ref ns) = namespace { Api::namespaced(client.clone(), ns) } else { Api::all(client.clone()) };
    let eps_api: Api<Endpoints> = if let Some(ref ns) = namespace { Api::namespaced(client.clone(), ns) } else { Api::all(client.clone()) };
    let deploys_api: Api<Deployment> = if let Some(ref ns) = namespace { Api::namespaced(client.clone(), ns) } else { Api::all(client.clone()) };

    let pods = pods_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    let svcs = svcs_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    let eps = eps_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    let deploys = deploys_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;

    // 1. Pod Checks
    for p in pods.items {
        let ns = p.namespace().unwrap_or_default();
        let name = p.name_any();
        
        if let Some(spec) = &p.spec {
            for container in &spec.containers {
                // Resource Limits
                if container.resources.as_ref().and_then(|r| r.limits.as_ref()).is_none() {
                    findings.push(LinterFinding {
                        resource_kind: "Pod".to_string(),
                        resource_name: name.clone(),
                        namespace: ns.clone(),
                        severity: "Warning".to_string(),
                        message: format!("Container '{}' has no resource limits", container.name),
                        code: "POD-001".to_string(),
                    });
                }
                
                // Probes
                if container.liveness_probe.is_none() && container.readiness_probe.is_none() {
                    findings.push(LinterFinding {
                        resource_kind: "Pod".to_string(),
                        resource_name: name.clone(),
                        namespace: ns.clone(),
                        severity: "Warning".to_string(),
                        message: format!("Container '{}' has no health probes", container.name),
                        code: "POD-002".to_string(),
                    });
                }
            }
        }
        
        // Status checks
        if let Some(status) = &p.status {
            if let Some(phase) = &status.phase {
                if phase == "Failed" || phase == "Unknown" {
                    findings.push(LinterFinding {
                        resource_kind: "Pod".to_string(),
                        resource_name: name.clone(),
                        namespace: ns.clone(),
                        severity: "Critical".to_string(),
                        message: format!("Pod is in phase '{}'", phase),
                        code: "POD-003".to_string(),
                    });
                }
            }
            
            if let Some(container_statuses) = &status.container_statuses {
                for cs in container_statuses {
                    if cs.restart_count > 10 {
                        findings.push(LinterFinding {
                            resource_kind: "Pod".to_string(),
                            resource_name: name.clone(),
                            namespace: ns.clone(),
                            severity: "Critical".to_string(),
                            message: format!("Container '{}' has restarted {} times", cs.name, cs.restart_count),
                            code: "POD-004".to_string(),
                        });
                    }
                }
            }
        }
    }

    // 2. Service Checks
    for s in svcs.items {
        let ns = s.namespace().unwrap_or_default();
        let name = s.name_any();
        
        // Skip Kubernetes internal service
        if name == "kubernetes" && ns == "default" { continue; }
        
        // Check if service has endpoints
        let has_endpoints = eps.items.iter().any(|e| e.name_any() == name && e.namespace().as_ref() == Some(&ns) && e.subsets.as_ref().map_or(false, |s| !s.is_empty()));
        
        if !has_endpoints && s.spec.as_ref().map_or(false, |spec| spec.type_.as_ref().map_or(true, |t| t != "ExternalName")) {
            findings.push(LinterFinding {
                resource_kind: "Service".to_string(),
                resource_name: name.clone(),
                namespace: ns.clone(),
                severity: "Critical".to_string(),
                message: "Service has no active endpoints".to_string(),
                code: "SVC-001".to_string(),
            });
        }
    }

    // 3. Deployment Checks
    for d in deploys.items {
        let ns = d.namespace().unwrap_or_default();
        let name = d.name_any();
        
        if let Some(status) = &d.status {
            let desired = d.spec.as_ref().and_then(|s| s.replicas).unwrap_or(1);
            let available = status.available_replicas.unwrap_or(0);
            
            if available < desired {
                findings.push(LinterFinding {
                    resource_kind: "Deployment".to_string(),
                    resource_name: name.clone(),
                    namespace: ns.clone(),
                    severity: "Critical".to_string(),
                    message: format!("Deployment has {}/{} available replicas", available, desired),
                    code: "DEP-001".to_string(),
                });
            }
        }
    }

    // Calculate health score
    let critical_count = findings.iter().filter(|f| f.severity == "Critical").count();
    let warning_count = findings.iter().filter(|f| f.severity == "Warning").count();
    
    let mut score = 100 - (critical_count as i32 * 10) - (warning_count as i32 * 2);
    if score < 0 { score = 0; }

    Ok(LinterReport { findings, health_score: score })
}
