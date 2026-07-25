// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::{Api, ResourceExt, api::{Patch, PatchParams, PostParams}};
use serde_json::Value;
use serde_yaml;
use tracing::{error, info};

use crate::app_state::AppState;

#[tauri::command]
pub async fn kuboard_get_resource_yaml(
    kind: String,
    name: String,
    namespace: Option<String>,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // We use dynamic API to fetch any resource type
    // This requires knowing the GVK (Group Version Kind)
    // For simplicity in this initial version, we'll map common kinds
    
    let (_resource, api_version) = match kind.to_lowercase().as_str() {
        "pod" => ("pods", "v1"),
        "deployment" => ("deployments", "apps/v1"),
        "service" => ("services", "v1"),
        "namespace" => ("namespaces", "v1"),
        "node" => ("nodes", "v1"),
        "configmap" => ("configmaps", "v1"),
        "secret" => ("secrets", "v1"),
        "ingress" => ("ingresses", "networking.k8s.io/v1"),
        "networkpolicy" => ("networkpolicies", "networking.k8s.io/v1"),
        "persistentvolume" => ("persistentvolumes", "v1"),
        "persistentvolumeclaim" => ("persistentvolumeclaims", "v1"),
        "storageclass" => ("storageclasses", "storage.k8s.io/v1"),
        "statefulset" => ("statefulsets", "apps/v1"),
        "daemonset" => ("daemonsets", "apps/v1"),
        "replicaset" => ("replicasets", "apps/v1"),
        "job" => ("jobs", "batch/v1"),
        "cronjob" => ("cronjobs", "batch/v1"),
        "role" => ("roles", "rbac.authorization.k8s.io/v1"),
        "clusterrole" => ("clusterroles", "rbac.authorization.k8s.io/v1"),
        "rolebinding" => ("rolebindings", "rbac.authorization.k8s.io/v1"),
        "clusterrolebinding" => ("clusterrolebindings", "rbac.authorization.k8s.io/v1"),
        "serviceaccount" => ("serviceaccounts", "v1"),
        _ => return Err(format!("Unsupported resource kind for YAML export: {}", kind)),
    };

    let gvk = kube::api::GroupVersionKind::gvk(
        api_version.split('/').next().unwrap_or(""),
        api_version.split('/').last().unwrap_or(api_version),
        &kind
    );

    // Dynamic API call
    let ar = kube::discovery::ApiResource::from_gvk(&gvk);
    let api: Api<kube::core::DynamicObject> = if let Some(ns) = namespace {
        if ns.is_empty() || ns == "all" {
             Api::all_with(client.clone(), &ar)
        } else {
             Api::namespaced_with(client.clone(), &ns, &ar)
        }
    } else {
        Api::all_with(client.clone(), &ar)
    };

    match api.get(&name).await {
        Ok(obj) => {
            // Convert to YAML
            // We strip some internal fields like managedFields for better readability
            let mut val = serde_json::to_value(obj).map_err(|e| e.to_string())?;
            if let Some(metadata) = val.get_mut("metadata") {
                if let Some(m) = metadata.as_object_mut() {
                    m.remove("managedFields");
                }
            }
            
            serde_yaml::to_string(&val).map_err(|e| e.to_string())
        },
        Err(e) => Err(format!("Failed to get resource {}: {}", name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_apply_resource_yaml(
    yaml_content: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Parse YAML
    let val: Value = serde_yaml::from_str(&yaml_content).map_err(|e| format!("Invalid YAML: {}", e))?;
    
    let kind = val.get("kind").and_then(|v| v.as_str()).ok_or("Missing 'kind' in YAML")?;
    let api_version = val.get("apiVersion").and_then(|v| v.as_str()).ok_or("Missing 'apiVersion' in YAML")?;
    let metadata = val.get("metadata").ok_or("Missing 'metadata' in YAML")?;
    let name = metadata.get("name").and_then(|v| v.as_str()).ok_or("Missing 'metadata.name' in YAML")?;
    let namespace = metadata.get("namespace").and_then(|v| v.as_str());

    let gvk = kube::api::GroupVersionKind::gvk(
        api_version.split('/').next().unwrap_or(""),
        api_version.split('/').last().unwrap_or(api_version),
        kind
    );

    let ar = kube::discovery::ApiResource::from_gvk(&gvk);
    let api: Api<kube::core::DynamicObject> = if let Some(ns) = namespace {
        Api::namespaced_with(client.clone(), ns, &ar)
    } else {
        Api::all_with(client.clone(), &ar)
    };

    // We use server-side apply (Patch with Apply params)
    let ssaparams = PatchParams::apply("kuboard-editor").force();
    let patch = Patch::Apply(&val);

    match api.patch(name, &ssaparams, &patch).await {
        Ok(_) => {
            info!("Applied YAML for {}/{}", kind, name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to apply YAML: {}", e)),
    }
}
