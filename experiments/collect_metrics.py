#!/usr/bin/env python3
"""
Scalability metrics collection script for WASMBED
Collects device statistics, gateway distribution, and resource usage
"""

import json
import time
import requests
import subprocess
from datetime import datetime
from collections import defaultdict
import statistics
import math

def get_devices_from_api():
    """Query API server for device information"""
    try:
        response = requests.get('http://localhost:3001/api/v1/devices', timeout=5)
        if response.status_code == 200:
            return response.json()
        else:
            print(f"API returned status {response.status_code}")
            return None
    except Exception as e:
        print(f"Error querying API: {e}")
        return None

def get_pod_metrics():
    """Get pod resource usage from kubectl"""
    try:
        # Get pod CPU/memory if metrics-server is available
        result = subprocess.run(
            ['kubectl', 'top', 'pods', '-n', 'wasmbed', '--no-headers'],
            capture_output=True,
            text=True,
            timeout=10
        )
        if result.returncode == 0:
            return result.stdout
        return None
    except:
        return None

def get_pod_status():
    """Get pod status information"""
    result = subprocess.run(
        ['kubectl', 'get', 'pods', '-n', 'wasmbed', '-o', 'json'],
        capture_output=True,
        text=True
    )
    if result.returncode == 0:
        return json.loads(result.stdout)
    return None

def analyze_devices(devices_data):
    """Analyze device distribution and statistics"""
    if not devices_data or 'devices' not in devices_data:
        return None
    
    devices = devices_data['devices']
    stats = {
        'total_devices': len(devices),
        'gateway_distribution': defaultdict(int),
        'mcu_types': defaultdict(int),
        'connection_states': defaultdict(int)
    }
    
    for device in devices:
        # Gateway distribution
        gateway = device.get('gateway', 'none')
        stats['gateway_distribution'][gateway] += 1
        
        # MCU types
        mcu = device.get('mcuType', 'unknown')
        stats['mcu_types'][mcu] += 1
        
        # Connection state
        status = device.get('status', 'unknown')
        stats['connection_states'][status] += 1
    
    return stats

def measure_api_latency(n_requests: int = 20, timeout: float = 5.0):
    """Run a burst of GET /devices to measure latency and throughput.

    Returns:
        dict with keys: latencies_ms (list), min, max, avg, p50, p95, p99, throughput_rps
    """
    latencies = []
    start = time.perf_counter()
    for _ in range(n_requests):
        t0 = time.perf_counter()
        try:
            r = requests.get('http://localhost:3001/api/v1/devices', timeout=timeout)
            _ = r.status_code  # ensure request executed
        except Exception:
            # Count failed request as timeout duration
            pass
        dt_ms = (time.perf_counter() - t0) * 1000.0
        latencies.append(dt_ms)
    total_time = time.perf_counter() - start
    lat_sorted = sorted(latencies)
    def pct(p):
        if not lat_sorted:
            return 0.0
        k = (len(lat_sorted)-1) * p
        f = math.floor(k)
        c = math.ceil(k)
        if f == c:
            return lat_sorted[int(k)]
        d0 = lat_sorted[int(f)] * (c - k)
        d1 = lat_sorted[int(c)] * (k - f)
        return d0 + d1
    return {
        'latencies_ms': latencies,
        'min': min(latencies) if latencies else 0.0,
        'max': max(latencies) if latencies else 0.0,
        'avg': (sum(latencies) / len(latencies)) if latencies else 0.0,
        'p50': pct(0.5),
        'p95': pct(0.95),
        'p99': pct(0.99),
        'throughput_rps': (n_requests / total_time) if total_time > 0 else 0.0,
        'requests': n_requests,
        'window_s': total_time
    }

