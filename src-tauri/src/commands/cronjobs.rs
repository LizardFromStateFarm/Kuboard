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



// CronJob Commands
#[tauri::command]
pub async fn kuboard_get_cronjobs(state: State<'_, AppState>) -> Result<Vec<CronJob>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::all(client.clone());
    match cronjobs_api.list(&Default::default()).await {
        Ok(cronjobs) => Ok(cronjobs.items),
        Err(e) => Err(format!("Failed to get cronjobs: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<CronJob, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    match cronjobs_api.get(&name).await {
        Ok(cronjob) => Ok(cronjob),
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("CronJob {}/{} not found", namespace, name))
        }
        Err(e) => Err(format!("Failed to get cronjob: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_trigger_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Job, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    // Get the cronjob to extract its job template
    let cronjob = match cronjobs_api.get(&name).await {
        Ok(cj) => cj,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("CronJob {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get cronjob: {}", e)),
    };

    // Extract job template from cronjob spec
    let job_template = match cronjob.spec.as_ref() {
        Some(spec) => &spec.job_template,
        None => return Err("CronJob has no spec".to_string()),
    };

    // Create a new Job from the template
    let mut job_metadata = k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta {
        name: Some(format!("{}-manual-{}", name, chrono::Utc::now().timestamp())),
        namespace: Some(namespace.clone()),
        ..Default::default()
    };

    // Copy labels from job template metadata if present
    if let Some(template_metadata) = job_template.metadata.as_ref() {
        if let Some(labels) = template_metadata.labels.as_ref() {
            job_metadata.labels = Some(labels.clone());
        }
    }

    let job = Job {
        metadata: job_metadata,
        spec: job_template.spec.clone(),
        ..Default::default()
    };

    // Create the job
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    match jobs_api.create(&Default::default(), &job).await {
        Ok(created_job) => Ok(created_job),
        Err(e) => Err(format!("Failed to trigger cronjob: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_suspend_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<CronJob, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    // Get current cronjob
    let mut cronjob = match cronjobs_api.get(&name).await {
        Ok(cj) => cj,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("CronJob {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get cronjob: {}", e)),
    };

    // Set suspend to true
    if let Some(spec) = cronjob.spec.as_mut() {
        spec.suspend = Some(true);
    } else {
        return Err("CronJob has no spec".to_string());
    }

    // Apply the update
    match cronjobs_api.replace(&name, &Default::default(), &cronjob).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to suspend cronjob: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_resume_cronjob(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<CronJob, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    
    // Get current cronjob
    let mut cronjob = match cronjobs_api.get(&name).await {
        Ok(cj) => cj,
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("CronJob {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get cronjob: {}", e)),
    };

    // Set suspend to false
    if let Some(spec) = cronjob.spec.as_mut() {
        spec.suspend = Some(false);
    } else {
        return Err("CronJob has no spec".to_string());
    }

    // Apply the update
    match cronjobs_api.replace(&name, &Default::default(), &cronjob).await {
        Ok(updated) => Ok(updated),
        Err(e) => Err(format!("Failed to resume cronjob: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_cronjob_jobs(
    name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<Vec<Job>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    // Verify the cronjob exists
    let cronjobs_api: Api<CronJob> = Api::namespaced(client.clone(), &namespace);
    match cronjobs_api.get(&name).await {
        Ok(_) => {},
        Err(kube::Error::Api(e)) if e.code == 404 => {
            return Err(format!("CronJob {}/{} not found", namespace, name));
        }
        Err(e) => return Err(format!("Failed to get cronjob: {}", e)),
    }

    // List all jobs in the namespace
    let jobs_api: Api<Job> = Api::namespaced(client.clone(), &namespace);
    let jobs = match jobs_api.list(&Default::default()).await {
        Ok(job_list) => job_list.items,
        Err(e) => return Err(format!("Failed to list jobs: {}", e)),
    };

    // Filter jobs by owner reference (jobs created by this cronjob)
    let matching_jobs: Vec<Job> = jobs
        .into_iter()
        .filter(|job| {
            if let Some(owner_refs) = job.metadata.owner_references.as_ref() {
                owner_refs.iter().any(|owner| {
                    owner.kind == "CronJob" && 
                    owner.name == name &&
                    owner.controller == Some(true)
                })
            } else {
                false
            }
        })
        .collect();

    // Sort by creation timestamp (newest first)
    let mut sorted_jobs = matching_jobs;
    sorted_jobs.sort_by(|a, b| {
        let time_a = a.metadata.creation_timestamp.as_ref()
            .map(|ts| ts.0.timestamp())
            .unwrap_or(0);
        let time_b = b.metadata.creation_timestamp.as_ref()
            .map(|ts| ts.0.timestamp())
            .unwrap_or(0);
        time_b.cmp(&time_a) // Reverse order (newest first)
    });

    Ok(sorted_jobs)
}

#[tauri::command]
pub async fn kuboard_get_configmaps(state: State<'_, AppState>) -> Result<Vec<ConfigMap>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let configmaps_api: Api<ConfigMap> = Api::all(client.clone());
    match configmaps_api.list(&Default::default()).await {
        Ok(configmaps) => Ok(configmaps.items),
        Err(e) => Err(format!("Failed to get configmaps: {}", e)),
    }
}

#[tauri::command]
pub async fn kuboard_get_secrets(state: State<'_, AppState>) -> Result<Vec<Secret>, String> {
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let secrets_api: Api<Secret> = Api::all(client.clone());
    match secrets_api.list(&Default::default()).await {
        Ok(secrets) => Ok(secrets.items),
        Err(e) => Err(format!("Failed to get secrets: {}", e)),
    }
}

