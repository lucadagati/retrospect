#!/usr/bin/env python3
"""Generate publication figures for the RETROSPECT paper from experiment JSON."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt


COLOR_BLUE = "#0B4F6C"
COLOR_GOLD = "#C88A04"
COLOR_RED = "#B63A2B"
COLOR_GREEN = "#2A7F62"
COLOR_GRID = "#D9D9D9"


def load_payload(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def records_by_experiment(payload: dict, name: str) -> list[dict]:
    return [r for r in payload.get("records", []) if r.get("experiment") == name]


def transaction_latencies(payload: dict) -> list[float]:
    enrollment = {r["trial"]: r for r in records_by_experiment(payload, "enrollment")}
    deployment = {r["trial"]: r for r in records_by_experiment(payload, "deployment")}
    latencies = []
    for trial in sorted(set(enrollment) & set(deployment)):
        e = enrollment[trial].get("duration_ms")
        d = deployment[trial].get("duration_ms")
        if e is not None and d is not None:
            latencies.append(float(e) + float(d))
    return latencies


def save_latency_ci_plot(payload: dict, out_path: Path) -> None:
    summary = payload["summary"]
    labels = ["Enrollment", "Heartbeat", "Deployment", "End-to-end"]
    entries = [
        summary["enrollment"]["latency_ms"],
        summary["heartbeat"]["latency_ms"],
        summary["deployment"]["latency_ms"],
        summary["transactional"]["end_to_end_latency_ms"],
    ]

    means = [e["mean"] for e in entries]
    p95 = [e["p95"] for e in entries]
    lower = [e["mean"] - e["ci95"][0] for e in entries]
    upper = [e["ci95"][1] - e["mean"] for e in entries]
    colors = [COLOR_BLUE, COLOR_GOLD, COLOR_RED, COLOR_GREEN]

    fig, ax = plt.subplots(figsize=(7.2, 4.0), constrained_layout=True)
    x = list(range(len(labels)))
    bars = ax.bar(x, means, color=colors, width=0.6, edgecolor="black", linewidth=0.7)
    ax.errorbar(x, means, yerr=[lower, upper], fmt="none", ecolor="black", elinewidth=1.0, capsize=4)
    ax.scatter(x, p95, marker="^", color="black", s=38, zorder=3, label="p95")
    ax.set_yscale("log")
    ax.set_ylabel("Latency [ms] (log scale)")
    ax.set_xticks(x, labels)
    ax.grid(axis="y", which="both", color=COLOR_GRID, linewidth=0.8)
    ax.set_axisbelow(True)
    ax.legend(loc="upper left", frameon=True, fontsize=8)

    for bar, mean in zip(bars, means):
        ax.text(
            bar.get_x() + bar.get_width() / 2,
            mean,
            f"{mean:.1f}",
            ha="center",
            va="bottom",
            fontsize=8,
        )

    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path, dpi=220, bbox_inches="tight")
    plt.close(fig)


def save_latency_plot(payload: dict, out_path: Path) -> None:
    enrollment = [r["duration_ms"] for r in records_by_experiment(payload, "enrollment")]
    heartbeat = [r["duration_ms"] for r in records_by_experiment(payload, "heartbeat")]
    deployment = [r["duration_ms"] for r in records_by_experiment(payload, "deployment")]
    transactional = transaction_latencies(payload)

    data = [enrollment, heartbeat, deployment, transactional]
    labels = ["Enrollment", "Heartbeat", "Deployment", "End-to-end"]
    colors = [COLOR_BLUE, COLOR_GOLD, COLOR_RED, COLOR_GREEN]

    fig, ax = plt.subplots(figsize=(7.2, 4.2), constrained_layout=True)
    box = ax.boxplot(
        data,
        tick_labels=labels,
        patch_artist=True,
        medianprops={"color": "black", "linewidth": 1.2},
        boxprops={"linewidth": 1.0},
        whiskerprops={"linewidth": 1.0},
        capprops={"linewidth": 1.0},
        flierprops={"marker": "o", "markersize": 3, "markerfacecolor": "black", "alpha": 0.4},
    )

    for patch, color in zip(box["boxes"], colors):
        patch.set_facecolor(color)
        patch.set_alpha(0.78)

    ax.set_yscale("log")
    ax.set_ylabel("Latency [ms] (log scale)")
    ax.grid(axis="y", which="both", color=COLOR_GRID, linewidth=0.8)
    ax.set_axisbelow(True)

    means = [sum(vals) / len(vals) for vals in data]
    for idx, mean in enumerate(means, start=1):
        ax.scatter(idx, mean, marker="D", s=26, color="white", edgecolor="black", zorder=3)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path, dpi=220, bbox_inches="tight")
    plt.close(fig)


def save_latency_cdf_plot(payload: dict, out_path: Path) -> None:
    """Empirical CDF for enrollment, deployment, and end-to-end latencies."""
    import numpy as np

    enrollment = sorted(r["duration_ms"] for r in records_by_experiment(payload, "enrollment"))
    deployment = sorted(r["duration_ms"] for r in records_by_experiment(payload, "deployment"))
    transactional = sorted(transaction_latencies(payload))

    fig, ax = plt.subplots(figsize=(7.2, 4.0), constrained_layout=True)

    for vals, label, color in [
        (enrollment, "Enrollment", COLOR_BLUE),
        (deployment, "Deployment", COLOR_RED),
        (transactional, "End-to-end", COLOR_GREEN),
    ]:
        y = [(i + 1) / len(vals) for i in range(len(vals))]
        ax.step(vals, y, where="post", color=color, linewidth=1.8, label=label)

    ax.axhline(0.95, color="grey", linewidth=0.8, linestyle="--", label="p95")
    ax.axhline(0.99, color="grey", linewidth=0.8, linestyle=":", label="p99")

    ax.set_xlabel("Latency [ms]")
    ax.set_ylabel("Empirical CDF")
    ax.set_ylim(0, 1.05)
    ax.legend(loc="lower right", frameon=True, fontsize=8)
    ax.grid(color=COLOR_GRID, linewidth=0.8)
    ax.set_axisbelow(True)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path, dpi=220, bbox_inches="tight")
    plt.close(fig)


def save_trial_scatter_plot(payload: dict, out_path: Path) -> None:
    """Per-trial latency scatter to show stationarity across the campaign."""
    enrollment_map = {r["trial"]: r["duration_ms"] for r in records_by_experiment(payload, "enrollment")}
    deployment_map = {r["trial"]: r["duration_ms"] for r in records_by_experiment(payload, "deployment")}
    heartbeat_map = {r["trial"]: r["duration_ms"] for r in records_by_experiment(payload, "heartbeat")}

    trials_enr = sorted(enrollment_map)
    trials_dep = sorted(deployment_map)
    trials_hb = sorted(heartbeat_map)

    fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(7.2, 5.5), constrained_layout=True, sharex=False)

    # Top: enrollment and deployment on same axis
    ax1.scatter(trials_enr, [enrollment_map[t] for t in trials_enr],
                s=22, color=COLOR_BLUE, alpha=0.85, label="Enrollment", zorder=3)
    ax1.scatter(trials_dep, [deployment_map[t] for t in trials_dep],
                s=22, marker="s", color=COLOR_RED, alpha=0.85, label="Deployment", zorder=3)

    for trials, vals, color in [(trials_enr, enrollment_map, COLOR_BLUE), (trials_dep, deployment_map, COLOR_RED)]:
        mean_v = sum(vals[t] for t in trials) / len(trials)
        ax1.axhline(mean_v, color=color, linewidth=1.0, linestyle="--", alpha=0.6)

    ax1.set_ylabel("Latency [ms]")
    ax1.set_xlabel("Trial index")
    ax1.legend(loc="upper right", frameon=True, fontsize=8)
    ax1.grid(color=COLOR_GRID, linewidth=0.8)
    ax1.set_axisbelow(True)

    # Bottom: heartbeat (separate y-axis due to scale difference)
    ax2.scatter(trials_hb, [heartbeat_map[t] for t in trials_hb],
                s=22, marker="^", color=COLOR_GOLD, alpha=0.85, label="Heartbeat", zorder=3)
    mean_hb = sum(heartbeat_map.values()) / len(heartbeat_map)
    ax2.axhline(mean_hb, color=COLOR_GOLD, linewidth=1.0, linestyle="--", alpha=0.6)

    ax2.set_ylabel("Latency [ms]")
    ax2.set_xlabel("Trial index")
    ax2.legend(loc="upper right", frameon=True, fontsize=8)
    ax2.grid(color=COLOR_GRID, linewidth=0.8)
    ax2.set_axisbelow(True)

    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path, dpi=220, bbox_inches="tight")
    plt.close(fig)


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate RETROSPECT paper figures")
    parser.add_argument(
        "--input",
        default="../experiments/scalability_metrics_1778929086.json",
        help="Path to experiment JSON relative to this script or absolute",
    )
    parser.add_argument("--output-dir", default="figures", help="Output directory for figures")
    args = parser.parse_args()

    script_dir = Path(__file__).resolve().parent
    input_path = Path(args.input)
    if not input_path.is_absolute():
        input_path = (script_dir / input_path).resolve()
    output_dir = Path(args.output_dir)
    if not output_dir.is_absolute():
        output_dir = (script_dir / output_dir).resolve()

    payload = load_payload(input_path)
    save_latency_ci_plot(payload, output_dir / "latency_ci_profile.png")
    save_latency_plot(payload, output_dir / "latency_boxplot.png")
    save_latency_cdf_plot(payload, output_dir / "latency_cdf.png")
    save_trial_scatter_plot(payload, output_dir / "latency_trial_scatter.png")
    print(f"Wrote {output_dir / 'latency_ci_profile.png'}")
    print(f"Wrote {output_dir / 'latency_boxplot.png'}")
    print(f"Wrote {output_dir / 'latency_cdf.png'}")
    print(f"Wrote {output_dir / 'latency_trial_scatter.png'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
