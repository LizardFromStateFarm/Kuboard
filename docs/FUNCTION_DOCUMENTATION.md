# 📖 Kuboard Function Documentation & API Inventory

This document provides a comprehensive inventory of all backend Tauri command handlers, helper modules, and frontend functions in Kuboard.

---

## 🔧 **Backend Command Inventory (Rust - Tauri RPC)**

Below is the complete, reconciled list of registered Tauri commands exposed via `tauri::generate_handler!` in `src-tauri/src/lib.rs`.

### **1. Context Management Commands (`commands/contexts.rs`)**
| Function Name | Parameters | Description | Status |
|---------------|------------|-------------|--------|
| `kuboard_list_contexts` | None | Reads local `~/.kube/config` and returns available context objects | ✅ Working |
| `kuboard_set_context` | `context_name: String` | Sets the active kubeconfig context and initializes the Kubernetes client | ✅ Working |
| `kuboard_get_current_context` | None | Returns the active Kubernetes context name | ✅ Working |

### **2. Cluster & Resource Overview Commands (`commands/cluster.rs`, `commands/resources.rs`)**
| Function Name | Parameters | Description | Status |
|---------------|------------|-------------|--------|
| `kuboard_get_cluster_overview` | None | Returns node/pod/namespace counts, Kubernetes API version, and cluster health | ✅ Working |
| `kuboard_get_cluster_metrics` | None | Computes cluster-wide aggregated CPU, Memory, and Disk utilization | ✅ Working |
| `kuboard_get_cluster_events` | None | Fetches cluster-wide Kubernetes warning/error events | ✅ Working |
| `kuboard_get_nodes` | None | Fetches all node manifests with conditions, specs, and status | ✅ Working |
| `kuboard_get_namespaces` | None | Fetches all namespaces in the cluster | ✅ Working |
| `kuboard_get_configmaps` | None | Fetches ConfigMaps across all or specified namespaces | ✅ Working |
| `kuboard_get_secrets` | None | Fetches Secrets metadata and masked data | ✅ Working |

