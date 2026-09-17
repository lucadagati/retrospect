#!/usr/bin/env python3
"""Generate all figures for the Computer Communications paper from measured artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np


def load_jsonl(path: Path) -> list[dict]:
    rows = []
    if not path.exists():
        return rows
    for line in path.read_text().splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def fig_latency_boxplot(exp_dir: Path, out: Path) -> None:
    stages = {
        "Enrollment": load_jsonl(exp_dir / "raw/enrollment_trials.jsonl"),
        "Heartbeat": load_jsonl(exp_dir / "raw/heartbeat_trials.jsonl"),
        "Deployment": load_jsonl(exp_dir / "raw/deployment_trials.jsonl"),
    }
    data = [
        [r["latency_ms"] for r in rows if r.get("latency_ms") is not None]
        for rows in stages.values()
    ]
    labels = list(stages.keys())
    fig, ax = plt.subplots(figsize=(5.5, 3.4), constrained_layout=True)
    bp = ax.boxplot(data, tick_labels=labels, showfliers=True, patch_artist=True)
    colors = ["#4C78A8", "#72B7B2", "#E45756"]
    for patch, c in zip(bp["boxes"], colors):
        patch.set_facecolor(c)
        patch.set_alpha(0.75)
    means = [float(np.mean(d)) for d in data]
    ax.scatter(range(1, len(labels) + 1), means, marker="D", color="black", s=28, zorder=3, label="Mean")
    ax.set_yscale("log")
    ax.set_ylabel("Latency (ms, log scale)")
    ax.set_title("Trial-level stage latencies ($n=100$)")
    ax.legend(fontsize=8, loc="upper left")
    ax.grid(axis="y", alpha=0.3)
    fig.savefig(out, dpi=220, bbox_inches="tight")
    plt.close(fig)


def fig_latency_cdf(exp_dir: Path, out: Path) -> None:
    enr = [r["latency_ms"] for r in load_jsonl(exp_dir / "raw/enrollment_trials.jsonl")]
    dep = [r["latency_ms"] for r in load_jsonl(exp_dir / "raw/deployment_trials.jsonl")]
    e2e = [r["latency_ms"] for r in load_jsonl(exp_dir / "raw/transactional_trials.jsonl")]
    fig, ax = plt.subplots(figsize=(5.5, 3.4), constrained_layout=True)
    for vals, label, color in [
        (sorted(enr), "Enrollment", "#4C78A8"),
        (sorted(dep), "Deployment", "#E45756"),
        (sorted(e2e), "End-to-end", "#54A24B"),
    ]:
        y = np.arange(1, len(vals) + 1) / len(vals)
        ax.plot(vals, y, label=label, color=color, linewidth=1.8)
        p95 = float(np.percentile(vals, 95))
        p99 = float(np.percentile(vals, 99))
        ax.axhline(0.95, color="gray", linestyle="--", linewidth=0.8, alpha=0.6)
        ax.axhline(0.99, color="gray", linestyle=":", linewidth=0.8, alpha=0.6)
        ax.axvline(p95, color=color, linestyle="--", linewidth=0.8, alpha=0.5)
    ax.set_xlabel("Latency (ms)")
    ax.set_ylabel("Empirical CDF")
    ax.set_title("Latency CDF ($n=100$)")
    ax.legend(fontsize=8)
    ax.grid(alpha=0.3)
    fig.savefig(out, dpi=220, bbox_inches="tight")
    plt.close(fig)


def fig_latency_scatter(exp_dir: Path, out: Path) -> None:
    rows = load_jsonl(exp_dir / "raw/transactional_trials.jsonl")
    trials = [r["trial_id"] for r in rows]
    e2e = [r["latency_ms"] for r in rows]
    fig, ax = plt.subplots(figsize=(5.8, 3.2), constrained_layout=True)
    ax.scatter(trials, e2e, s=18, alpha=0.75, color="#4C78A8", edgecolors="none")
    ax.axhline(float(np.mean(e2e)), color="#E45756", linestyle="--", linewidth=1.2, label=f"Mean = {np.mean(e2e):.1f} ms")
    ax.axhline(float(np.percentile(e2e, 95)), color="#54A24B", linestyle=":", linewidth=1.2, label=f"p95 = {np.percentile(e2e, 95):.1f} ms")
    ax.set_xlabel("Trial index")
    ax.set_ylabel("End-to-end latency (ms)")
    ax.set_title("Per-trial transactional latency")
    ax.legend(fontsize=8)
    ax.grid(alpha=0.3)
    fig.savefig(out, dpi=220, bbox_inches="tight")
    plt.close(fig)


def fig_wire_sizes(sust_path: Path, out: Path) -> None:
    data = json.loads(sust_path.read_text())
    wire = data["wire_sizes_bytes"]
    order = [
        ("client_heartbeat", "HB client"),
        ("server_heartbeat_ack", "HB ack"),
        ("client_enrollment_request", "Enroll req"),
        ("client_public_key", "Public key"),
        ("client_enrollment_ack", "Enroll ack"),
        ("server_enrollment_accepted", "Enroll OK"),
        ("server_device_uuid", "Device UUID"),
        ("server_enrollment_completed", "Enroll done"),
        ("server_deploy_minimal_wasm", "Deploy WASM"),
        ("client_deploy_ack", "Deploy ack"),
    ]
    labels = [x[1] for x in order]
    vals = [wire[x[0]] for x in order]
    fig, ax = plt.subplots(figsize=(6.2, 3.4), constrained_layout=True)
    bars = ax.barh(labels, vals, color="#72B7B2")
    ax.set_xlabel("Wire size (bytes, incl. 4-byte length prefix)")
    ax.set_title("Measured CBOR/TLS application payloads")
    for b, v in zip(bars, vals):
        ax.text(v + 1, b.get_y() + b.get_height() / 2, str(v), va="center", fontsize=7)
    fig.savefig(out, dpi=220, bbox_inches="tight")
    plt.close(fig)


def fig_pod_resources(sust_path: Path, out: Path) -> None:
    data = json.loads(sust_path.read_text())
    pods = data["gateway_pod_resources_idle"]
    names, cpu, mem = [], [], []
    mapping = {
        "gateway": "Gateway",
        "wasmbed-api-server": "API server",
        "wasmbed-application-controller": "App controller",
    }
    for key, vals in pods.items():
        for prefix, label in mapping.items():
            if key.startswith(prefix):
                names.append(label)
                cpu.append(vals["cpu_millicores_mean"])
                mem.append(vals["memory_mib_mean"])
    x = np.arange(len(names))
    w = 0.35
    fig, ax = plt.subplots(figsize=(5.5, 3.4), constrained_layout=True)
    ax.bar(x - w / 2, cpu, w, label="CPU (millicores)", color="#4C78A8")
    ax2 = ax.twinx()
    ax2.bar(x + w / 2, mem, w, label="RAM (MiB)", color="#E45756", alpha=0.85)
    ax.set_xticks(x, names, rotation=15, ha="right")
    ax.set_ylabel("CPU (millicores)")
    ax2.set_ylabel("RAM (MiB)")
    ax.set_title("Control-plane pod resources (mean of 5 samples)")
    lines1, lab1 = ax.get_legend_handles_labels()
    lines2, lab2 = ax2.get_legend_handles_labels()
    ax.legend(lines1 + lines2, lab1 + lab2, fontsize=8, loc="upper left")
    fig.savefig(out, dpi=220, bbox_inches="tight")
    plt.close(fig)


def fig_ablation(ablation_path: Path, out: Path) -> None:
    if not ablation_path.exists():
        return
    data = json.loads(ablation_path.read_text())
    labels = ["Txn success", "Stale CRD", "False disc."]
    h = data["hardened"]
    u = data["unhardened"]
    vals_h = [h["txn_rate"], h["stale_phases"], h["false_disconnect"]]
    vals_u = [u["txn_rate"], u["stale_phases"], u["false_disconnect"]]
    x = range(len(labels))
    fig, ax = plt.subplots(figsize=(5.5, 3.2), constrained_layout=True)
    ax.bar([i - 0.18 for i in x], vals_h, 0.36, label="Hardened", color="#2A7F62")
    ax.bar([i + 0.18 for i in x], vals_u, 0.36, label="Unhardened", color="#B63A2B")
    ax.set_xticks(list(x), labels)
    ax.set_ylabel("Rate / count")
    ax.legend(fontsize=8)
    fig.savefig(out, dpi=220, bbox_inches="tight")
    plt.close(fig)


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--experiment-dir", default="experiments/20260609-070246")
    p.add_argument("--sustainability-json", default="experiments/sustainability_20260611-093255/sustainability_metrics.json")
    p.add_argument("--output-dir", default="RETROSPECT_submission/figures")
    args = p.parse_args()
    exp = Path(args.experiment_dir)
    out = Path(args.output_dir)
    out.mkdir(parents=True, exist_ok=True)
    sust = Path(args.sustainability_json)

    fig_latency_boxplot(exp, out / "latency_boxplot.png")
    fig_latency_cdf(exp, out / "latency_cdf.png")
    fig_latency_scatter(exp, out / "latency_trial_scatter.png")
    if sust.exists():
        fig_wire_sizes(sust, out / "wire_sizes_bar.png")
        fig_pod_resources(sust, out / "pod_resources_bar.png")
    fig_ablation(exp / "summary/ablation_metrics.json", out / "ablation_comparison.png")
    print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
