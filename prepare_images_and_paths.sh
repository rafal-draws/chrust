#!/bin/sh

set -x

mkdir /server_data 
chown 755 -R /server_data


mkdir /metadata
chown 755 -R /metadata


docker build -t backend-etl data/.

docker compose up -d 

docker build -t backend-rust back/.

docker run -p 3000:3000  -v /server_data:/server_data -v /metadata:/util --network masters_db backend-rust
