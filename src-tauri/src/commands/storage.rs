// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::Api;
use kube::api::{ListParams, DeleteParams};
use k8s_openapi::api::core::v1::{PersistentVolume, PersistentVolumeClaim};
use k8s_openapi::api::storage::v1::StorageClass;
use tracing::{error, info};

use crate::app_state::AppState;

// Persistent Volume Commands
#[tauri::command]
pub async fn kuboard_list_persistent_volumes(
    state: State<'_, AppState>
) -> Result<Vec<PersistentVolume>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pv_api: Api<PersistentVolume> = Api::all(client.clone());
    match pv_api.list(&ListParams::default()).await {
        Ok(pv_list) => Ok(pv_list.items),
        Err(e) => Err(format!("Failed to list PersistentVolumes: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_persistent_volume(
    name: String,
    state: State<'_, AppState>
) -> Result<PersistentVolume, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pv_api: Api<PersistentVolume> = Api::all(client.clone());
    match pv_api.get(&name).await {
        Ok(pv) => Ok(pv),
        Err(e) => Err(format!("Failed to get PersistentVolume {}: {}", name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_persistent_volume(
    name: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pv_api: Api<PersistentVolume> = Api::all(client.clone());
    match pv_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted PersistentVolume: {}", name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete PersistentVolume {}: {}", name, e)),
    }
}

// Persistent Volume Claim Commands
#[tauri::command]
pub async fn kuboard_list_persistent_volume_claims(
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<PersistentVolumeClaim>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pvc_api: Api<PersistentVolumeClaim> = if namespace.is_empty() || namespace == "all" {
        Api::all(client.clone())
    } else {
        Api::namespaced(client.clone(), &namespace)
    };

    match pvc_api.list(&ListParams::default()).await {
        Ok(pvc_list) => Ok(pvc_list.items),
        Err(e) => Err(format!("Failed to list PersistentVolumeClaims: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_persistent_volume_claim(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<PersistentVolumeClaim, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), &namespace);
    match pvc_api.get(&name).await {
        Ok(pvc) => Ok(pvc),
        Err(e) => Err(format!("Failed to get PersistentVolumeClaim {}/{}: {}", namespace, name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_persistent_volume_claim(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pvc_api: Api<PersistentVolumeClaim> = Api::namespaced(client.clone(), &namespace);
    match pvc_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted PersistentVolumeClaim: {}/{}", namespace, name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete PersistentVolumeClaim {}/{}: {}", namespace, name, e)),
    }
}

// Storage Class Commands
#[tauri::command]
pub async fn kuboard_list_storage_classes(
    state: State<'_, AppState>
) -> Result<Vec<StorageClass>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let sc_api: Api<StorageClass> = Api::all(client.clone());
    match sc_api.list(&ListParams::default()).await {
        Ok(sc_list) => Ok(sc_list.items),
        Err(e) => Err(format!("Failed to list StorageClasses: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_storage_class(
    name: String,
    state: State<'_, AppState>
) -> Result<StorageClass, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let sc_api: Api<StorageClass> = Api::all(client.clone());
    match sc_api.get(&name).await {
        Ok(sc) => Ok(sc),
        Err(e) => Err(format!("Failed to get StorageClass {}: {}", name, e)),
    }
}

#[tauri::command]
pub async fn kuboard_delete_storage_class(
    name: String,
    state: State<'_, AppState>
) -> Result<(), String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let sc_api: Api<StorageClass> = Api::all(client.clone());
    match sc_api.delete(&name, &DeleteParams::default()).await {
        Ok(_) => {
            info!("Deleted StorageClass: {}", name);
            Ok(())
        },
        Err(e) => Err(format!("Failed to delete StorageClass {}: {}", name, e)),
    }
}
