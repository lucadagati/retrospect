#!/usr/bin/env python3
"""
Generate scalability plots for RETROSPECT deliverable
Creates English-labeled plots from collected metrics
"""

import json
import glob
import matplotlib.pyplot as plt
import matplotlib
import numpy as np
from datetime import datetime

# Use Agg backend for headless operation
matplotlib.use('Agg')

# Set font sizes for publication quality
plt.rcParams.update({
    'font.size': 12,
    'axes.labelsize': 14,
    'axes.titlesize': 16,
    'xtick.labelsize': 12,
    'ytick.labelsize': 12,
    'legend.fontsize': 11,
    'figure.titlesize': 18
})

def load_all_metrics(directory='/home/lucadag/18_10_23_retrospect'):
    """Load all metrics JSON files"""
    pattern = f'{directory}/scalability_metrics_*.json'
    files = sorted(glob.glob(pattern))
    
    metrics_list = []
    for file in files:
        with open(file, 'r') as f:
            metrics = json.load(f)
            metrics['filename'] = file
            metrics_list.append(metrics)
    
    return metrics_list

def plot_device_count_comparison(metrics_list, output_dir):
    """Plot device count vs number of gateways"""
    fig, ax = plt.subplots(figsize=(10, 6))
    
    scenarios = []
    device_counts = []
    gateway_counts = []
    devices_per_gateway = []
    
    for metrics in metrics_list:
        config = metrics['config']
        scenarios.append(metrics['scenario'])
        device_counts.append(config['num_devices'])
        gateway_counts.append(config['num_gateways'])
        devices_per_gateway.append(config['devices_per_gateway'])
    
    # Bar chart showing device distribution
    x = np.arange(len(scenarios))
    width = 0.35
    
    bars1 = ax.bar(x - width/2, device_counts, width, label='Total Devices', color='#2E86AB')
    bars2 = ax.bar(x + width/2, gateway_counts, width, label='Gateways', color='#A23B72')
    
    ax.set_xlabel('Scenario', fontweight='bold')
    ax.set_ylabel('Count', fontweight='bold')
    ax.set_title('WASMBED Scalability Test Configuration', fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels([s.replace('_', '\n') for s in scenarios])
    ax.legend()
    ax.grid(True, alpha=0.3, linestyle='--')
    
    # Add value labels on bars
    for bars in [bars1, bars2]:
        for bar in bars:
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                   f'{int(height)}',
                   ha='center', va='bottom', fontsize=10)
    
    plt.tight_layout()
    output_file = f'{output_dir}/scalability_configuration.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()

def plot_devices_per_gateway(metrics_list, output_dir):
    """Plot devices per gateway ratio"""
    fig, ax = plt.subplots(figsize=(10, 6))
    
    scenarios = []
    devices_per_gw = []
    
    for metrics in metrics_list:
        config = metrics['config']
        scenarios.append(metrics['scenario'])
        devices_per_gw.append(config['devices_per_gateway'])
    
    bars = ax.bar(scenarios, devices_per_gw, color='#F18F01', edgecolor='black', linewidth=1.5)
    
    ax.set_xlabel('Scenario', fontweight='bold')
    ax.set_ylabel('Devices per Gateway', fontweight='bold')
    ax.set_title('Device-to-Gateway Ratio Analysis', fontweight='bold')
    ax.set_xticklabels([s.replace('_', '\n') for s in scenarios])
    ax.grid(True, alpha=0.3, linestyle='--', axis='y')
    ax.axhline(y=20, color='red', linestyle='--', alpha=0.7, label='Target: 20 devices/gateway')
    ax.legend()
    
    # Add value labels
    for bar in bars:
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height,
               f'{height:.1f}',
               ha='center', va='bottom', fontsize=11, fontweight='bold')
    
    plt.tight_layout()
    output_file = f'{output_dir}/devices_per_gateway.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()

