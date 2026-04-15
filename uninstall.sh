#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="$HOME/.local/bin"
SERVICE_DIR="$HOME/.config/systemd/user"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

echo "=== simbridge uninstaller ==="
echo ""

# ── Service ──
echo "[1/4] Stopping and removing systemd service..."
if systemctl --user is-active simbridge-server.service &>/dev/null; then
    systemctl --user stop simbridge-server.service
fi
if systemctl --user is-enabled simbridge-server.service &>/dev/null; then
    systemctl --user disable simbridge-server.service
fi
rm -f "$SERVICE_DIR/simbridge-server.service"
systemctl --user daemon-reload
echo "  done"
echo ""

# ── Binaries ──
echo "[2/4] Removing binaries..."
rm -f "$BIN_DIR/simbridge.exe"
rm -f "$BIN_DIR/simbridge-server"
rm -f "$BIN_DIR/simbridge-launch"
echo "  done"
echo ""

# ── Desktop entry ──
echo "[3/4] Removing desktop entry..."
rm -f "$DESKTOP_DIR/simbridge.desktop"
echo "  done"
echo ""

# ── Icon ──
echo "[4/4] Removing icon..."
rm -f "$ICON_DIR/simbridge.svg"
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi
echo "  done"
echo ""

echo "=== simbridge uninstalled ==="
