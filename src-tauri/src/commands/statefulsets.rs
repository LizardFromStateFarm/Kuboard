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



// StatefulSet Commands
#[tauri::command]
pub async fn kuboard_get_statefulsets(state: State<'_, AppState>) -> Result<Vec<StatefulSet>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::all(client.clone());
    match statefulsets_api.list(&Default::default()).await {
        Ok(statefulsets) => Ok(statefulsets.items),
        Err(e) => Err(format!("Failed to get statefulsets: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_statefulset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<StatefulSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    match statefulsets_api.get(&name).await {
        Ok(statefulset) => Ok(statefulset),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("StatefulSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get statefulset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_scale_statefulset(
    name: String,
    namespace: String,
    replicas: i32,
    state: State<'_, AppState>
) -> Result<StatefulSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    
    // Get current statefulset
    let mut statefulset = match statefulsets_api.get(&name).await {
        Ok(ss) => ss,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("StatefulSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get statefulset: {}", e)),
    };

    // Update replica count
    if let Some(spec) = statefulset.spec.as_mut() {
        spec.replicas = Some(replicas);
    } else {
        return Err("StatefulSet spec is missing".to_string());
    }

    // Apply the update
    match statefulsets_api.replace(&name, &Default::default(), &statefulset).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to scale statefulset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_restart_statefulset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<StatefulSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    
    // Get current statefulset
    let mut statefulset = match statefulsets_api.get(&name).await {
        Ok(ss) => ss,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("StatefulSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get statefulset: {}", e)),
    };

    // Add restart annotation to trigger pod recreation
    // The annotation must be in spec.template.metadata.annotations, not metadata.annotations
    let spec = statefulset.spec.as_mut().ok_or_else(|| "StatefulSet spec is missing".to_string())?;
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
    match statefulsets_api.replace(&name, &Default::default(), &statefulset).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to restart statefulset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_statefulset_pods(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the statefulset to find its selector
    let statefulsets_api: Api<StatefulSet> = Api::namespaced(client.clone(), &namespace);
    let statefulset = match statefulsets_api.get(&name).await {
        Ok(ss) => ss,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("StatefulSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get statefulset: {}", e)),
    };

    // Get selector from statefulset
    let selector = match statefulset.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("StatefulSet has no spec".to_string()),
    };

    // List pods with matching labels
    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    let pods = match pods_api.list(&Default::default()).await {
        Ok(pod_list) => pod_list.items,
        Err(e) => return Err(format!("Failed to list pods: {}", e)),
    };

    // Filter pods by selector and sort by ordinal (StatefulSet pods are named with ordinal suffix)
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

    // Sort by pod name (which contains ordinal) for StatefulSet ordering
    let mut sorted_pods = matching_pods;
    sorted_pods.sort_by(|a, b| {
        let name_a = a.metadata.name.as_deref().unwrap_or("");
        let name_b = b.metadata.name.as_deref().unwrap_or("");
        name_a.cmp(name_b)
    });

    Ok(sorted_pods)
}

