# Start all backend services: MySQL, Redis, Core Operations (gRPC), GraphQL (all in Docker)
# Stops existing sudattas* containers, then starts fresh via docker-compose.
# Run from backend/: .\start-services.ps1

$ErrorActionPreference = "Stop"
$BackendRoot = $PSScriptRoot

Write-Host "Stopping existing sudattas containers..." -ForegroundColor Yellow
$containers = docker ps -a --format "{{.Names}}" 2>$null | Where-Object { $_ -match "^sudattas" }
if ($containers) {
    $containers | ForEach-Object { docker rm -f $_ 2>$null }
    Write-Host "Removed: $($containers -join ', ')" -ForegroundColor Gray
} else {
    Write-Host "No existing sudattas containers found." -ForegroundColor Gray
}

Write-Host "Building and starting all services (MySQL, Redis, Core Operations, GraphQL)..." -ForegroundColor Cyan
Push-Location $BackendRoot
try {
    docker-compose up -d --build
    if ($LASTEXITCODE -ne 0) { throw "docker-compose failed" }
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "Done. All services running in Docker:" -ForegroundColor Green
Write-Host "  MySQL (3306), Redis (6379), Core Operations (50051), GraphQL (8080)" -ForegroundColor Gray
Write-Host "  Stop with: docker-compose down" -ForegroundColor Gray
