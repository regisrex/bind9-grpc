#!/bin/bash

# Directory to watch (mounted volume from docker-compose)
ZONES_DIR="/var/lib/docker/volumes/dnsscriptylabscom_bind_zones/_data"
CONTAINER_NAME="dns_bind9"

echo "Starting zone file watcher for directory: $ZONES_DIR"
echo "Watching for changes in BIND9 container: $CONTAINER_NAME"

# Install inotify-tools if not present
if ! command -v inotifywait &> /dev/null; then
    echo "Installing inotify-tools..."
    apt-get update && apt-get install -y inotify-tools
fi

# Watch for file changes and reload BIND9
inotifywait -m -r -e modify,create,delete,move "$ZONES_DIR" |
while read path action file; do
    echo "Zone file change detected: $action $path$file"
    
    # Wait a moment for file operations to complete
    sleep 1
    
    # Try rndc reload first, fallback to SIGHUP
    if docker exec "$CONTAINER_NAME" rndc reload 2>/dev/null; then
        echo "BIND9 reloaded successfully via rndc"
    else
        echo "rndc failed, trying SIGHUP..."
        if docker exec "$CONTAINER_NAME" pkill -HUP named; then
            echo "BIND9 reloaded successfully via SIGHUP"
        else
            echo "Failed to reload BIND9"
        fi
    fi
done
