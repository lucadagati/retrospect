#!/usr/bin/env python3
"""
RETROSPECT — Measurement Harness (Fase 1 piano esperimenti)
Esegue i 5 esperimenti di metriche per il Capitolo 4 della tesi.

Prerequisiti:
  - K8s cluster up con gateway-1 running
  - kubectl port-forward svc/wasmbed-gateway 8080:8080 (HTTP) già attivo
  - gateway-1-service ClusterIP 10.43.222.9:8443 (TLS) raggiungibile
  - zephyr.exe compilato con --device-key support

Usage:
  python3 measure_experiments.py [--exp all|scalability|decomp|throughput|size|reliability]
  python3 measure_experiments.py --quick  # 1 device, smoke test
"""

import argparse
import base64
import csv
import json
import math
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
import threading
from pathlib import Path

# ─── CONFIG ────────────────────────────────────────────────────────────────────
GATEWAY_TLS    = "10.43.222.9:8443"   # ClusterIP TLS port (stable across pod restarts)
GATEWAY_HTTP   = "http://127.0.0.1:8080"  # kubectl port-forward to gateway HTTP
K8S_NAMESPACE  = "wasmbed"
ZEPHYR_EXE     = "/home/ubuntu/Thesis/retrospect/ocre-workspace/build/zephyr/zephyr.exe"
WASM_FILE      = "/home/ubuntu/Thesis/retrospect/ocre-workspace/ocre-runtime/src/samples/mini/hello-world.wasm"
RESULTS_DIR    = "/home/ubuntu/Thesis/retrospect/results"
SCRATCHPAD     = "/tmp/claude-1000/-home-ubuntu-Thesis/bc02d793-c98b-49ac-8ecd-23d9e983e9a4/scratchpad"

# Timeouts
ENROLL_TIMEOUT_S   = 60    # max seconds for device to reach "Connected"
DEPLOY_TIMEOUT_S   = 120   # max seconds for app to reach "Running" for all devices
POLL_INTERVAL_S    = 1.0   # CRD poll interval


# ─── HELPERS ──────────────────────────────────────────────────────────────────

def ensure_dirs():
    os.makedirs(RESULTS_DIR, exist_ok=True)
    os.makedirs(SCRATCHPAD, exist_ok=True)


def hex_key(index: int) -> str:
    """Generate a unique 32-byte key as 64 hex chars from index.
    Uses index spread across bytes 0-3 and bytes 16-19 for uniqueness.
    """
    # Spread the index across multiple byte positions to avoid trivial patterns
    b = bytearray(32)
    b[0] = (index >> 24) & 0xFF
    b[1] = (index >> 16) & 0xFF
    b[2] = (index >> 8) & 0xFF
    b[3] = index & 0xFF
    b[16] = ~b[0] & 0xFF
    b[17] = ~b[1] & 0xFF
    b[18] = ~b[2] & 0xFF
    b[19] = ~b[3] & 0xFF
    b[8] = 0xDE
    b[9] = 0xAD
    b[10] = 0xBE
    b[11] = 0xEF
    return b.hex()


def key_to_urlsafe_b64_no_pad(hex64: str) -> str:
    """Convert 64 hex chars to URL-safe base64 WITHOUT padding.
    This matches base64::engine::general_purpose::URL_SAFE_NO_PAD used by the gateway.
    """
    raw = bytes.fromhex(hex64)
    return base64.urlsafe_b64encode(raw).rstrip(b'=').decode()


def device_name_from_index(index: int) -> str:
    """Generate a device CRD name in 'device-<32hexchars>' format (valid UUID format)."""
    # Use index as a UUID — encode in little-endian at the end
    # Must be parseable by uuid::Uuid::parse_str() in Rust
    uuid_hex = f"{index:032x}"
    return f"device-{uuid_hex}"


def make_flash_path(index: int, flash_dir: str) -> str:
    """Return a unique flash file path for this device instance.
    The file does NOT need to exist; native_sim will create + format it.
    """
    return os.path.join(flash_dir, f"flash_{index:06d}.bin")


def get_wasm_bytes() -> bytes:
    """Load WASM bytes from the bundled hello-world.wasm file."""
    with open(WASM_FILE, "rb") as f:
        return f.read()


def get_wasm_bytes_b64() -> str:
    """Return hello-world.wasm encoded as standard base64 (for Application CRD spec)."""
    data = get_wasm_bytes()
    return base64.b64encode(data).decode()


# ─── KUBERNETES OPS ───────────────────────────────────────────────────────────

def enable_pairing_mode(enabled: bool = True):
    """Enable or disable gateway pairing mode via HTTP API."""
    import urllib.request
    data = json.dumps({"enabled": enabled}).encode()
    req = urllib.request.Request(
        f"{GATEWAY_HTTP}/api/v1/admin/pairing-mode",
        data=data,
        headers={"Content-Type": "application/json"},
        method="POST"
    )
    with urllib.request.urlopen(req, timeout=10) as r:
        resp = json.loads(r.read())
    return resp


def create_device_crd(name: str, pub_key_b64: str) -> bool:
    """Create a Device CRD in Kubernetes.
    name must be 'device-<32hexchars>' format for gateway reconnect to work.
    pub_key_b64 must be URL-safe base64 without padding (URL_SAFE_NO_PAD).
    """
    manifest = {
        "apiVersion": "wasmbed.github.io/v0",
        "kind": "Device",
        "metadata": {"name": name, "namespace": K8S_NAMESPACE},
        "spec": {
            "mcuType": "native_sim",
            "publicKey": pub_key_b64,
        }
    }
    try:
        proc = subprocess.run(
            ["kubectl", "apply", "-f", "-"],
            input=json.dumps(manifest),
            capture_output=True, text=True
        )
        return proc.returncode == 0
    except Exception as e:
        print(f"[ERROR] create_device_crd {name}: {e}")
        return False


def delete_device_crd(name: str):
    subprocess.run(
        ["kubectl", "delete", "device", name, "-n", K8S_NAMESPACE,
         "--ignore-not-found=true"],
        capture_output=True
    )


def get_device_phase(name: str) -> str:
    try:
        r = subprocess.run(
            ["kubectl", "get", "device", name, "-n", K8S_NAMESPACE,
             "-o", "jsonpath={.status.phase}"],
            capture_output=True, text=True
        )
        return r.stdout.strip() or "Unknown"
    except Exception:
        return "Unknown"


