// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::Api;
use kube::api::{ListParams, DeleteParams};
use k8s_openapi::api::networking::v1::{Ingress, IngressClass, NetworkPolicy};
use k8s_openapi::api::core::v1::Service;
use tracing::{error, info};

use crate::app_state::AppState;

// Ingress Commands
#[tauri::command]
pub async fn kuboard_list_ingresses(
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Ingress>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<Ingress> = if namespace.is_empty() || namespace == "all" {
        Api::all(client.clone())
    } else {
        Api::namespaced(client.clone(), &namespace)
    };

    match api.list(&ListParams::default()).await {
        Ok(list) => Ok(list.items),
        Err(e) => Err(format!("Failed to list Ingresses: {}", e)),
    }
}

// IngressClass Commands
#[tauri::command]
pub async fn kuboard_list_ingress_classes(
    state: State<'_, AppState>
) -> Result<Vec<IngressClass>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<IngressClass> = Api::all(client.clone());
    match api.list(&ListParams::default()).await {
        Ok(list) => Ok(list.items),
        Err(e) => Err(format!("Failed to list IngressClasses: {}", e)),
    }
}

// NetworkPolicy Commands
#[tauri::command]
pub async fn kuboard_list_network_policies(
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<NetworkPolicy>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<NetworkPolicy> = if namespace.is_empty() || namespace == "all" {
        Api::all(client.clone())
    } else {
        Api::namespaced(client.clone(), &namespace)
    };

    match api.list(&ListParams::default()).await {
        Ok(list) => Ok(list.items),
        Err(e) => Err(format!("Failed to list NetworkPolicies: {}", e)),
    }
}

// Delete Commands
#[tauri::command]
pub async fn kuboard_delete_ingress(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<Ingress> = Api::namespaced(client.clone(), &namespace);
    match api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted Ingress: {}/{}", namespace, name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete Ingress {}/{}: {}", namespace, name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_ingress_class(
    name: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<IngressClass> = Api::all(client.clone());
    match api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted IngressClass: {}", name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete IngressClass {}: {}", name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_network_policy(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<NetworkPolicy> = Api::namespaced(client.clone(), &namespace);
    match api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted NetworkPolicy: {}/{}", namespace, name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete NetworkPolicy {}/{}: {}", namespace, name, e)),
    }
}
