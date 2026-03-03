#!/bin/bash
set -e

echo "Running DB migrations..."
diesel migration run --database-url "$DATABASE_URL"

mkdir -p /etc/bind/zones

echo "Starting dns_scriptylabs_com..."
exec ./dns_scriptylabs_com