def create_application_crd(name: str, device_names: list, wasm_b64: str) -> bool:
    """Create an Application CRD targeting the given devices.
    Uses server-side apply (--server-side) to avoid the 262144-byte annotation limit
    that standard kubectl apply would hit for large WASM modules (>200KB).
    """
    manifest = {
        "apiVersion": "wasmbed.github.io/v1alpha1",
        "kind": "Application",
        "metadata": {"name": name, "namespace": K8S_NAMESPACE},
        "spec": {
            "name": name,
            "targetDevices": {"deviceNames": device_names},
            "targetRuntime": "OcreZephyr",
            "wasmBytes": wasm_b64
        }
    }
    try:
        proc = subprocess.run(
            ["kubectl", "apply", "--server-side", "--field-manager=measure-harness",
             "--force-conflicts", "-f", "-"],
            input=json.dumps(manifest),
            capture_output=True, text=True
        )
        if proc.returncode != 0:
            # Fall back to kubectl create if server-side apply fails
            proc2 = subprocess.run(
                ["kubectl", "create", "-f", "-"],
                input=json.dumps(manifest),
                capture_output=True, text=True
            )
            if proc2.returncode != 0:
                print(f"[ERROR] create_application_crd {name}: {proc.stderr[:200]}")
            return proc2.returncode == 0
        return True
    except Exception as e:
        print(f"[ERROR] create_application_crd {name}: {e}")
        return False


def delete_application_crd(name: str):
    subprocess.run(
        ["kubectl", "delete", "application", name, "-n", K8S_NAMESPACE,
         "--ignore-not-found=true"],
        capture_output=True
    )


def get_application_phase(name: str) -> str:
    try:
        r = subprocess.run(
            ["kubectl", "get", "application", name, "-n", K8S_NAMESPACE,
             "-o", "jsonpath={.status.phase}"],
            capture_output=True, text=True
        )
        return r.stdout.strip() or "Unknown"
    except Exception:
        return "Unknown"


def get_application_device_statuses(name: str) -> dict:
    try:
        r = subprocess.run(
            ["kubectl", "get", "application", name, "-n", K8S_NAMESPACE,
             "-o", "jsonpath={.status.deviceStatuses}"],
            capture_output=True, text=True
        )
        raw = r.stdout.strip()
        if raw:
            return json.loads(raw)
        return {}
    except Exception:
        return {}


# ─── PROCESS MANAGEMENT ───────────────────────────────────────────────────────

def launch_zephyr(index: int, hex_key_str: str, flash_path: str,
                   log_path: str) -> subprocess.Popen:
    """Launch a native_sim zephyr.exe instance.
    NOTE: do NOT use --no-rt — it advances simulated time faster than real time,
    causing the 10-second TLS connect timeout to expire before real network I/O completes.
    We run in default real-time mode (simulated time ≈ wall clock).
    """
    cmd = [
        ZEPHYR_EXE,
        f"--gateway={GATEWAY_TLS}",
        f"--device-key={hex_key_str}",
        f"--flash={flash_path}",
    ]
    logf = open(log_path, "w")
    proc = subprocess.Popen(cmd, stdout=logf, stderr=logf)
    proc._logf = logf  # keep reference to close later
    return proc


def kill_proc(proc):
    try:
        proc.kill()
        proc.wait(timeout=5)
    except Exception:
        pass
    try:
        if hasattr(proc, '_logf'):
            proc._logf.close()
    except Exception:
        pass


# ─── WAIT HELPERS ─────────────────────────────────────────────────────────────

def wait_all_connected(device_names: list, timeout_s: float) -> tuple:
    """
    Wait until all devices reach 'Connected' phase (or timeout).
    Returns (all_connected: bool, times: dict[name -> elapsed_s], errors: list).
    Devices must already exist as CRDs before calling this.
    """
    t0 = time.time()
    connected_at = {}
    pending = set(device_names)
    errors = []

    while pending and (time.time() - t0) < timeout_s:
        for name in list(pending):
            phase = get_device_phase(name)
            if phase == "Connected":
                connected_at[name] = time.time() - t0
                pending.discard(name)
            elif phase in ("Failed", "Unreachable"):
                errors.append(f"{name}: phase={phase}")
                pending.discard(name)
        if pending:
            time.sleep(POLL_INTERVAL_S)

    if pending:
        for name in pending:
            errors.append(f"{name}: timeout (phase={get_device_phase(name)})")

    all_ok = len(connected_at) == len(device_names)
    return all_ok, connected_at, errors


def wait_app_running(app_name: str, device_names: list, timeout_s: float) -> tuple:
    """
    Wait until Application deviceStatuses shows Running for all devices (or timeout).
    Returns (all_running: bool, elapsed_s: float, device_statuses: dict, errors: list).
    """
    t0 = time.time()
    while (time.time() - t0) < timeout_s:
        phase = get_application_phase(app_name)
        statuses = get_application_device_statuses(app_name)
        all_running = all(
            statuses.get(d, {}).get("status") == "Running"
            for d in device_names
        )
        if all_running or phase in ("Failed",):
            elapsed = time.time() - t0
            errors = [
                f"{d}: {statuses.get(d,{}).get('status','?')}"
                for d in device_names
                if statuses.get(d, {}).get("status") != "Running"
            ]
            return all_running, elapsed, statuses, errors
        time.sleep(POLL_INTERVAL_S)

    elapsed = time.time() - t0
    statuses = get_application_device_statuses(app_name)
    errors = [
        f"{d}: timeout (status={statuses.get(d,{}).get('status','?')})"
        for d in device_names
        if statuses.get(d, {}).get("status") != "Running"
    ]
    return False, elapsed, statuses, errors


# ─── LOG PARSING ──────────────────────────────────────────────────────────────

def parse_device_timings(log_path: str) -> dict:
    """Parse the LAST set of TIMING entries from firmware log file."""
    result = {}
    try:
        if not os.path.exists(log_path):
            return result
        with open(log_path, errors="replace") as f:
            content = f.read()
        for key in ["transfer_ms", "lfs_write_ms", "wamr_load_ms", "wamr_start_ms"]:
            pattern = rf"TIMING {key}=(\d+)"
            matches = re.findall(pattern, content)
            if matches:
                result[key] = int(matches[-1])
    except Exception as e:
        print(f"  [WARN] parse_device_timings: {e}")
    return result


