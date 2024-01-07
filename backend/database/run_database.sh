#!/usr/bin/env bash

echo "Building Docker image with no cache..."
docker build --no-cache -t sudattas_local .

container_name="sudattas-database-$(openssl rand -hex 6)"

echo "Running container: $container_name"
docker run --name $container_name -e MYSQL_ROOT_PASSWORD=12345678 -p 3306:3306 -d sudattas_local

echo "Container started: $container_name"
