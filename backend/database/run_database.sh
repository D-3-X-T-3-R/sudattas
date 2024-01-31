#!/usr/bin/env bash

echo "Building Docker image with no cache..."
sudo docker build --no-cache -t sudattas-database-local .

# sudo docker run --name db -e MYSQL_ROOT_PASSWORD=12345678 -p 3306:3306 -d sudattas-database-local
# container_name="sudattas-database-$(openssl rand -hex 6)"

# echo "Container started: $container_name"
