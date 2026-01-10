#!/bin/bash

# SPDX-License-Identifier: AGPL-3.0
# Copyright © 2025 Wasmbed contributors

# Check Zephyr SDK download progress

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

FILE="zephyr-sdk-0.16.5_linux-x86_64.tar.xz"
EXPECTED_SIZE=1406994864

if [ -f "$FILE" ]; then
    CURRENT_SIZE=$(stat -c%s "$FILE" 2>/dev/null || echo "0")
    PERCENT=$((CURRENT_SIZE * 100 / EXPECTED_SIZE))
    SIZE_H=$(du -h "$FILE" | cut -f1)
    
    echo "=== Zephyr SDK Download Status ==="
    echo "File: $FILE"
    echo "Dimensione attuale: $SIZE_H"
    echo "Dimensione attesa: 1.3G"
    echo "Progresso: ~${PERCENT}%"
    echo ""
    
    if [ "$CURRENT_SIZE" -ge "$EXPECTED_SIZE" ]; then
        echo "✅ Download COMPLETATO!"
        echo ""
        echo "Puoi procedere con:"
        echo "  source .venv/bin/activate"
        echo "  ./scripts/setup-zephyr-workspace.sh"
    else
        echo "⏳ Download in corso..."
        echo ""
        echo "Per monitorare: watch -n 5 './scripts/check-download.sh'"
    fi
else
    echo "❌ File non trovato"
    echo "Eseguire: ./scripts/setup-zephyr-workspace.sh"
fi


