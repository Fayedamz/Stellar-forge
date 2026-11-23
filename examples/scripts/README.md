# Scripts

This folder contains operational shell and Python utilities for protocol upgrades, storage benchmarking, and key-ceremony workflows.

- [`arm-upgrade.sh`](./arm-upgrade.sh) arms, clears, or inspects a stellar-core protocol upgrade on a validator pod through `kubectl exec`.
- [`run-fio-benchmark.sh`](./run-fio-benchmark.sh) benchmarks a mounted filesystem with fio workloads that model Stellar Core I/O and compares the results with validator and archive targets.
- [`shamir-split.py`](./shamir-split.py) generates, derives, splits, combines, verifies, and self-tests k-of-n Shamir shares for Stellar validator seeds.