def plot_api_latency(metrics_list, output_dir):
    """Plot API latency percentiles (avg, p95, p99) per scenario if available"""
    # Filter scenarios that have api latency in summary
    data = [m for m in metrics_list if 'summary' in m and 'api_latency_ms' in m['summary']]
    if not data:
        print("No API latency data found in metrics; skipping latency plot.")
        return
    
    scenarios = [m['scenario'] for m in data]
    avg = [m['summary']['api_latency_ms']['avg'] for m in data]
    p95 = [m['summary']['api_latency_ms']['p95'] for m in data]
    p99 = [m['summary']['api_latency_ms']['p99'] for m in data]
    
    x = np.arange(len(scenarios))
    width = 0.25
    
    fig, ax = plt.subplots(figsize=(12, 6))
    b1 = ax.bar(x - width, avg, width, label='Avg (ms)', color='#2E86AB')
    b2 = ax.bar(x, p95, width, label='P95 (ms)', color='#F18F01')
    b3 = ax.bar(x + width, p99, width, label='P99 (ms)', color='#A23B72')
    
    ax.set_xlabel('Scenario', fontweight='bold')
    ax.set_ylabel('Latency (ms)', fontweight='bold')
    ax.set_title('API Latency Percentiles for GET /devices', fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels([s.replace('_', '\n') for s in scenarios])
    ax.grid(True, alpha=0.3, axis='y', linestyle='--')
    ax.legend()
    
    for bars in [b1, b2, b3]:
        for bar in bars:
            h = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., h, f"{h:.0f}", ha='center', va='bottom', fontsize=10)
    
    plt.tight_layout()
    output_file = f'{output_dir}/api_latency_percentiles.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()

def plot_api_throughput(metrics_list, output_dir):
    """Plot average API throughput (req/s) per scenario if available"""
    data = [m for m in metrics_list if 'summary' in m and 'throughput_rps_avg' in m['summary']]
    if not data:
        print("No API throughput data found in metrics; skipping throughput plot.")
        return
    
    scenarios = [m['scenario'] for m in data]
    thr = [m['summary']['throughput_rps_avg'] for m in data]
    
    fig, ax = plt.subplots(figsize=(10, 6))
    bars = ax.bar(scenarios, thr, color='#06A77D', edgecolor='black', linewidth=1.5)
    ax.set_xlabel('Scenario', fontweight='bold')
    ax.set_ylabel('Throughput (requests/sec)', fontweight='bold')
    ax.set_title('API Throughput for GET /devices', fontweight='bold')
    ax.set_xticklabels([s.replace('_', '\n') for s in scenarios])
    ax.grid(True, alpha=0.3, linestyle='--', axis='y')
    for bar in bars:
        h = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., h, f"{h:.2f}", ha='center', va='bottom', fontsize=11, fontweight='bold')
    plt.tight_layout()
    output_file = f'{output_dir}/api_throughput.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()

