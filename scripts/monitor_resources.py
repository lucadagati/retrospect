#!/usr/bin/env python3
"""
monitor_resources.py — campiona risorse host + gateway pod ogni ~3s.

Usage:
    python3 scripts/monitor_resources.py <output_csv> [--interval 3]

Output CSV columns:
    epoch, elapsed_s,
    cpu_used_pct, load1, load5, load15,
    mem_used_mb, mem_avail_mb, swap_used_mb,
    nproc_zephyr, rss_zephyr_mb,
    open_fds,
    gw_cpu_millicores, gw_mem_mi
"""

import csv
import os
import signal
import subprocess
import sys
import time
import argparse

# ── helpers ───────────────────────────────────────────────────────────────────

def _read_proc_stat():
    """Return (user, nice, system, idle, iowait, irq, softirq) total ticks."""
    with open("/proc/stat") as f:
        line = f.readline()   # "cpu  ..."
    fields = list(map(int, line.split()[1:8]))
    return tuple(fields)


def _cpu_used_pct(prev, curr):
    """Compute %CPU used between two /proc/stat snapshots."""
    prev_idle = prev[3] + prev[4]
    curr_idle = curr[3] + curr[4]
    prev_total = sum(prev)
    curr_total = sum(curr)
    d_total = curr_total - prev_total
    d_idle  = curr_idle  - prev_idle
    if d_total == 0:
        return 0.0
    return round(100.0 * (1.0 - d_idle / d_total), 2)


def _read_loadavg():
    with open("/proc/loadavg") as f:
        parts = f.read().split()
    return float(parts[0]), float(parts[1]), float(parts[2])


def _read_meminfo():
    mem = {}
    with open("/proc/meminfo") as f:
        for line in f:
            k, v = line.split(":", 1)
            mem[k.strip()] = int(v.split()[0])   # kB
    mem_total = mem.get("MemTotal", 0)
    mem_free  = mem.get("MemFree",  0)
    mem_avail = mem.get("MemAvailable", mem_free)
    buffers   = mem.get("Buffers", 0)
    cached    = mem.get("Cached",  0)
    swap_total= mem.get("SwapTotal", 0)
    swap_free = mem.get("SwapFree",  0)
    mem_used  = mem_total - mem_free - buffers - cached
    swap_used = swap_total - swap_free
    return (
        round(mem_used  / 1024, 1),
        round(mem_avail / 1024, 1),
        round(swap_used / 1024, 1),
    )


def _count_zephyr():
    """Return (count, total_rss_mb) for all zephyr.exe processes."""
    try:
        out = subprocess.check_output(
            ["pgrep", "-a", "zephyr.exe"], text=True, stderr=subprocess.DEVNULL
        )
        pids = [line.split()[0] for line in out.strip().splitlines() if line]
    except subprocess.CalledProcessError:
        return 0, 0.0
    total_rss_kb = 0
    for pid in pids:
        try:
            with open(f"/proc/{pid}/status") as f:
                for line in f:
                    if line.startswith("VmRSS:"):
                        total_rss_kb += int(line.split()[1])
                        break
        except (OSError, ValueError):
            pass
    return len(pids), round(total_rss_kb / 1024, 1)


def _read_open_fds():
    try:
        with open("/proc/sys/fs/file-nr") as f:
            return int(f.read().split()[0])
    except Exception:
        return None


def _kubectl_top_gateway():
    """Return (cpu_millicores, mem_mi) or (None, None) on error."""
    try:
        out = subprocess.check_output(
            ["kubectl", "top", "pod", "-n", "wasmbed",
             "-l", "app=wasmbed-gateway", "--no-headers"],
            text=True, stderr=subprocess.DEVNULL, timeout=8
        )
        for line in out.strip().splitlines():
            parts = line.split()
            if len(parts) >= 3:
                cpu_s = parts[1].rstrip("m")
                mem_s = parts[2].rstrip("Mi").rstrip("m")
                cpu_m = int(cpu_s) if "m" in parts[1] else int(cpu_s) * 1000
                mem_i = int(parts[2].rstrip("Mi")) if "Mi" in parts[2] else None
                return cpu_m, mem_i
    except Exception:
        pass
    return None, None


