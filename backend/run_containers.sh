
prefix="sudattas"

containers_to_delete=$(docker ps -a --format "{{.Names}}" | grep "^$prefix" | xargs)

if [ -z "$containers_to_delete" ]; then
    echo "No containers found with names starting with $prefix"
else
    echo "Deleting containers: $containers_to_delete"
    docker rm -f $containers_to_delete
fi

cd database/

database_container_name=$(./run_database.sh)

cd -

echo "Wainting for Db build to complete..."
sleep 30

gql_container_name="sudattas-GraohQL-$(openssl rand -hex 6)"
core_operations_container_name="sudattas-core_operations-$(openssl rand -hex 6)"

docker build -t my-graphql-app --target graphql-runner . 
docker build -t my-core-operations-app --target core-operations-runner .

docker run --rm --name $gql_container_name -p 8080:8080 my-graphql-app &
docker run --rm --name $core_operations_container_name -p 50051:50051 my-core-operations-app &
