#!/usr/bin/env bash
set -euo pipefail

BIN_DIR="$HOME/.local/bin"
SERVICE_DIR="$HOME/.config/systemd/user"
DESKTOP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor/scalable/apps"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== simbridge installer ==="
echo ""

# ── Stop running service ──
if systemctl --user is-active simbridge-server.service &>/dev/null; then
    echo "[0/5] Stopping running service..."
    systemctl --user stop simbridge-server.service
    echo ""
fi

# ── Build or locate binaries ──
if [ -f "$SCRIPT_DIR/bin/simbridge-server" ] && [ -f "$SCRIPT_DIR/bin/simbridge.exe" ]; then
    # Release tarball — pre-built binaries
    echo "[1/5] Using pre-built binaries..."
    BRIDGE_BIN="$SCRIPT_DIR/bin/simbridge.exe"
    SERVER_BIN="$SCRIPT_DIR/bin/simbridge-server"
    LAUNCH_BIN="$SCRIPT_DIR/bin/simbridge-launch"
elif [ -f "$SCRIPT_DIR/Cargo.toml" ]; then
    # Source checkout — build from source
    echo "[1/5] Building from source..."
    cd "$SCRIPT_DIR"
    make build
    BRIDGE_BIN="$SCRIPT_DIR/target/x86_64-pc-windows-gnu/release/simbridge.exe"
    SERVER_BIN="$SCRIPT_DIR/target/release/simbridge-server"
    LAUNCH_BIN="$SCRIPT_DIR/scripts/simbridge-launch"
else
    echo "Error: no pre-built binaries in bin/ and no Cargo.toml for source build."
    echo "Download a release from GitHub or clone the repo."
    exit 1
fi
echo ""

# ── Binaries ──
echo "[2/5] Installing binaries to $BIN_DIR/"
mkdir -p "$BIN_DIR"
cp "$BRIDGE_BIN" "$BIN_DIR/simbridge.exe"
cp "$SERVER_BIN" "$BIN_DIR/simbridge-server"
cp "$LAUNCH_BIN" "$BIN_DIR/simbridge-launch"
chmod +x "$BIN_DIR/simbridge-launch"
echo "  simbridge.exe"
echo "  simbridge-server"
echo "  simbridge-launch"
echo ""

# ── Systemd service ──
echo "[3/5] Installing systemd user service..."
mkdir -p "$SERVICE_DIR"
cp "$SCRIPT_DIR/simbridge-server.service" "$SERVICE_DIR/simbridge-server.service"
systemctl --user daemon-reload
systemctl --user enable simbridge-server.service
systemctl --user restart simbridge-server.service
echo "  simbridge-server.service (enabled + started)"
echo ""

# ── Desktop entry ──
echo "[4/5] Installing desktop entry..."
mkdir -p "$DESKTOP_DIR"
cp "$SCRIPT_DIR/simbridge.desktop" "$DESKTOP_DIR/simbridge.desktop"
echo "  simbridge.desktop"
echo ""

# ── Icon ──
echo "[5/5] Installing icon..."
mkdir -p "$ICON_DIR"
cp "$SCRIPT_DIR/assets/simbridge.svg" "$ICON_DIR/simbridge.svg"
if command -v gtk-update-icon-cache &>/dev/null; then
    gtk-update-icon-cache -f -t "$HOME/.local/share/icons/hicolor" 2>/dev/null || true
fi
echo "  simbridge.svg"
echo ""

echo "=== Done ==="
echo ""
echo "simbridge is installed. You should see it in your app launcher."
echo ""
echo "Admin panel:  http://localhost:8888"
echo ""
echo "Steam launch options (add to your game):"
echo "  ACC:      simbridge-launch acc %command%"
echo "  AC:       simbridge-launch ac %command%"
echo "  AC Rally: simbridge-launch acrally %command%"
echo "  AC Evo:   simbridge-launch acevo %command%"
echo "  rF2:      simbridge-launch rf2 %command%"
echo "  BeamNG:   simbridge-launch beamng %command%"
echo ""
echo "Dirt Rally 2.0 uses native UDP — no bridge needed."
echo "Enable UDP in hardware_settings_config.xml (see README)."
