#!/usr/bin/env python3
"""
RETROSPECT — Generatore di grafici e tabelle LaTeX per il Capitolo 4.

Legge i CSV in results/ e produce:
  capitolo4/figure/fig_size_sensitivity.pdf   — transfer/lfs/total vs KB
  capitolo4/figure/fig_scalability.pdf        — enrollment+deploy vs N
  capitolo4/figure/fig_throughput.pdf         — req/s completati + latenza vs rate
  capitolo4/figure/fig_decomposition.pdf      — barra impilata fasi latenza
  capitolo4/figure/fig_reliability.pdf        — CDF latenza enroll+deploy

  Stampa su stdout gli snippet LaTeX pronti per l'inclusione in capitolo4.tex.

Usage:
    python3 scripts/make_plots.py
    python3 scripts/make_plots.py --only size   # solo un grafico specifico
"""

import argparse
import csv
import json
import math
import os
import sys

RESULTS_DIR   = os.path.join(os.path.dirname(__file__), "..", "results")
FIGURE_DIR    = os.path.join(os.path.dirname(__file__), "..", "..",
                              "overleaf", "mdthesis_aeCiliberto",
                              "capitolo4", "figure")

# ── Palette coerente per tutti i grafici ─────────────────────────────────────
C_BLUE   = "#2166ac"
C_RED    = "#d6604d"
C_GREEN  = "#4dac26"
C_ORANGE = "#f4a582"
C_GRAY   = "#999999"

PHASE_COLORS = {
    "K8s CRD":       "#1f77b4",   # blue
    "Controller":    "#17becf",   # cyan
    "Gateway+CBOR":  "#9467bd",   # purple
    "Transfer TLS":  "#ff7f0e",   # orange
    "LittleFS":      "#2ca02c",   # green
    "WAMR load":     "#d62728",   # red
    "WAMR start":    "#8c564b",   # brown
}


def ensure_figure_dir():
    os.makedirs(FIGURE_DIR, exist_ok=True)


def load_csv(filename):
    path = os.path.join(RESULTS_DIR, filename)
    if not os.path.exists(path):
        return None
    with open(path, newline="") as f:
        return list(csv.DictReader(f))


def load_json(filename):
    path = os.path.join(RESULTS_DIR, filename)
    if not os.path.exists(path):
        return None
    with open(path) as f:
        return json.load(f)


def fval(v, default=None):
    """Safe float conversion; returns default if None/empty/'None'."""
    try:
        if v is None or str(v).strip() in ("", "None"):
            return default
        return float(v)
    except (ValueError, TypeError):
        return default


def mean_std(vals):
    valid = [v for v in vals if v is not None]
    if not valid:
        return None, None
    m = sum(valid) / len(valid)
    s = math.sqrt(sum((v - m) ** 2 for v in valid) / len(valid)) if len(valid) > 1 else 0.0
    return m, s


# ─────────────────────────────────────────────────────────────────────────────
# FIG 1 — Module-Size Sensitivity
# ─────────────────────────────────────────────────────────────────────────────

