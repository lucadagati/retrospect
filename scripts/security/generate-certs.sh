#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

set -e

echo " GENERAZIONE CERTIFICATI TLS"
echo "=============================="

CERT_DIR="certs"
mkdir -p "$CERT_DIR"

# Generate CA private key
echo " Generando CA private key..."
openssl genrsa -out "$CERT_DIR/ca-key.pem" 4096

# Generate CA certificate
echo " Generando CA certificate..."
openssl req -new -x509 -days 365 -key "$CERT_DIR/ca-key.pem" -out "$CERT_DIR/ca-cert.pem" \
    -subj "/C=IT/ST=Italy/L=Milan/O=Wasmbed/OU=Development/CN=Wasmbed-CA"

# Generate server private key
echo " Generando server private key..."
openssl genrsa -out "$CERT_DIR/server-key.pem" 4096

# Generate server certificate request
echo " Generando server certificate request..."
openssl req -new -key "$CERT_DIR/server-key.pem" -out "$CERT_DIR/server.csr" \
    -subj "/C=IT/ST=Italy/L=Milan/O=Wasmbed/OU=Development/CN=wasmbed-gateway"

# Generate server certificate signed by CA
echo " Generando server certificate..."
openssl x509 -req -days 365 -in "$CERT_DIR/server.csr" -CA "$CERT_DIR/ca-cert.pem" \
    -CAkey "$CERT_DIR/ca-key.pem" -CAcreateserial -out "$CERT_DIR/server-cert.pem"

# Generate client private key
echo " Generando client private key..."
openssl genrsa -out "$CERT_DIR/client-key.pem" 4096

# Generate client certificate request
echo " Generando client certificate request..."
openssl req -new -key "$CERT_DIR/client-key.pem" -out "$CERT_DIR/client.csr" \
    -subj "/C=IT/ST=Italy/L=Milan/O=Wasmbed/OU=Development/CN=wasmbed-client"

# Generate client certificate signed by CA
echo " Generando client certificate..."
openssl x509 -req -days 365 -in "$CERT_DIR/client.csr" -CA "$CERT_DIR/ca-cert.pem" \
    -CAkey "$CERT_DIR/ca-key.pem" -CAcreateserial -out "$CERT_DIR/client-cert.pem"

# Clean up CSR files
rm "$CERT_DIR/server.csr" "$CERT_DIR/client.csr"

echo ""
echo " Certificati generati con successo!"
echo " Directory: $CERT_DIR"
echo ""
echo " File generati:"
ls -la "$CERT_DIR/"
echo ""
echo " Per verificare i certificati:"
echo "  openssl x509 -in $CERT_DIR/ca-cert.pem -text -noout"
echo "  openssl x509 -in $CERT_DIR/server-cert.pem -text -noout"
echo "  openssl x509 -in $CERT_DIR/client-cert.pem -text -noout"