def collect_metrics_sample(scenario_name, num_devices, num_gateways, duration=180, interval=15):
    """
    Collect metrics over a time period
    
    Args:
        scenario_name: Name of the scenario (e.g., "60_devices_3_gateways")
        num_devices: Expected number of devices
        num_gateways: Expected number of gateways
        duration: How long to collect metrics (seconds)
        interval: Sampling interval (seconds)
    """
    print(f"\n{'='*60}")
    print(f"Collecting metrics for: {scenario_name}")
    print(f"Expected: {num_devices} devices, {num_gateways} gateways")
    print(f"Duration: {duration}s, Interval: {interval}s")
    print(f"{'='*60}\n")
    
    metrics = {
        'scenario': scenario_name,
        'timestamp': datetime.now().isoformat(),
        'config': {
            'num_devices': num_devices,
            'num_gateways': num_gateways,
            'devices_per_gateway': num_devices / num_gateways
        },
        'samples': []
    }
    
    start_time = time.time()
    sample_count = 0
    
    while time.time() - start_time < duration:
        sample_start = time.time()
        sample_count += 1
        
        print(f"[{sample_count}] Collecting sample at {datetime.now().strftime('%H:%M:%S')}...")
        
        # Get devices from API
        devices_data = get_devices_from_api()
        
        # Get pod status
        pod_status = get_pod_status()
        
        # Analyze data
        device_stats = analyze_devices(devices_data) if devices_data else None
        # Measure API latency/throughput with a short burst
        api_stats = measure_api_latency(n_requests=20)
        # Parse pod resource metrics if available
        raw_top = get_pod_metrics()
        res = {
            'device_cpu_m': 0,
            'device_mem_Mi': 0,
            'gateway_cpu_m': 0,
            'gateway_mem_Mi': 0
        }
        if raw_top:
            for line in raw_top.strip().splitlines():
                parts = line.split()
                if len(parts) < 3:
                    continue
                name, cpu, mem = parts[0], parts[1], parts[2]
                # cpu like '108m' or '1m' or '0m' or '1'
                def cpu_to_m(v):
                    try:
                        if v.endswith('m'):
                            return int(v[:-1])
                        # cores to millicores
                        return int(float(v) * 1000)
                    except:
                        return 0
                # mem like '5Mi', '200Mi', '1Gi'
                def mem_to_Mi(v):
                    try:
                        if v.endswith('Mi'):
                            return int(v[:-2])
                        if v.endswith('Gi'):
                            return int(float(v[:-2]) * 1024)
                        if v.endswith('Ki'):
                            return int(int(v[:-2]) / 1024)
                        return 0
                    except:
                        return 0
                cpu_m = cpu_to_m(cpu)
                mem_Mi = mem_to_Mi(mem)
                if name.startswith('scale-device-') or name.endswith('-pod'):
                    res['device_cpu_m'] += cpu_m
                    res['device_mem_Mi'] += mem_Mi
                elif name.startswith('gateway-'):
                    res['gateway_cpu_m'] += cpu_m
                    res['gateway_mem_Mi'] += mem_Mi
        
        sample = {
            'sample_id': sample_count,
            'timestamp': datetime.now().isoformat(),
            'elapsed_time': time.time() - start_time,
            'device_stats': device_stats,
            'pod_count': len(pod_status['items']) if pod_status else 0,
            'api_stats': api_stats,
            'resources': res
        }
        
        # Add pod phase counts
        if pod_status:
            pod_phases = defaultdict(int)
            for pod in pod_status['items']:
                phase = pod['status']['phase']
                pod_phases[phase] += 1
            sample['pod_phases'] = dict(pod_phases)
        
        metrics['samples'].append(sample)
        
        if device_stats:
            print(f"  Devices: {device_stats['total_devices']}")
            print(f"  Gateway distribution: {dict(device_stats['gateway_distribution'])}")
            print(f"  Connection states: {dict(device_stats['connection_states'])}")
        
        # Wait for next interval
        elapsed = time.time() - sample_start
        sleep_time = max(0, interval - elapsed)
        if sleep_time > 0 and time.time() - start_time + sleep_time < duration:
            time.sleep(sleep_time)
    
    # Calculate aggregated statistics
    if metrics['samples']:
        device_counts = [s['device_stats']['total_devices'] for s in metrics['samples'] 
                        if s['device_stats']]
        # Aggregate API latency/throughput
        avg_lat = [s['api_stats']['avg'] for s in metrics['samples'] if s.get('api_stats')]
        p95_lat = [s['api_stats']['p95'] for s in metrics['samples'] if s.get('api_stats')]
        p99_lat = [s['api_stats']['p99'] for s in metrics['samples'] if s.get('api_stats')]
        thr = [s['api_stats']['throughput_rps'] for s in metrics['samples'] if s.get('api_stats')]
        
        # time to all enrolled (first sample where counts match expectation)
        tta = None
        for s in metrics['samples']:
            ds = s.get('device_stats') or {}
            total = ds.get('total_devices', 0)
            enrolled = (ds.get('connection_states') or {}).get('Enrolled', 0)
            if total >= num_devices and enrolled >= num_devices:
                tta = s['elapsed_time']
                break

        # aggregate resources
        dev_cpu = [s['resources']['device_cpu_m'] for s in metrics['samples'] if s.get('resources')]
        gw_cpu = [s['resources']['gateway_cpu_m'] for s in metrics['samples'] if s.get('resources')]
        dev_mem = [s['resources']['device_mem_Mi'] for s in metrics['samples'] if s.get('resources')]
        gw_mem = [s['resources']['gateway_mem_Mi'] for s in metrics['samples'] if s.get('resources')]

        metrics['summary'] = {
            'total_samples': len(metrics['samples']),
            'avg_device_count': statistics.mean(device_counts) if device_counts else 0,
            'min_device_count': min(device_counts) if device_counts else 0,
            'max_device_count': max(device_counts) if device_counts else 0,
            'final_gateway_distribution': metrics['samples'][-1]['device_stats']['gateway_distribution'] 
                                         if metrics['samples'][-1]['device_stats'] else {},
            'final_connection_states': metrics['samples'][-1]['device_stats']['connection_states']
                                      if metrics['samples'][-1]['device_stats'] else {},
            'api_latency_ms': {
                'avg': statistics.mean(avg_lat) if avg_lat else 0.0,
                'p95': statistics.mean(p95_lat) if p95_lat else 0.0,
                'p99': statistics.mean(p99_lat) if p99_lat else 0.0
            },
            'throughput_rps_avg': statistics.mean(thr) if thr else 0.0,
            'time_to_all_enrolled_s': tta if tta is not None else -1,
            'resources': {
                'device_cpu_m_avg': statistics.mean(dev_cpu) if dev_cpu else 0,
                'gateway_cpu_m_avg': statistics.mean(gw_cpu) if gw_cpu else 0,
                'device_mem_Mi_avg': statistics.mean(dev_mem) if dev_mem else 0,
                'gateway_mem_Mi_avg': statistics.mean(gw_mem) if gw_mem else 0
            }
        }
    
    return metrics

