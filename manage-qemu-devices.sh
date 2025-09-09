#!/bin/bash

# Wasmbed QEMU Device Manager
# Manages real QEMU devices alongside simulated ones

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
QEMU_FIRMWARE_PATH="crates/wasmbed-firmware-hifive1-qemu"
FIRMWARE_BINARY="target/riscv32imac-unknown-none-elf/release/wasmbed-firmware-hifive1-qemu"
QEMU_PIDS_FILE="/tmp/wasmbed-qemu-pids"
GATEWAY_HOST="172.19.0.2"
GATEWAY_TLS_PORT="30423"
DEVICE_COUNT=2  # Number of real QEMU devices to start

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_qemu() {
    echo -e "${PURPLE}[QEMU]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    log_info "Checking prerequisites..."
    
    # Check if QEMU is installed
    if ! command -v qemu-system-riscv32 >/dev/null 2>&1; then
        log_error "QEMU not found. Please install qemu-system-riscv32"
        exit 1
    fi
    
    # Check if firmware exists
    if [ ! -f "$FIRMWARE_BINARY" ]; then
        log_error "Firmware binary not found: $FIRMWARE_BINARY"
        log_info "Please compile the firmware first:"
        log_info "cargo build --release --bin wasmbed-firmware-hifive1-qemu --target riscv32imac-unknown-none-elf"
        exit 1
    fi
    
    # Check if socat is available
    if ! command -v socat >/dev/null 2>&1; then
        log_warning "socat not found. QEMU serial communication may not work properly"
    fi
    
    log_success "Prerequisites check completed"
}

# Start QEMU device
start_qemu_device() {
    local device_id=$1
    local device_port=$2
    
    log_qemu "Starting QEMU device $device_id on port $device_port"
    
    # Create unique serial socket for this device
    local serial_sock="/tmp/wasmbed-qemu-$device_id.sock"
    
    # Start QEMU in background (simplified configuration for HiFive1)
    qemu-system-riscv32 \
        -nographic \
        -monitor none \
        -machine sifive_e,revb=true \
        -serial unix:"$serial_sock",server,nowait \
        -kernel "$FIRMWARE_BINARY" \
        -m 16K \
        > "/tmp/wasmbed-qemu-$device_id.log" 2>&1 &
    
    local qemu_pid=$!
    echo "$qemu_pid" >> "$QEMU_PIDS_FILE"
    
    # Wait for QEMU to start
    sleep 2
    
    # Check if QEMU is running
    if kill -0 "$qemu_pid" 2>/dev/null; then
        log_success "QEMU device $device_id started (PID: $qemu_pid)"
        log_info "Serial socket: $serial_sock"
        log_info "Log file: /tmp/wasmbed-qemu-$device_id.log"
        return 0
    else
        log_error "Failed to start QEMU device $device_id"
        return 1
    fi
}

# Start all QEMU devices
start_all_qemu_devices() {
    log_info "Starting $DEVICE_COUNT QEMU devices..."
    
    # Clear previous PIDs
    > "$QEMU_PIDS_FILE"
    
    local success_count=0
    
    for i in $(seq 1 $DEVICE_COUNT); do
        local device_id="qemu-device-$i"
        local device_port=$((30424 + i))  # Use different ports for each device
        
        if start_qemu_device "$device_id" "$device_port"; then
            ((success_count++))
        fi
        
        sleep 1  # Small delay between device starts
    done
    
    log_success "Started $success_count out of $DEVICE_COUNT QEMU devices"
    return $success_count
}

# Stop all QEMU devices
stop_all_qemu_devices() {
    log_info "Stopping all QEMU devices..."
    
    if [ -f "$QEMU_PIDS_FILE" ]; then
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                log_qemu "Stopping QEMU device (PID: $pid)"
                kill "$pid"
                sleep 1
                if kill -0 "$pid" 2>/dev/null; then
                    log_warning "Force killing QEMU device (PID: $pid)"
                    kill -9 "$pid"
                fi
            fi
        done < "$QEMU_PIDS_FILE"
        
        rm -f "$QEMU_PIDS_FILE"
        log_success "All QEMU devices stopped"
    else
        log_warning "No QEMU PIDs file found"
    fi
    
    # Clean up socket files
    rm -f /tmp/wasmbed-qemu-*.sock
    rm -f /tmp/wasmbed-qemu-*.log
}

# Check QEMU device status
check_qemu_status() {
    log_info "Checking QEMU device status..."
    
    if [ -f "$QEMU_PIDS_FILE" ]; then
        local running_count=0
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                ((running_count++))
                log_success "QEMU device running (PID: $pid)"
            else
                log_warning "QEMU device not running (PID: $pid)"
            fi
        done < "$QEMU_PIDS_FILE"
        
        log_info "Running QEMU devices: $running_count"
    else
        log_warning "No QEMU devices found"
    fi
}

