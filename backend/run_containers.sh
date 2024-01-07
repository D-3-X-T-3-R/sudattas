#!/bin/bash

echo "Running cargo clean..."
cargo clean 

prefix="sudattas"
echo "Searching for containers to delete..."
containers_to_delete=$(docker ps -a --format "{{.Names}}" | grep "^$prefix" | xargs)

if [ -z "$containers_to_delete" ]; then
    echo "No containers found with names starting with $prefix"
else
    echo "Deleting containers: $containers_to_delete"
    docker rm -f $containers_to_delete
fi

echo "Building the database..."
cd database/
database_container_name=$(./run_database.sh)
cd -

echo "Waiting for the database to build..."
for i in {1..120}; do echo -ne '\033[1;32m#\033[0m'; sleep 0.25; done; echo

echo "Setting up application containers..."
gql_container_name="sudattas-GraphQL-$(openssl rand -hex 6)"
core_operations_container_name="sudattas-core_operations-$(openssl rand -hex 6)"

echo "Building GraphQL app with no cache..."
docker build --no-cache -t graphql-app-local --target graphql-runner . 

echo "Building Core Operations app with no cache..."
docker build --no-cache -t core-operations-app-local --target core-operations-runner .

echo "Running GraphQL container: $gql_container_name"
docker run --name "$gql_container_name" -p 8080:8080 graphql-app-local &

echo "Running Core Operations container: $core_operations_container_name"
docker run --name "$core_operations_container_name" -p 50051:50051 core-operations-app-local &
