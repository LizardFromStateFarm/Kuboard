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



// Context Management Commands
#[tauri::command]
pub async fn kuboard_list_contexts(state: State<'_, AppState>) -> Result<ContextListResponse, String> {
    info!("Listing Kubernetes contexts");
    
    let kubeconfig = match kuboard_load_kubeconfig().await {
        Ok(config) => {
            *state.kubeconfig.write().await = Some(config.clone());
            config
        }
        Err(e) => {
            error!("Failed to load kubeconfig: {}", e);
            return Err(format!("Failed to load kubeconfig: {}", e));
        }
    };

    let current_context = kubeconfig.current_context.clone();
    let mut contexts = Vec::new();

    // Simple iteration over contexts
    for context_entry in &kubeconfig.contexts {
        let context_name = &context_entry.name;
        
        if let Some(context) = &context_entry.context {
            contexts.push(KubeContext {
                name: context_name.clone(),
                cluster: context.cluster.clone(),
                user: context.user.clone().unwrap_or_default(),
                namespace: context.namespace.clone(),
                is_current: current_context.as_ref() == Some(context_name),
            });
        }
    }

    // Automatically set the current context if one exists and no context is currently set
    if let Some(_current_context_name) = &current_context {
        let _current_state = state.current_context.read().await;
        // Don't auto-select context - let user choose
        // if current_state.is_none() {
        //     drop(current_state); // Release the read lock
        //     
        //     // Set the current context automatically
        //     match kuboard_create_client_from_context(&kubeconfig, current_context_name).await {
        //         Ok(client) => {
        //             *state.current_client.write().await = Some(client);
        //             *state.current_context.write().await = Some(current_context_name.clone());
        //             info!("Automatically set current context to: {}", current_context_name);
        //         }
        //         Err(e) => {
        //             warn!("Failed to automatically set current context '{}': {}", current_context_name, e);
        //         }
        //     }
        // }
    }

    Ok(ContextListResponse {
        contexts,
        current_context,
    })
}

#[tauri::command]
pub async fn kuboard_set_context(context_name: String, state: State<'_, AppState>) -> Result<String, String> {
    info!("Setting context to: {}", context_name);
    
    let kubeconfig = state.kubeconfig.read().await;
    let kubeconfig = kubeconfig
        .as_ref()
        .ok_or_else(|| "Kubeconfig not loaded. Call list_contexts first.".to_string())?;

    // Verify the context exists
    let context_exists = kubeconfig.contexts
        .iter()
        .any(|c| c.name == context_name);

    if !context_exists {
        return Err(format!("Context '{}' not found", context_name));
    }

    // Create client for the new context
    match kuboard_create_client_from_context(kubeconfig, &context_name).await {
        Ok(client) => {
            *state.current_client.write().await = Some(client);
            *state.current_context.write().await = Some(context_name.clone());
            Ok(format!("Context switched to: {}", context_name))
        }
        Err(e) => {
            error!("Failed to create client for context '{}': {}", context_name, e);
            Err(format!("Failed to switch context: {}", e))
        }
    }
}

#[tauri::command]
pub async fn kuboard_get_current_context(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let current_context = state.current_context.read().await.clone();
    Ok(current_context)
}

