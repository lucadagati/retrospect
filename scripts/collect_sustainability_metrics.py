#!/usr/bin/env python3
"""Collect sustainability-oriented metrics for the Computer Communications SI paper.

Measures wire-format sizes, edge footprint, gateway/control-plane resource use,
and control-traffic overhead proxies on the live testbed.
"""

from __future__ import annotations

import argparse
import base64
import json
import re
import statistics
import subprocess
import time
from datetime import datetime, timezone
from pathlib import Path

try:
    import requests
except ImportError as exc:
    raise SystemExit("Missing dependency: requests") from exc


DEFAULT_NAMESPACE = "wasmbed"
DEFAULT_API = "http://127.0.0.1:3001"
DEFAULT_GATEWAY = "http://127.0.0.1:8080"
HEARTBEAT_PERIOD_S = 25


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def run(cmd: list[str], check: bool = False) -> str:
    return subprocess.run(cmd, check=check, text=True, capture_output=True).stdout


def parse_kubectl_top() -> dict[str, dict[str, float]]:
    out = run(["kubectl", "top", "pods", "-n", DEFAULT_NAMESPACE])
    rows: dict[str, dict[str, float]] = {}
    for line in out.splitlines()[1:]:
        parts = line.split()
        if len(parts) < 3:
            continue
        name = parts[0]
        cpu_raw = parts[1]
        mem_raw = parts[2]
        if cpu_raw.endswith("m"):
            cpu_m = float(cpu_raw[:-1])
        elif cpu_raw.endswith("n"):
            cpu_m = float(cpu_raw[:-1]) / 1_000_000
        else:
            cpu_m = float(cpu_raw) * 1000
        if mem_raw.endswith("Mi"):
            mem_mi = float(mem_raw[:-2])
        elif mem_raw.endswith("Ki"):
            mem_mi = float(mem_raw[:-2]) / 1024
        elif mem_raw.endswith("Gi"):
            mem_mi = float(mem_raw[:-2]) * 1024
        else:
            mem_mi = float(mem_raw)
        rows[name] = {"cpu_millicores": cpu_m, "memory_mib": mem_mi}
    return rows


def sample_pod_resources(samples: int, interval_s: float) -> dict[str, dict]:
    series: dict[str, list[dict[str, float]]] = {}
    for _ in range(samples):
        snap = parse_kubectl_top()
        for pod, vals in snap.items():
            series.setdefault(pod, []).append(vals)
        time.sleep(interval_s)

    summary: dict[str, dict] = {}
    for pod, rows in series.items():
        cpus = [r["cpu_millicores"] for r in rows]
        mems = [r["memory_mib"] for r in rows]
        summary[pod] = {
            "samples": len(rows),
            "cpu_millicores_mean": statistics.fmean(cpus),
            "cpu_millicores_max": max(cpus),
            "memory_mib_mean": statistics.fmean(mems),
            "memory_mib_max": max(mems),
        }
    return summary


def _cbor_uint(n: int) -> bytes:
    if n < 24:
        return bytes([n])
    if n < 256:
        return bytes([0x18, n])
    raise ValueError(f"uint too large: {n}")


def _cbor_array(items: list[bytes]) -> bytes:
    head = bytes([0x80 + len(items)])
    return head + b"".join(items)


def _cbor_text(s: str) -> bytes:
    b = s.encode()
    if len(b) < 24:
        return bytes([0x60 + len(b)]) + b
    return bytes([0x78, len(b)]) + b


def _cbor_bytes(b: bytes) -> bytes:
    if len(b) < 24:
        return bytes([0x40 + len(b)]) + b
    if len(b) < 256:
        return bytes([0x58, len(b)]) + b
    raise ValueError("byte string too large")


def _cbor_null() -> bytes:
    return b"\xf6"


def _cbor_bool(v: bool) -> bytes:
    return b"\xf5" if v else b"\xf4"


def _wire(cbor_payload: bytes) -> int:
    return 4 + len(cbor_payload)


