# Wasmbed Platform - Environment Setup
# Add this to your ~/.bashrc or ~/.zshrc

# Wasmbed Platform Management
alias wasmbed='$(pwd)/scripts/wasmbed.sh'

# Quick access commands
alias wb-clean='wasmbed clean'
alias wb-build='wasmbed build'
alias wb-deploy='wasmbed deploy'
alias wb-stop='wasmbed stop'
alias wb-status='wasmbed status'
alias wb-restart='wasmbed restart'

# Resource management
alias wb-devices='wasmbed devices'
alias wb-apps='wasmbed applications'
alias wb-monitor='wasmbed monitor'

# Testing
alias wb-test='wasmbed test'
alias wb-test-devices='wasmbed test-devices'
alias wb-test-apps='wasmbed test-applications'
alias wb-test-gateways='wasmbed test-gateways'

# Service endpoints
export WASMBED_INFRASTRUCTURE_URL="http://localhost:30460"
export WASMBED_GATEWAY_URL="http://localhost:30451"
export WASMBED_DASHBOARD_URL="http://localhost:30470"

echo "Wasmbed Platform environment loaded!"
echo "Use 'wasmbed help' for available commands"
