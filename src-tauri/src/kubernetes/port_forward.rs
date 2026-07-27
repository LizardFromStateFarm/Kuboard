use kube::{Api, Client};
use k8s_openapi::api::core::v1::{Pod, Service};
use anyhow::{Result, anyhow};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tracing::{info, error, warn};
use uuid::Uuid;

pub struct PortForwardSession {
    pub session_id: String,
    pub resource_type: String, // "pod" or "service"
    pub resource_name: String,
    pub namespace: String,
    pub local_port: u16,
    pub remote_port: u16,
    pub container_name: Option<String>,
    pub stop_tx: Option<oneshot::Sender<()>>,
}

impl Clone for PortForwardSession {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            resource_type: self.resource_type.clone(),
            resource_name: self.resource_name.clone(),
            namespace: self.namespace.clone(),
            local_port: self.local_port,
            remote_port: self.remote_port,
            container_name: self.container_name.clone(),
            stop_tx: None,
        }
    }
}

impl PortForwardSession {
    pub fn new(
        resource_type: String,
        resource_name: String,
        namespace: String,
        local_port: u16,
        remote_port: u16,
        container_name: Option<String>,
        stop_tx: oneshot::Sender<()>,
    ) -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            resource_type,
            resource_name,
            namespace,
            local_port,
            remote_port,
            container_name,
            stop_tx: Some(stop_tx),
        }
    }
    
    pub fn url(&self) -> String {
        format!("http://localhost:{}", self.local_port)
    }
}

pub async fn start_port_forward_session(
    client: &Client,
    resource_type: &str,
    resource_name: &str,
    namespace: &str,
    local_port: u16,
    remote_port: u16,
    container_name: Option<&str>,
) -> Result<PortForwardSession> {
    info!("Starting port forward: {} {}:{}/{} -> localhost:{}", 
          resource_type, namespace, resource_name, remote_port, local_port);
    
    // Resolve target pod name
    let target_pod_name = match resource_type {
        "pod" => {
            let pods_api: Api<Pod> = Api::namespaced(client.clone(), namespace);
            let pod = pods_api.get(resource_name).await
                .map_err(|e| anyhow!("Pod not found: {}", e))?;
            pod.metadata.name.unwrap_or_else(|| resource_name.to_string())
        }
        "service" => {
            let services_api: Api<Service> = Api::namespaced(client.clone(), namespace);
            let svc = services_api.get(resource_name).await
                .map_err(|e| anyhow!("Service not found: {}", e))?;
            
            // Find pod matching service selector
            let selector = svc.spec.as_ref().and_then(|s| s.selector.as_ref())
                .ok_or_else(|| anyhow!("Service has no label selector"))?;
            
            let pods_api: Api<Pod> = Api::namespaced(client.clone(), namespace);
            let lp = kube::api::ListParams::default();
            let pods = pods_api.list(&lp).await.map_err(|e| anyhow!("Failed to list pods: {}", e))?;
            
            let matching_pod = pods.into_iter().find(|p| {
                if let Some(labels) = &p.metadata.labels {
                    selector.iter().all(|(k, v)| labels.get(k) == Some(v))
                } else {
                    false
                }
            }).ok_or_else(|| anyhow!("No active pod found matching service selector"))?;
            
            matching_pod.metadata.name.ok_or_else(|| anyhow!("Pod has no name"))?
        }
        _ => return Err(anyhow!("Invalid resource type: {}", resource_type)),
    };
    
    let listener = TcpListener::bind(format!("127.0.0.1:{}", local_port)).await
        .map_err(|e| anyhow!("Local port {} is not available: {}", local_port, e))?;

    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

    let client_clone = client.clone();
    let namespace_clone = namespace.to_string();

    // Spawn background TCP listener proxy loop
    tokio::spawn(async move {
        info!("🔌 Port-forward TCP listener bound on 127.0.0.1:{}", local_port);
        let pods_api: Api<Pod> = Api::namespaced(client_clone, &namespace_clone);

        loop {
            tokio::select! {
                _ = &mut stop_rx => {
                    info!("🔌 Port-forward listener on port {} stopped", local_port);
                    break;
                }
                accept_res = listener.accept() => {
                    match accept_res {
                        Ok((mut client_stream, peer_addr)) => {
                            info!("🔌 New connection on port {} from {}", local_port, peer_addr);
                            let pods_api = pods_api.clone();
                            let pod_name = target_pod_name.clone();

                            tokio::spawn(async move {
                                match pods_api.portforward(&pod_name, &[remote_port]).await {
                                    Ok(mut pf) => {
                                        if let Some(mut pod_stream) = pf.take_stream(remote_port) {
                                            if let Err(e) = tokio::io::copy_bidirectional(&mut client_stream, &mut pod_stream).await {
                                                warn!("Port forward stream copy error: {}", e);
                                            }
                                        } else {
                                            error!("Failed to obtain port stream for {}", remote_port);
                                        }
                                    }
                                    Err(e) => {
                                        error!("K8s API portforward failed for {}: {}", pod_name, e);
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            error!("Accept failed on local port {}: {}", local_port, e);
                        }
                    }
                }
            }
        }
    });

    let session = PortForwardSession::new(
        resource_type.to_string(),
        resource_name.to_string(),
        namespace.to_string(),
        local_port,
        remote_port,
        container_name.map(|s| s.to_string()),
        stop_tx,
    );
    
    info!("Created active port forward session: {}", session.session_id);
    Ok(session)
}

