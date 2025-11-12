// Copyright 2025 Kuboard Contributors
// Licensed under the MIT License - see LICENSE file for details

// Kubernetes Watch Streams
// Real-time watch streams for Kubernetes resources

use kube::{Api, Client};
use kube::runtime::watcher;
use k8s_openapi::api::core::v1::{Pod, Service};
use k8s_openapi::api::apps::v1::{Deployment, StatefulSet, DaemonSet, ReplicaSet};
use k8s_openapi::api::batch::v1::CronJob;
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

// Deployment Watch Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentWatchEvent {
    pub event_type: WatchEventType,
    pub deployment: Deployment,
}

pub struct DeploymentWatcher {
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl DeploymentWatcher {
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
        self.stop();

        info!("Starting deployment watcher");

        let deployments_api: Api<Deployment> = Api::all(client);
        let app_handle_clone = app_handle.clone();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        let handle = tokio::spawn(async move {
            let stream = watcher(deployments_api, Default::default());
            tokio::pin!(stream);

            info!("Deployment watcher started, listening for events");
            let mut seen_deployments = std::collections::HashSet::new();

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        info!("Deployment watcher stopped by user");
                        break;
                    }
                    result = stream.next() => {
                        match result {
                            Some(Ok(event)) => {
                                match event {
                                    watcher::Event::Apply(deployment) => {
                                        if let Some(name) = deployment.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                deployment.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            let is_new = !seen_deployments.contains(&key);
                                            seen_deployments.insert(key.clone());
                                            
                                            let event_type = if is_new {
                                                info!("Deployment watch event: Added {}", name);
                                                WatchEventType::Added
                                            } else {
                                                info!("Deployment watch event: Modified {}", name);
                                                WatchEventType::Modified
                                            };
                                            
                                            if let Err(e) = app_handle_clone.emit("deployment-watch-event", DeploymentWatchEvent {
                                                event_type,
                                                deployment: deployment.clone(),
                                            }) {
                                                error!("Failed to emit deployment watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Delete(deployment) => {
                                        if let Some(name) = deployment.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                deployment.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            seen_deployments.remove(&key);
                                            info!("Deployment watch event: Deleted {}", name);
                                            if let Err(e) = app_handle_clone.emit("deployment-watch-event", DeploymentWatchEvent {
                                                event_type: WatchEventType::Deleted,
                                                deployment: deployment.clone(),
                                            }) {
                                                error!("Failed to emit deployment watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Init | watcher::Event::InitApply(_) | watcher::Event::InitDone => {
                                        info!("Deployment watcher initialization event");
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("Deployment watcher error: {}", e);
                                let _ = app_handle_clone.emit("deployment-watch-error", serde_json::json!({
                                    "error": format!("Watch error: {}", e)
                                }));
                            }
                            None => {
                                warn!("Deployment watcher stream ended");
                                let _ = app_handle_clone.emit("deployment-watch-error", serde_json::json!({
                                    "error": "Watch stream ended"
                                }));
                                break;
                            }
                        }
                    }
                }
            }

            info!("Deployment watcher task completed");
        });

        self.handle = Some(handle);
        self.stop_tx = Some(stop_tx);

        Ok(())
    }
}

impl Drop for DeploymentWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

// StatefulSet Watch Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatefulSetWatchEvent {
    pub event_type: WatchEventType,
    pub statefulset: StatefulSet,
}

pub struct StatefulSetWatcher {
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl StatefulSetWatcher {
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
        self.stop();

        info!("Starting statefulset watcher");

        let statefulsets_api: Api<StatefulSet> = Api::all(client);
        let app_handle_clone = app_handle.clone();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        let handle = tokio::spawn(async move {
            let stream = watcher(statefulsets_api, Default::default());
            tokio::pin!(stream);

            info!("StatefulSet watcher started, listening for events");
            let mut seen_statefulsets = std::collections::HashSet::new();

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        info!("StatefulSet watcher stopped by user");
                        break;
                    }
                    result = stream.next() => {
                        match result {
                            Some(Ok(event)) => {
                                match event {
                                    watcher::Event::Apply(statefulset) => {
                                        if let Some(name) = statefulset.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                statefulset.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            let is_new = !seen_statefulsets.contains(&key);
                                            seen_statefulsets.insert(key.clone());
                                            
                                            let event_type = if is_new {
                                                info!("StatefulSet watch event: Added {}", name);
                                                WatchEventType::Added
                                            } else {
                                                info!("StatefulSet watch event: Modified {}", name);
                                                WatchEventType::Modified
                                            };
                                            
                                            if let Err(e) = app_handle_clone.emit("statefulset-watch-event", StatefulSetWatchEvent {
                                                event_type,
                                                statefulset: statefulset.clone(),
                                            }) {
                                                error!("Failed to emit statefulset watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Delete(statefulset) => {
                                        if let Some(name) = statefulset.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                statefulset.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            seen_statefulsets.remove(&key);
                                            info!("StatefulSet watch event: Deleted {}", name);
                                            if let Err(e) = app_handle_clone.emit("statefulset-watch-event", StatefulSetWatchEvent {
                                                event_type: WatchEventType::Deleted,
                                                statefulset: statefulset.clone(),
                                            }) {
                                                error!("Failed to emit statefulset watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Init | watcher::Event::InitApply(_) | watcher::Event::InitDone => {
                                        info!("StatefulSet watcher initialization event");
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("StatefulSet watcher error: {}", e);
                                let _ = app_handle_clone.emit("statefulset-watch-error", serde_json::json!({
                                    "error": format!("Watch error: {}", e)
                                }));
                            }
                            None => {
                                warn!("StatefulSet watcher stream ended");
                                let _ = app_handle_clone.emit("statefulset-watch-error", serde_json::json!({
                                    "error": "Watch stream ended"
                                }));
                                break;
                            }
                        }
                    }
                }
            }

            info!("StatefulSet watcher task completed");
        });

        self.handle = Some(handle);
        self.stop_tx = Some(stop_tx);

        Ok(())
    }
}

impl Drop for StatefulSetWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

// DaemonSet Watch Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonSetWatchEvent {
    pub event_type: WatchEventType,
    pub daemonset: DaemonSet,
}

pub struct DaemonSetWatcher {
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl DaemonSetWatcher {
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
        self.stop();

        info!("Starting daemonset watcher");

        let daemonsets_api: Api<DaemonSet> = Api::all(client);
        let app_handle_clone = app_handle.clone();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        let handle = tokio::spawn(async move {
            let stream = watcher(daemonsets_api, Default::default());
            tokio::pin!(stream);

            info!("DaemonSet watcher started, listening for events");
            let mut seen_daemonsets = std::collections::HashSet::new();

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        info!("DaemonSet watcher stopped by user");
                        break;
                    }
                    result = stream.next() => {
                        match result {
                            Some(Ok(event)) => {
                                match event {
                                    watcher::Event::Apply(daemonset) => {
                                        if let Some(name) = daemonset.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                daemonset.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            let is_new = !seen_daemonsets.contains(&key);
                                            seen_daemonsets.insert(key.clone());
                                            
                                            let event_type = if is_new {
                                                info!("DaemonSet watch event: Added {}", name);
                                                WatchEventType::Added
                                            } else {
                                                info!("DaemonSet watch event: Modified {}", name);
                                                WatchEventType::Modified
                                            };
                                            
                                            if let Err(e) = app_handle_clone.emit("daemonset-watch-event", DaemonSetWatchEvent {
                                                event_type,
                                                daemonset: daemonset.clone(),
                                            }) {
                                                error!("Failed to emit daemonset watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Delete(daemonset) => {
                                        if let Some(name) = daemonset.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                daemonset.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            seen_daemonsets.remove(&key);
                                            info!("DaemonSet watch event: Deleted {}", name);
                                            if let Err(e) = app_handle_clone.emit("daemonset-watch-event", DaemonSetWatchEvent {
                                                event_type: WatchEventType::Deleted,
                                                daemonset: daemonset.clone(),
                                            }) {
                                                error!("Failed to emit daemonset watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Init | watcher::Event::InitApply(_) | watcher::Event::InitDone => {
                                        info!("DaemonSet watcher initialization event");
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("DaemonSet watcher error: {}", e);
                                let _ = app_handle_clone.emit("daemonset-watch-error", serde_json::json!({
                                    "error": format!("Watch error: {}", e)
                                }));
                            }
                            None => {
                                warn!("DaemonSet watcher stream ended");
                                let _ = app_handle_clone.emit("daemonset-watch-error", serde_json::json!({
                                    "error": "Watch stream ended"
                                }));
                                break;
                            }
                        }
                    }
                }
            }

            info!("DaemonSet watcher task completed");
        });

        self.handle = Some(handle);
        self.stop_tx = Some(stop_tx);

        Ok(())
    }
}

impl Drop for DaemonSetWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

// ReplicaSet Watch Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaSetWatchEvent {
    pub event_type: WatchEventType,
    pub replicaset: ReplicaSet,
}

pub struct ReplicaSetWatcher {
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl ReplicaSetWatcher {
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
        self.stop();

        info!("Starting replicaset watcher");

        let replicasets_api: Api<ReplicaSet> = Api::all(client);
        let app_handle_clone = app_handle.clone();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        let handle = tokio::spawn(async move {
            let stream = watcher(replicasets_api, Default::default());
            tokio::pin!(stream);

            info!("ReplicaSet watcher started, listening for events");
            let mut seen_replicasets = std::collections::HashSet::new();

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        info!("ReplicaSet watcher stopped by user");
                        break;
                    }
                    result = stream.next() => {
                        match result {
                            Some(Ok(event)) => {
                                match event {
                                    watcher::Event::Apply(replicaset) => {
                                        if let Some(name) = replicaset.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                replicaset.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            let is_new = !seen_replicasets.contains(&key);
                                            seen_replicasets.insert(key.clone());
                                            
                                            let event_type = if is_new {
                                                info!("ReplicaSet watch event: Added {}", name);
                                                WatchEventType::Added
                                            } else {
                                                info!("ReplicaSet watch event: Modified {}", name);
                                                WatchEventType::Modified
                                            };
                                            
                                            if let Err(e) = app_handle_clone.emit("replicaset-watch-event", ReplicaSetWatchEvent {
                                                event_type,
                                                replicaset: replicaset.clone(),
                                            }) {
                                                error!("Failed to emit replicaset watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Delete(replicaset) => {
                                        if let Some(name) = replicaset.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                replicaset.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            seen_replicasets.remove(&key);
                                            info!("ReplicaSet watch event: Deleted {}", name);
                                            if let Err(e) = app_handle_clone.emit("replicaset-watch-event", ReplicaSetWatchEvent {
                                                event_type: WatchEventType::Deleted,
                                                replicaset: replicaset.clone(),
                                            }) {
                                                error!("Failed to emit replicaset watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Init | watcher::Event::InitApply(_) | watcher::Event::InitDone => {
                                        info!("ReplicaSet watcher initialization event");
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("ReplicaSet watcher error: {}", e);
                                let _ = app_handle_clone.emit("replicaset-watch-error", serde_json::json!({
                                    "error": format!("Watch error: {}", e)
                                }));
                            }
                            None => {
                                warn!("ReplicaSet watcher stream ended");
                                let _ = app_handle_clone.emit("replicaset-watch-error", serde_json::json!({
                                    "error": "Watch stream ended"
                                }));
                                break;
                            }
                        }
                    }
                }
            }

            info!("ReplicaSet watcher task completed");
        });

        self.handle = Some(handle);
        self.stop_tx = Some(stop_tx);

        Ok(())
    }
}

impl Drop for ReplicaSetWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

// Service Watch Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceWatchEvent {
    pub event_type: WatchEventType,
    pub service: Service,
}

pub struct ServiceWatcher {
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl ServiceWatcher {
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
        self.stop();

        info!("Starting service watcher");

        let services_api: Api<Service> = Api::all(client);
        let app_handle_clone = app_handle.clone();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        let handle = tokio::spawn(async move {
            let stream = watcher(services_api, Default::default());
            tokio::pin!(stream);

            info!("Service watcher started, listening for events");
            let mut seen_services = std::collections::HashSet::new();

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        info!("Service watcher stopped by user");
                        break;
                    }
                    result = stream.next() => {
                        match result {
                            Some(Ok(event)) => {
                                match event {
                                    watcher::Event::Apply(service) => {
                                        if let Some(name) = service.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                service.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            let is_new = !seen_services.contains(&key);
                                            seen_services.insert(key.clone());
                                            
                                            let event_type = if is_new {
                                                info!("Service watch event: Added {}", name);
                                                WatchEventType::Added
                                            } else {
                                                info!("Service watch event: Modified {}", name);
                                                WatchEventType::Modified
                                            };
                                            
                                            if let Err(e) = app_handle_clone.emit("service-watch-event", ServiceWatchEvent {
                                                event_type,
                                                service: service.clone(),
                                            }) {
                                                error!("Failed to emit service watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Delete(service) => {
                                        if let Some(name) = service.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                service.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            seen_services.remove(&key);
                                            info!("Service watch event: Deleted {}", name);
                                            if let Err(e) = app_handle_clone.emit("service-watch-event", ServiceWatchEvent {
                                                event_type: WatchEventType::Deleted,
                                                service: service.clone(),
                                            }) {
                                                error!("Failed to emit service watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Init | watcher::Event::InitApply(_) | watcher::Event::InitDone => {
                                        info!("Service watcher initialization event");
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("Service watcher error: {}", e);
                                let _ = app_handle_clone.emit("service-watch-error", serde_json::json!({
                                    "error": format!("Watch error: {}", e)
                                }));
                            }
                            None => {
                                warn!("Service watcher stream ended");
                                let _ = app_handle_clone.emit("service-watch-error", serde_json::json!({
                                    "error": "Watch stream ended"
                                }));
                                break;
                            }
                        }
                    }
                }
            }

            info!("Service watcher task completed");
        });

        self.handle = Some(handle);
        self.stop_tx = Some(stop_tx);

        Ok(())
    }
}

impl Drop for ServiceWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

// CronJob Watch Event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobWatchEvent {
    pub event_type: WatchEventType,
    pub cronjob: CronJob,
}

pub struct CronJobWatcher {
    handle: Option<JoinHandle<()>>,
    stop_tx: Option<mpsc::Sender<()>>,
}

impl CronJobWatcher {
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
        self.stop();

        info!("Starting cronjob watcher");

        let cronjobs_api: Api<CronJob> = Api::all(client);
        let app_handle_clone = app_handle.clone();
        let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);

        let handle = tokio::spawn(async move {
            let stream = watcher(cronjobs_api, Default::default());
            tokio::pin!(stream);

            info!("CronJob watcher started, listening for events");
            let mut seen_cronjobs = std::collections::HashSet::new();

            loop {
                tokio::select! {
                    _ = stop_rx.recv() => {
                        info!("CronJob watcher stopped by user");
                        break;
                    }
                    result = stream.next() => {
                        match result {
                            Some(Ok(event)) => {
                                match event {
                                    watcher::Event::Apply(cronjob) => {
                                        if let Some(name) = cronjob.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                cronjob.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            let is_new = !seen_cronjobs.contains(&key);
                                            seen_cronjobs.insert(key.clone());
                                            
                                            let event_type = if is_new {
                                                info!("CronJob watch event: Added {}", name);
                                                WatchEventType::Added
                                            } else {
                                                info!("CronJob watch event: Modified {}", name);
                                                WatchEventType::Modified
                                            };
                                            
                                            if let Err(e) = app_handle_clone.emit("cronjob-watch-event", CronJobWatchEvent {
                                                event_type,
                                                cronjob: cronjob.clone(),
                                            }) {
                                                error!("Failed to emit cronjob watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Delete(cronjob) => {
                                        if let Some(name) = cronjob.metadata.name.as_ref() {
                                            let key = format!("{}/{}", 
                                                cronjob.metadata.namespace.as_ref().unwrap_or(&"default".to_string()),
                                                name
                                            );
                                            seen_cronjobs.remove(&key);
                                            info!("CronJob watch event: Deleted {}", name);
                                            if let Err(e) = app_handle_clone.emit("cronjob-watch-event", CronJobWatchEvent {
                                                event_type: WatchEventType::Deleted,
                                                cronjob: cronjob.clone(),
                                            }) {
                                                error!("Failed to emit cronjob watch event: {}", e);
                                            }
                                        }
                                    }
                                    watcher::Event::Init | watcher::Event::InitApply(_) | watcher::Event::InitDone => {
                                        info!("CronJob watcher initialization event");
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                error!("CronJob watcher error: {}", e);
                                let _ = app_handle_clone.emit("cronjob-watch-error", serde_json::json!({
                                    "error": format!("Watch error: {}", e)
                                }));
                            }
                            None => {
                                warn!("CronJob watcher stream ended");
                                let _ = app_handle_clone.emit("cronjob-watch-error", serde_json::json!({
                                    "error": "Watch stream ended"
                                }));
                                break;
                            }
                        }
                    }
                }
            }

            info!("CronJob watcher task completed");
        });

        self.handle = Some(handle);
        self.stop_tx = Some(stop_tx);

        Ok(())
    }
}

impl Drop for CronJobWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