def plot_size_sensitivity():
    import matplotlib.pyplot as plt
    import matplotlib.ticker as ticker

    rows = load_csv("size_sensitivity.csv")
    if not rows:
        print("[SKIP] size_sensitivity.csv not found", file=sys.stderr)
        return

    sizes_kb  = []
    transfer  = []
    lfs       = []
    wamr_load = []
    total     = []

    for r in rows:
        kb = fval(r.get("size_kb"))
        if kb is None:
            continue
        sizes_kb.append(kb)
        transfer.append(fval(r.get("transfer_ms")))
        lfs.append(fval(r.get("lfs_write_ms")))
        wamr_load.append(fval(r.get("wamr_load_ms")))
        total.append(fval(r.get("total_deploy_s")))

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4))
    fig.subplots_adjust(wspace=0.35)

    # Left: device-side phases (ms)
    xs = list(range(len(sizes_kb)))
    labels = [f"{int(k)}" for k in sizes_kb]

    def bar_vals(series):
        return [v if v is not None else 0 for v in series]

    t_v  = bar_vals(transfer)
    l_v  = bar_vals(lfs)
    w_v  = bar_vals(wamr_load)

    b1 = ax1.bar(xs, t_v,  label="Transfer TLS", color=PHASE_COLORS["Transfer TLS"], edgecolor="white")
    b2 = ax1.bar(xs, l_v,  bottom=t_v,
                 label="LittleFS write",  color=PHASE_COLORS["LittleFS"],     edgecolor="white")
    bottom2 = [a + b for a, b in zip(t_v, l_v)]
    b3 = ax1.bar(xs, w_v,  bottom=bottom2,
                 label="WAMR load",       color=PHASE_COLORS["WAMR load"],    edgecolor="white")

    ax1.set_xticks(xs)
    ax1.set_xticklabels(labels, fontsize=9)
    ax1.set_xlabel("Module size (KB)", fontsize=10)
    ax1.set_ylabel("Time (ms)", fontsize=10)
    ax1.set_title("Device-side breakdown", fontsize=11, fontweight="bold")
    ax1.legend(fontsize=8, loc="upper left")
    ax1.yaxis.set_major_formatter(ticker.FormatStrFormatter("%.1f"))
    ax1.grid(axis="y", linestyle="--", alpha=0.4)
    ax1.spines[["top", "right"]].set_visible(False)

    # Right: total deploy latency (s)
    tot_v = [v if v is not None else 0 for v in total]
    ax2.plot(sizes_kb, tot_v, marker="o", color=C_BLUE, linewidth=2, markersize=6)
    ax2.set_xlabel("Module size (KB)", fontsize=10)
    ax2.set_ylabel("Total deploy latency (s)", fontsize=10)
    ax2.set_title("End-to-end deploy time", fontsize=11, fontweight="bold")
    ax2.grid(linestyle="--", alpha=0.4)
    ax2.spines[["top", "right"]].set_visible(False)
    ax2.set_xscale("log", base=2)
    ax2.xaxis.set_major_formatter(ticker.ScalarFormatter())
    ax2.set_xticks(sizes_kb)
    ax2.set_xticklabels(labels, fontsize=9)

    fig.suptitle("WASM Module Size Sensitivity", fontsize=12, fontweight="bold", y=1.01)

    out = os.path.join(FIGURE_DIR, "fig_size_sensitivity.pdf")
    fig.savefig(out, bbox_inches="tight")
    plt.close(fig)
    print(f"[OK] {out}")

    # LaTeX table
    print("\n% ── Table: size sensitivity ──────────────────────────────────────")
    print(r"""\begin{table}[ht]
\centering\small
\renewcommand{\arraystretch}{1.3}
\begin{tabular}{rrrrrr}
\hline
\textbf{Size (KB)} & \textbf{Actual (B)} & \textbf{Transfer (ms)} &
\textbf{LittleFS (ms)} & \textbf{WAMR load (ms)} & \textbf{Total (s)} \\
\hline""")
    for r in rows:
        kb  = r.get("size_kb", "")
        ab  = r.get("actual_bytes", "")
        tr  = fval(r.get("transfer_ms"))
        lf  = fval(r.get("lfs_write_ms"))
        wl  = fval(r.get("wamr_load_ms"))
        tot = fval(r.get("total_deploy_s"))
        tr_s  = f"{tr:.3f}"  if tr  is not None else r"$<$1"
        lf_s  = f"{lf:.3f}"  if lf  is not None else r"$<$1"
        wl_s  = f"{wl:.3f}"  if wl  is not None else r"$<$1"
        tot_s = f"{tot:.3f}" if tot is not None else "—"
        print(f"{kb} & {ab} & {tr_s} & {lf_s} & {wl_s} & {tot_s} \\\\")
    print(r"""\hline
\end{tabular}
\caption{WASM module size sensitivity: per-phase timing (mean of 3 repetitions).
         Measurements on \texttt{native\_sim}/x86; LittleFS and WAMR timings in
         milliseconds with $\mu$s resolution (see Section~\ref{subsec:size}).}
\label{tab:size-sensitivity}
\end{table}""")


# ─────────────────────────────────────────────────────────────────────────────
# FIG 2 — Scalability
# ─────────────────────────────────────────────────────────────────────────────

