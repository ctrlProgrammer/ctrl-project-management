#!/usr/bin/env bash
set -euo pipefail

BIN="ctrl-project-management"
DESKTOP="scripts/ctrl-project-management.desktop"

if [ "${1:-}" = "--system" ]; then
    sudo install -m 755 "target/release/$BIN" /usr/local/bin/
    sudo install -m 644 "$DESKTOP" /usr/local/share/applications/
else
    mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications"
    install -m 755 "target/release/$BIN" "$HOME/.local/bin/"
    install -m 644 "$DESKTOP" "$HOME/.local/share/applications/"
fi

echo "Installed $BIN. Launch with: $BIN"
echo "MCP server mode: $BIN mcp"
