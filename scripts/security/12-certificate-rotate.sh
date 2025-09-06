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

echo "🔄 Converting certificates from DER to PEM format..."

# Configuration
CERT_DIR="resources/dev-certs"
PEM_DIR="resources/dev-certs/pem"

# Create PEM directory
mkdir -p "$PEM_DIR"

convert_cert() {
    local der_file="$1"
    local pem_file="$2"
    local cert_type="$3"
    
    echo "  🔄 Converting $cert_type: $der_file -> $pem_file"
    
    if [[ "$cert_type" == "certificate" ]]; then
        openssl x509 -in "$der_file" -inform DER -out "$pem_file" -outform PEM
    elif [[ "$cert_type" == "private_key" ]]; then
        # For private keys, we need to try different formats
        if openssl rsa -in "$der_file" -inform DER -out "$pem_file" -outform PEM 2>/dev/null; then
            echo "    PASS: Converted as RSA key"
        elif openssl ec -in "$der_file" -inform DER -out "$pem_file" -outform PEM 2>/dev/null; then
            echo "    PASS: Converted as EC key"
        else
            echo "    FAIL: Failed to convert private key"
            return 1
        fi
    fi
    
    echo "    PASS: Conversion successful"
}

main() {
    echo -e "${BLUE}Converting certificates...${NC}"
    
    # Convert CA certificates
    convert_cert "$CERT_DIR/server-ca.der" "$PEM_DIR/server-ca.pem" "certificate"
    convert_cert "$CERT_DIR/client-ca.der" "$PEM_DIR/client-ca.pem" "certificate"
    
    # Convert server certificate and key
    convert_cert "$CERT_DIR/server-0.der" "$PEM_DIR/server-0-cert.pem" "certificate"
    convert_cert "$CERT_DIR/server-0.key" "$PEM_DIR/server-0-key.pem" "private_key"
    
    # Convert client certificate and key
    convert_cert "$CERT_DIR/client-0.der" "$PEM_DIR/client-0-cert.pem" "certificate"
    convert_cert "$CERT_DIR/client-0.key" "$PEM_DIR/client-0-key.pem" "private_key"
    
    echo ""
    echo -e "${GREEN}Certificate conversion completed!${NC}"
    echo ""
    echo -e "${BLUE}PEM certificates available in: $PEM_DIR${NC}"
    echo "  - server-ca.pem"
    echo "  - client-ca.pem"
    echo "  - server-0-cert.pem"
    echo "  - server-0-key.pem"
    echo "  - client-0-cert.pem"
    echo "  - client-0-key.pem"
    echo ""
    echo -e "${YELLOW}Note: Private keys are now in PEM format for easier testing${NC}"
}

main "$@"
