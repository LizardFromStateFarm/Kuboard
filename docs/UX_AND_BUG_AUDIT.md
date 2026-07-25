# 🛠️ Kuboard Comprehensive UX & Bug Audit Report

**Date**: July 24, 2026  
**Auditor**: Antigravity AI Pair Programmer  
**Target Application**: Kuboard Kubernetes Desktop Application (Rust backend + Svelte 5 frontend)

---

## 📐 **Executive Summary & UX Vision**

Kuboard is designed to be an enterprise-grade Kubernetes dashboard competing directly with Lens and k9s. High-performance desktop monitoring requires zero friction:
1. **Context Isolation**: Switching cluster contexts (profiles/tabs) must instantly clear all active cluster state, open log streams, terminals, and editor overlays to prevent cross-cluster data pollution.
2. **Predictable Stacking Layers**: Floating slide-outs (Logs, Exec Terminals, Port Forwards) must cleanly layer over list and detail views without requiring users to navigate backwards to see triggered windows.
3. **Resilient Fallbacks**: Network or API errors (e.g. 404 GVK lookup failures) must gracefully degrade with informative user-facing alerts instead of breaking UI modals.

---

## 🔍 **Detailed Audit Findings & UX Friction Scorecard**

### 🚨 **Issue 1: Pod Details Logs Window Obscured / Hidden**
- **Symptom**: Clicking "📋 Logs" inside `PodDetails.svelte` does not display the `LogsWindow` until the user backs out to the main `PodsPanel` resource list.
- **Root Cause**:
  - `LogsWindow.svelte` possessed `z-index: 1000;`.
  - `PodDetails.svelte` and modal overlays possessed `z-index: 2100;` and separate stacking contexts.
  - Because `LogsWindow` was mounted at `z-index: 1000`, the active `PodDetails` sheet completely covered and obscured the log panel.
- **UX Impact**: Frustrating experience where users assume clicking "📋 Logs" failed, only to find the log panel open after closing the detail view.
- **Resolution**: Elevated `LogsWindow` to `z-index: 9000;` and ensured global viewport positioning.

---

### 🚨 **Issue 2: Stale Log Window & Overlays Persist Across Cluster Profile Swaps**
- **Symptom**: When switching cluster contexts (profiles/tabs in `Header.svelte`), active log tabs in `LogsWindow.svelte` remain open displaying logs from the previous cluster context.
- **Root Cause**:
  - `LogsWindow.svelte` did not receive or reactively watch `currentContext`.
  - Log tailing intervals (`refreshInterval`) and `tabs` array in `LogsWindow.svelte` were not cleared upon context change.
  - Global stores (`editorStore`, `xrayStore`) lacked context change listeners.
- **UX Impact**: Dangerous cross-cluster confusion where users analyze logs from Cluster A while believing they are connected to Cluster B.
- **Resolution**:
  - Bound `currentContext` prop to `LogsWindow` in `PodsPanel.svelte`.
  - Added reactive context watcher in `LogsWindow.svelte` that stops active log polling, closes open log tabs, and resets `isOpen = false` on context switch.
  - Added context switch reset hooks in `+page.svelte` for `editorStore` and `xrayStore`.

---

### 💡 **Issue 3: Detail View State Leak on Context & Namespace Change**
- **Symptom**: Swapping cluster contexts or namespaces while inspecting a resource detail view (`PodDetails`, `DeploymentDetails`, etc.) leaves `showFullDetails` set to `true`, displaying stale or empty data for a resource that does not exist in the new context.
- **Root Cause**:
  - Resource panel components (`PodsPanel`, `DeploymentsPanel`, `ReplicaSetsPanel`, `StatefulSetsPanel`, `DaemonSetsPanel`, `CronJobsPanel`, `ServicesPanel`) stored `showFullDetails` as local state without resetting it when `currentContext` changed.
- **UX Impact**: Broken blank screens when switching tabs with a detail view open.
- **Resolution**: Added reactive `$: if (currentContext) { showFullDetails = false; selectedItem = null; }` resets across all 7 workload panel components.

---

### 💡 **Issue 5: Sticky Viewport Bottom Logs Window**
- **UX Requirement**: The `LogsWindow.svelte` panel must be pinned to the bottom of the viewport (`position: fixed; bottom: 0; left: 0; right: 0;`) so it remains constantly visible and sticky as the user scrolls up or down through resource lists and detail views.

---

### 💡 **Issue 6: Multi-Cluster Profile Session Isolation & State Persistence**
- **UX Requirement**: When swapping cluster profile tabs (`Header.svelte`), active log windows and overlay panels from Cluster Tab A should **not** leak into or float over Cluster Tab B.
- **Persistence Goal**: When the user navigates back to Cluster Tab A, its previously open log window, active log tabs, and scroll positions must be preserved and remain open for that specific profile session.

---

## 🎯 **Audit Summary Table**

| # | Component | Bug / Feature Description | Severity | Status |
|---|---|---|---|---|
| 1 | `LogsWindow.svelte` | Obscured by `PodDetails` (z-index 1000 < 2100) | High | 🟢 Fixed |
| 2 | `LogsWindow.svelte` | Logs persist across cluster context profile swaps | High | 🟢 Fixed |
| 3 | Workload Panels | Detail view state (`showFullDetails`) leaks on context switch | Medium | 🟢 Fixed |
| 4 | `QuickActionsMenu.svelte` | Missing `dispatch` & state variable declarations | High | 🟢 Fixed |
| 5 | `yaml.rs` | Core `v1` GVK group mapped to `"v1"` causing 404s | High | 🟢 Fixed |
| 6 | `LogsWindow.svelte` | Viewport-sticky bottom positioning during page scroll | Medium | 🟢 Fixed |
| 7 | Global Session | Per-profile tab log window state isolation & persistence | High | 🟢 Fixed |