def measure_wire_sizes(_repo_root: Path) -> dict:
    """Encode representative protocol messages (4-byte BE length + CBOR array tag)."""
    ed_key = bytes([0xAB] * 32)
    minimal_wasm = bytes([
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
        0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00,
        0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
    ])
    uuid = bytes([0x11] * 16)
    wire = {
        "client_heartbeat": _wire(_cbor_array([_cbor_uint(0)])),
        "client_enrollment_request": _wire(_cbor_array([_cbor_uint(1)])),
        "client_public_key": _wire(_cbor_array([_cbor_uint(2), _cbor_bytes(ed_key)])),
        "client_enrollment_ack": _wire(_cbor_array([_cbor_uint(3)])),
        "client_deploy_ack": _wire(_cbor_array([
            _cbor_uint(5), _cbor_text("experiment-app"), _cbor_bool(True), _cbor_null(),
        ])),
        "server_heartbeat_ack": _wire(_cbor_array([_cbor_uint(0)])),
        "server_enrollment_accepted": _wire(_cbor_array([_cbor_uint(1)])),
        "server_device_uuid": _wire(_cbor_array([_cbor_uint(3), _cbor_bytes(uuid)])),
        "server_enrollment_completed": _wire(_cbor_array([_cbor_uint(4)])),
        "server_deploy_minimal_wasm": _wire(_cbor_array([
            _cbor_uint(5),
            _cbor_text("experiment-app"),
            _cbor_text("benchmark"),
            _cbor_bytes(minimal_wasm),
            _cbor_null(),
        ])),
    }
    return {"wire_sizes_bytes": wire, "minimal_wasm_cbor_payload_bytes": len(minimal_wasm)}


def _elf_loadable_footprint(data: bytes) -> dict | None:
    """Compute on-device flash/RAM footprint from ELF section headers.

    The ELF file on disk is much larger than the flashed image because it keeps
    debug symbols and section metadata. This replicates ``size`` semantics on the
    allocatable sections: ``text`` (read-only code and rodata), ``data``
    (initialized writable) and ``bss`` (uninitialized writable). The on-device
    footprint is then flash = text + data and RAM = data + bss.
    """
    import struct

    if len(data) < 52 or data[:4] != b"\x7fELF":
        return None
    is_64 = data[4] == 2
    endian = "<" if data[5] == 1 else ">"

    if is_64:
        e_shoff = struct.unpack_from(endian + "Q", data, 40)[0]
        e_shentsize = struct.unpack_from(endian + "H", data, 58)[0]
        e_shnum = struct.unpack_from(endian + "H", data, 60)[0]
    else:
        e_shoff = struct.unpack_from(endian + "I", data, 32)[0]
        e_shentsize = struct.unpack_from(endian + "H", data, 46)[0]
        e_shnum = struct.unpack_from(endian + "H", data, 48)[0]

    SHT_NOBITS, SHF_WRITE, SHF_ALLOC = 8, 0x1, 0x2
    text = data_sz = bss = 0
    for i in range(e_shnum):
        off = e_shoff + i * e_shentsize
        if is_64:
            sh_type = struct.unpack_from(endian + "I", data, off + 4)[0]
            sh_flags = struct.unpack_from(endian + "Q", data, off + 8)[0]
            sh_size = struct.unpack_from(endian + "Q", data, off + 32)[0]
        else:
            sh_type = struct.unpack_from(endian + "I", data, off + 4)[0]
            sh_flags = struct.unpack_from(endian + "I", data, off + 8)[0]
            sh_size = struct.unpack_from(endian + "I", data, off + 20)[0]
        if not (sh_flags & SHF_ALLOC):
            continue
        if sh_flags & SHF_WRITE:
            if sh_type == SHT_NOBITS:
                bss += sh_size
            else:
                data_sz += sh_size
        else:
            text += sh_size
    return {
        "text_bytes": text,
        "data_bytes": data_sz,
        "bss_bytes": bss,
        "flash_bytes": text + data_sz,
        "ram_bytes": data_sz + bss,
    }


