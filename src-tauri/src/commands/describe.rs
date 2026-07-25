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



// Resource Describe Commands
#[tauri::command]
pub async fn kuboard_describe_pod(
    pod_name: String,
    namespace: String,
    state: State<'_, AppState>
) -> Result<serde_json::Value, String> {
    info!("Describing pod: {}/{}", namespace, pod_name);
    
    let client_guard = state.current_client.read().await;
    let client = client_guard
        .as_ref()
        .ok_or_else(|| "No active context. Please set a context first.".to_string())?;

    let pods_api: Api<Pod> = Api::namespaced(client.clone(), &namespace);
    
    match pods_api.get(&pod_name).await {
        Ok(pod) => {
            // Get pod events
            let events = kuboard_fetch_pod_events(client, &pod_name, &namespace).await.unwrap_or_default();
            
            // Build describe output structure
            let describe = json!({
                "name": pod.metadata.name.as_ref().unwrap_or(&"Unknown".to_string()),
                "namespace": pod.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                "labels": pod.metadata.labels.as_ref().unwrap_or(&std::collections::BTreeMap::new()),
                "annotations": pod.metadata.annotations.as_ref().unwrap_or(&std::collections::BTreeMap::new()),
                "status": {
                    "phase": pod.status.as_ref().and_then(|s| s.phase.as_ref()).unwrap_or(&"Unknown".to_string()),
                    "podIP": pod.status.as_ref().and_then(|s| s.pod_ip.as_ref()).unwrap_or(&"None".to_string()),
                    "hostIP": pod.status.as_ref().and_then(|s| s.host_ip.as_ref()).unwrap_or(&"None".to_string()),
                    "nodeName": pod.spec.as_ref().and_then(|s| s.node_name.as_ref()).unwrap_or(&"None".to_string()),
                    "qosClass": pod.status.as_ref().and_then(|s| s.qos_class.as_ref()).unwrap_or(&"Unknown".to_string()),
                    "startTime": pod.status.as_ref().and_then(|s| s.start_time.as_ref()).map(|t| t.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                },
                "conditions": pod.status.as_ref()
                    .and_then(|s| s.conditions.as_ref())
                    .map(|conditions| conditions.iter().map(|c| json!({
                        "type": c.type_,
                        "status": c.status,
                        "reason": c.reason.as_ref().unwrap_or(&"None".to_string()),
                        "message": c.message.as_ref().unwrap_or(&"None".to_string()),
                        "lastTransitionTime": c.last_transition_time.as_ref().map(|t| t.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                    })).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "containers": pod.spec.as_ref()
                    .map(|s| s.containers.iter().map(|c| {
                        let status = pod.status.as_ref()
                            .and_then(|s| s.container_statuses.as_ref())
                            .and_then(|statuses| statuses.iter().find(|cs| cs.name == c.name));
                        json!({
                            "name": c.name,
                            "image": c.image,
                            "imagePullPolicy": c.image_pull_policy.as_ref().unwrap_or(&"IfNotPresent".to_string()),
                            "resources": c.resources.as_ref().map(|r| json!({
                                "requests": r.requests.as_ref().map(|reqs| reqs.iter().map(|(k, v)| (k, v.0.clone())).collect::<std::collections::BTreeMap<_, _>>()).unwrap_or_default(),
                                "limits": r.limits.as_ref().map(|lims| lims.iter().map(|(k, v)| (k, v.0.clone())).collect::<std::collections::BTreeMap<_, _>>()).unwrap_or_default(),
                            })),
                            "ports": c.ports.as_ref().map(|ports| ports.iter().map(|p| json!({
                                "name": p.name.as_ref().unwrap_or(&"None".to_string()),
                                "containerPort": p.container_port,
                                "protocol": p.protocol.as_ref().unwrap_or(&"TCP".to_string()),
                            })).collect::<Vec<_>>()).unwrap_or_default(),
                            "env": c.env.as_ref().map(|envs| envs.iter().map(|e| {
                                let mut env_json = serde_json::Map::new();
                                env_json.insert("name".to_string(), json!(e.name));
                                env_json.insert("value".to_string(), json!(e.value.as_ref().unwrap_or(&"None".to_string())));
                                if let Some(vf) = e.value_from.as_ref() {
                                    let mut value_from_json = serde_json::Map::new();
                                    if let Some(fr) = vf.field_ref.as_ref() {
                                        let mut field_ref_json: serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
                                        field_ref_json.insert("fieldPath".to_string(), serde_json::Value::String(fr.field_path.clone()));
                                        value_from_json.insert("fieldRef".to_string(), serde_json::Value::Object(field_ref_json));
                                    }
                                    env_json.insert("valueFrom".to_string(), json!(value_from_json));
                                }
                                json!(env_json)
                            }).collect::<Vec<_>>()).unwrap_or_default(),
                            "status": status.map(|s| json!({
                                "ready": s.ready,
                                "restartCount": s.restart_count,
                                "state": {
                                    "running": s.state.as_ref().and_then(|st| st.running.as_ref()).map(|_| json!({"startedAt": "Running"})),
                                    "waiting": s.state.as_ref().and_then(|st| st.waiting.as_ref()).map(|w| json!({
                                        "reason": w.reason.as_ref().unwrap_or(&"None".to_string()),
                                        "message": w.message.as_ref().unwrap_or(&"None".to_string()),
                                    })),
                                    "terminated": s.state.as_ref().and_then(|st| st.terminated.as_ref()).map(|t| json!({
                                        "reason": t.reason.as_ref().unwrap_or(&"None".to_string()),
                                        "exitCode": t.exit_code,
                                        "startedAt": t.started_at.as_ref().map(|dt| dt.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                                        "finishedAt": t.finished_at.as_ref().map(|dt| dt.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                                    })),
                                },
                            })).unwrap_or(json!({})),
                        })
                    }).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "volumes": pod.spec.as_ref()
                    .and_then(|s| s.volumes.as_ref())
                    .map(|volumes| volumes.iter().map(|v| json!({
                        "name": v.name,
                        "type": if v.config_map.is_some() { "ConfigMap" } 
                               else if v.secret.is_some() { "Secret" }
                               else if v.persistent_volume_claim.is_some() { "PVC" }
                               else if v.empty_dir.is_some() { "EmptyDir" }
                               else { "Other" },
                    })).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "tolerations": pod.spec.as_ref()
                    .and_then(|s| s.tolerations.as_ref())
                    .map(|tolerations| tolerations.iter().map(|t| json!({
                        "key": t.key.as_ref().unwrap_or(&"".to_string()),
                        "operator": t.operator.as_ref().unwrap_or(&"Equal".to_string()),
                        "value": t.value.as_ref().unwrap_or(&"None".to_string()),
                        "effect": t.effect.as_ref().unwrap_or(&"None".to_string()),
                        "tolerationSeconds": t.toleration_seconds,
                    })).collect::<Vec<_>>())
                    .unwrap_or_default(),
                "events": events.iter().map(|e| json!({
                    "type": e.type_,
                    "reason": e.reason,
                    "message": e.message,
                    "count": e.count,
                    "firstTimestamp": e.first_timestamp.as_deref().unwrap_or("None"),
                    "lastTimestamp": e.last_timestamp.as_deref().unwrap_or("None"),
                })).collect::<Vec<_>>(),
                "metadata": {
                    "uid": pod.metadata.uid.as_ref().unwrap_or(&"None".to_string()),
                    "resourceVersion": pod.metadata.resource_version.as_ref().unwrap_or(&"None".to_string()),
                    "creationTimestamp": pod.metadata.creation_timestamp.as_ref().map(|t| t.0.to_rfc3339()).unwrap_or_else(|| "None".to_string()),
                    "generation": pod.metadata.generation.unwrap_or(0),
                },
            });
            
            info!("✅ Successfully described pod: {}/{}", namespace, pod_name);
            Ok(describe)
        }
        Err(kube::Error::Api(e)) if e.code == 404 => {
            Err(format!("Pod {}/{} not found", namespace, pod_name))
        }
        Err(e) => {
            error!("Failed to describe pod {}/{}: {}", namespace, pod_name, e);
            Err(format!("Failed to describe pod: {}", e))
        }
    }
}

