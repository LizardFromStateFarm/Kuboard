// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::{Api, api::ListParams};
use serde::{Serialize, Deserialize};

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomResourceDefinition {
    pub name: String,
    pub group: String,
    pub version: String,
    pub scope: String,
    pub kind: String,
}

#[tauri::command]
pub async fn kuboard_list_crds(
    state: State<'_, AppState>
) -> Result<Vec<serde_json::Value>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context".to_string())?;

    let crds_api: Api<kube::core::DynamicObject> = Api::all_with(
        client.clone(),
        &kube::discovery::ApiResource::from_gvk(&kube::api::GroupVersionKind::gvk(
            "apiextensions.k8s.io",
            "v1",
            "CustomResourceDefinition",
        )),
    );

    let list = crds_api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    
    Ok(list.items.into_iter().map(|o| serde_json::to_value(o).unwrap()).collect())
}

#[tauri::command]
pub async fn kuboard_list_custom_resource_instances(
    group: String,
    version: String,
    kind: String,
    namespace: Option<String>,
    state: State<'_, AppState>
) -> Result<Vec<serde_json::Value>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context".to_string())?;

    let ar = kube::discovery::ApiResource::from_gvk(&kube::api::GroupVersionKind::gvk(
        &group,
        &version,
        &kind,
    ));

    let api: Api<kube::core::DynamicObject> = if let Some(ns) = namespace {
        Api::namespaced_with(client.clone(), &ns, &ar)
    } else {
        Api::all_with(client.clone(), &ar)
    };

    let list = api.list(&ListParams::default()).await.map_err(|e| e.to_string())?;
    
    Ok(list.items.into_iter().map(|o| serde_json::to_value(o).unwrap()).collect())
}