def firmware_footprint() -> dict:
    listing = run(
        [
            "docker",
            "run",
            "--rm",
            "-v",
            "wasmbed-firmware-store:/firmware",
            "alpine",
            "sh",
            "-c",
            "find /firmware -name zephyr.elf -exec ls -la {} \\;",
        ]
    )
    files = []
    for line in listing.splitlines():
        parts = line.split()
        if len(parts) >= 9 and parts[-1].endswith("zephyr.elf"):
            path = parts[-1]
            entry = {"path": path, "elf_file_bytes": int(parts[4])}
            b64 = run(
                [
                    "docker",
                    "run",
                    "--rm",
                    "-v",
                    "wasmbed-firmware-store:/firmware",
                    "alpine",
                    "sh",
                    "-c",
                    f"base64 {path}",
                ]
            )
            try:
                elf = base64.b64decode(b64)
                fp = _elf_loadable_footprint(elf)
                if fp:
                    entry.update(fp)
            except Exception:
                pass
            files.append(entry)
    return {"firmware_images": files}


def minimal_wasm_bytes() -> int:
    wasm = bytes([
        0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
        0x03, 0x02, 0x01, 0x00, 0x07, 0x07, 0x01, 0x03, 0x72, 0x75, 0x6e, 0x00, 0x00,
        0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
    ])
    return len(wasm)


def control_traffic_model(wire: dict[str, int], heartbeat_period_s: int = HEARTBEAT_PERIOD_S) -> dict:
  hb = wire.get("client_heartbeat", 6) + wire.get("server_heartbeat_ack", 6)
  enroll_roundtrip = sum(
      wire.get(k, 0)
      for k in (
          "client_enrollment_request",
          "server_enrollment_accepted",
          "client_public_key",
          "server_device_uuid",
          "client_enrollment_ack",
          "server_enrollment_completed",
      )
  )
  deploy_roundtrip = wire.get("server_deploy_minimal_wasm", 0) + wire.get("client_deploy_ack", 0)
  per_hour_hb = int(3600 / heartbeat_period_s)
  return {
      "heartbeat_period_s": heartbeat_period_s,
      "heartbeat_roundtrip_bytes": hb,
      "heartbeat_bytes_per_hour_per_device": hb * per_hour_hb,
      "enrollment_roundtrip_bytes_once": enroll_roundtrip,
      "deploy_roundtrip_bytes_once": deploy_roundtrip,
      "steady_state_control_bytes_per_hour_per_device": hb * per_hour_hb,
  }


def gateway_board_scaling(gateway_http: str, counts: list[int]) -> list[dict]:
    """Register synthetic boards and sample gateway memory (HTTP registry only)."""
    results = []
    for n in counts:
        for i in range(n):
            device_id = f"scale-synth-{n}-{i}"
            try:
                requests.delete(f"{gateway_http}/api/v1/board/{device_id}", timeout=5)
            except Exception:
                pass
            requests.post(
                f"{gateway_http}/api/v1/board/register",
                json={
                    "device_id": device_id,
                    "endpoint": f"10.0.0.{(i % 250) + 1}:8443",
                    "mcu_type": "Stm32F746gDisco",
                    "capabilities": {"has_ethernet": True, "has_network": True},
                },
                timeout=5,
            )
        time.sleep(1)
        top = parse_kubectl_top()
        gw = next((v for k, v in top.items() if k.startswith("gateway-")), {})
        board_list = requests.get(f"{gateway_http}/api/v1/board", timeout=5).json()
        results.append({
            "synthetic_boards_registered": n,
            "gateway_cpu_millicores": gw.get("cpu_millicores"),
            "gateway_memory_mib": gw.get("memory_mib"),
            "boards_visible": len(board_list) if isinstance(board_list, list) else None,
        })
        for i in range(n):
            device_id = f"scale-synth-{n}-{i}"
            try:
                requests.delete(f"{gateway_http}/api/v1/board/{device_id}", timeout=5)
            except Exception:
                pass
    return results