### **3. Workload Management Commands (`commands/pods.rs`, `deployments.rs`, `replicasets.rs`, etc.)**
| Function Name | Parameters | Description | Status |
|---------------|------------|-------------|--------|
| `kuboard_get_pods` | None | Fetches all pods in active namespace / cluster | ✅ Working |
| `kuboard_get_pod_events` | `name: String, namespace: String` | Fetches K8s events related to specific pod | ✅ Working |
| `kuboard_get_pod_logs` | `name: String, namespace: String, container: Option<String>, tail: Option<i64>` | Streams pod/container logs | ✅ Working |
| `kuboard_get_workload_logs` | `resource_type: String, resource_name: String, namespace: String, tail_lines: Option<u32>` | Aggregates and streams color-coded merged logs across all pods matching workload label selectors | ✅ Working |
| `kuboard_delete_pod` | `name: String, namespace: String` | Deletes specified pod | ✅ Working |
| `kuboard_restart_pod` | `name: String, namespace: String` | Restarts pod (deletes for controller recreation) | ✅ Working |
| `kuboard_get_pod_yaml` | `name: String, namespace: String` | Returns raw Pod YAML string | ✅ Working |
| `kuboard_update_pod_from_yaml` | `yaml: String` | Applies updated Pod manifest | ✅ Working |
| `kuboard_describe_pod` | `name: String, namespace: String` | Generates structured describe view output | ✅ Working |
| `kuboard_get_deployments` | None | Fetches all Deployments | ✅ Working |
| `kuboard_get_deployment` | `name: String, namespace: String` | Fetches single Deployment by name | ✅ Working |
| `kuboard_scale_deployment` | `name: String, namespace: String, replicas: i32` | Scales Deployment replica count | ✅ Working |
| `kuboard_rollback_deployment` | `name: String, namespace: String` | Rollback Deployment revision | ✅ Working |
| `kuboard_restart_deployment` | `name: String, namespace: String` | Triggers rolling restart of Deployment | ✅ Working |
| `kuboard_get_deployment_replicasets` | `name: String, namespace: String` | Fetches owned ReplicaSets for Deployment | ✅ Working |
| `kuboard_get_deployment_pods` | `name: String, namespace: String` | Fetches managed Pods for Deployment | ✅ Working |
| `kuboard_delete_deployment` | `name: String, namespace: String` | Deletes Deployment | ✅ Working |
| `kuboard_get_deployment_yaml` | `name: String, namespace: String` | Gets Deployment manifest YAML | ✅ Working |
| `kuboard_get_replicasets` | None | Fetches all ReplicaSets | ✅ Working |
| `kuboard_get_replicaset` | `name: String, namespace: String` | Fetches single ReplicaSet by name | ✅ Working |
| `kuboard_scale_replicaset` | `name: String, namespace: String, replicas: i32` | Scales ReplicaSet replica count | ✅ Working |
| `kuboard_get_replicaset_pods` | `name: String, namespace: String` | Fetches Pods owned by ReplicaSet | ✅ Working |
| `kuboard_delete_replicaset` | `name: String, namespace: String` | Deletes ReplicaSet | ✅ Working |
| `kuboard_get_replicaset_yaml` | `name: String, namespace: String` | Gets ReplicaSet YAML | ✅ Working |
| `kuboard_get_statefulsets` | None | Fetches all StatefulSets | ✅ Working |
| `kuboard_get_statefulset` | `name: String, namespace: String` | Fetches single StatefulSet by name | ✅ Working |
| `kuboard_scale_statefulset` | `name: String, namespace: String, replicas: i32` | Scales StatefulSet replica count | ✅ Working |
| `kuboard_restart_statefulset` | `name: String, namespace: String` | Triggers rolling restart of StatefulSet | ✅ Working |
| `kuboard_get_statefulset_pods` | `name: String, namespace: String` | Fetches Pods owned by StatefulSet | ✅ Working |
| `kuboard_delete_statefulset` | `name: String, namespace: String` | Deletes StatefulSet | ✅ Working |
| `kuboard_get_statefulset_yaml` | `name: String, namespace: String` | Gets StatefulSet YAML | ✅ Working |
| `kuboard_get_daemonsets` | None | Fetches all DaemonSets | ✅ Working |
| `kuboard_get_daemonset` | `name: String, namespace: String` | Fetches single DaemonSet by name | ✅ Working |
| `kuboard_restart_daemonset` | `name: String, namespace: String` | Restarts DaemonSet pods | ✅ Working |
| `kuboard_get_daemonset_pods` | `name: String, namespace: String` | Fetches Pods owned by DaemonSet | ✅ Working |
| `kuboard_delete_daemonset` | `name: String, namespace: String` | Deletes DaemonSet | ✅ Working |
| `kuboard_get_daemonset_yaml` | `name: String, namespace: String` | Gets DaemonSet YAML | ✅ Working |
| `kuboard_get_cronjobs` | None | Fetches all CronJobs | ✅ Working |
| `kuboard_get_cronjob` | `name: String, namespace: String` | Fetches single CronJob by name | ✅ Working |
| `kuboard_trigger_cronjob` | `name: String, namespace: String` | Triggers immediate execution (creates Job) | ✅ Working |
| `kuboard_suspend_cronjob` | `name: String, namespace: String` | Suspends CronJob schedule | ✅ Working |
| `kuboard_resume_cronjob` | `name: String, namespace: String` | Resumes CronJob schedule | ✅ Working |
| `kuboard_get_cronjob_jobs` | `name: String, namespace: String` | Fetches historical Jobs triggered by CronJob | ✅ Working |
| `kuboard_delete_cronjob` | `name: String, namespace: String` | Deletes CronJob | ✅ Working |
| `kuboard_get_cronjob_yaml` | `name: String, namespace: String` | Gets CronJob YAML | ✅ Working |

