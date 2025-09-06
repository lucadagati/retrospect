#!/bin/bash
# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CLUSTER_NAME="wasmbed"
NAMESPACE="wasmbed"

echo "🛠️  Wasmbed Development Helper"

check_environment() {
    # Check if we're in Nix shell
    if [ -z "${IN_NIX_SHELL:-}" ]; then
        echo -e "${YELLOW}⚠️  Not in Nix shell. Running 'nix develop' first...${NC}"
        exec nix develop --command "$0" "$@"
    fi
    
    # Set kubeconfig if not already set
    if [ -z "${KUBECONFIG:-}" ]; then
        export KUBECONFIG=$(k3d kubeconfig write "$CLUSTER_NAME" 2>/dev/null || echo "")
    fi
}

rebuild_and_deploy() {
    echo -e "${BLUE}🔄 Rebuilding and deploying gateway...${NC}"
    
    # Build new image
    echo "  🔨 Building gateway image..."
    nix build '.#dockerImages.x86_64-linux.wasmbed-gateway'
    
    # Load and get tag
    local load_output
    load_output=$(docker load -i "$(readlink result)")
    local image_tag
    image_tag=$(echo "$load_output" | grep "Loaded image:" | sed 's/Loaded image: //')
    
    echo "  📦 Built image: $image_tag"
    
    # Import to k3d
    echo "  📥 Importing to k3d..."
    k3d image import -c "$CLUSTER_NAME" "$image_tag"
    
    # Update StatefulSet
    echo "  🚢 Updating StatefulSet..."
    kubectl -n "$NAMESPACE" set image statefulset/wasmbed-gateway wasmbed-gateway="$image_tag"
    
    # Wait for rollout
    echo "  ⏳ Waiting for rollout..."
    kubectl -n "$NAMESPACE" rollout status statefulset/wasmbed-gateway --timeout=300s
    
    echo -e "${GREEN}✅ Gateway updated successfully!${NC}"
}

run_tests() {
    echo -e "${BLUE}🧪 Running tests...${NC}"
    
    echo "  📋 Cargo tests..."
    cargo test
    
    echo "  🔍 Clippy checks..."
    cargo clippy -- -D warnings
    
    echo "  🎨 Format check..."
    cargo fmt --check
    
    echo "  🧪 System tests..."
    ./scripts/test.sh
    
    echo -e "${GREEN}✅ All tests passed!${NC}"
}

port_forward() {
    echo -e "${BLUE}🔌 Setting up port forwarding...${NC}"
    
    # Kill existing port forwards
    pkill -f "kubectl.*port-forward.*wasmbed-gateway-service" || true
    sleep 2
    
    # Start new port forward
    echo "  🔗 Forwarding localhost:4423 -> service:4423"
    kubectl -n "$NAMESPACE" port-forward service/wasmbed-gateway-service 4423:4423 &
    local pf_pid=$!
    
    echo "  ✅ Port forward started (PID: $pf_pid)"
    echo "  💡 Gateway accessible at: https://localhost:4423"
    echo "  ⏹️  To stop: kill $pf_pid"
    
    # Save PID for cleanup
    echo $pf_pid > /tmp/wasmbed-port-forward.pid
}

tail_logs() {
    echo -e "${BLUE}📜 Tailing gateway logs...${NC}"
    echo "  (Press Ctrl+C to stop)"
    kubectl -n "$NAMESPACE" logs -f wasmbed-gateway-0
}

watch_resources() {
    echo -e "${BLUE}👀 Watching Kubernetes resources...${NC}"
    echo "  (Press Ctrl+C to stop)"
    kubectl -n "$NAMESPACE" get pods,services,devices --watch
}

shell_into_pod() {
    echo -e "${BLUE}🐚 Opening shell in gateway pod...${NC}"
    kubectl -n "$NAMESPACE" exec -it wasmbed-gateway-0 -- /bin/sh
}