def parse_device_timings_since(log_path: str, byte_offset: int,
                                wait_timeout: float = 60.0) -> dict:
    """Parse TIMING entries (µs precision) from log_path starting at byte_offset.

    Firmware now logs `_us` fields (CLOCK_MONOTONIC via clock_gettime on native_sim).
    Returns a dict with keys `transfer_ms`, `lfs_write_ms`, `wamr_load_ms`,
    `wamr_start_ms` as floats (µs → ms with 3 decimal places).

    Phase 1: wait up to `wait_timeout` seconds for `TIMING transfer_us=` to appear.
    Phase 2: wait up to 30 more seconds for wamr_load_us + wamr_start_us.
    """
    deadline_transfer = time.time() + wait_timeout
    transfer_found_at = None
    while time.time() < deadline_transfer:
        try:
            with open(log_path, "rb") as f:
                f.seek(byte_offset)
                tail = f.read().decode(errors="replace")
            if "TIMING transfer_us=" in tail:
                transfer_found_at = time.time()
                break
        except Exception as e:
            print(f"  [WARN] parse_device_timings_since(phase1): {e}")
            return {}
        time.sleep(0.5)

    if transfer_found_at is None:
        return {}  # never received frame

    # Phase 2: wait for wamr_load_us + wamr_start_us (up to 30 more seconds)
    deadline_wamr = transfer_found_at + 30.0
    result = {}
    while time.time() < deadline_wamr:
        try:
            with open(log_path, "rb") as f:
                f.seek(byte_offset)
                tail = f.read().decode(errors="replace")
            result = {}
            # firmware logs _us fields; convert to ms (float, 3dp) for CSV/display
            for us_key, ms_key in [
                ("transfer_us",  "transfer_ms"),
                ("lfs_write_us", "lfs_write_ms"),
                ("wamr_load_us", "wamr_load_ms"),
                ("wamr_start_us","wamr_start_ms"),
            ]:
                m = re.search(rf"TIMING {us_key}=(\d+)", tail)
                if m:
                    result[ms_key] = round(int(m.group(1)) / 1000.0, 3)
            if "wamr_load_ms" in result and "wamr_start_ms" in result:
                return result  # all phases complete
        except Exception as e:
            print(f"  [WARN] parse_device_timings_since(phase2): {e}")
            return result
        time.sleep(0.5)
    return result  # partial (wamr phases may be missing if WAMR failed)


def parse_cloud_timings_for_app(app_name: str) -> dict:
    """Parse gateway pod logs to extract deployment timing for app_name."""
    result = {}
    try:
        r = subprocess.run(
            ["kubectl", "logs", "-n", K8S_NAMESPACE,
             "deployment/gateway-1-deployment", "--tail=500"],
            capture_output=True, text=True
        )
        lines = r.stdout.splitlines()
        recv_ts = send_ts = ack_ts = None
        for line in lines:
            if app_name in line:
                ts_match = re.search(r'(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)', line)
                ts = ts_match.group(1) if ts_match else None
                if "Received deployment request" in line or "deploy" in line.lower():
                    recv_ts = recv_ts or ts
                if "Successfully sent deployment" in line or "sending deployment" in line.lower():
                    send_ts = ts
                if "deployment acknowledgment" in line.lower() or "DeployAck" in line:
                    ack_ts = ts
        result = {"t_gw_recv": recv_ts, "t_gw_send": send_ts, "t_deploy_ack": ack_ts}
    except Exception as e:
        print(f"  [WARN] parse_cloud_timings: {e}")
    return result


# ─── WASM PADDING ─────────────────────────────────────────────────────────────

def encode_leb128_u32(n: int) -> bytes:
    """Encode a non-negative integer as LEB128."""
    result = []
    while True:
        byte = n & 0x7F
        n >>= 7
        if n:
            result.append(byte | 0x80)
        else:
            result.append(byte)
            break
    return bytes(result)


def pad_wasm_module(wasm_bytes: bytes, target_size: int) -> bytes:
    """
    Pad a WASM module to EXACTLY target_size bytes by appending a WebAssembly custom
    section.  Custom section format:
        0x00  [leb128(payload_len)]  [name_len_byte][name_bytes][zero_padding_bytes]
    Iterates over possible LEB128 sizes (1-5 bytes) to find the unique size that
    yields exactly target_size bytes without any truncation (truncation would produce
    invalid WASM that WAMR rejects).
    """
    if len(wasm_bytes) >= target_size:
        return wasm_bytes

    section_name = b"padding"
    name_part = bytes([len(section_name)]) + section_name  # 1 + 7 = 8 bytes

    for leb_size in range(1, 6):
        payload_data_size = target_size - len(wasm_bytes) - 1 - leb_size - len(name_part)
        if payload_data_size < 0:
            continue
        payload_len = len(name_part) + payload_data_size
        leb = encode_leb128_u32(payload_len)
        if len(leb) != leb_size:
            continue  # LEB128 encoding has a different length — try next
        # Exact match: build section
        payload = name_part + b'\x00' * payload_data_size
        section = b'\x00' + leb + payload
        result = wasm_bytes + section
        assert len(result) == target_size, f"pad_wasm_module: {len(result)} != {target_size}"
        return result

    # Fallback for very small targets: return unmodified valid WASM
    return wasm_bytes


# ─── EXPERIMENT 1: SCALABILITY ────────────────────────────────────────────────