### **4. Networking, Storage & RBAC Commands**
| Function Name | Parameters | Description | Status |
|---------------|------------|-------------|--------|
| `kuboard_get_services` | None | Fetches all Services | ✅ Working |
| `kuboard_get_service` | `name: String, namespace: String` | Fetches single Service | ✅ Working |
| `kuboard_get_service_endpoints` | `name: String, namespace: String` | Fetches Endpoints for Service | ✅ Working |
| `kuboard_delete_service` | `name: String, namespace: String` | Deletes Service | ✅ Working |
| `kuboard_get_service_yaml` | `name: String, namespace: String` | Gets Service YAML | ✅ Working |
| `kuboard_list_ingresses` | None | Fetches all Ingresses | ✅ Working |
| `kuboard_list_ingress_classes` | None | Fetches IngressClasses | ✅ Working |
| `kuboard_list_network_policies` | None | Fetches NetworkPolicies | ✅ Working |
| `kuboard_delete_ingress` | `name: String, namespace: String` | Deletes Ingress | ✅ Working |
| `kuboard_delete_ingress_class` | `name: String` | Deletes IngressClass | ✅ Working |
| `kuboard_delete_network_policy` | `name: String, namespace: String` | Deletes NetworkPolicy | ✅ Working |
| `kuboard_list_persistent_volumes` | None | Fetches PersistentVolumes | ✅ Working |
| `kuboard_get_persistent_volume` | `name: String` | Fetches single PersistentVolume | ✅ Working |
| `kuboard_delete_persistent_volume` | `name: String` | Deletes PersistentVolume | ✅ Working |
| `kuboard_list_persistent_volume_claims` | None | Fetches PersistentVolumeClaims | ✅ Working |
| `kuboard_get_persistent_volume_claim` | `name: String, namespace: String` | Fetches single PVC | ✅ Working |
| `kuboard_delete_persistent_volume_claim` | `name: String, namespace: String` | Deletes PVC | ✅ Working |
| `kuboard_list_storage_classes` | None | Fetches StorageClasses | ✅ Working |
| `kuboard_get_storage_class` | `name: String` | Fetches StorageClass details | ✅ Working |
| `kuboard_delete_storage_class` | `name: String` | Deletes StorageClass | ✅ Working |
| `kuboard_list_roles` | None | Fetches Roles | ✅ Working |
| `kuboard_list_cluster_roles` | None | Fetches ClusterRoles | ✅ Working |
| `kuboard_list_role_bindings` | None | Fetches RoleBindings | ✅ Working |
| `kuboard_list_cluster_role_bindings` | None | Fetches ClusterRoleBindings | ✅ Working |
| `kuboard_list_service_accounts` | None | Fetches ServiceAccounts | ✅ Working |
| `kuboard_delete_role` / `delete_cluster_role` | `name: String` | Deletes Role/ClusterRole | ✅ Working |
| `kuboard_delete_role_binding` / `delete_cluster_role_binding` | `name: String` | Deletes RoleBinding/ClusterRoleBinding | ✅ Working |
| `kuboard_delete_service_account` | `name: String, namespace: String` | Deletes ServiceAccount | ✅ Working |

