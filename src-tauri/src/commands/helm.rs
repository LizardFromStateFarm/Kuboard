// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

use tauri::State;
use kube::{Api, api::ListParams};
use k8s_openapi::api::core::v1::Secret;
use serde::{Serialize, Deserialize};
use base64::{engine::general_purpose, Engine as _};
use flate2::read::GzDecoder;
use std::io::Read;

use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct HelmRelease {
    pub name: String,
    pub namespace: String,
    pub revision: i32,
    pub updated: String,
    pub status: String,
    pub chart: String,
    pub app_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelmReleaseDetail {
    pub name: String,
    pub namespace: String,
    pub revision: i32,
    pub config: serde_json::Value,
    pub manifest: String,
    pub info: HelmReleaseInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HelmReleaseInfo {
    pub first_deployed: String,
    pub last_deployed: String,
    pub deleted: String,
    pub description: String,
    pub status: String,
    pub notes: String,
}

#[tauri::command]
pub async fn kuboard_list_helm_releases(
    state: State<'_, AppState>
) -> Result<Vec<HelmRelease>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context".to_string())?;

    let secrets_api: Api<Secret> = Api::all(client.clone());
    let lp = ListParams::default().labels("owner=helm");
    
    let secrets = secrets_api.list(&lp).await.map_err(|e| e.to_string())?;
    
    let mut releases: Vec<HelmRelease> = Vec::new();
    
    for secret in secrets.items {
        if let Some(data) = secret.data {
            if let Some(release_data) = data.get("release") {
                // Decode Helm release data
                // It's base64 encoded, then gzipped, then base64 encoded again (Helm 3)
                // Actually it's base64 encoded once, then gzipped, then base64 encoded again?
                // Standard Helm 3: Base64 -> Gzip -> Base64
                
                if let Ok(decoded) = decode_helm_release(&release_data.0) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&decoded) {
                        let name = v["name"].as_str().unwrap_or_default().to_string();
                        let namespace = v["namespace"].as_str().unwrap_or_default().to_string();
                        let version = v["version"].as_i64().unwrap_or(0) as i32;
                        let status = v["info"]["status"].as_str().unwrap_or_default().to_string();
                        let chart = v["chart"]["metadata"]["name"].as_str().unwrap_or_default().to_string() 
                                    + "-" + v["chart"]["metadata"]["version"].as_str().unwrap_or_default();
                        let app_version = v["chart"]["metadata"]["appVersion"].as_str().unwrap_or_default().to_string();
                        let updated = v["info"]["last_deployed"].as_str().unwrap_or_default().to_string();

                        // Only keep the latest version for listing
                        if let Some(existing) = releases.iter_mut().find(|r| r.name == name && r.namespace == namespace) {
                            if version > existing.revision {
                                existing.revision = version;
                                existing.status = status;
                                existing.chart = chart;
                                existing.app_version = app_version;
                                existing.updated = updated;
                            }
                        } else {
                            releases.push(HelmRelease {
                                name,
                                namespace,
                                revision: version,
                                status,
                                chart,
                                app_version,
                                updated,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(releases)
}

fn decode_helm_release(data: &[u8]) -> Result<String, String> {
    // 1. Base64 decode step 1
    let step1 = general_purpose::STANDARD
        .decode(data)
        .or_else(|_| general_purpose::URL_SAFE.decode(data))
        .map_err(|e| format!("Initial base64 decode failed: {}", e))?;

    // Try double base64 + Gzip (Standard Helm 3 Secret Storage)
    if let Ok(step2) = general_purpose::STANDARD.decode(&step1) {
        let mut decoder = GzDecoder::new(&step2[..]);
        let mut decoded = String::new();
        if decoder.read_to_string(&mut decoded).is_ok() && !decoded.is_empty() {
            return Ok(decoded);
        }
    }

    // Try single base64 + Gzip
    let mut decoder = GzDecoder::new(&step1[..]);
    let mut decoded = String::new();
    if decoder.read_to_string(&mut decoded).is_ok() && !decoded.is_empty() {
        return Ok(decoded);
    }

    // Fallback: Raw UTF-8 string from step1 (uncompressed JSON)
    if let Ok(utf8_str) = String::from_utf8(step1.clone()) {
        if utf8_str.trim().starts_with('{') {
            return Ok(utf8_str);
        }
    }

    // Fallback: Raw UTF-8 string from raw bytes
    if let Ok(utf8_str) = String::from_utf8(data.to_vec()) {
        if utf8_str.trim().starts_with('{') {
            return Ok(utf8_str);
        }
    }

    Err("Failed to decode Helm release data with any supported encoding".to_string())
}

#[tauri::command]
pub async fn kuboard_get_helm_release_details(
    name: String,
    namespace: String,
    revision: i32,
    state: State<'_, AppState>
) -> Result<HelmReleaseDetail, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context".to_string())?;

    let secrets_api: Api<Secret> = Api::namespaced(client.clone(), &namespace);
    let secret_name = format!("sh.helm.release.v1.{}.v{}", name, revision);
    
    let secret = secrets_api.get(&secret_name).await.map_err(|e| e.to_string())?;
    
    if let Some(data) = secret.data {
        if let Some(release_data) = data.get("release") {
            let decoded = decode_helm_release(&release_data.0)?;
            let v: serde_json::Value = serde_json::from_str(&decoded).map_err(|e| e.to_string())?;
            
            return Ok(HelmReleaseDetail {
                name: v["name"].as_str().unwrap_or_default().to_string(),
                namespace: v["namespace"].as_str().unwrap_or_default().to_string(),
                revision: v["version"].as_i64().unwrap_or(0) as i32,
                config: v["config"].clone(),
                manifest: v["manifest"].as_str().unwrap_or_default().to_string(),
                info: HelmReleaseInfo {
                    first_deployed: v["info"]["first_deployed"].as_str().unwrap_or_default().to_string(),
                    last_deployed: v["info"]["last_deployed"].as_str().unwrap_or_default().to_string(),
                    deleted: v["info"]["deleted"].as_str().unwrap_or_default().to_string(),
                    description: v["info"]["description"].as_str().unwrap_or_default().to_string(),
                    status: v["info"]["status"].as_str().unwrap_or_default().to_string(),
                    notes: v["info"]["notes"].as_str().unwrap_or_default().to_string(),
                },
            });
        }
    }

    Err("Release data not found".to_string())
}