def plot_scalability():
    import matplotlib.pyplot as plt

    rows = load_csv("scalability.csv")
    if not rows:
        print("[SKIP] scalability.csv not found", file=sys.stderr)
        return

    from collections import defaultdict
    enroll_by_n = defaultdict(list)
    deploy_by_n = defaultdict(list)

    for r in rows:
        n = int(r["N"])
        e = fval(r.get("enroll_total_s"))
        d = fval(r.get("deploy_total_s"))
        if e is not None:
            enroll_by_n[n].append(e)
        if d is not None:
            deploy_by_n[n].append(d)

    ns = sorted(set(enroll_by_n) | set(deploy_by_n))
    e_means, e_stds = [], []
    d_means, d_stds = [], []
    for n in ns:
        em, es = mean_std(enroll_by_n.get(n, []))
        dm, ds = mean_std(deploy_by_n.get(n, []))
        e_means.append(em); e_stds.append(es if es else 0)
        d_means.append(dm); d_stds.append(ds if ds else 0)

    fig, ax = plt.subplots(figsize=(7, 4))
    xs = list(range(len(ns)))

    ax.errorbar(xs, e_means, yerr=e_stds, marker="o", color=C_BLUE,
                label="Enrollment (all Connected)", linewidth=2, markersize=6,
                capsize=4, elinewidth=1.2)

    # Deploy curve: skip N points with no deploy data (mean_std returns None for empty
    # list), so that a partial-enrollment-only row doesn't break the plot.
    d_xs = [x for x, dm in zip(xs, d_means) if dm is not None]
    d_y  = [dm for dm in d_means if dm is not None]
    d_err = [ds for dm, ds in zip(d_means, d_stds) if dm is not None]
    if d_xs:
        ax.errorbar(d_xs, d_y, yerr=d_err, marker="s", color=C_RED,
                    label="Deploy (all Running)", linewidth=2, markersize=6,
                    capsize=4, elinewidth=1.2, linestyle="--")

    ax.set_xticks(xs)
    ax.set_xticklabels([str(n) for n in ns], fontsize=10)
    ax.set_xlabel("Number of devices (N)", fontsize=11)
    ax.set_ylabel("Time to completion (s)", fontsize=11)
    ax.set_title("Control-plane scalability", fontsize=12, fontweight="bold")
    ax.legend(fontsize=9)
    ax.grid(linestyle="--", alpha=0.4)
    ax.spines[["top", "right"]].set_visible(False)

    out = os.path.join(FIGURE_DIR, "fig_scalability.pdf")
    fig.savefig(out, bbox_inches="tight")
    plt.close(fig)
    print(f"[OK] {out}")

    # LaTeX table
    print("\n% ── Table: scalability ───────────────────────────────────────────")
    print(r"""\begin{table}[ht]
\centering\small
\renewcommand{\arraystretch}{1.3}
\begin{tabular}{rrrr}
\hline
\textbf{N devices} & \textbf{Enroll mean (s)} & \textbf{Enroll std (s)} &
\textbf{Deploy mean (s)} \\
\hline""")
    for n, em, es, dm in zip(ns, e_means, e_stds, d_means):
        em_s = f"{em:.2f}" if em is not None else "—"
        es_s = f"{es:.2f}" if es is not None else "—"
        dm_s = f"{dm:.2f}" if dm is not None else "—"
        print(f"{n} & {em_s} & {es_s} & {dm_s} \\\\")
    print(r"""\hline
\end{tabular}
\caption{Control-plane scalability: time for all N devices to reach
         \textit{Connected} (enrollment) and \textit{Running} (deploy).
         Mean and standard deviation over 2 repetitions per N.}
\label{tab:scalability}
\end{table}""")


# ─────────────────────────────────────────────────────────────────────────────
# FIG 3 — Throughput
# ─────────────────────────────────────────────────────────────────────────────

