# Reviewer experimental summary

Testbed: single-node K3s (wasmbed namespace), Renode-emulated Zephyr STM32F746, TAP+DNAT to gateway TLS on 192.168.1.1:30443.

Campaign: 1 warm-up + 50 independent trials measuring enrollment, heartbeat supervision, deployment, and transactional workflow.

## Success rates
## Success rates

| Stage | Successes | Trials | Rate |
| --- | --- | --- | --- |
| enrollment | 50 | 50 | 100.0% |
| heartbeat | 50 | 50 | 100.0% |
| deployment | 50 | 50 | 100.0% |
| transactional_end_to_end | 50 | 50 | 100.0% |


## Latency
## Latency profile

| Stage | Mean | Median | p95 | p99 | CV | 95% CI |
| --- | --- | --- | --- | --- | --- | --- |
| enrollment | 589.0 | 567.3 | 773.5 | 823.5 | 0.138 | [566.0, 612.1] |
| heartbeat | 4.5 | 4.6 | 5.3 | 5.6 | 0.093 | [4.4, 4.7] |
| deployment | 618.8 | 560.7 | 941.5 | 984.2 | 0.238 | [576.8, 660.7] |
| transactional_end_to_end | 1207.8 | 1160.2 | 1516.3 | 1543.4 | 0.116 | [1168.0, 1247.6] |


Reconciliation hardening: active application-controller repairs stale Application CRD phase; ablation via controller scale-down documented separately.

Limitations: single-node K3s, emulated device (no physical hardware), controlled port-forward path.

Commit: dad717c8c04703004c39dbb7d19151453cd526e0
Artifacts: /home/ubuntu/retrospect/experiments/20260609-064519
