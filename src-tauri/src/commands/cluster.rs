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



// Cluster Overview Commands
#[tauri::command]
pub async fn kuboard_get_cluster_overview(state: State<'_, AppState>) -> Result<ClusterOverview, String> {
    info!("Getting cluster overview");
    
    // Check if we have a client, if not try to set the current context automatically
    let client_guard = state.current_client.read().await;
    let client = if client_guard.is_some() {
        client_guard.as_ref().unwrap().clone()
    } else {
        drop(client_guard); // Release the read lock
        
        // Try to automatically set the current context from kubeconfig
        let kubeconfig_guard = state.kubeconfig.read().await;
        if let Some(kubeconfig) = kubeconfig_guard.as_ref() {
            if let Some(current_context) = &kubeconfig.current_context {
                match kuboard_create_client_from_context(kubeconfig, current_context).await {
                    Ok(client) => {
                        *state.current_client.write().await = Some(client.clone());
                        *state.current_context.write().await = Some(current_context.clone());
                        info!("Automatically set context to: {}", current_context);
                        client
                    }
                    Err(e) => {
                        return Err(format!("Failed to automatically set context: {}", e));
                    }
                }
            } else {
                return Err("No current context found in kubeconfig. Please set a context first.".to_string());
            }
        } else {
            return Err("No kubeconfig loaded. Please call list_contexts first.".to_string());
        }
    };

    let context_name = state.current_context.read().await
        .clone()
        .unwrap_or_else(|| "unknown".to_string());

    // Get cluster info
    let cluster_info = ClusterInfo {
        name: context_name,
        server: "unknown".to_string(), // Client doesn't expose apiserver_url in this version
        version: None,
    };

    // Count nodes
    let nodes_api: Api<Node> = Api::all(client.clone());
    let node_count = match nodes_api.list(&Default::default()).await {
        Ok(nodes) => nodes.items.len(),
        Err(e) => {
            warn!("Failed to get nodes: {}", e);
            0
        }
    };

    // Count namespaces
    let namespaces_api: Api<Namespace> = Api::all(client.clone());
    let namespace_count = match namespaces_api.list(&Default::default()).await {
        Ok(namespaces) => namespaces.items.len(),
        Err(e) => {
            warn!("Failed to get namespaces: {}", e);
            0
        }
    };

    // Count pods
    let pods_api: Api<Pod> = Api::all(client.clone());
    let pod_count = match pods_api.list(&Default::default()).await {
        Ok(pods) => pods.items.len(),
        Err(e) => {
            warn!("Failed to get pods: {}", e);
            0
        }
    };

    // Count deployments
    let deployments_api: Api<Deployment> = Api::all(client.clone());
    let deployment_count = match deployments_api.list(&Default::default()).await {
        Ok(deployments) => deployments.items.len(),
        Err(e) => {
            warn!("Failed to get deployments: {}", e);
            0
        }
    };

    // Try to get Kubernetes version
    let kubernetes_version = match client.apiserver_version().await {
        Ok(version) => Some(format!("{}.{}", version.major, version.minor)),
        Err(e) => {
            warn!("Failed to get Kubernetes version: {}", e);
            None
        }
    };

    // Calculate cluster metrics
    let cluster_metrics = match kuboard_calculate_cluster_metrics(&client).await {
        Ok(metrics) => Some(metrics),
        Err(e) => {
            warn!("Failed to calculate cluster metrics: {}", e);
            None
        }
    };

    Ok(ClusterOverview {
        cluster_info,
        node_count,
        namespace_count,
        pod_count,
        deployment_count,
        kubernetes_version,
        cluster_metrics,
    })
}