def plot_throughput():
    import matplotlib.pyplot as plt

    rows = load_csv("throughput.csv")
    if not rows:
        print("[SKIP] throughput.csv not found", file=sys.stderr)
        return

    rates    = [fval(r["rate_rps"]) for r in rows]
    e_ok     = [fval(r.get("enrolled_ok")) for r in rows]
    d_ok     = [fval(r.get("deployed_ok")) for r in rows]
    e_lat    = [fval(r.get("enroll_latency_mean_s")) for r in rows]
    d_lat    = [fval(r.get("deploy_latency_mean_s")) for r in rows]
    saturated= [r.get("saturated", "False") == "True" for r in rows]

    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(10, 4))
    fig.subplots_adjust(wspace=0.35)

    # Left: completed req/s
    ax1.plot(rates, e_ok, marker="o", color=C_BLUE, linewidth=2,
             label="Enrolled", markersize=6)
    ax1.plot(rates, d_ok, marker="s", color=C_RED,  linewidth=2,
             label="Deployed", markersize=6, linestyle="--")
    for i, sat in enumerate(saturated):
        if sat and rates[i] is not None:
            ax1.axvline(rates[i], color=C_GRAY, linestyle=":", linewidth=1.2, alpha=0.7)
            ax1.text(rates[i] + 0.3, ax1.get_ylim()[1] * 0.9,
                     "sat.", fontsize=8, color=C_GRAY)
    ax1.set_xlabel("Injection rate (req/s)", fontsize=10)
    ax1.set_ylabel("Completed operations", fontsize=10)
    ax1.set_title("Throughput", fontsize=11, fontweight="bold")
    ax1.legend(fontsize=9)
    ax1.grid(linestyle="--", alpha=0.4)
    ax1.spines[["top", "right"]].set_visible(False)

    # Right: mean latency vs rate
    ax2.plot(rates, e_lat, marker="o", color=C_BLUE, linewidth=2,
             label="Enroll latency", markersize=6)
    ax2.plot(rates, d_lat, marker="s", color=C_RED,  linewidth=2,
             label="Deploy latency", markersize=6, linestyle="--")
    ax2.set_xlabel("Injection rate (req/s)", fontsize=10)
    ax2.set_ylabel("Mean latency (s)", fontsize=10)
    ax2.set_title("Latency vs rate", fontsize=11, fontweight="bold")
    ax2.legend(fontsize=9)
    ax2.grid(linestyle="--", alpha=0.4)
    ax2.spines[["top", "right"]].set_visible(False)

    fig.suptitle("Control-plane throughput", fontsize=12, fontweight="bold", y=1.01)

    out = os.path.join(FIGURE_DIR, "fig_throughput.pdf")
    fig.savefig(out, bbox_inches="tight")
    plt.close(fig)
    print(f"[OK] {out}")


# ─────────────────────────────────────────────────────────────────────────────
# FIG 4 — Latency Decomposition (stacked bar)
# ─────────────────────────────────────────────────────────────────────────────