def run_experiment_scalability(scales=None, n_reps=2):
    """
    Esp. 1: enrollment e deployment su N ∈ {1,10,50,100,500} device.
    Misura: enrollment_total_s (tutti Connected), deploy_total_s (tutti Running).
    """
    if scales is None:
        scales = [1, 10, 50, 100, 500]

    ensure_dirs()
    csv_path = os.path.join(RESULTS_DIR, "scalability.csv")
    wasm_b64 = get_wasm_bytes_b64()

    fieldnames = ["N", "rep", "enroll_total_s", "deploy_total_s",
                  "enroll_success", "deploy_success", "enroll_errors", "deploy_errors"]
    rows = []

    print("\n" + "="*60)
    print("EXPERIMENT 1: Scalability")
    print("="*60)

    # Index offset to avoid collisions across experiments
    IDX_OFFSET = 100_000

    for N in scales:
        for rep in range(1, n_reps + 1):
            print(f"\n[EXP1] N={N}, rep={rep}/{n_reps}")
            flash_dir = tempfile.mkdtemp(prefix=f"exp1_n{N}_r{rep}_", dir=SCRATCHPAD)
            log_dir = os.path.join(flash_dir, "logs")
            os.makedirs(log_dir, exist_ok=True)

            # Generate device indices: IDX_OFFSET + N*10000 + rep*1000 + i
            base_idx = IDX_OFFSET + N * 10_000 + rep * 1_000
            indices = list(range(base_idx, base_idx + N))
            device_names = [device_name_from_index(i) for i in indices]
            key_hexes = [hex_key(i) for i in indices]
            app_name = f"exp1-n{N:04d}-r{rep}"
            processes = []

            try:
                enable_pairing_mode(True)

                # Pre-create Device CRDs
                print(f"  Creating {N} Device CRDs...")
                for dname, kh in zip(device_names, key_hexes):
                    create_device_crd(dname, key_to_urlsafe_b64_no_pad(kh))
                time.sleep(1)  # CRD watch propagation

                # Launch all firmware instances concurrently
                print(f"  Launching {N} zephyr.exe instances (--no-rt)...")
                t_launch = time.time()
                for i, (dname, kh) in enumerate(zip(device_names, key_hexes)):
                    flash_path = make_flash_path(i, flash_dir)
                    log_path = os.path.join(log_dir, f"device_{i:04d}.log")
                    proc = launch_zephyr(i, kh, flash_path, log_path)
                    processes.append(proc)

                # Wait for enrollment
                enroll_deadline = max(ENROLL_TIMEOUT_S, N * 3)
                ok_e, connected_at, err_e = wait_all_connected(device_names, enroll_deadline)
                enroll_total = max(connected_at.values(), default=enroll_deadline)

                print(f"  Enrollment: {'OK' if ok_e else 'PARTIAL'} "
                      f"({len(connected_at)}/{N} connected, {enroll_total:.2f}s)")
                if err_e:
                    print(f"  Errors: {err_e[:3]}")

                # Determine deploy targets: all devices if enrollment 100%, else the
                # connected subset (allows measuring deploy even on partial enrollment).
                if not ok_e:
                    targets = [d for d in device_names if d in connected_at]
                    if not targets:
                        # Zero devices connected – nothing to deploy; record and skip.
                        rows.append({
                            "N": N, "rep": rep,
                            "enroll_total_s": round(enroll_total, 3),
                            "deploy_total_s": None,
                            "enroll_success": 0,
                            "deploy_success": 0,
                            "enroll_errors": "; ".join(err_e[:5]),
                            "deploy_errors": "enrollment incomplete; zero targets"
                        })
                        continue
                    # Partial enrollment: deploy to connected subset only.
                    print(f"  Enrollment partial ({len(targets)}/{N}); "
                          f"deploying to connected subset...")
                else:
                    targets = device_names

                # Deploy application (to targets – full list or subset)
                print(f"  Deploying to {len(targets)} devices...")
                print(f"  [TS] deploy_start_epoch={time.time():.3f}", flush=True)
                create_application_crd(app_name, targets, wasm_b64)
                deploy_deadline = max(DEPLOY_TIMEOUT_S, N * 4)
                ok_d, deploy_total, statuses, err_d = wait_app_running(
                    app_name, targets, deploy_deadline)
                print(f"  [TS] deploy_end_epoch={time.time():.3f}", flush=True)

                running_count = sum(
                    1 for d in targets
                    if statuses.get(d, {}).get("status") == "Running"
                )
                print(f"  Deploy: {'OK' if ok_d else 'PARTIAL'} "
                      f"({running_count}/{len(targets)} Running, {deploy_total:.2f}s)")

                # If enrollment was partial, annotate deploy_errors with subset info.
                deploy_err_parts = list(err_d[:5])
                if not ok_e:
                    deploy_err_parts.insert(
                        0, f"enroll partial {len(connected_at)}/{N}; "
                           f"deploy to {len(targets)}-device subset")
                rows.append({
                    "N": N, "rep": rep,
                    "enroll_total_s": round(enroll_total, 3),
                    "deploy_total_s": round(deploy_total, 3),
                    "enroll_success": len(connected_at),
                    "deploy_success": running_count,
                    "enroll_errors": "; ".join(err_e[:5]),
                    "deploy_errors": "; ".join(deploy_err_parts)
                })
                print(f"  → N={N} rep={rep}: enroll={enroll_total:.2f}s "
                      f"deploy={deploy_total:.2f}s")

            except Exception as e:
                print(f"  [ERROR] N={N} rep={rep}: {e}")
                rows.append({
                    "N": N, "rep": rep,
                    "enroll_total_s": None, "deploy_total_s": None,
                    "enroll_success": 0, "deploy_success": 0,
                    "enroll_errors": str(e), "deploy_errors": ""
                })
            finally:
                for proc in processes:
                    kill_proc(proc)
                delete_application_crd(app_name)
                for dname in device_names:
                    delete_device_crd(dname)
                try:
                    shutil.rmtree(flash_dir, ignore_errors=True)
                except Exception:
                    pass
                time.sleep(5)

        # Stop scaling if last scale point failed
        last_reps = [r for r in rows if r["N"] == N]
        if last_reps and any(r["enroll_success"] < N for r in last_reps):
            print(f"\n[EXP1] Saturation at N={N}. Stopping.")
            break

    with open(csv_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"\n[EXP1] Results saved to {csv_path}")
    return rows


# ─── EXPERIMENT 2: LATENCY DECOMPOSITION ─────────────────────────────────────

