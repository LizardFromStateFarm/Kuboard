// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kuboard Tauri Commands Module
// This module contains all Tauri command functions with kuboard_ prefix

use tauri::State;
use kube::Api;
use kube::api::DeleteParams;
use k8s_openapi::api::{
    apps::v1::{Deployment, ReplicaSet, StatefulSet, DaemonSet},
    batch::v1::{CronJob, Job},
    core::v1::{Node, Namespace, Pod, Service, ConfigMap, Secret, Endpoints},
};
use tracing::{error, info, warn};

use crate::app_state::AppState;
use crate::types::*;
use crate::kubernetes::{
    kuboard_load_kubeconfig,
    kuboard_create_client_from_context,
    kuboard_calculate_cluster_metrics,
};
use crate::metrics::{
    kuboard_fetch_node_metrics_real,
    kuboard_fetch_node_metrics_history,
    kuboard_fetch_pod_metrics_real,
    kuboard_fetch_pod_metrics_history,
    kuboard_check_metrics_server_availability,
};
use crate::kubernetes::{kuboard_fetch_pod_events, kuboard_fetch_pod_logs};
use crate::kubernetes::exec::start_exec_session;
use crate::kubernetes::port_forward::start_port_forward_session;
use serde_json::json;



// Deployment Commands
#[tauri::command]
pub async fn kuboard_get_deployment(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Deployment, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    match deployments_api.get(&name).await {
        Ok(deployment) => Ok(deployment),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Deployment {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get deployment: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_scale_deployment(
    name: String,
    namespace: String,
    replicas: i32,
    state: State<'_, AppState>
) -> Result<Deployment, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    // Get current deployment
    let mut deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // Update replica count
    if let Some(spec) = deployment.spec.as_mut() {
        spec.replicas = Some(replicas);
    } else {
        return Err("Deployment spec is missing".to_string());
    }

    // Apply the update
    match deployments_api.replace(&name, &Default::default(), &deployment).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to scale deployment: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_rollback_deployment(
    name: String,
    namespace: String,
    _revision: Option<i64>,
    state: State<'_, AppState>
) -> Result<Deployment, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    // Get current deployment (verify it exists)
    let _deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // For rollback, we need to use the rollout subresource
    // This is a simplified version - in production, you'd use kubectl rollout undo
    // For now, we'll return an error indicating this needs kubectl
    Err("Rollback requires kubectl rollout undo command. This feature will be enhanced in Phase 2.".to_string())
}

#[tauri::command]
pub async fn kuboard_restart_deployment(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Deployment, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    // Get current deployment
    let mut deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // Add restart annotation to trigger pod recreation
    // The annotation must be in spec.template.metadata.annotations, not metadata.annotations
    let spec = deployment.spec.as_mut().ok_or_else(|| "Deployment spec is missing".to_string())?;
    let metadata = spec.template.metadata.get_or_insert_with(Default::default);
    let annotations = metadata.annotations.get_or_insert_with(Default::default);
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    annotations.insert(
        "kubectl.kubernetes.io/restartedAt".to_string(),
        timestamp.to_string(),
    );

    // Apply the update
    match deployments_api.replace(&name, &Default::default(), &deployment).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to restart deployment: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_deployment_replicasets(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<ReplicaSet>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the deployment to find its selector
    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // Get selector from deployment (currently unused, filtering by owner reference instead)
    let _selector = match deployment.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("Deployment has no spec".to_string()),
    };

    // List all replicasets in namespace
    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    let replicasets = match replicasets_api.list(&Default::default()).await {
        Ok(rs_list) => rs_list.items,
        Err(e) => return Err(format!("Failed to list replicasets: {}", e)),
    };

    // Filter replicasets by owner reference (owned by this deployment)
    let matching_replicasets: Vec<ReplicaSet> = replicasets
        .into_iter()
        .filter(|rs| {
            if let Some(owner_refs) = rs.metadata.owner_references.as_ref() {
                owner_refs.iter().any(|owner| {
                    owner.kind == "Deployment" && owner.name == name
                })
            } else {
                false
            }
        })
        .collect();

    Ok(matching_replicasets)
}

#[tauri::command]
pub async fn kuboard_get_deployment_pods(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the deployment to find its selector
    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    let deployment = match deployments_api.get(&name).await {
        Ok(dep) => dep,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("Deployment {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get deployment: {}", e)),
    };

    // Get selector from deployment
    let selector = match deployment.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("Deployment has no spec".to_string()),
    };

    // List pods with matching labels
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let pods = match pods_api.list(&Default::default()).await {
        Ok(pod_list) => pod_list.items,
        Err(e) => return Err(format!("Failed to list pods: {}", e)),
    };

    // Filter pods by selector
    let matching_pods: Vec<Pod> = pods
        .into_iter()
        .filter(|pod| {
            if let Some(pod_labels) = pod.metadata.labels.as_ref() {
                if let Some(match_labels) = selector.match_labels.as_ref() {
                    match_labels.iter().all(|(key, value)| {
                        pod_labels.get(key).map_or(false, |v| v == value)
                    })
                } else {
                    false
                }
            } else {
                false
            }
        })
        .collect();

    Ok(matching_pods)
}

