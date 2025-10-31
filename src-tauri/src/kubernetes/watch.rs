// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kubernetes Watch Streams
// Real-time watch streams for Kubernetes resources

use kube::{Api, Client};
use kube::runtime::watcher;
use k8s_openapi::api::core::v1::Pod;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Emitter};
use tracing::{error, info, warn};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::StreamExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WatchEventType {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodWatchEvent {
    pub event_type: WatchEventType,
    pub pod: Pod,
}

pub struct PodWatcher {
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl PodWatcher {
    pub fn new() -> Self {
        Self {
            handle: None,
            stop_tx: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.handle.is_some()
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.try_send(());
        }
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }

    pub async fn start(
        &mut self,
        client: Client,
        app_handle: AppHandle,
    ) -> Result<(), String> {
        // Stop existing watcher if any
        self.stop();

        info!("Starting pod watcher");

        let pods_api: Api<Pod> = Api::all(client);
        let app_handle_clone = app_handle.clone();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        let handle = tokio::spawn(async move {
            let stream = watcher(pods_api, Default::default());
            tokio::pin!(stream);

            info!("Pod watcher started, listening for events");
            
            // Track seen pods to distinguish Added vs Modified on Applied events
            let mut seen_pods = std::collections::HashSet::new();

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        info!("Pod watcher stopped by user");
                        break;
                    }
                    result = stream.next() => {
                        match result {
                            Some(Ok(event)) => {
                                match event {
                                    watcher::Event::Apply(pod) => {
                                        if let Some(name) = pod.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                pod.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            let is_new = !seen_pods.contains(&key);
                                            seen_pods.insert(key.clone());
                                            
                                            let event_type = if is_new {
                                                info!("Pod watch event: Added {}", name);
                                                WatchEventType::Added
                                            } else {
                                                info!("Pod watch event: Modified {}", name);
                                                WatchEventType::Modified
                                            };
                                            
                                            if let Err(e) = app_handle_clone.emit("pod-watch-event", PodWatchEvent {
                                                event_type,
                                                pod: pod.clone(),
                                            }) {
                                                error!("Failed to emit pod watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Delete(pod) => {
                                        if let Some(name) = pod.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                pod.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            seen_pods.remove(&key);
                                            info!("Pod watch event: Deleted {}", name);
                                            if let Err(e) = app_handle_clone.emit("pod-watch-event", PodWatchEvent {
                                                event_type: WatchEventType::Deleted,
                                                pod: pod.clone(),
                                            }) {
                                                error!("Failed to emit pod watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Init | watcher::Event::InitApply(_) | watcher::Event::InitDone => {
                                        // These events are part of the initial sync and don't need special handling
                                        // The Apply events during initialization will be handled above
                                        info!("Pod watcher initialization event");
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("Pod watcher error: {}", e);
                                let _ = app_handle_clone.emit("pod-watch-error", serde_json::json!({
                                    "error": format!("Watch error: {}", e)
                                }));
                                // Try to continue, but log the error
                            }
                            None => {
                                warn!("Pod watcher stream ended");
                                let _ = app_handle_clone.emit("pod-watch-error", serde_json::json!({
                                    "error": "Watch stream ended"
                                }));
                                break;
                            }
                        }
                    }
                }
            }

            info!("Pod watcher task completed");
        });

        self.handle = Some(handle);
        self.stop_tx = Some(stop_tx);

        Ok(())
    }
}

impl Drop for PodWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