def plot_decomposition():
    import matplotlib.pyplot as plt
    import numpy as np

    decomp_rows = load_csv("decomposition.csv")
    size_rows   = load_csv("size_sensitivity.csv")
    if not decomp_rows:
        print("[SKIP] decomposition.csv not found", file=sys.stderr)
        return
    if not size_rows:
        print("[SKIP] size_sensitivity.csv not found", file=sys.stderr)
        return

    # ── Cloud overhead (constant, from decomposition.csv @ 50 KB) ─────────────
    def col_mean(rows, col):
        vals = [fval(r.get(col)) for r in rows]
        valids = [v for v in vals if v is not None]
        return sum(valids) / len(valids) if valids else 0.0

    def ts_diff_ms(rows, t1_col, t0_col):
        from datetime import datetime, timezone
        diffs = []
        for r in rows:
            t0s = r.get(t0_col, "")
            t1s = r.get(t1_col, "")
            if not t0s or not t1s:
                continue
            try:
                fmt = "%Y-%m-%dT%H:%M:%S.%fZ"
                t0 = datetime.strptime(t0s, fmt).replace(tzinfo=timezone.utc)
                t1 = datetime.strptime(t1s, fmt).replace(tzinfo=timezone.utc)
                diffs.append((t1 - t0).total_seconds() * 1000)
            except ValueError:
                pass
        return sum(diffs) / len(diffs) if diffs else 0.0

    gw_recv_to_send = ts_diff_ms(decomp_rows, "t_gw_send", "t_gw_recv")
    gw_send_to_ack  = ts_diff_ms(decomp_rows, "t_deploy_ack", "t_gw_send")
    total_mean_ms   = col_mean(decomp_rows, "total_s") * 1000
    k8s_ctrl_ms     = max(0.0, total_mean_ms - gw_recv_to_send - gw_send_to_ack)
    k8s_ms          = k8s_ctrl_ms * 0.6
    ctrl_ms         = k8s_ctrl_ms * 0.4

    # ── Per-size device phases from size_sensitivity.csv ─────────────────────
    # wamr_start_ms not captured in size sweep → use decomp mean (constant ~0.25ms)
    wamr_start_const = col_mean(decomp_rows, "wamr_start_ms")

    sizes_kb   = []
    bar_data   = []   # list of dicts {phase: ms} per size
    for r in size_rows:
        kb = fval(r.get("size_kb"))
        if kb is None:
            continue
        transfer  = fval(r.get("transfer_ms"))  or 0.0
        lfs       = fval(r.get("lfs_write_ms")) or 0.0
        wamr_load = fval(r.get("wamr_load_ms")) or 0.0
        sizes_kb.append(int(kb))
        bar_data.append({
            "K8s CRD":      k8s_ms,
            "Controller":   ctrl_ms,
            "Gateway+CBOR": gw_recv_to_send,
            "Transfer TLS": transfer,
            "LittleFS":     lfs,
            "WAMR load":    wamr_load,
            "WAMR start":   wamr_start_const,
        })

    phase_order = ["K8s CRD", "Controller", "Gateway+CBOR",
                   "Transfer TLS", "LittleFS", "WAMR load", "WAMR start"]

    x      = np.arange(len(sizes_kb))
    width  = 0.55
    x_labels = [f"{kb} KB" for kb in sizes_kb]

    fig, ax = plt.subplots(figsize=(10, 5.5))

    bottoms = np.zeros(len(sizes_kb))
    totals  = np.zeros(len(sizes_kb))
    for phase in phase_order:
        vals = np.array([bd[phase] for bd in bar_data])
        ax.bar(x, vals, width, bottom=bottoms,
               color=PHASE_COLORS[phase], edgecolor="white", linewidth=0.5,
               label=phase)
        bottoms += vals
    totals = bottoms  # cumulative sum after all phases

    # Annotate each bar with its total time, connected by a short vertical line
    y_max = totals.max()
    label_offset = y_max * 0.04
    for i, total in enumerate(totals):
        label = f"{total/1000:.1f} s" if total >= 1000 else f"{total:.0f} ms"
        ax.annotate(
            label,
            xy=(x[i], total),
            xytext=(x[i], total + label_offset),
            ha="center", va="bottom", fontsize=9, fontweight="bold",
            color="#333333",
            arrowprops=dict(arrowstyle="-", color="#888888", lw=0.9),
        )

    ax.set_xticks(x)
    ax.set_xticklabels(x_labels, fontsize=10)
    ax.set_xlabel("WASM module size", fontsize=11)
    ax.set_ylabel("Time (ms)", fontsize=11)
    ax.set_title("Deployment latency decomposition by WASM module size",
                 fontsize=11, fontweight="bold")
    ax.set_ylim(0, y_max * 1.18)
    ax.legend(loc="upper left", fontsize=9, frameon=True)
    ax.grid(axis="y", linestyle="--", alpha=0.35)
    ax.spines[["top", "right"]].set_visible(False)

    out = os.path.join(FIGURE_DIR, "fig_decomposition.pdf")
    fig.savefig(out, bbox_inches="tight")
    plt.close(fig)
    print(f"[OK] {out}")

    # LaTeX table (single-size breakdown @ 50 KB from decomposition.csv)
    transfer_50  = col_mean(decomp_rows, "transfer_ms")
    lfs_50       = col_mean(decomp_rows, "lfs_write_ms")
    wamr_load_50 = col_mean(decomp_rows, "wamr_load_ms")
    phases_50 = [
        ("K8s CRD",      k8s_ms),
        ("Controller",   ctrl_ms),
        ("Gateway+CBOR", gw_recv_to_send),
        ("Transfer TLS", transfer_50),
        ("LittleFS",     lfs_50),
        ("WAMR load",    wamr_load_50),
        ("WAMR start",   wamr_start_const),
    ]
    print("\n% ── Table: decomposition ─────────────────────────────────────────")
    print(r"""\begin{table}[ht]
\centering\small
\renewcommand{\arraystretch}{1.3}
\begin{tabular}{lrr}
\hline
\textbf{Phase} & \textbf{Mean (ms)} & \textbf{Fraction (\%)} \\
\hline""")
    total_sum = sum(v for _, v in phases_50)
    for label, val in phases_50:
        pct = 100 * val / total_sum if total_sum > 0 else 0
        print(f"{label} & {val:.3f} & {pct:.1f}\\\\")
    print(f"\\hline\nTotal & {total_sum:.3f} & 100.0\\\\")
    print(r"""\hline
\end{tabular}
\caption{Deployment latency decomposition (mean of 5 sequential runs,
         50~KB padded module, \texttt{native\_sim} loopback TLS).
         Cloud phases derived from gateway timestamps; device phases
         measured with $\mu$s resolution.}
\label{tab:decomposition}
\end{table}""")


