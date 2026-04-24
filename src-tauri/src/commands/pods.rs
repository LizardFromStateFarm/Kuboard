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



// Pod Actions Commands
#[tauri::command]
pub async fn kuboard_delete_pod(
    pod_name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting pod: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    match pods_api.delete(&pod_name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted pod: {}/{}", namespace, pod_name);
            Ok(format!("Pod {}/{} deleted successfully", namespace, pod_name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            // Treat 404 as successful deletion (already gone)
            warn!("Pod {}/{} not found during delete - treating as already deleted", namespace, pod_name);
            Ok(format!("Pod {}/{} not found (already deleted)", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to delete pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to delete pod: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_restart_pod(
    pod_name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Restarting pod: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    // Delete directly to trigger recreation by controller. If already gone, treat as success.
    match pods_api.delete(&pod_name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully restarted (deleted for recreation) pod: {}/{}", namespace, pod_name);
            Ok(format!("Pod {}/{} restarted (deleted for recreation)", namespace, pod_name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("Pod {}/{} not found during restart - treating as already restarted", namespace, pod_name);
            Ok(format!("Pod {}/{} not found (already restarted)", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to restart pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to restart pod: {}", e))
        }
    }
}

// Delete Commands for All Resource Types
#[tauri::command]
pub async fn kuboard_delete_deployment(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting deployment: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    match deployments_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted deployment: {}/{}", namespace, name);
            Ok(format!("Deployment {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("Deployment {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("Deployment {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete deployment {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete deployment: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_statefulset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting statefulset: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    
    match statefulsets_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted statefulset: {}/{}", namespace, name);
            Ok(format!("StatefulSet {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("StatefulSet {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("StatefulSet {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete statefulset {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete statefulset: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_daemonset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting daemonset: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    
    match daemonsets_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted daemonset: {}/{}", namespace, name);
            Ok(format!("DaemonSet {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("DaemonSet {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("DaemonSet {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete daemonset {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete daemonset: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_replicaset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting replicaset: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    
    match replicasets_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted replicaset: {}/{}", namespace, name);
            Ok(format!("ReplicaSet {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("ReplicaSet {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("ReplicaSet {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete replicaset {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete replicaset: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_service(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting service: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let services_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
    
    match services_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted service: {}/{}", namespace, name);
            Ok(format!("Service {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("Service {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("Service {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete service {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete service: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_delete_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Deleting cronjob: {}/{}", namespace, name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    match cronjobs_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("✅ Successfully deleted cronjob: {}/{}", namespace, name);
            Ok(format!("CronJob {}/{} deleted successfully", namespace, name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            warn!("CronJob {}/{} not found during delete - treating as already deleted", namespace, name);
            Ok(format!("CronJob {}/{} not found (already deleted)", namespace, name))
        }
        Err(e) => {
            error!("Failed to delete cronjob {}/{}: {}", namespace, name, e);
            Err(format!("Failed to delete cronjob: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_get_pod_yaml(
    pod_name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Getting YAML for pod: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    match pods_api.get(&pod_name).await {
        Ok(pod) => {
            // Convert to JSON first, then format as YAML-like structure
            // Note: We'll use JSON for now, YAML can be added later if needed
            match serde_json::to_string_pretty(&pod) {
                Ok(json) => {
                    info!("✅ Successfully retrieved pod data: {}/{}", namespace, pod_name);
                    Ok(json)
                }
                Err(e) => {
                    error!("Failed to serialize pod to JSON: {}", e);
                    Err(format!("Failed to serialize pod: {}", e))
                }
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Pod {}/{} not found", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to get pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to get pod: {}", e))
        }
    }
}

// YAML Get Commands for All Resource Types
#[tauri::command]
pub async fn kuboard_get_deployment_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::namespaced(client.clone(), &namespace);
    
    match deployments_api.get(&name).await {
        Ok(deployment) => {
            match serde_json::to_string_pretty(&deployment) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize deployment: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Deployment {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get deployment: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_statefulset_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    
    match statefulsets_api.get(&name).await {
        Ok(statefulset) => {
            match serde_json::to_string_pretty(&statefulset) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize statefulset: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("StatefulSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get statefulset: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_daemonset_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    
    match daemonsets_api.get(&name).await {
        Ok(daemonset) => {
            match serde_json::to_string_pretty(&daemonset) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize daemonset: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("DaemonSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get daemonset: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_replicaset_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    
    match replicasets_api.get(&name).await {
        Ok(replicaset) => {
            match serde_json::to_string_pretty(&replicaset) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize replicaset: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("ReplicaSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get replicaset: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_service_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let services_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
    
    match services_api.get(&name).await {
        Ok(service) => {
            match serde_json::to_string_pretty(&service) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize service: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Service {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get service: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_get_cronjob_yaml(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    match cronjobs_api.get(&name).await {
        Ok(cronjob) => {
            match serde_json::to_string_pretty(&cronjob) {
                Ok(json) => Ok(json),
                Err(e) => Err(format!("Failed to serialize cronjob: {}", e))
            }
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("CronJob {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get cronjob: {}", e))
    }
}

#[tauri::command]
pub async fn kuboard_update_pod_from_yaml(
    pod_name: String,
    namespace: String,
    yaml_content: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Updating pod from YAML: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    // Parse JSON/YAML content
    let mut updated_pod: Pod = match serde_json::from_str(&yaml_content) {
        Ok(pod) => pod,
        Err(e) => {
            error!("Failed to parse YAML/JSON: {}", e);
            return Err(format!("Invalid YAML/JSON format: {}", e));
        }
    };
    
    // Verify the pod name matches (metadata.name is Option<String>)
    match &updated_pod.metadata.name {
        Some(name) if name != &pod_name => {
            return Err(format!("Pod name mismatch: expected {}, got {}", 
                pod_name, name));
        }
        None => {
            // If name is None, set it to the expected name
            updated_pod.metadata.name = Some(pod_name.clone());
        }
        _ => {} // Name matches or will be set
    }
    
    // Replace the pod
    match pods_api.replace(&pod_name, &Default::default(), &updated_pod).await {
        Ok(_) => {
            info!("✅ Successfully updated pod: {}/{}", namespace, pod_name);
            Ok(format!("Pod {}/{} updated successfully", namespace, pod_name))
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Pod {}/{} not found", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to update pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to update pod: {}", e))
        }
    }
}

// Pod Watch Commands
#[tauri::command]
pub async fn kuboard_start_pod_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting pod watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.pod_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ Pod watch started successfully");
            Ok("Pod watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start pod watch: {}", e);
            Err(format!("Failed to start pod watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_pod_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping pod watch");

    let mut watcher_guard = state.pod_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ Pod watch stopped");
    Ok("Pod watch stopped".to_string())
}

// Deployment Watch Commands
#[tauri::command]
pub async fn kuboard_start_deployment_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting deployment watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.deployment_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ Deployment watch started successfully");
            Ok("Deployment watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start deployment watch: {}", e);
            Err(format!("Failed to start deployment watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_deployment_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping deployment watch");

    let mut watcher_guard = state.deployment_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ Deployment watch stopped");
    Ok("Deployment watch stopped".to_string())
}

// StatefulSet Watch Commands
#[tauri::command]
pub async fn kuboard_start_statefulset_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting statefulset watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.statefulset_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ StatefulSet watch started successfully");
            Ok("StatefulSet watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start statefulset watch: {}", e);
            Err(format!("Failed to start statefulset watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_statefulset_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping statefulset watch");

    let mut watcher_guard = state.statefulset_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ StatefulSet watch stopped");
    Ok("StatefulSet watch stopped".to_string())
}

// DaemonSet Watch Commands
#[tauri::command]
pub async fn kuboard_start_daemonset_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting daemonset watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.daemonset_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ DaemonSet watch started successfully");
            Ok("DaemonSet watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start daemonset watch: {}", e);
            Err(format!("Failed to start daemonset watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_daemonset_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping daemonset watch");

    let mut watcher_guard = state.daemonset_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ DaemonSet watch stopped");
    Ok("DaemonSet watch stopped".to_string())
}

// ReplicaSet Watch Commands
#[tauri::command]
pub async fn kuboard_start_replicaset_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting replicaset watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.replicaset_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ ReplicaSet watch started successfully");
            Ok("ReplicaSet watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start replicaset watch: {}", e);
            Err(format!("Failed to start replicaset watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_replicaset_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping replicaset watch");

    let mut watcher_guard = state.replicaset_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ ReplicaSet watch stopped");
    Ok("ReplicaSet watch stopped".to_string())
}

// Service Watch Commands
#[tauri::command]
pub async fn kuboard_start_service_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting service watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.service_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ Service watch started successfully");
            Ok("Service watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start service watch: {}", e);
            Err(format!("Failed to start service watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_service_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping service watch");

    let mut watcher_guard = state.service_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ Service watch stopped");
    Ok("Service watch stopped".to_string())
}

// CronJob Watch Commands
#[tauri::command]
pub async fn kuboard_start_cronjob_watch(
    app: tauri::AppHandle,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Starting cronjob watch");

    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?
        .clone();
    drop(client_guard);

    let mut watcher_guard = state.cronjob_watcher.write().await;
    
    match watcher_guard.start(client, app).await {
        Ok(_) => {
            info!("✅ CronJob watch started successfully");
            Ok("CronJob watch started".to_string())
        }
        Err(e) => {
            error!("Failed to start cronjob watch: {}", e);
            Err(format!("Failed to start cronjob watch: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_stop_cronjob_watch(
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping cronjob watch");

    let mut watcher_guard = state.cronjob_watcher.write().await;
    watcher_guard.stop();
    
    info!("✅ CronJob watch stopped");
    Ok("CronJob watch stopped".to_string())
}

