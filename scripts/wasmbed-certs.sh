#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

# Wasmbed Platform - Certificate Management Script
# This script manages TLS certificates for the Wasmbed platform

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    local status=$1
    local message=$2
    case $status in
        "SUCCESS")
            echo -e "${GREEN}✓ $message${NC}"
            ;;
        "ERROR")
            echo -e "${RED}✗ $message${NC}"
            ;;
        "WARNING")
            echo -e "${YELLOW}⚠ $message${NC}"
            ;;
        "INFO")
            echo -e "${BLUE}ℹ $message${NC}"
            ;;
    esac
}

show_help() {
    echo "Wasmbed Certificate Management Script"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  generate                Generate new certificates"
    echo "  renew                   Renew existing certificates"
    echo "  validate                Validate certificate chain"
    echo "  info                    Show certificate information"
    echo "  clean                   Clean up certificates"
    echo ""
    echo "Examples:"
    echo "  $0 generate"
    echo "  $0 validate"
    echo "  $0 info"
}

generate_certificates() {
    print_status "INFO" "Generating Wasmbed certificates..."
    
    # Create certs directory
    mkdir -p certs
    
    # Generate CA certificate
    print_status "INFO" "Generating CA certificate..."
    openssl req -x509 -newkey rsa:4096 -keyout certs/ca-key.pem -out certs/ca-cert.pem -days 365 -nodes -subj "/C=US/ST=State/L=City/O=Wasmbed/CN=Wasmbed-CA"
    
    # Generate server certificate
    print_status "INFO" "Generating server certificate..."
    openssl req -newkey rsa:4096 -keyout certs/server-key.pem -out certs/server.csr -nodes -subj "/C=US/ST=State/L=City/O=Wasmbed/CN=localhost"
    openssl x509 -req -in certs/server.csr -CA certs/ca-cert.pem -CAkey certs/ca-key.pem -out certs/server-cert.pem -days 365 -CAcreateserial
    rm certs/server.csr
    
    # Generate client certificate
    print_status "INFO" "Generating client certificate..."
    openssl req -newkey rsa:4096 -keyout certs/client-key.pem -out certs/client.csr -nodes -subj "/C=US/ST=State/L=City/O=Wasmbed/CN=client"
    openssl x509 -req -in certs/client.csr -CA certs/ca-cert.pem -CAkey certs/ca-key.pem -out certs/client-cert.pem -days 365 -CAcreateserial
    rm certs/client.csr
    
    print_status "SUCCESS" "Certificates generated successfully"
    print_status "INFO" "Certificate files created in certs/ directory"
}

renew_certificates() {
    print_status "INFO" "Renewing certificates..."
    
    if [ ! -d "certs" ]; then
        print_status "ERROR" "Certificates directory not found"
        exit 1
    fi
    
    # Backup existing certificates
    print_status "INFO" "Backing up existing certificates..."
    cp -r certs certs.backup.$(date +%Y%m%d_%H%M%S)
    
    # Generate new certificates
    generate_certificates
    
    print_status "SUCCESS" "Certificates renewed successfully"
}

validate_certificates() {
    print_status "INFO" "Validating certificate chain..."
    
    if [ ! -d "certs" ]; then
        print_status "ERROR" "Certificates directory not found"
        exit 1
    fi
    
    # Validate CA certificate
    if [ -f "certs/ca-cert.pem" ]; then
        print_status "INFO" "Validating CA certificate..."
        openssl x509 -in certs/ca-cert.pem -text -noout >/dev/null 2>&1
        if [ $? -eq 0 ]; then
            print_status "SUCCESS" "CA certificate is valid"
        else
            print_status "ERROR" "CA certificate is invalid"
        fi
    else
        print_status "ERROR" "CA certificate not found"
    fi
    
    # Validate server certificate
    if [ -f "certs/server-cert.pem" ] && [ -f "certs/ca-cert.pem" ]; then
        print_status "INFO" "Validating server certificate..."
        openssl verify -CAfile certs/ca-cert.pem certs/server-cert.pem >/dev/null 2>&1
        if [ $? -eq 0 ]; then
            print_status "SUCCESS" "Server certificate is valid"
        else
            print_status "ERROR" "Server certificate is invalid"
        fi
    else
        print_status "ERROR" "Server certificate or CA not found"
    fi
    
    # Validate client certificate
    if [ -f "certs/client-cert.pem" ] && [ -f "certs/ca-cert.pem" ]; then
        print_status "INFO" "Validating client certificate..."
        openssl verify -CAfile certs/ca-cert.pem certs/client-cert.pem >/dev/null 2>&1
        if [ $? -eq 0 ]; then
            print_status "SUCCESS" "Client certificate is valid"
        else
            print_status "ERROR" "Client certificate is invalid"
        fi
    else
        print_status "ERROR" "Client certificate or CA not found"
    fi
}

show_certificate_info() {
    print_status "INFO" "Certificate Information"
    echo ""
    
    if [ ! -d "certs" ]; then
        print_status "ERROR" "Certificates directory not found"
        exit 1
    fi
    
    # CA certificate info
    if [ -f "certs/ca-cert.pem" ]; then
        print_status "INFO" "CA Certificate:"
        openssl x509 -in certs/ca-cert.pem -text -noout | grep -E "(Subject:|Issuer:|Not Before:|Not After:)"
        echo ""
    fi
    
    # Server certificate info
    if [ -f "certs/server-cert.pem" ]; then
        print_status "INFO" "Server Certificate:"
        openssl x509 -in certs/server-cert.pem -text -noout | grep -E "(Subject:|Issuer:|Not Before:|Not After:)"
        echo ""
    fi
    
    # Client certificate info
    if [ -f "certs/client-cert.pem" ]; then
        print_status "INFO" "Client Certificate:"
        openssl x509 -in certs/client-cert.pem -text -noout | grep -E "(Subject:|Issuer:|Not Before:|Not After:)"
        echo ""
    fi
}

clean_certificates() {
    print_status "INFO" "Cleaning up certificates..."
    
    if [ -d "certs" ]; then
        rm -rf certs/
        print_status "SUCCESS" "Certificates cleaned up"
    else
        print_status "WARNING" "Certificates directory not found"
    fi
}

# Main script logic
case "$1" in
    "generate")
        generate_certificates
        ;;
    "renew")
        renew_certificates
        ;;
    "validate")
        validate_certificates
        ;;
    "info")
        show_certificate_info
        ;;
    "clean")
        clean_certificates
        ;;
    "help"|"-h"|"--help")
        show_help
        ;;
    *)
        print_status "ERROR" "Unknown command: $1"
        echo ""
        show_help
        exit 1
        ;;
esac