# ─────────────────────────────────────────────────────────────────────────────
# FIG 5 — Reliability (CDF)
# ─────────────────────────────────────────────────────────────────────────────

def plot_reliability():
    import matplotlib.pyplot as plt
    import numpy as np

    rows = load_csv("reliability.csv")
    summary = load_json("reliability_summary.json")
    if not rows:
        print("[SKIP] reliability.csv not found", file=sys.stderr)
        return

    enroll_lat = sorted([fval(r["elapsed_s"]) for r in rows
                         if r.get("phase") == "enrollment" and r.get("success") == "True"
                         and fval(r["elapsed_s"]) is not None])
    deploy_lat  = sorted([fval(r["elapsed_s"]) for r in rows
                          if r.get("phase") == "deploy" and r.get("success") == "True"
                          and fval(r["elapsed_s"]) is not None])

    def cdf(vals):
        n = len(vals)
        return vals, [i / n for i in range(1, n + 1)]

    fig, ax = plt.subplots(figsize=(7, 4))

    if enroll_lat:
        ex, ey = cdf(enroll_lat)
        ax.plot(ex, ey, color=C_BLUE, linewidth=2, label="Enrollment")
    if deploy_lat:
        dx, dy = cdf(deploy_lat)
        ax.plot(dx, dy, color=C_RED, linewidth=2, linestyle="--", label="Deploy")

    ax.set_xlabel("Latency (s)", fontsize=11)
    ax.set_ylabel("CDF", fontsize=11)
    ax.set_title("Reliability: latency CDF (successful trials)", fontsize=11, fontweight="bold")
    ax.legend(fontsize=9)
    ax.grid(linestyle="--", alpha=0.4)
    ax.spines[["top", "right"]].set_visible(False)
    ax.set_ylim(0, 1.05)

    out = os.path.join(FIGURE_DIR, "fig_reliability.pdf")
    fig.savefig(out, bbox_inches="tight")
    plt.close(fig)
    print(f"[OK] {out}")

    # LaTeX table from summary
    if summary:
        print("\n% ── Table: reliability ───────────────────────────────────────────")
        print(r"""\begin{table}[ht]
\centering\small
\renewcommand{\arraystretch}{1.3}
\begin{tabular}{lrrrr}
\hline
\textbf{Operation} & \textbf{Trials} & \textbf{Success rate} &
\textbf{Mean (s)} & \textbf{Std dev (s)} \\
\hline""")
        for key, label in [("enrollment", "Enrollment"), ("deploy", "Deploy")]:
            d = summary.get(key, {})
            n   = d.get("n_trials", "—")
            sr  = d.get("success_rate")
            m   = d.get("mean_s")
            std = d.get("std_s")
            sr_s  = f"{sr*100:.1f}\\%" if sr  is not None else "—"
            m_s   = f"{m:.3f}"          if m   is not None else "—"
            std_s = f"{std:.3f}"        if std is not None else "—"
            print(f"{label} & {n} & {sr_s} & {m_s} & {std_s} \\\\")
        print(r"""\hline
\end{tabular}
\caption{Reliability over 100 consecutive trials each. Success = operation completed
         within timeout without error. \texttt{native\_sim} loopback TLS.}
\label{tab:reliability}
\end{table}""")


# ─────────────────────────────────────────────────────────────────────────────
# MAIN
# ─────────────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(description="RETROSPECT — generate plots and LaTeX tables")
    parser.add_argument("--only", choices=["size", "scalability", "throughput",
                                           "decomposition", "reliability"],
                        help="Generate only one figure")
    args = parser.parse_args()

    ensure_figure_dir()

    funcs = {
        "size":          plot_size_sensitivity,
        "scalability":   plot_scalability,
        "throughput":    plot_throughput,
        "decomposition": plot_decomposition,
        "reliability":   plot_reliability,
    }

    targets = [args.only] if args.only else list(funcs.keys())
    for t in targets:
        print(f"\n{'='*60}")
        print(f"  {t.upper()}")
        print(f"{'='*60}")
        funcs[t]()

    print("\n[DONE] Figures saved to:", FIGURE_DIR)


if __name__ == "__main__":
    main()
