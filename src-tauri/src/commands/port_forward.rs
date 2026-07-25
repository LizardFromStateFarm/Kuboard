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



// Port Forwarding Commands
// Note: Port forwarding requires WebSocket support for data streaming
// This is a placeholder that will be enhanced with proper WebSocket integration

#[tauri::command]
pub async fn kuboard_port_forward(
    resource_type: String, // "pod" or "service"
    resource_name: String,
    namespace: String,
    local_port: u16,
    remote_port: u16,
    container_name: Option<String>,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    info!("Port forward: {} {}:{}/{} -> localhost:{}", resource_type, namespace, resource_name, remote_port, local_port);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Start port forward session
    let session = start_port_forward_session(
        client,
        &resource_type,
        &resource_name,
        &namespace,
        local_port,
        remote_port,
        container_name.as_deref(),
    ).await.map_err(|e| format!("Failed to start port forward: {}", e))?;

    // Store session in app state
    {
        let mut sessions = state.port_forward_sessions.write().await;
        sessions.insert(session.session_id.clone(), session.clone());
    }

    // TODO: Implement WebSocket streaming and TCP forwarding
    // For now, return session info
    // The actual port forwarding will be implemented with WebSocket
    
    Ok(json!({
        "sessionId": session.session_id,
        "resourceType": session.resource_type,
        "resourceName": session.resource_name,
        "namespace": session.namespace,
        "localPort": session.local_port,
        "remotePort": session.remote_port,
        "url": session.url(),
        "status": "connecting",
        "message": "Port forward session created. Full forwarding support coming soon."
    }))
}

#[tauri::command]
pub async fn kuboard_list_port_forwards(
    state: State<'_, AppState>
) -> Result<Vec<serde_json::Value>, String> {
    info!("Listing active port forwards");
    
    let sessions = state.port_forward_sessions.read().await;
    let forwards: Vec<serde_json::Value> = sessions.values().map(|session| {
        json!({
            "id": session.session_id,
            "podName": if session.resource_type == "pod" { Some(&session.resource_name) } else { None },
            "serviceName": if session.resource_type == "service" { Some(&session.resource_name) } else { None },
            "namespace": session.namespace,
            "localPort": session.local_port,
            "remotePort": session.remote_port,
            "url": session.url(),
            "status": "active"
        })
    }).collect();
    
    Ok(forwards)
}

#[tauri::command]
pub async fn kuboard_stop_port_forward(
    forward_id: String,
    state: State<'_, AppState>
) -> Result<String, String> {
    info!("Stopping port forward: {}", forward_id);
    
    let mut sessions = state.port_forward_sessions.write().await;
    
    if let Some(session) = sessions.remove(&forward_id) {
        info!("Stopped port forward session: {} ({}:{}/{} -> localhost:{})", 
              forward_id, session.namespace, session.resource_name, session.remote_port, session.local_port);
        
        // TODO: Close WebSocket connection and TCP listener
        // For now, just remove from state
        
        Ok(format!("Port forward {} stopped", forward_id))
    } else {
        Err(format!("Port forward {} not found", forward_id))
    }
}
