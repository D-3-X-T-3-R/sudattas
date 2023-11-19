#!/usr/bin/env bash
docker build -t sudattas_local .

container_name="sudattas-$(openssl rand -hex 6)"

docker run --name $container_name -e MYSQL_ROOT_PASSWORD=12345678 -p 3306:3306 -d sudattas_local

echo "container_name => $container_name"

# Verify
# docker exec -it $container_name bash
