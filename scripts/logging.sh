#!/bin/bash
# Wasmbed Unified Logging System
# Provides consistent logging across all scripts

# Colors for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly BLUE='\033[0;34m'
readonly PURPLE='\033[0;35m'
readonly CYAN='\033[0;36m'
readonly WHITE='\033[1;37m'
readonly NC='\033[0m' # No Color

# Log levels
readonly LOG_ERROR=1
readonly LOG_WARN=2
readonly LOG_INFO=3
readonly LOG_DEBUG=4

# Default log level
LOG_LEVEL=${LOG_LEVEL:-$LOG_INFO}

# Logging functions
log_error() {
    if [ $LOG_LEVEL -ge $LOG_ERROR ]; then
        echo -e "${RED}[ERROR]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1" >&2
    fi
}

log_warn() {
    if [ $LOG_LEVEL -ge $LOG_WARN ]; then
        echo -e "${YELLOW}[WARN]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
    fi
}

log_info() {
    if [ $LOG_LEVEL -ge $LOG_INFO ]; then
        echo -e "${GREEN}[INFO]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
    fi
}

log_debug() {
    if [ $LOG_LEVEL -ge $LOG_DEBUG ]; then
        echo -e "${BLUE}[DEBUG]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
    fi
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_step() {
    echo -e "${CYAN}[STEP]${NC} $(date '+%Y-%m-%d %H:%M:%S') $1"
}

log_header() {
    echo -e "${WHITE}================================${NC}"
    echo -e "${WHITE}$1${NC}"
    echo -e "${WHITE}================================${NC}"
}

# Command execution with logging
execute_cmd() {
    local cmd="$1"
    local desc="${2:-Executing command}"
    
    log_debug "Running: $cmd"
    
    if eval "$cmd"; then
        log_success "$desc completed"
        return 0
    else
        log_error "$desc failed"
        return 1
    fi
}

# Check if command exists
check_command() {
    local cmd="$1"
    local desc="${2:-$cmd}"
    
    if command -v "$cmd" >/dev/null 2>&1; then
        log_debug "$desc found"
        return 0
    else
        log_error "$desc not found"
        return 1
    fi
}

# Check prerequisites
check_prerequisites() {
    log_header "Checking Prerequisites"
    
    local missing=0
    
    check_command "cargo" "Rust Cargo" || missing=$((missing + 1))
    check_command "docker" "Docker" || missing=$((missing + 1))
    check_command "kubectl" "Kubernetes CLI" || missing=$((missing + 1))
    check_command "k3d" "k3d" || missing=$((missing + 1))
    
    if [ $missing -eq 0 ]; then
        log_success "All prerequisites satisfied"
        return 0
    else
        log_error "$missing prerequisites missing"
        return 1
    fi
}

# Initialize logging
init_logging() {
    # Set log level if provided
    if [ -n "${1:-}" ]; then
        case "$1" in
            "error"|"ERROR"|1) LOG_LEVEL=$LOG_ERROR ;;
            "warn"|"WARN"|2) LOG_LEVEL=$LOG_WARN ;;
            "info"|"INFO"|3) LOG_LEVEL=$LOG_INFO ;;
            "debug"|"DEBUG"|4) LOG_LEVEL=$LOG_DEBUG ;;
            *) LOG_LEVEL=$LOG_INFO ;;
        esac
    fi
    
    log_info "Logging initialized (level: $LOG_LEVEL)"
}