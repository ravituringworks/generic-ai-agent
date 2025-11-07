#!/bin/bash

# Run The Agency Workflow Builder Desktop App
# This script ensures the daemon is running and launches the desktop app

set -e

echo "🚀 Starting The Agency Workflow Builder..."
echo ""

# Check if port 8080 is available or daemon is running
if lsof -Pi :8080 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "✓ Daemon already running on port 8080"
else
    echo "Starting agency daemon..."
    cargo run --bin agency-daemon --release > /tmp/agency-daemon.log 2>&1 &
    DAEMON_PID=$!

    echo "Waiting for daemon to start (PID: $DAEMON_PID)..."

    # Wait up to 30 seconds for daemon
    for i in {1..30}; do
        if curl -s http://localhost:8080/health > /dev/null 2>&1; then
            echo "✓ Daemon is ready after ${i} seconds"
            break
        fi
        sleep 1
        echo -n "."
    done
    echo ""
fi

# Launch the desktop app
echo ""
echo "🎨 Launching Workflow Builder Desktop App..."
cargo run --bin workflow-builder --features tauri

# Cleanup
echo ""
echo "Desktop app closed"
