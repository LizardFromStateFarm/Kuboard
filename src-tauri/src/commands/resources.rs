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



// Resource Commands
#[tauri::command]
pub async fn kuboard_get_nodes(state: State<'_, AppState>) -> Result<Vec<Node>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let nodes_api: Api<Node> = Api::all(client.clone());
    match nodes_api.list(&Default::default()).await {
        Ok(nodes) => Ok(nodes.items),
        Err(e) => Err(format!("Failed to get nodes: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_namespaces(state: State<'_, AppState>) -> Result<Vec<Namespace>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    match namespaces_api.list(&Default::default()).await {
        Ok(namespaces) => Ok(namespaces.items),
        Err(e) => Err(format!("Failed to get namespaces: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_pods(state: State<'_, AppState>) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::all(client.clone());
    match pods_api.list(&Default::default()).await {
        Ok(pods) => Ok(pods.items),
        Err(e) => Err(format!("Failed to get pods: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_deployments(state: State<'_, AppState>) -> Result<Vec<Deployment>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let deployments_api: Api<Deployment> = Api::all(client.clone());
    match deployments_api.list(&Default::default()).await {
        Ok(deployments) => Ok(deployments.items),
        Err(e) => Err(format!("Failed to get deployments: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_services(state: State<'_, AppState>) -> Result<Vec<Service>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let services_api: Api<Service> = Api::all(client.clone());
    match services_api.list(&Default::default()).await {
        Ok(services) => Ok(services.items),
        Err(e) => Err(format!("Failed to get services: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_service(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Service, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let services_api: Api<Service> = Api::namespaced(client.clone(), &namespace);
    match services_api.get(&name).await {
        Ok(service) => Ok(service),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Service {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get service: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_service_endpoints(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Endpoints, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let endpoints_api: Api<Endpoints> = Api::namespaced(client.clone(), &namespace);
    match endpoints_api.get(&name).await {
        Ok(endpoints) => Ok(endpoints),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Endpoints {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get service endpoints: {}", e)),
    }
}



#[tauri::command]
pub async fn kuboard_get_replicasets(state: State<'_, AppState>) -> Result<Vec<ReplicaSet>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::all(client.clone());
    match replicasets_api.list(&Default::default()).await {
        Ok(replicasets) => Ok(replicasets.items),
        Err(e) => Err(format!("Failed to get replicasets: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_replicaset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<ReplicaSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    match replicasets_api.get(&name).await {
        Ok(replicaset) => Ok(replicaset),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("ReplicaSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get replicaset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_scale_replicaset(
    name: String,
    namespace: String,
    replicas: i32,
    state: State<'_, AppState>
) -> Result<ReplicaSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    
    // Get current replicaset
    let mut replicaset = match replicasets_api.get(&name).await {
        Ok(rs) => rs,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("ReplicaSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get replicaset: {}", e)),
    };

    // Update replica count
    if let Some(spec) = replicaset.spec.as_mut() {
        spec.replicas = Some(replicas);
    } else {
        return Err("ReplicaSet spec is missing".to_string());
    }

    // Apply the update
    match replicasets_api.replace(&name, &Default::default(), &replicaset).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to scale replicaset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_replicaset_pods(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the replicaset to find its selector
    let replicasets_api: Api<ReplicaSet> = Api::namespaced(client.clone(), &namespace);
    let replicaset = match replicasets_api.get(&name).await {
        Ok(rs) => rs,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("ReplicaSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get replicaset: {}", e)),
    };

    // Get selector from replicaset
    let selector = match replicaset.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("ReplicaSet has no spec".to_string()),
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

