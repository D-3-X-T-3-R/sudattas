@echo off
setlocal enabledelayedexpansion

set "prefix=sudattas"

:: Get a list of containers with names starting with the prefix
for /f "tokens=*" %%i in ('docker ps -a --format "{{.Names}}" ^| findstr /r /c:"^%prefix%"') do (
    set "containers_to_delete=!containers_to_delete! %%i"
)

:: Check if there are containers to delete
if "%containers_to_delete%"=="" (
    echo No containers found with names starting with %prefix%
) else (
    echo Deleting containers:%containers_to_delete%
    docker rm -f %containers_to_delete%
)

:: Change directory to 'database' and run 'run_database.sh'
cd database
call run_database_windows.bat
cd ..

echo Waiting for Db build to complete...
timeout /t 30 /nobreak

set "gql_container_name=sudattas-GraohQL-!random!"
set "core_operations_container_name=sudattas-core_operations-!random!"

:: Build the GraphQL image
docker build -t my-graphql-app --target graphql-runner .

:: Build the Core Operations image
docker build -t my-core-operations-app --target core-operations-runner .

:: Run the GraphQL container
start "GraphQL Container" docker run --rm --name !gql_container_name! -p 8080:8080 my-graphql-app

:: Run the Core Operations container
start "Core Operations Container" docker run --rm --name !core_operations_container_name! -p 50051:50051 my-core-operations-app