def run_experiment_decomposition(n_reps=5, decomp_size_kb=50):
    """
    Esp. 2: decomposizione latenza deployment in fasi cloud + device.
    Un device stabile, deploy sequenziali. Parsing log gateway + stdout firmware.
    decomp_size_kb: modulo padded a questa dimensione (default 50KB) per dare
    transfer_ms misurabile; 3KB dà 0ms per il transfer (buffered immediately).
    """
    ensure_dirs()
    csv_path = os.path.join(RESULTS_DIR, "decomposition.csv")
    # Use padded module so device-side phases (especially transfer) are measurable
    base_wasm = get_wasm_bytes()
    target_bytes = decomp_size_kb * 1024
    if target_bytes > len(base_wasm):
        padded = pad_wasm_module(base_wasm, target_bytes)
        wasm_b64 = base64.b64encode(padded).decode()
        print(f"  Using padded WASM: {len(base_wasm)}B → {len(padded)}B ({decomp_size_kb} KB)")
    else:
        wasm_b64 = base64.b64encode(base_wasm).decode()
        print(f"  Using native WASM: {len(base_wasm)}B")

    print("\n" + "="*60)
    print("EXPERIMENT 2: Latency Decomposition")
    print("="*60)

    rows = []
    fieldnames = [
        "rep", "total_s",
        "t_gw_recv", "t_gw_send", "t_deploy_ack",
        "transfer_ms", "lfs_write_ms", "wamr_load_ms", "wamr_start_ms",
        "success"
    ]

    IDX = 999_999
    device_name = device_name_from_index(IDX)
    key_hex = hex_key(IDX)
    flash_dir = tempfile.mkdtemp(prefix="exp2_decomp_", dir=SCRATCHPAD)
    log_dir = os.path.join(flash_dir, "logs")
    os.makedirs(log_dir, exist_ok=True)
    flash_path = make_flash_path(0, flash_dir)
    log_path = os.path.join(log_dir, "device.log")

    try:
        enable_pairing_mode(True)
        create_device_crd(device_name, key_to_urlsafe_b64_no_pad(key_hex))
        time.sleep(1)

        proc = launch_zephyr(0, key_hex, flash_path, log_path)

        ok_e, connected_at, err_e = wait_all_connected([device_name], ENROLL_TIMEOUT_S)
        if not ok_e:
            print(f"  [ERROR] Enrollment failed: {err_e}")
            return
        print(f"  Device connected in {connected_at[device_name]:.2f}s")
        time.sleep(2)

        for rep in range(1, n_reps + 1):
            print(f"  [EXP2] rep={rep}/{n_reps}")
            app_name = f"exp2-decomp-r{rep}"
            delete_application_crd(app_name)
            time.sleep(0.5)

            log_size_before = os.path.getsize(log_path) if os.path.exists(log_path) else 0

            t_api_create = time.time()
            create_application_crd(app_name, [device_name], wasm_b64)

            ok_d, deploy_total, statuses, err_d = wait_app_running(
                app_name, [device_name], DEPLOY_TIMEOUT_S)

            total_s = time.time() - t_api_create

            cloud_timings = parse_cloud_timings_for_app(app_name)
            device_timings = parse_device_timings_since(log_path, log_size_before)

            row = {
                "rep": rep,
                "total_s": round(total_s, 3),
                **cloud_timings,
                **device_timings,
                "success": ok_d
            }
            rows.append(row)
            print(f"    total={total_s:.2f}s "
                  f"transfer={device_timings.get('transfer_ms','?')}ms "
                  f"lfs={device_timings.get('lfs_write_ms','?')}ms "
                  f"wamr_load={device_timings.get('wamr_load_ms','?')}ms "
                  f"wamr_start={device_timings.get('wamr_start_ms','?')}ms")

            delete_application_crd(app_name)
            time.sleep(3)

    finally:
        try: kill_proc(proc)
        except Exception: pass
        delete_device_crd(device_name)
        shutil.rmtree(flash_dir, ignore_errors=True)

    with open(csv_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames, extrasaction='ignore')
        w.writeheader()
        w.writerows(rows)
    print(f"[EXP2] Results saved to {csv_path}")
    return rows


# ─── EXPERIMENT 3: THROUGHPUT ─────────────────────────────────────────────────

def run_experiment_throughput(rates=None):
    """
    Esp. 3: max enrollment e deploy throughput.
    Lancia N device a tasso 'rate' req/s, misura latenza aggregata.
    """
    if rates is None:
        rates = [1, 5, 10, 20, 50]

    ensure_dirs()
    csv_path = os.path.join(RESULTS_DIR, "throughput.csv")
    wasm_b64 = get_wasm_bytes_b64()

    print("\n" + "="*60)
    print("EXPERIMENT 3: Throughput")
    print("="*60)

    rows = []
    fieldnames = ["rate_rps", "n_devices", "enrolled_ok", "deployed_ok",
                  "enroll_latency_mean_s", "enroll_latency_std_s",
                  "deploy_latency_mean_s",
                  "saturated"]
    # Note: deploy_latency_std_s intentionally omitted — the throughput experiment
    # deploys a single Application CRD to all N devices simultaneously and measures
    # one aggregate batch time, not N per-device samples, so std is meaningless here.
    # enroll_latency_std_s is real (measured per-device).
    saturated = False
    IDX_OFFSET = 200_000

    for rate in rates:
        if saturated:
            break
        N = min(max(10, rate * 10), 100)
        print(f"\n[EXP3] rate={rate}rps, N={N} devices")

        base_idx = IDX_OFFSET + rate * 1_000
        indices = list(range(base_idx, base_idx + N))
        device_names = [device_name_from_index(i) for i in indices]
        key_hexes = [hex_key(i) for i in indices]
        app_name = f"exp3-r{rate:03d}-app"
        processes = []
        start_times = {}
        enroll_times = {}

        flash_dir = tempfile.mkdtemp(prefix=f"exp3_r{rate}_", dir=SCRATCHPAD)
        log_dir = os.path.join(flash_dir, "logs")
        os.makedirs(log_dir, exist_ok=True)

        try:
            enable_pairing_mode(True)

            # Pre-create Device CRDs
            for dname, kh in zip(device_names, key_hexes):
                create_device_crd(dname, key_to_urlsafe_b64_no_pad(kh))
            time.sleep(0.5)

            # Launch at specified rate
            inter_arrival = 1.0 / rate
            for i, (dname, kh) in enumerate(zip(device_names, key_hexes)):
                flash_path = make_flash_path(i, flash_dir)
                log_path = os.path.join(log_dir, f"device_{i:04d}.log")
                t_start = time.time()
                proc = launch_zephyr(i, kh, flash_path, log_path)
                processes.append(proc)
                start_times[dname] = t_start
                time.sleep(inter_arrival)

            # Poll enrollment
            t0_enroll = time.time()
            deadline_e = t0_enroll + max(ENROLL_TIMEOUT_S, N * 3)
            remaining = set(device_names)
            while remaining and time.time() < deadline_e:
                for dname in list(remaining):
                    if get_device_phase(dname) == "Connected":
                        enroll_times[dname] = time.time() - start_times[dname]
                        remaining.discard(dname)
                time.sleep(POLL_INTERVAL_S)

            enrolled_count = len(enroll_times)
            enroll_vals = list(enroll_times.values())
            enroll_mean = sum(enroll_vals) / len(enroll_vals) if enroll_vals else 0.0
            enroll_std = (math.sqrt(sum((v-enroll_mean)**2 for v in enroll_vals)/len(enroll_vals))
                          if len(enroll_vals) > 1 else 0.0)

            sat = enrolled_count < N * 0.95

            # Deploy
            connected_devices = [d for d in device_names
                                  if get_device_phase(d) == "Connected"]
            deploy_ok = 0
            deploy_mean = 0.0
            if connected_devices:
                create_application_crd(app_name, connected_devices, wasm_b64)
                ok_d, dt, statuses, _ = wait_app_running(
                    app_name, connected_devices,
                    max(DEPLOY_TIMEOUT_S, len(connected_devices) * 4))
                deploy_ok = sum(1 for d in connected_devices
                                if statuses.get(d, {}).get("status") == "Running")
                deploy_mean = dt

            rows.append({
                "rate_rps": rate, "n_devices": N,
                "enrolled_ok": enrolled_count, "deployed_ok": deploy_ok,
                "enroll_latency_mean_s": round(enroll_mean, 3),
                "enroll_latency_std_s": round(enroll_std, 3),
                "deploy_latency_mean_s": round(deploy_mean, 3),
                "saturated": sat
            })
            print(f"  enrolled={enrolled_count}/{N} deploy={deploy_ok}/{len(connected_devices)} "
                  f"sat={sat} enroll_mean={enroll_mean:.2f}s")
            if sat:
                saturated = True

        except Exception as e:
            print(f"  [ERROR] rate={rate}: {e}")
        finally:
            for proc in processes:
                kill_proc(proc)
            delete_application_crd(app_name)
            for dname in device_names:
                delete_device_crd(dname)
            shutil.rmtree(flash_dir, ignore_errors=True)
            time.sleep(5)

    with open(csv_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"[EXP3] Results saved to {csv_path}")
    return rows


