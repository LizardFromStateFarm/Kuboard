// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::Api;
use kube::api::{ListParams, DeleteParams};
use k8s_openapi::api::rbac::v1::{Role, ClusterRole, RoleBinding, ClusterRoleBinding};
use k8s_openapi::api::core::v1::ServiceAccount;
use tracing::{error, info};

use crate::app_state::AppState;

// Role Commands
#[tauri::command]
pub async fn kuboard_list_roles(
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Role>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<Role> = if namespace.is_empty() || namespace == "all" {
        Api::all(client.clone())
    } else {
        Api::namespaced(client.clone(), &namespace)
    };

    match api.list(&ListParams::default()).await {
        Ok(list) => Ok(list.items),
        Err(e) => Err(format!("Failed to list Roles: {}", e)),
    }
}

// ClusterRole Commands
#[tauri::command]
pub async fn kuboard_list_cluster_roles(
    state: State<'_, AppState>
) -> Result<Vec<ClusterRole>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<ClusterRole> = Api::all(client.clone());
    match api.list(&ListParams::default()).await {
        Ok(list) => Ok(list.items),
        Err(e) => Err(format!("Failed to list ClusterRoles: {}", e)),
    }
}

// RoleBinding Commands
#[tauri::command]
pub async fn kuboard_list_role_bindings(
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<RoleBinding>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<RoleBinding> = if namespace.is_empty() || namespace == "all" {
        Api::all(client.clone())
    } else {
        Api::namespaced(client.clone(), &namespace)
    };

    match api.list(&ListParams::default()).await {
        Ok(list) => Ok(list.items),
        Err(e) => Err(format!("Failed to list RoleBindings: {}", e)),
    }
}

// ClusterRoleBinding Commands
#[tauri::command]
pub async fn kuboard_list_cluster_role_bindings(
    state: State<'_, AppState>
) -> Result<Vec<ClusterRoleBinding>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<ClusterRoleBinding> = Api::all(client.clone());
    match api.list(&ListParams::default()).await {
        Ok(list) => Ok(list.items),
        Err(e) => Err(format!("Failed to list ClusterRoleBindings: {}", e)),
    }
}

// ServiceAccount Commands
#[tauri::command]
pub async fn kuboard_list_service_accounts(
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<ServiceAccount>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<ServiceAccount> = if namespace.is_empty() || namespace == "all" {
        Api::all(client.clone())
    } else {
        Api::namespaced(client.clone(), &namespace)
    };

    match api.list(&ListParams::default()).await {
        Ok(list) => Ok(list.items),
        Err(e) => Err(format!("Failed to list ServiceAccounts: {}", e)),
    }
}

// Delete Commands
#[tauri::command]
pub async fn kuboard_delete_role(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<Role> = Api::namespaced(client.clone(), &namespace);
    match api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted Role: {}/{}", namespace, name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete Role {}/{}: {}", namespace, name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_cluster_role(
    name: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<ClusterRole> = Api::all(client.clone());
    match api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted ClusterRole: {}", name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete ClusterRole {}: {}", name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_role_binding(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<RoleBinding> = Api::namespaced(client.clone(), &namespace);
    match api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted RoleBinding: {}/{}", namespace, name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete RoleBinding {}/{}: {}", namespace, name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_cluster_role_binding(
    name: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<ClusterRoleBinding> = Api::all(client.clone());
    match api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted ClusterRoleBinding: {}", name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete ClusterRoleBinding {}: {}", name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_service_account(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let api: Api<ServiceAccount> = Api::namespaced(client.clone(), &namespace);
    match api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted ServiceAccount: {}/{}", namespace, name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete ServiceAccount {}/{}: {}", namespace, name, e)),
    }
}