# Monitor QEMU devices
monitor_qemu_devices() {
    log_info "Monitoring QEMU devices..."
    
    if [ -f "$QEMU_PIDS_FILE" ]; then
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                log_info "Monitoring QEMU device (PID: $pid)"
                # Show last few lines of log
                if [ -f "/tmp/wasmbed-qemu-qemu-device-*.log" ]; then
                    tail -5 /tmp/wasmbed-qemu-qemu-device-*.log 2>/dev/null || true
                fi
            fi
        done < "$QEMU_PIDS_FILE"
    fi
}

# Create hybrid device configuration
create_hybrid_config() {
    log_info "Creating hybrid device configuration..."
    
    # Create QEMU device resources in Kubernetes
    for i in $(seq 1 $DEVICE_COUNT); do
        local device_id="qemu-device-$i"
        
        cat <<EOF | kubectl apply -f -
apiVersion: wasmbed.github.io/v0
kind: Device
metadata:
  name: $device_id
  namespace: wasmbed
spec:
  deviceId: "$device_id"
  publicKey: "LS0tLS1CRUdJTiBQVUJMSUMgS0VZLS0tLS0KTUlJQ0lqQU5CZ2txaGtpRzl3MEJBUUVGQUFPQ0FnOEFNSUlDQ2dLQ0FnRUF0b0VTUzk5STRFVjdMa3B3KzlKSQpkRzZkcDJIRldIT2lrWUMwSGF2ZE5sYWVsc1ZHUGNvbmJKTi9Zc0t5d2F4aC9CTm0vYU5HdU9iK1ZKTm5UK21HCi9jQ3RPZ0I5NjFaVGRjZ2xhR0tWTEJHWWFwZ1VLNC9ocFVUL0VCMlpqaEJUazNoYUxwdFZJR08yU2NZSFc2KzgKUVZxT0ZIWFgwOSs3MHNMZUJPdzc5ZGhzOUZXelBUSElmUGVHUWpPNFR0S0VjZlhYbjM3WlBYZTBDMTRMVXpEYgorWWIwVmE1VEllWW5sOWhVaFFLV3ZQYlQxOXM3T2p4U0JuY1BwVkVOTjBoRnFzVUh2SVJrZGo4VjY2My94NkZoCmhhdlNTbDU0Wk1WeHVtcXlYaGFCOTBEYjVUUU5kUW52RGZxSmVLOFJDUzgzTUdjdjlEMUpUTHNYZzFNcm9LRjAKak9hc08vYnd6QWVYcWlkSGlwdG5YVWJtYzFveVRiWXIwZUFGV2piUjJmdDF3bjhPV1libW9YS3RkeWNCVmRjYwpxUVFpcnpzSCtNUW9QbXdqMEVPandwMTJnTnBqRnloSnVDWWlESVhKUENrV0U5ZlhubjZVRk11bks5YkxSZ3NkCklPblJKbUk3ODh2ZTFkaExYd21qdSt2MFQ0elBVRkNPYUpYSEU1NWhsNUZqdkZOdjFNSlFHYytpK1B1a1pUTUsKRmFNQVlEQ0IxaXh4RUo4aC8veDZIU0t2Z0ViYzFSUWhNTXhkQlRiUFQ4ZVBqQXpiNHNQeXhHcUhUVzJ4dVY5OQovMWk4V3JnSjA4YW5FN2Z0bW1hd0F6OGd3ZHY1Tytjc3ErZW50djY0ZUtxSU45dmdHQUhLZnB0ZS9hZkRuaTFRCnJrQTFOSzIxTW1DVEZrYWxSMHN5QmlrQ0F3RUFBUT09Ci0tLS0tRU5EIFBVQkxJQyBLRVktLS0tLQo"
  deviceType: "riscv-hifive1-qemu"
  capabilities: ["wasm-execution", "tls-client", "microROS", "qemu-emulation"]
EOF
    done
    
    log_success "Hybrid device configuration created"
}

# Main function
main() {
    case "${1:-start}" in
        "start")
            check_prerequisites
            start_all_qemu_devices
            create_hybrid_config
            log_success "Hybrid QEMU + Simulator system started!"
            log_info "QEMU devices: $DEVICE_COUNT"
            log_info "Simulated devices: 4 (mcu-device-1 to mcu-device-4)"
            log_info "Total devices: $((DEVICE_COUNT + 4))"
            ;;
        "stop")
            stop_all_qemu_devices
            log_success "QEMU devices stopped"
            ;;
        "status")
            check_qemu_status
            ;;
        "monitor")
            monitor_qemu_devices
            ;;
        "restart")
            stop_all_qemu_devices
            sleep 2
            check_prerequisites
            start_all_qemu_devices
            create_hybrid_config
            log_success "QEMU devices restarted"
            ;;
        *)
            echo "Usage: $0 {start|stop|status|monitor|restart}"
            echo "  start   - Start QEMU devices (default)"
            echo "  stop    - Stop all QEMU devices"
            echo "  status  - Check QEMU device status"
            echo "  monitor - Monitor QEMU devices"
            echo "  restart - Restart QEMU devices"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"