def plot_resource_efficiency(metrics_list, output_dir):
    """Plot average CPU(m) and Memory(Mi) for devices and gateways per scenario"""
    data = [m for m in metrics_list if 'summary' in m and 'resources' in m['summary']]
    if not data:
        print("No resource data found; skipping CPU/Memory plots.")
        return
    scenarios = [m['scenario'] for m in data]
    dev_cpu = [m['summary']['resources'].get('device_cpu_m_avg', 0) for m in data]
    gw_cpu = [m['summary']['resources'].get('gateway_cpu_m_avg', 0) for m in data]
    dev_mem = [m['summary']['resources'].get('device_mem_Mi_avg', 0) for m in data]
    gw_mem = [m['summary']['resources'].get('gateway_mem_Mi_avg', 0) for m in data]

    x = np.arange(len(scenarios))
    width = 0.2

    # CPU plot
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.bar(x - width/2, dev_cpu, width, label='Devices CPU (m)', color='#2E86AB')
    ax.bar(x + width/2, gw_cpu, width, label='Gateways CPU (m)', color='#A23B72')
    ax.set_xlabel('Scenario', fontweight='bold')
    ax.set_ylabel('CPU (millicores)', fontweight='bold')
    ax.set_title('Average CPU Usage by Scenario')
    ax.set_xticks(x)
    ax.set_xticklabels([s.replace('_', '\n') for s in scenarios])
    ax.legend()
    ax.grid(True, alpha=0.3, axis='y', linestyle='--')
    plt.tight_layout()
    out1 = f'{output_dir}/cpu_usage.png'
    plt.savefig(out1, dpi=300, bbox_inches='tight')
    print(f"Saved: {out1}")
    plt.close()

    # Memory plot
    fig, ax = plt.subplots(figsize=(12, 6))
    ax.bar(x - width/2, dev_mem, width, label='Devices Memory (Mi)', color='#06A77D')
    ax.bar(x + width/2, gw_mem, width, label='Gateways Memory (Mi)', color='#F18F01')
    ax.set_xlabel('Scenario', fontweight='bold')
    ax.set_ylabel('Memory (Mi)', fontweight='bold')
    ax.set_title('Average Memory Usage by Scenario')
    ax.set_xticks(x)
    ax.set_xticklabels([s.replace('_', '\n') for s in scenarios])
    ax.legend()
    ax.grid(True, alpha=0.3, axis='y', linestyle='--')
    plt.tight_layout()
    out2 = f'{output_dir}/memory_usage.png'
    plt.savefig(out2, dpi=300, bbox_inches='tight')
    print(f"Saved: {out2}")
    plt.close()

def plot_pod_count_timeline(metrics_list, output_dir):
    """Plot pod count over time for each scenario"""
    fig, axes = plt.subplots(len(metrics_list), 1, figsize=(12, 4 * len(metrics_list)))
    
    if len(metrics_list) == 1:
        axes = [axes]
    
    for idx, metrics in enumerate(metrics_list):
        ax = axes[idx]
        scenario = metrics['scenario']
        samples = metrics['samples']
        
        times = [s['elapsed_time'] / 60 for s in samples]  # Convert to minutes
        pod_counts = [s['pod_count'] for s in samples]
        
        ax.plot(times, pod_counts, marker='o', linewidth=2, markersize=4, color='#06A77D')
        ax.set_xlabel('Elapsed Time (minutes)', fontweight='bold')
        ax.set_ylabel('Active Pods', fontweight='bold')
        ax.set_title(f'Pod Count Over Time: {scenario.replace("_", " ").title()}', fontweight='bold')
        ax.grid(True, alpha=0.3)
        
        # Add expected count line
        expected_pods = metrics['config']['num_devices'] + metrics['config']['num_gateways']
        ax.axhline(y=expected_pods, color='red', linestyle='--', alpha=0.7, 
                  label=f'Expected: {expected_pods} pods')
        ax.legend()
    
    plt.tight_layout()
    output_file = f'{output_dir}/pod_count_timeline.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()

def plot_summary_table(metrics_list, output_dir):
    """Create a summary table as an image"""
    fig, ax = plt.subplots(figsize=(14, 3 + len(metrics_list) * 0.5))
    ax.axis('tight')
    ax.axis('off')
    
    # Prepare table data
    headers = ['Scenario', 'Devices', 'Gateways', 'Devices/Gateway', 
               'Total Pods', 'Samples', 'Enrollment Rate']
    
    table_data = []
    for metrics in metrics_list:
        config = metrics['config']
        summary = metrics['summary']
        
        # Calculate enrollment rate
        enrolled_devices = summary.get('final_connection_states', {}).get('Enrolled', 0)
        enrollment_rate = (enrolled_devices / config['num_devices'] * 100) if config['num_devices'] > 0 else 0
        
        row = [
            metrics['scenario'].replace('_', ' ').title(),
            str(config['num_devices']),
            str(config['num_gateways']),
            f"{config['devices_per_gateway']:.1f}",
            f"{config['num_devices'] + config['num_gateways']}",
            str(summary['total_samples']),
            f"{enrollment_rate:.0f}%"
        ]
        table_data.append(row)
    
    table = ax.table(cellText=table_data, colLabels=headers, cellLoc='center',
                     loc='center', colWidths=[0.2, 0.1, 0.1, 0.15, 0.1, 0.1, 0.15])
    
    table.auto_set_font_size(False)
    table.set_fontsize(11)
    table.scale(1, 2.5)
    
    # Style header
    for i in range(len(headers)):
        table[(0, i)].set_facecolor('#2E86AB')
        table[(0, i)].set_text_props(weight='bold', color='white')
    
    # Style data rows
    for i in range(1, len(table_data) + 1):
        for j in range(len(headers)):
            if i % 2 == 0:
                table[(i, j)].set_facecolor('#E8F4F8')
            else:
                table[(i, j)].set_facecolor('white')
    
    plt.title('WASMBED Scalability Test Results Summary', 
             fontsize=16, fontweight='bold', pad=20)
    
    plt.tight_layout()
    output_file = f'{output_dir}/scalability_summary_table.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()

