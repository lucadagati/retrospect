# Reproduce experiment 20260609-064519
1. ./scripts/ensure-experiment-runtime.sh
2. sudo ./scripts/setup-renode-net.sh
3. curl -X POST http://127.0.0.1:3001/api/v1/devices/native-sim-1/renode/start -d '{}'
4. python3 scripts/collect_experiment_metrics.py --trials 50
Commit: dad717c8c04703004c39dbb7d19151453cd526e0
