# Ablation: reconciliation hardening

Disabling hardening requires scaling wasmbed-application-controller to 0, which prevents CRD phase repair. Code evidence: crates/wasmbed-application-controller/src/main.rs handle_running() reconciles stale phase to Running.
Hardened campaign results are in summary_metrics.json; unhardened mini-campaign can be run with `kubectl scale deployment/wasmbed-application-controller -n wasmbed --replicas=0`.
