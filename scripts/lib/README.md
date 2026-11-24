# `scripts/lib`

Shared helper libraries sourced by repository scripts for error reporting, health verification steps, and version checks[span_1](start_span)[span_1](end_span).

- `errors.sh`: Shared error-handling and failure-reporting helpers for repository scripts.
- `health-steps.sh`: Shared health-check step functions for verifying service and environment readiness.
- `versions.sh`: Defines pinned minimum tool versions and the `version_ge` version comparison helper[span_2](start_span)[span_2](end_span).
# scripts/lib

Shared shell libraries used by the scripts in this repository. This folder holds reusable bash helpers that are sourced by other scripts to provide common error handling, health-check steps, and version utilities, keeping that logic in one place instead of duplicating it across scripts.

## Files

- `errors.sh` - Error-handling helpers (e.g. failure messages and exit handling) shared by scripts.
- `health-steps.sh` - Definitions and helpers for the health-check steps run by the scripts.
- `versions.sh` - Utilities for reading, comparing, and managing version strings.
