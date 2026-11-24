# k6 Benchmarks

This directory holds the k6 load-test scripts used to benchmark Stellar-K8s operator behavior under normal, stress, and failure-oriented traffic. The scripts cover API and reconciliation latency, webhook and CRD performance, autoscaling, traffic shaping, upgrade availability, and security-style stress checks.

## Files

- `comprehensive-perf.js` — Measures the full StellarNode lifecycle by creating, waiting for reconciliation, scaling, and deleting nodes while tracking API and reconciliation latency.
- `crd-operations-load-test.js` — Ramps concurrent load over StellarNode CRD create, update, and delete requests and checks latency and error thresholds.
- `gas-autoscaling-test.js` — Sends ramped and sustained Soroban RPC `getTransactions` traffic and tracks RPC errors, estimated gas from resource fees, and epoch transitions for autoscaling checks.
- `operator-load-test.js` — Benchmarks operator health, metrics, REST API, CRD operations, and reconciliation under steady, stress, spike, and arrival-rate scenarios with optional baseline regression checks.
- `security-pen.js` — Stresses operator HTTP endpoints with high-volume and slow request patterns while checking response codes, server version leakage, and response size.
- `traffic-shaping-load-test.js` — Applies baseline and burst traffic with priority headers to the traffic dashboard endpoint and measures priority success, low-priority shedding, latency, and rejections.
- `upgrade-load-test.js` — Polls the operator health endpoint during an upgrade window, counting non-200 responses as downtime events and writing an error-rate summary.
- `webhook-load-test.js` — Benchmarks StellarNode validation and mutation admission webhooks under baseline, stress, spike, and sustained load, producing latency and throughput reports.
