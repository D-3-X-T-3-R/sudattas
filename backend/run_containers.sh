#!/bin/bash

start_time=$(date +%s)

echo "Building the database..."
cd database/
database_container_name=$(./run_database.sh)
cd -

echo "Setting up application containers..."

# echo "Building GraphQL app with no cache..."
sudo docker build --no-cache -t graphql-app-local --target graphql-runner . 

echo "Building Core Operations app with no cache..."
sudo docker build --no-cache -t core-operations-app-local --target core-operations-runner .

echo "Saving Images"
docker save sudattas-database-local:latest > sudattas-database-local.tar
docker save core-operations-app-local:latest > core-operations-app-local.tar
docker save graphql-app-local:latest > graphql-app-local.tar

scp -r -i sudattas.pem *.tar ubuntu@13.233.125.216:/home/ubuntu/backend

end_time=$(date +%s)
runtime=$(($end_time - $start_time))

echo "Time taken to execute : $runtime seconds"