def save_metrics(metrics, filename=None):
    """Save metrics to JSON file"""
    if filename is None:
        timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
        filename = f'/home/lucadag/18_10_23_retrospect/scalability_metrics_{timestamp}.json'
    
    with open(filename, 'w') as f:
        json.dump(metrics, f, indent=2)
    
    print(f"\n{'='*60}")
    print(f"Metrics saved to: {filename}")
    print(f"{'='*60}\n")
    
    return filename

if __name__ == '__main__':
    import sys
    
    if len(sys.argv) < 3:
        print("Usage: collect_metrics.py <scenario_name> <num_devices> <num_gateways> [duration] [interval]")
        print("Example: collect_metrics.py 60d_3g 60 3 180 15")
        sys.exit(1)
    
    scenario_name = sys.argv[1]
    num_devices = int(sys.argv[2])
    num_gateways = int(sys.argv[3])
    duration = int(sys.argv[4]) if len(sys.argv) > 4 else 180
    interval = int(sys.argv[5]) if len(sys.argv) > 5 else 15
    
    metrics = collect_metrics_sample(scenario_name, num_devices, num_gateways, duration, interval)
    filename = save_metrics(metrics)
    
    print("\nSummary:")
    print(f"  Samples collected: {metrics['summary']['total_samples']}")
    print(f"  Average devices: {metrics['summary']['avg_device_count']:.1f}")
    print(f"  Final distribution: {metrics['summary']['final_gateway_distribution']}")
