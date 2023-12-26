@echo off
setlocal enabledelayedexpansion

:: Build the Docker image with the specified tag
docker build -t sudattas_local .

:: Generate a random container name
set "container_name=sudattas-database-!random!"

:: Run a Docker container with the specified name, environment variables, and ports
docker run --name !container_name! -e MYSQL_ROOT_PASSWORD=12345678 -p 3306:3306 -d sudattas_local

echo !container_name!
