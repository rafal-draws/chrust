#!/bin/sh

set -x

mkdir /server_data 
chown 755 -R /server_data

mkdir /metadata
chown 755 -R /metadata

docker build -t backend-etl data/.

docker compose up -d

# Wait for database to be ready
sleep 5

# Build with network access using legacy builder or host network
DOCKER_BUILDKIT=0 docker build -t backend-rust --network chrust_db back/. || \
  docker build -t backend-rust --network host back/.

docker run -d -p 3000:3000 --restart unless-stopped  -v /server_data:/server_data -v /metadata:/metadata --network chrust_db backend-rust