def k8s_node_overhead_proxy() -> dict:
    nodes = run(["kubectl", "get", "nodes", "-o", "json"])
    node_data = json.loads(nodes)
    items = node_data.get("items", [])
    alloc = items[0].get("status", {}).get("allocatable", {}) if items else {}
    return {
        "node_count": len(items),
        "allocatable_cpu": alloc.get("cpu"),
        "allocatable_memory": alloc.get("memory"),
        "note": "Full Kubernetes worker agents (kubelet + runtime) are not deployed on embedded endpoints; this node capacity is control-plane overhead for comparison only.",
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Collect sustainability metrics")
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--output-dir", default="experiments")
    parser.add_argument("--gateway-http", default=DEFAULT_GATEWAY)
    parser.add_argument("--resource-samples", type=int, default=5)
    parser.add_argument("--resource-interval", type=float, default=2.0)
    parser.add_argument("--scale-boards", default="1,10,25,50")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    out_dir = Path(args.output_dir) / f"sustainability_{datetime.now().strftime('%Y%m%d-%H%M%S')}"
    out_dir.mkdir(parents=True, exist_ok=True)

    wire_payload = measure_wire_sizes(repo_root)
    wire = wire_payload["wire_sizes_bytes"]
    traffic = control_traffic_model(wire)
    footprint = firmware_footprint()
    wasm_bytes = minimal_wasm_bytes()

    payload = {
        "generated_at": now_iso(),
        "wire_sizes_bytes": wire,
        "minimal_wasm_module_bytes": wasm_bytes,
        "control_traffic_model": traffic,
        "edge_footprint": footprint,
        "gateway_pod_resources_idle": sample_pod_resources(args.resource_samples, args.resource_interval),
        "gateway_board_scaling": gateway_board_scaling(
            args.gateway_http,
            [int(x) for x in args.scale_boards.split(",") if x.strip()],
        ),
        "k8s_node_proxy": k8s_node_overhead_proxy(),
        "design_comparison": {
            "edge_runs_kubelet": False,
            "edge_runs_container_runtime": False,
            "edge_agent_model": "Zephyr firmware + WAMR + TLS/CBOR client only",
            "orchestration_mediated_by": "gateway fog component",
        },
    }

    out_path = out_dir / "sustainability_metrics.json"
    out_path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    md = out_dir / "sustainability_summary.md"
    fw_flash = max((f.get("flash_bytes", 0) for f in footprint["firmware_images"]), default=0)
    fw_ram = max((f.get("ram_bytes", 0) for f in footprint["firmware_images"]), default=0)
    fw_elf = max((f.get("elf_file_bytes", 0) for f in footprint["firmware_images"]), default=0)
    gw_idle = payload["gateway_pod_resources_idle"]
    gw_key = next((k for k in gw_idle if k.startswith("gateway-")), None)
    gw_cpu = gw_idle[gw_key]["cpu_millicores_mean"] if gw_key else 0
    gw_mem = gw_idle[gw_key]["memory_mib_mean"] if gw_key else 0
    md.write_text(
        "\n".join([
            "# Sustainability metrics summary",
            "",
            f"- Firmware on-device footprint: **{fw_flash/1024:.0f} KiB flash** ({fw_flash} B), **{fw_ram/1024:.0f} KiB RAM** ({fw_ram} B)",
            f"- Firmware ELF file (with debug symbols, not flashed): **{fw_elf/1e6:.2f} MB**",
            f"- Minimal WASM module: **{wasm_bytes} B**",
            f"- Heartbeat round-trip wire size: **{traffic['heartbeat_roundtrip_bytes']} B**",
            f"- Steady-state control traffic: **{traffic['heartbeat_bytes_per_hour_per_device']} B/h/device** (~{traffic['heartbeat_bytes_per_hour_per_device']/1024:.1f} KiB/h)",
            f"- Enrollment round-trip (once): **{traffic['enrollment_roundtrip_bytes_once']} B**",
            f"- Deploy round-trip (minimal WASM): **{traffic['deploy_roundtrip_bytes_once']} B**",
            f"- Gateway idle mean CPU: **{gw_cpu:.0f} m**; RAM: **{gw_mem:.1f} MiB**",
            "",
            "## Wire sizes (4-byte length prefix + CBOR)",
            "",
            "| Message | Bytes |",
            "|---------|------:|",
            *[f"| {k} | {v} |" for k, v in sorted(wire.items())],
            "",
            "## Gateway memory vs synthetic board registrations",
            "",
            "| Boards | Gateway RAM (MiB) | CPU (m) |",
            "|--------|------------------:|--------:|",
            *[
                f"| {r['synthetic_boards_registered']} | {r.get('gateway_memory_mib', '-')} | {r.get('gateway_cpu_millicores', '-')} |"
                for r in payload["gateway_board_scaling"]
            ],
        ]),
        encoding="utf-8",
    )
    print(out_path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