def plot_gateway_distribution(metrics_list, output_dir):
    """Plot how devices are distributed across gateways"""
    fig, axes = plt.subplots(1, len(metrics_list), figsize=(6 * len(metrics_list), 5))
    
    if len(metrics_list) == 1:
        axes = [axes]
    
    for idx, metrics in enumerate(metrics_list):
        ax = axes[idx]
        scenario = metrics['scenario']
        distribution = metrics['summary']['final_gateway_distribution']
        
        gateways = list(distribution.keys())
        device_counts = list(distribution.values())
        
        colors = plt.cm.Set3(np.linspace(0, 1, len(gateways)))
        bars = ax.bar(gateways, device_counts, color=colors, edgecolor='black', linewidth=1.5)
        
        ax.set_xlabel('Gateway ID', fontweight='bold')
        ax.set_ylabel('Number of Devices', fontweight='bold')
        ax.set_title(f'{scenario.replace("_", " ").title()}', fontweight='bold')
        ax.grid(True, alpha=0.3, linestyle='--', axis='y')
        
        # Add value labels
        for bar in bars:
            height = bar.get_height()
            ax.text(bar.get_x() + bar.get_width()/2., height,
                   f'{int(height)}',
                   ha='center', va='bottom', fontsize=10, fontweight='bold')
    
    plt.suptitle('Device Distribution Across Gateways', fontsize=16, fontweight='bold', y=1.02)
    plt.tight_layout()
    output_file = f'{output_dir}/gateway_distribution.png'
    plt.savefig(output_file, dpi=300, bbox_inches='tight')
    print(f"Saved: {output_file}")
    plt.close()

def main():
    """Main execution"""
    print("="*60)
    print("WASMBED Scalability Visualization Generator")
    print("="*60)
    
    # Load metrics
    metrics_list = load_all_metrics()
    print(f"\nLoaded {len(metrics_list)} metrics files:")
    for m in metrics_list:
        print(f"  - {m['scenario']}: {m['config']['num_devices']} devices, {m['config']['num_gateways']} gateways")
    
    # Output directory
    output_dir = '/home/lucadag/18_10_23_retrospect/RETROSPECT_Deliverable_D_4_2/images'
    
    print(f"\nGenerating plots to: {output_dir}")
    print("-"*60)
    
    # Generate plots
    plot_device_count_comparison(metrics_list, output_dir)
    plot_devices_per_gateway(metrics_list, output_dir)
    plot_pod_count_timeline(metrics_list, output_dir)
    plot_summary_table(metrics_list, output_dir)
    plot_gateway_distribution(metrics_list, output_dir)
    # New performance plots
    plot_api_latency(metrics_list, output_dir)
    plot_api_throughput(metrics_list, output_dir)
    plot_resource_efficiency(metrics_list, output_dir)
    
    print("-"*60)
    print("\n✓ All plots generated successfully!")
    print("="*60)

if __name__ == '__main__':
    main()
