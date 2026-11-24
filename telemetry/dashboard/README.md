# Telemetry Dashboard

This folder contains the web frontend for the Stellar-K8s live traffic-routing heatmap dashboard (`@stellar-k8s/heatmap-dashboard`). Built with Vite and WebGL2, it connects to the telemetry WebSocket backend to visualize real-time Soroban RPC pod traffic load and also includes a placeholder Three.js mempool visualizer.

## Files and Folders

- `index.html` — HTML entry page defining the dashboard layout, the `#heatmap-canvas` element with its default `data-ws-url`, and the module script loader.
- `package.json` — Package manifest configuring the Vite `dev`, `build`, and `preview` scripts for `@stellar-k8s/heatmap-dashboard`.
- `src/` — JavaScript source files for bootstrapping the dashboard and rendering WebGL visualizations.
  - `src/index.js` — Main entry point that initializes the `TrafficHeatmap` renderer, starts the telemetry Web Worker, and updates the legend and connection status badge.
  - `src/webgl/` — WebGL rendering and WebSocket worker modules for the dashboard visualizations.
    - `src/webgl/heatmap.js` — WebGL2 `TrafficHeatmap` renderer using custom GLSL shaders to render Soroban RPC pods as pulsating, load-colored particle nodes.
    - `src/webgl/heatmap.worker.js` — Web Worker that manages the telemetry WebSocket connection, reconnect backoff, snapshot validation, and 60fps rate-limited forwarding to the main thread.
    - `src/webgl/mempool.js` — Placeholder Three.js WebGL visualizer that renders instanced meshes representing mempool transactions.
