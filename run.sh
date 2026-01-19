#!/bin/bash
# Niri Keypad Helper Script

# Ensure NIRI_SOCKET is set (usually handled by niri/wayland environment)
if [ -z "$NIRI_SOCKET" ]; then
    echo "Warning: NIRI_SOCKET is not set. Niri IPC might fail."
fi

# Ensure we run from the script directory so cargo finds Cargo.toml
cd "$(dirname "$0")"

# Build and run
cargo run --quiet -- "$@"
