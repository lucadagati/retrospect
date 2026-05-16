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
    print(f"Wrote {output_dir / 'latency_ci_profile.png'}")
    print(f"Wrote {output_dir / 'latency_boxplot.png'}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