generate_certs() {
    echo -e "${BLUE}🔐 Generating development certificates...${NC}"
    
    # Check if certificates already exist
    if [ -f "resources/dev-certs/server-ca.der" ]; then
        echo "  ⚠️  Certificates already exist. Regenerate? (y/N)"
        read -n 1 -r
        echo ""
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            echo "  ⏹️  Certificate generation cancelled"
            return
        fi
    fi
    
    echo "  🏗️  Generating server CA..."
    cargo run -p wasmbed-cert-tool -- \
        generate-ca server \
        --common-name "Wasmbed Gateway Server Development CA" \
        --out-key resources/dev-certs/server-ca.key \
        --out-cert resources/dev-certs/server-ca.der
    
    echo "  🏗️  Generating client CA..."
    cargo run -p wasmbed-cert-tool -- \
        generate-ca client \
        --common-name "Wasmbed Gateway Client Development CA" \
        --out-key resources/dev-certs/client-ca.key \
        --out-cert resources/dev-certs/client-ca.der
    
    echo "  🎫 Generating server certificate..."
    cargo run -p wasmbed-cert-tool -- \
        issue-cert server \
        --ca-key resources/dev-certs/server-ca.key \
        --ca-cert resources/dev-certs/server-ca.der \
        --common-name "Wasmbed Gateway Server 0" \
        --out-key resources/dev-certs/server-0.key \
        --out-cert resources/dev-certs/server-0.der
    
    echo "  🎫 Generating client certificate..."
    cargo run -p wasmbed-cert-tool -- \
        issue-cert client \
        --ca-key resources/dev-certs/client-ca.key \
        --ca-cert resources/dev-certs/client-ca.der \
        --common-name "Wasmbed Gateway Client 0" \
        --out-key resources/dev-certs/client-0.key \
        --out-cert resources/dev-certs/client-0.der
    
    echo -e "${GREEN}✅ Certificates generated successfully!${NC}"
}

run_client() {
    echo -e "${BLUE}🤝 Running test client...${NC}"
    
    # Ensure port forward is running
    if ! pgrep -f "kubectl.*port-forward.*wasmbed-gateway-service" > /dev/null; then
        echo "  🔌 Starting port forward..."
        port_forward
        sleep 3
    fi
    
    echo "  🚀 Connecting to gateway..."
    cargo run -p wasmbed-gateway-test-client -- \
        --address 127.0.0.1:4423 \
        --server-ca resources/dev-certs/server-ca.der \
        --private-key resources/dev-certs/client-0.key \
        --certificate resources/dev-certs/client-0.der
}

show_help() {
    echo -e "${BLUE}🛠️  Wasmbed Development Helper${NC}"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  build, b        - Rebuild and deploy gateway"
    echo "  test, t         - Run all tests"
    echo "  port, p         - Setup port forwarding"
    echo "  logs, l         - Tail gateway logs"
    echo "  watch, w        - Watch Kubernetes resources"
    echo "  shell, s        - Shell into gateway pod"
    echo "  certs, c        - Generate development certificates"
    echo "  client          - Run test client"
    echo "  monitor, m      - Open interactive monitor"
    echo "  help, h         - Show this help"
    echo ""
    echo "Examples:"
    echo "  $0 build        - Rebuild gateway after code changes"
    echo "  $0 logs         - Watch gateway logs in real-time"
    echo "  $0 client       - Test connection to gateway"
    echo ""
}

main() {
    check_environment
    
    case "${1:-help}" in
        build|b)
            rebuild_and_deploy
            ;;
        test|t)
            run_tests
            ;;
        port|p)
            port_forward
            ;;
        logs|l)
            tail_logs
            ;;
        watch|w)
            watch_resources
            ;;
        shell|s)
            shell_into_pod
            ;;
        certs|c)
            generate_certs
            ;;
        client)
            run_client
            ;;
        monitor|m)
            ./scripts/monitor.sh --interactive
            ;;
        help|h|*)
            show_help
            ;;
    esac
}

main "$@"
