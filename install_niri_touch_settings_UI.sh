#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== Building niri-touch-settings (debug) ==="
cd "$SCRIPT_DIR"
cargo build

echo ""
echo "=== Stopping running instance ==="
killall niri-touch-settings 2>/dev/null && echo "Stopped." || echo "Not running."

echo ""
echo "=== Installing niri-touch-settings ==="
sudo cp "$SCRIPT_DIR/target/debug/niri-touch-settings" /usr/local/bin/niri-touch-settings

echo ""
echo "Done! Run 'niri-touch-settings' to open the settings app."
