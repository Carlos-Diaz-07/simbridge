#!/usr/bin/env bash
# Launch simbridge inside a game's Proton prefix.
#
# Usage: launch.sh <steam_app_id> <game_adapter>
#
# Examples:
#   ./launch.sh 805550 acc      # ACC
#   ./launch.sh 244210 ac       # Assetto Corsa
#   ./launch.sh 3058630 acevo   # AC Evo
#   ./launch.sh 3917090 acrally # AC Rally
#   ./launch.sh 365960 rf2      # rFactor 2
#   ./launch.sh 284160 beamng   # BeamNG.drive
#
# The game must already be running (its Proton prefix must exist).

set -euo pipefail

APP_ID="${1:?Usage: launch.sh <steam_app_id> <game_adapter>}"
GAME="${2:?Usage: launch.sh <steam_app_id> <game_adapter>}"
SIMBRIDGE="${SIMBRIDGE_PATH:-$HOME/.local/bin/simbridge.exe}"
PORT="${SIMBRIDGE_PORT:-20777}"

# Find the Proton prefix
COMPAT_PATH="$HOME/.local/share/Steam/steamapps/compatdata/$APP_ID"
if [ ! -d "$COMPAT_PATH" ]; then
    echo "Error: Proton prefix not found at $COMPAT_PATH"
    echo "Is the game installed? Has it been run at least once?"
    exit 1
fi

# Find the Proton installation the game uses
PROTON_PATH=$(grep -l "$APP_ID" "$HOME/.local/share/Steam/steamapps/"*.acf 2>/dev/null | head -1 | xargs grep -oP '"proton_[^"]*"' 2>/dev/null | tr -d '"' || true)

# Fallback: find any Proton installation
if [ -z "$PROTON_PATH" ] || [ ! -d "$HOME/.local/share/Steam/compatibilitytools.d/$PROTON_PATH" ]; then
    # Try to find Proton from the compatdata config
    PROTON_BIN=$(find "$HOME/.local/share/Steam/compatibilitytools.d/" -name "proton" -type f 2>/dev/null | head -1)
    if [ -z "$PROTON_BIN" ]; then
        PROTON_BIN=$(find "$HOME/.local/share/Steam/steamapps/common/" -path "*/Proton*/proton" -type f 2>/dev/null | head -1)
    fi
else
    PROTON_BIN="$HOME/.local/share/Steam/compatibilitytools.d/$PROTON_PATH/proton"
fi

if [ -z "$PROTON_BIN" ] || [ ! -f "$PROTON_BIN" ]; then
    echo "Error: Could not find Proton installation"
    echo "Try running with protontricks instead:"
    echo "  protontricks-launch --appid $APP_ID $SIMBRIDGE -- $GAME --port $PORT"
    exit 1
fi

echo "[launch.sh] App ID: $APP_ID"
echo "[launch.sh] Game adapter: $GAME"
echo "[launch.sh] Proton: $PROTON_BIN"
echo "[launch.sh] Binary: $SIMBRIDGE"
echo "[launch.sh] UDP port: $PORT"

export STEAM_COMPAT_DATA_PATH="$COMPAT_PATH"
export STEAM_COMPAT_CLIENT_INSTALL_PATH="$HOME/.local/share/Steam"

exec "$PROTON_BIN" run "$SIMBRIDGE" "$GAME" --port "$PORT"
