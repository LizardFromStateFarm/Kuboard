# Kuboard UI Testing Utilities & Test Suite

This directory contains Kubernetes YAML manifests designed to populate your Minikube cluster with various workloads, storage components, network rules, RBAC bindings, and metric-generating pods to thoroughly test the Kuboard UI.

---

## 1. Setting up Metrics Server on Minikube

To view CPU/Memory metrics and graphs in Kuboard:

1. **Enable the metrics-server addon:**
   ```bash
   minikube addons enable metrics-server
   ```

2. **Verify metrics-server status:**
   ```bash
   kubectl get pods -n kube-system -l k8s-app=metrics-server
   ```
   *Note: It takes 1–2 minutes after enabling for metrics data to start populating.*

3. **Verify metric collection:**
   ```bash
   kubectl top nodes
   kubectl top pods -A
   ```

---

## 2. Included Manifest Files

| File | Purpose / Resource Types Covered |
| :--- | :--- |
| [log-generator-pod.yaml](file:///D:/google_working/Kuboard/testing_utils/log-generator-pod.yaml) | Basic single-pod continuous log generator |
| [advanced-log-generator.yaml](file:///D:/google_working/Kuboard/testing_utils/advanced-log-generator.yaml) | JSON, multi-line, and formatted logs for testing log viewer filter & search |
| [test-workloads.yaml](file:///D:/google_working/Kuboard/testing_utils/test-workloads.yaml) | `StatefulSet`, `DaemonSet`, `CronJob`, multi-container Pod (with InitContainer), and failing Pods (`CrashLoopBackOff`, `ImagePullBackOff`) |
| [test-services-ingress-config.yaml](file:///D:/google_working/Kuboard/testing_utils/test-services-ingress-config.yaml) | `Service` (ClusterIP & NodePort), `Ingress`, `ConfigMap`, `Secret`, `PersistentVolumeClaim` (PVC), and `NetworkPolicy` |
| [test-rbac.yaml](file:///D:/google_working/Kuboard/testing_utils/test-rbac.yaml) | `ServiceAccount`, `Role`, `RoleBinding`, `ClusterRole`, and `ClusterRoleBinding` |
| [test-metrics-hpa.yaml](file:///D:/google_working/Kuboard/testing_utils/test-metrics-hpa.yaml) | CPU/Memory activity generator and `HorizontalPodAutoscaler` (HPA) |

---

## 3. Deploying the Test Suite

Apply all test manifests at once:
```bash
kubectl apply -f testing_utils/
```

Or apply individual files:
```bash
# Workloads & Error pods
kubectl apply -f testing_utils/test-workloads.yaml

# Networking, Storage & Config
kubectl apply -f testing_utils/test-services-ingress-config.yaml

# RBAC
kubectl apply -f testing_utils/test-rbac.yaml

# Metrics & HPA
kubectl apply -f testing_utils/test-metrics-hpa.yaml

# Log Generators
kubectl apply -f testing_utils/advanced-log-generator.yaml
```

---

## 4. Cleanup Test Resources

To clean up all test resources:
```bash
kubectl delete -f testing_utils/
```