# ── main loop ─────────────────────────────────────────────────────────────────

FIELDNAMES = [
    "epoch", "elapsed_s",
    "cpu_used_pct", "load1", "load5", "load15",
    "mem_used_mb", "mem_avail_mb", "swap_used_mb",
    "nproc_zephyr", "rss_zephyr_mb",
    "open_fds",
    "gw_cpu_millicores", "gw_mem_mi",
]

_stop = False


def _handle_signal(sig, frame):
    global _stop
    _stop = True


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("output_csv", help="Output CSV path")
    parser.add_argument("--interval", type=float, default=3.0,
                        help="Sampling interval in seconds (default: 3)")
    args = parser.parse_args()

    signal.signal(signal.SIGTERM, _handle_signal)
    signal.signal(signal.SIGINT,  _handle_signal)

    os.makedirs(os.path.dirname(os.path.abspath(args.output_csv)), exist_ok=True)

    t0 = time.time()
    prev_stat = _read_proc_stat()
    # First /proc/stat delta needs a sleep; take a short baseline.
    time.sleep(0.5)

    # kubectl top has ~12s granularity; cache the last good value.
    gw_cpu_cache, gw_mem_cache = None, None
    last_kubectl = 0.0

    with open(args.output_csv, "w", newline="") as csvf:
        writer = csv.DictWriter(csvf, fieldnames=FIELDNAMES)
        writer.writeheader()
        csvf.flush()

        while not _stop:
            now = time.time()
            elapsed = round(now - t0, 1)

            # CPU
            curr_stat = _read_proc_stat()
            cpu_pct = _cpu_used_pct(prev_stat, curr_stat)
            prev_stat = curr_stat

            # Load
            try:
                l1, l5, l15 = _read_loadavg()
            except Exception:
                l1, l5, l15 = None, None, None

            # Memory
            try:
                mem_used, mem_avail, swap_used = _read_meminfo()
            except Exception:
                mem_used, mem_avail, swap_used = None, None, None

            # Zephyr processes
            try:
                nz, rss_z = _count_zephyr()
            except Exception:
                nz, rss_z = None, None

            # Open FDs
            open_fds = _read_open_fds()

            # Gateway pod (rate-limited)
            if now - last_kubectl >= 12.0:
                gw_cpu_cache, gw_mem_cache = _kubectl_top_gateway()
                last_kubectl = now

            row = {
                "epoch":            round(now, 3),
                "elapsed_s":        elapsed,
                "cpu_used_pct":     cpu_pct,
                "load1":            l1,
                "load5":            l5,
                "load15":           l15,
                "mem_used_mb":      mem_used,
                "mem_avail_mb":     mem_avail,
                "swap_used_mb":     swap_used,
                "nproc_zephyr":     nz,
                "rss_zephyr_mb":    rss_z,
                "open_fds":         open_fds,
                "gw_cpu_millicores": gw_cpu_cache,
                "gw_mem_mi":        gw_mem_cache,
            }
            writer.writerow(row)
            csvf.flush()

            # Human-readable status line
            print(
                f"[MON t={elapsed:7.1f}s] "
                f"CPU={cpu_pct:5.1f}% load={l1} "
                f"RAM_avail={mem_avail}MB "
                f"zephyr={nz}({rss_z}MB) "
                f"fds={open_fds} "
                f"gw_cpu={gw_cpu_cache}m",
                flush=True,
            )

            # Sleep remainder of interval
            sleep_time = args.interval - (time.time() - now)
            if sleep_time > 0 and not _stop:
                time.sleep(sleep_time)

    print(f"[MON] Stopped. Wrote to {args.output_csv}", flush=True)


if __name__ == "__main__":
    main()