### **5. Special Operations (Metrics, Grafana, Helm, Graph, Search, Exec, Port Forward)**
| Function Name | Parameters | Description | Status |
|---------------|------------|-------------|--------|
| `kuboard_get_node_metrics` | None | Fetches live Node metrics from `/apis/metrics.k8s.io/v1beta1` | ✅ Working |
| `kuboard_get_node_metrics_history` | `duration: u64` | Fetches Node metrics history timeline | ✅ Working |
| `kuboard_get_pod_metrics` | None | Fetches live Pod metrics | ✅ Working |
| `kuboard_get_pod_metrics_history` | `duration: u64` | Fetches Pod metrics history timeline | ✅ Working |
| `kuboard_check_metrics_availability` | None | Checks if Metrics Server API is reachable | ✅ Working |
| `kuboard_configure_grafana` | `url: String, token: Option<String>` | Configures Grafana endpoint & credentials | 💡 Planned (Phase 4) |
| `kuboard_query_grafana_promql` | `query: String, start: u64, end: u64, step: String` | Executes PromQL range query via Grafana `/api/ds/query` bridge | 💡 Planned (Phase 4) |
| `kuboard_list_helm_releases` | None | Lists installed Helm releases via K8s secrets parsing | 🔄 Partial (Needs robust decoder) |
| `kuboard_get_helm_release_details` | `name: String, namespace: String, revision: i32` | Gets Helm values & manifests | 🔄 Partial |
| `kuboard_get_resource_graph` | None | Builds resource relationship DAG for X-Ray visualization | ✅ Working |
| `kuboard_list_crds` | None | Discovers installed CustomResourceDefinitions | ✅ Working |
| `kuboard_list_custom_resource_instances` | `crd_name: String` | Fetches custom resource instances | ✅ Working |
| `kuboard_run_linter` | None | Evaluates K8s resources against security/best-practice rules | ✅ Working |
| `kuboard_search_resources` | `query: String` | Performs fuzzy search across all cluster resources | ✅ Working |
| `kuboard_get_resource_yaml` | `kind: String, name: String, namespace: String` | Generic YAML fetcher for any resource | ✅ Working |
| `kuboard_apply_resource_yaml` | `yaml: String` | Applies generic YAML manifest to cluster | ✅ Working |
| `kuboard_exec_into_pod` | `pod: String, container: String, command: String` | Launches PTY exec session for pod | 🔄 Partial |
| `kuboard_port_forward` | `pod: String, namespace: String, target_port: u16, local_port: u16` | Establishes port forward tunnel | 🔄 Partial |
| `kuboard_list_port_forwards` / `stop_port_forward` | `id: String` | Manages active port forward tunnels | 🔄 Partial |

---

## ⚠️ Identified Discrepancies & Backend/Frontend Alignment Fixes Required

1. **`ReplicaSetDetails.svelte` Mismatches**:
   - Frontend calls `kuboard_get_replicaset_details` -> Backend actual function name is `kuboard_get_replicaset`.
   - Frontend calls `kuboard_get_pods_by_selector` -> Backend actual function name is `kuboard_get_replicaset_pods`.
2. **Generic Resource YAML Editing**:
   - `ReplicaSetsPanel.svelte` calls `kuboard_update_replicaset` (unregistered). Frontend should be standardized to use `kuboard_apply_resource_yaml`.
3. **Helm Secret Decompression**:
   - `helm.rs` uses double base64 decoding prior to GzDecoder. Certain Helm 3 releases format storage with single base64 or protobuf encoding. Requires fallback decoder logic.

---

## 🎨 **Recent Frontend UI Helper Functions**

| Component | Function / Feature | Description | Status |
|-----------|--------------------|-------------|--------|
| `PodDetails.svelte` | `copyPodName()` | Copies active pod name string to clipboard on click with animated copy confirmation feedback | ✅ Working |
| `MetricsGraph.svelte` | `setDuration(dur)` | Switches Chart.js time-series historical data resolution (`15m`, `30m`, `1h`, `6h`, `24h`) | ✅ Working |
| `TabbedContent.svelte` | `sessionTabMap[tabSessionId]` | Per-session reactive tab map preventing tab cross-talk when multiple tabs run in the same cluster context | ✅ Working |
| `ConfigTab.svelte` | `switchSubTab(id)` | Searchable ConfigMaps table view with dedicated `ConfigMapDetails.svelte` payload editor & Helm Releases sub-tab | ✅ Working |
| `SecurityTab.svelte` | `SecuritySubNav` | Integrated sub-tabs for Secrets, Roles, ClusterRoles, RoleBindings, ClusterRoleBindings, and ServiceAccounts | ✅ Working |
