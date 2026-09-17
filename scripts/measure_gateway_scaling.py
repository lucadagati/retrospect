#!/usr/bin/env python3
"""Measure gateway CPU/RAM as emulated TLS devices are added."""

from __future__ import annotations

import json
import statistics
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

import requests

API = "http://127.0.0.1:3001"
GATEWAY = "http://127.0.0.1:8080"
NS = "wasmbed"
PREFIX = "scale-tls"


def run(cmd: list[str]) -> str:
    return subprocess.run(cmd, text=True, capture_output=True).stdout


def kubectl_top_gateway() -> dict:
    out = run(["kubectl", "top", "pods", "-n", NS])
    for line in out.splitlines()[1:]:
        parts = line.split()
        if parts and parts[0].startswith("gateway-"):
            cpu = float(parts[1].rstrip("m"))
            mem = float(parts[2].rstrip("Mi"))
            return {"cpu_millicores": cpu, "memory_mib": mem}
    return {}


def gateway_connected_count() -> int:
    r = requests.get(f"{GATEWAY}/api/v1/devices", timeout=10)
    r.raise_for_status()
    devices = r.json().get("devices") or []
    return sum(1 for d in devices if d.get("connected"))


def create_and_start(n: int) -> list[str]:
  ids = []
  for i in range(1, n + 1):
    name = f"{PREFIX}-{i}"
    ids.append(name)
    requests.post(
        f"{API}/api/v1/devices",
        json={
            "name": name,
            "count": 1,
            "mcuType": "Stm32F746gDisco",
            "gatewayId": "gateway-1",
            "type": "MCU",
        },
        timeout=60,
    )
    time.sleep(1)
    requests.post(f"{API}/api/v1/devices/{name}/renode/start", json={}, timeout=120)
    time.sleep(5)
  return ids


def cleanup(ids: list[str]) -> None:
    for name in ids:
        try:
            requests.post(f"{API}/api/v1/devices/{name}/renode/stop", json={}, timeout=30)
        except Exception:
            pass
        try:
            requests.delete(f"{API}/api/v1/devices/{name}", timeout=30)
        except Exception:
            pass
        run(["kubectl", "delete", "device", name, "-n", NS, "--ignore-not-found=true"])


def main() -> int:
    run(["bash", "-c", "sudo /home/ubuntu/retrospect/scripts/setup-renode-net.sh"], )
    baseline = kubectl_top_gateway()
    baseline_conn = gateway_connected_count()

    results = [{
        "devices_target": 0,
        "devices_connected": baseline_conn,
        "gateway": baseline,
    }]

    all_ids: list[str] = []
    try:
        for n in [1, 2, 3]:
            new_id = f"{PREFIX}-{n}"
            if new_id not in all_ids:
                requests.post(
                    f"{API}/api/v1/devices",
                    json={"name": new_id, "count": 1, "mcuType": "Stm32F746gDisco", "gatewayId": "gateway-1"},
                    timeout=60,
                )
                time.sleep(1)
                requests.post(f"{API}/api/v1/devices/{new_id}/renode/start", json={}, timeout=120)
                all_ids.append(new_id)
                time.sleep(15)
            samples = [kubectl_top_gateway() for _ in range(5)]
            cpus = [s["cpu_millicores"] for s in samples if s]
            mems = [s["memory_mib"] for s in samples if s]
            results.append({
                "devices_target": n,
                "devices_connected": gateway_connected_count(),
                "gateway_cpu_millicores_mean": statistics.fmean(cpus) if cpus else None,
                "gateway_memory_mib_mean": statistics.fmean(mems) if mems else None,
                "device_ids": list(all_ids),
            })
    finally:
        cleanup(all_ids)
        # restore native-sim-1
        requests.post(f"{API}/api/v1/devices/native-sim-1/renode/start", json={}, timeout=120)

    out = Path("experiments") / f"gateway_scaling_{datetime.now().strftime('%Y%m%d-%H%M%S')}"
    out.mkdir(parents=True, exist_ok=True)
    path = out / "gateway_tls_scaling.json"
    path.write_text(json.dumps({
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "baseline": results[0],
        "scaling": results[1:],
    }, indent=2))
    print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
