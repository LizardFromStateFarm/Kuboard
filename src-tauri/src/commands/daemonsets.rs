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



// DaemonSet Commands
#[tauri::command]
pub async fn kuboard_get_daemonsets(state: State<'_, AppState>) -> Result<Vec<DaemonSet>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::all(client.clone());
    match daemonsets_api.list(&Default::default()).await {
        Ok(daemonsets) => Ok(daemonsets.items),
        Err(e) => Err(format!("Failed to get daemonsets: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_daemonset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<DaemonSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    match daemonsets_api.get(&name).await {
        Ok(daemonset) => Ok(daemonset),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("DaemonSet {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get daemonset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_restart_daemonset(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<DaemonSet, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    
    // Get current daemonset
    let mut daemonset = match daemonsets_api.get(&name).await {
        Ok(ds) => ds,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("DaemonSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get daemonset: {}", e)),
    };

    // Add restart annotation to trigger pod recreation
    // The annotation must be in spec.template.metadata.annotations, not metadata.annotations
    let spec = daemonset.spec.as_mut().ok_or_else(|| "DaemonSet spec is missing".to_string())?;
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
    match daemonsets_api.replace(&name, &Default::default(), &daemonset).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to restart daemonset: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_daemonset_pods(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Pod>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Get the daemonset to find its selector
    let daemonsets_api: Api<DaemonSet> = Api::namespaced(client.clone(), &namespace);
    let daemonset = match daemonsets_api.get(&name).await {
        Ok(ds) => ds,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("DaemonSet {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get daemonset: {}", e)),
    };

    // Get selector from daemonset
    let selector = match daemonset.spec.as_ref() {
        Some(spec) => &spec.selector,
        None => return Err("DaemonSet has no spec".to_string()),
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

    // Sort by node name, then by pod name for consistent ordering
    let mut sorted_pods = matching_pods;
    sorted_pods.sort_by(|a, b| {
        let node_a = a.spec.as_ref().and_then(|s| s.node_name.as_deref()).unwrap_or("");
        let node_b = b.spec.as_ref().and_then(|s| s.node_name.as_deref()).unwrap_or("");
        match node_a.cmp(node_b) {
            std::cmp::Ordering::Equal => {
                let name_a = a.metadata.name.as_deref().unwrap_or("");
                let name_b = b.metadata.name.as_deref().unwrap_or("");
                name_a.cmp(name_b)
            }
            other => other,
        }
    });

    Ok(sorted_pods)
}