# ─── EXPERIMENT 4: MODULE SIZE SENSITIVITY ───────────────────────────────────

def run_experiment_size_sensitivity(sizes_kb=None, n_reps=3):
    """
    Esp. 4: sensibilità alla dimensione del modulo WASM.
    Misura transfer / LittleFS-write / WAMR-load / WAMR-start per dimensioni diverse.
    """
    if sizes_kb is None:
        sizes_kb = [3, 50, 100, 250, 500, 1024]  # 1024 = 1 MB

    ensure_dirs()
    csv_path = os.path.join(RESULTS_DIR, "size_sensitivity.csv")
    base_wasm = get_wasm_bytes()

    print("\n" + "="*60)
    print("EXPERIMENT 4: Module Size Sensitivity")
    print("="*60)
    print(f"  Base module: {len(base_wasm)} bytes")
    print(f"  Target sizes (KB): {sizes_kb}")

    rows = []
    fieldnames = ["size_kb", "actual_bytes", "transfer_ms", "lfs_write_ms",
                  "wamr_load_ms", "wamr_start_ms", "total_deploy_s", "success"]

    IDX = 888_888
    device_name = device_name_from_index(IDX)
    key_hex = hex_key(IDX)
    flash_dir = tempfile.mkdtemp(prefix="exp4_size_", dir=SCRATCHPAD)
    log_dir = os.path.join(flash_dir, "logs")
    os.makedirs(log_dir, exist_ok=True)
    flash_path = make_flash_path(0, flash_dir)
    log_path = os.path.join(log_dir, "device.log")

    try:
        enable_pairing_mode(True)
        create_device_crd(device_name, key_to_urlsafe_b64_no_pad(key_hex))
        time.sleep(1)

        proc = launch_zephyr(0, key_hex, flash_path, log_path)
        ok_e, connected_at, err_e = wait_all_connected([device_name], ENROLL_TIMEOUT_S)
        if not ok_e:
            print(f"  [ERROR] Enrollment failed: {err_e}")
            return
        print(f"  Device connected in {connected_at[device_name]:.2f}s")

        for size_kb in sizes_kb:
            target_bytes = size_kb * 1024
            padded = pad_wasm_module(base_wasm, target_bytes)
            actual_bytes = len(padded)
            wasm_b64_padded = base64.b64encode(padded).decode()
            print(f"\n  Testing size={size_kb}KB (actual={actual_bytes}B)...")

            rep_transfer = []
            rep_lfs = []
            rep_load = []
            rep_start = []
            rep_total = []
            rep_success = []

            for rep in range(1, n_reps + 1):
                app_name = f"exp4-sz{size_kb:04d}-r{rep}"
                delete_application_crd(app_name)
                time.sleep(0.5)

                log_size_before = os.path.getsize(log_path) if os.path.exists(log_path) else 0

                t0 = time.time()
                create_application_crd(app_name, [device_name], wasm_b64_padded)
                ok_d, deploy_total, statuses, err_d = wait_app_running(
                    app_name, [device_name], DEPLOY_TIMEOUT_S * 4)
                total_s = time.time() - t0

                timings = parse_device_timings_since(log_path, log_size_before)
                rep_transfer.append(timings.get("transfer_ms"))
                rep_lfs.append(timings.get("lfs_write_ms"))
                rep_load.append(timings.get("wamr_load_ms"))
                rep_start.append(timings.get("wamr_start_ms"))
                rep_total.append(total_s)
                rep_success.append(ok_d)

                print(f"    rep={rep}: total={total_s:.2f}s "
                      f"transfer={timings.get('transfer_ms','?')}ms "
                      f"lfs={timings.get('lfs_write_ms','?')}ms "
                      f"wamr_load={timings.get('wamr_load_ms','?')}ms")

                delete_application_crd(app_name)
                time.sleep(3)

            def mean_or_none(vals):
                valid = [v for v in vals if v is not None]
                if not valid:
                    return None
                # Use median to reduce impact of race-condition outliers (esp. rep1
                # of large modules may pick up a retransmit from the previous size)
                s = sorted(valid)
                n = len(s)
                median = s[n // 2] if n % 2 == 1 else (s[n//2 - 1] + s[n//2]) / 2
                return round(median, 3)

            rows.append({
                "size_kb": size_kb,
                "actual_bytes": actual_bytes,
                "transfer_ms": mean_or_none(rep_transfer),
                "lfs_write_ms": mean_or_none(rep_lfs),
                "wamr_load_ms": mean_or_none(rep_load),
                "wamr_start_ms": mean_or_none(rep_start),
                "total_deploy_s": round(sum(rep_total)/len(rep_total), 3) if rep_total else None,
                "success": sum(1 for s in rep_success if s)
            })

    finally:
        try: kill_proc(proc)
        except Exception: pass
        delete_device_crd(device_name)
        shutil.rmtree(flash_dir, ignore_errors=True)

    with open(csv_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)
    print(f"[EXP4] Results saved to {csv_path}")
    return rows


# ─── EXPERIMENT 5: RELIABILITY ────────────────────────────────────────────────

def run_experiment_reliability(n_trials=100):
    """
    Esp. 5: 100 enrollment + 100 deploy consecutivi.
    Riporta success rate, mean, std dev, n. timeout.
    """
    ensure_dirs()
    csv_path = os.path.join(RESULTS_DIR, "reliability.csv")
    wasm_b64 = get_wasm_bytes_b64()

    print("\n" + "="*60)
    print("EXPERIMENT 5: Reliability")
    print("="*60)

    enroll_results = []
    deploy_results = []
    fieldnames = ["trial", "phase", "elapsed_s", "success", "error"]
    all_rows = []
    IDX_OFFSET = 300_000

    # Phase 1: N enrollment trials (sequential, each a fresh device)
    print(f"\n  Phase 1: {n_trials} enrollment trials (sequential)")
    for trial in range(1, n_trials + 1):
        idx = IDX_OFFSET + trial
        device_name = device_name_from_index(idx)
        key_hex_t = hex_key(idx)
        flash_dir = tempfile.mkdtemp(prefix=f"exp5_e{trial}_", dir=SCRATCHPAD)
        log_path = os.path.join(flash_dir, f"enroll_{trial:04d}.log")
        proc = None
        try:
            enable_pairing_mode(True)
            create_device_crd(device_name, key_to_urlsafe_b64_no_pad(key_hex_t))
            time.sleep(0.2)

            t0 = time.time()
            proc = launch_zephyr(trial, key_hex_t,
                                  make_flash_path(0, flash_dir), log_path)
            ok_e, connected_at, err_e = wait_all_connected([device_name], ENROLL_TIMEOUT_S)
            elapsed = time.time() - t0

            success = ok_e and bool(connected_at)
            enroll_results.append(elapsed if success else None)
            all_rows.append({
                "trial": trial, "phase": "enrollment",
                "elapsed_s": round(elapsed, 3),
                "success": success,
                "error": "; ".join(err_e[:2])
            })
            if trial % 10 == 0:
                ok_count = sum(1 for v in enroll_results if v is not None)
                print(f"  trial {trial}/{n_trials}: {ok_count} succeeded")

        except Exception as e:
            enroll_results.append(None)
            all_rows.append({
                "trial": trial, "phase": "enrollment",
                "elapsed_s": None, "success": False, "error": str(e)
            })
        finally:
            if proc:
                kill_proc(proc)
            delete_device_crd(device_name)
            shutil.rmtree(flash_dir, ignore_errors=True)

    enroll_ok = [v for v in enroll_results if v is not None]
    enroll_success_rate = len(enroll_ok) / n_trials
    enroll_mean = sum(enroll_ok) / len(enroll_ok) if enroll_ok else 0.0
    enroll_std = (math.sqrt(sum((v-enroll_mean)**2 for v in enroll_ok)/len(enroll_ok))
                  if len(enroll_ok) > 1 else 0.0)
    print(f"\n  Enrollment: rate={enroll_success_rate*100:.1f}% "
          f"mean={enroll_mean:.2f}s std={enroll_std:.2f}s "
          f"timeouts={n_trials - len(enroll_ok)}")

    # Phase 2: N deploy trials on a persistent connected device
    print(f"\n  Phase 2: {n_trials} deploy trials")
    IDX_D = 399_999
    deploy_device = device_name_from_index(IDX_D)
    key_hex_d = hex_key(IDX_D)
    flash_dir_d = tempfile.mkdtemp(prefix="exp5_deploy_", dir=SCRATCHPAD)
    log_path_d = os.path.join(flash_dir_d, "deploy_device.log")
    proc_d = None

    try:
        enable_pairing_mode(True)
        create_device_crd(deploy_device, key_to_urlsafe_b64_no_pad(key_hex_d))
        time.sleep(1)
        proc_d = launch_zephyr(0, key_hex_d,
                                make_flash_path(0, flash_dir_d), log_path_d)
        ok_e, connected_at, err_e = wait_all_connected([deploy_device], ENROLL_TIMEOUT_S)
        if not ok_e:
            print(f"  [ERROR] Deploy device enrollment failed: {err_e}")
        else:
            print(f"  Deploy device connected in {connected_at[deploy_device]:.2f}s")
            time.sleep(2)

            for trial in range(1, n_trials + 1):
                app_name = f"exp5-deploy-{trial:04d}"
                delete_application_crd(app_name)
                time.sleep(0.2)
                try:
                    t0 = time.time()
                    create_application_crd(app_name, [deploy_device], wasm_b64)
                    ok_d, deploy_total, statuses, err_d = wait_app_running(
                        app_name, [deploy_device], DEPLOY_TIMEOUT_S)
                    elapsed = time.time() - t0

                    deploy_results.append(elapsed if ok_d else None)
                    all_rows.append({
                        "trial": trial, "phase": "deploy",
                        "elapsed_s": round(elapsed, 3),
                        "success": ok_d,
                        "error": "; ".join(err_d[:2])
                    })
                    if trial % 10 == 0:
                        ok_count = sum(1 for v in deploy_results if v is not None)
                        print(f"  deploy trial {trial}/{n_trials}: {ok_count} succeeded")

                except Exception as e:
                    deploy_results.append(None)
                    all_rows.append({
                        "trial": trial, "phase": "deploy",
                        "elapsed_s": None, "success": False, "error": str(e)
                    })
                finally:
                    delete_application_crd(app_name)
                time.sleep(2)

    finally:
        if proc_d:
            kill_proc(proc_d)
        delete_device_crd(deploy_device)
        shutil.rmtree(flash_dir_d, ignore_errors=True)

    deploy_ok = [v for v in deploy_results if v is not None]
    deploy_success_rate = len(deploy_ok) / n_trials if deploy_results else 0
    deploy_mean = sum(deploy_ok) / len(deploy_ok) if deploy_ok else 0.0
    deploy_std = (math.sqrt(sum((v-deploy_mean)**2 for v in deploy_ok)/len(deploy_ok))
                  if len(deploy_ok) > 1 else 0.0)
    print(f"\n  Deploy: rate={deploy_success_rate*100:.1f}% "
          f"mean={deploy_mean:.2f}s std={deploy_std:.2f}s "
          f"timeouts={n_trials - len(deploy_ok)}")

    summary_path = os.path.join(RESULTS_DIR, "reliability_summary.json")
    with open(summary_path, "w") as f:
        json.dump({
            "enrollment": {
                "n_trials": n_trials, "success_rate": enroll_success_rate,
                "mean_s": enroll_mean, "std_s": enroll_std,
                "timeouts": n_trials - len(enroll_ok)
            },
            "deploy": {
                "n_trials": n_trials, "success_rate": deploy_success_rate,
                "mean_s": deploy_mean, "std_s": deploy_std,
                "timeouts": n_trials - len(deploy_ok)
            }
        }, f, indent=2)

    with open(csv_path, "w", newline="") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(all_rows)
    print(f"[EXP5] Results saved to {csv_path} and {summary_path}")
    return all_rows


# ─── QUICK SMOKE TEST ──────────────────────────────────────────────────────────

def run_quick_test():
    """Single device enrollment + deploy smoke test."""
    print("\n" + "="*60)
    print("QUICK TEST: 1 device enrollment + deploy")
    print("="*60)

    ensure_dirs()
    IDX = 1  # unique index for quick test
    device_name = device_name_from_index(IDX)
    key_hex_t = hex_key(IDX)
    wasm_b64 = get_wasm_bytes_b64()

    flash_dir = tempfile.mkdtemp(prefix="quick_test_", dir=SCRATCHPAD)
    log_path = os.path.join(flash_dir, "device.log")
    proc = None
    success = False

    try:
        enable_pairing_mode(True)
        print(f"  Pairing mode enabled.")
        create_device_crd(device_name, key_to_urlsafe_b64_no_pad(key_hex_t))
        print(f"  Device CRD created: {device_name}")
        time.sleep(0.5)

        t0 = time.time()
        proc = launch_zephyr(0, key_hex_t, make_flash_path(0, flash_dir), log_path)
        print(f"  zephyr.exe launched (PID {proc.pid})")

        ok_e, connected_at, err_e = wait_all_connected([device_name], ENROLL_TIMEOUT_S)
        enroll_s = connected_at.get(device_name, ENROLL_TIMEOUT_S)
        print(f"  Enrollment: {'OK' if ok_e else 'FAILED'} in {enroll_s:.2f}s")
        if not ok_e:
            print(f"  Errors: {err_e}")
            return

        app_name = "quick-test-app"
        delete_application_crd(app_name)
        create_application_crd(app_name, [device_name], wasm_b64)
        ok_d, deploy_s, statuses, err_d = wait_app_running(
            app_name, [device_name], DEPLOY_TIMEOUT_S)
        print(f"  Deploy: {'OK' if ok_d else 'FAILED'} in {deploy_s:.2f}s")
        if not ok_d:
            print(f"  Errors: {err_d}")

        success = ok_e and ok_d
        print(f"\n{'✓ Quick test PASSED!' if success else '✗ Quick test FAILED'}")
        print(f"  Enrollment: {enroll_s:.2f}s, Deploy: {deploy_s:.2f}s")

    finally:
        if proc:
            kill_proc(proc)
        delete_application_crd("quick-test-app")
        delete_device_crd(device_name)
        shutil.rmtree(flash_dir, ignore_errors=True)

    return success


# ─── MAIN ──────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="RETROSPECT Measurement Harness")
    parser.add_argument("--exp", default="all",
                        choices=["all", "scalability", "decomp", "throughput", "size", "reliability"],
                        help="Which experiment to run")
    parser.add_argument("--quick", action="store_true", help="Run smoke test only (N=1)")
    parser.add_argument("--scales", type=int, nargs="+",
                        default=[1, 10, 50, 100, 500],
                        help="Scalability N values")
    parser.add_argument("--reps", type=int, default=2, help="Repetitions per N")
    parser.add_argument("--trials", type=int, default=100, help="Trials for reliability")
    args = parser.parse_args()

    ensure_dirs()

    # Verify prerequisites
    if not os.path.exists(ZEPHYR_EXE):
        print(f"[ERROR] zephyr.exe not found: {ZEPHYR_EXE}")
        sys.exit(1)
    if not os.path.exists(WASM_FILE):
        print(f"[ERROR] WASM file not found: {WASM_FILE}")
        sys.exit(1)

    wasm = get_wasm_bytes()
    print(f"Gateway TLS : {GATEWAY_TLS}")
    print(f"Gateway HTTP: {GATEWAY_HTTP}")
    print(f"Zephyr      : {ZEPHYR_EXE}")
    print(f"WASM file   : {WASM_FILE} ({len(wasm)} bytes)")
    print(f"Results dir : {RESULTS_DIR}")

    if args.quick:
        run_quick_test()
        return

    exp = args.exp
    if exp in ("all", "scalability"):
        run_experiment_scalability(scales=args.scales, n_reps=args.reps)
    if exp in ("all", "decomp"):
        run_experiment_decomposition(n_reps=5)
    if exp in ("all", "throughput"):
        run_experiment_throughput()
    if exp in ("all", "size"):
        run_experiment_size_sensitivity()
    if exp in ("all", "reliability"):
        run_experiment_reliability(n_trials=args.trials)

    print("\n[DONE] All experiments completed. Results in:", RESULTS_DIR)


if __name__ == "__main__":
    main()
