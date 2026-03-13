# Start all backend services: MySQL, Redis, Core Operations (gRPC), GraphQL (all in Docker)
# 1) Backup current DB (if container is running)
# 2) Stop existing sudattas* containers, then start fresh via docker-compose
# 3) Restore DB from latest backup (if any)
# Run from backend/: .\start-services.ps1

$ErrorActionPreference = "Stop"
$BackendRoot = $PSScriptRoot

# 1) Backup DB before tearing down containers
Write-Host "Backing up DB (if running)..." -ForegroundColor Yellow
& "$BackendRoot\backup-db.ps1"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Backup skipped or failed (DB container may not be running)." -ForegroundColor Gray
}

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

# 3) Restore DB from latest backup (if any)
Write-Host "Restoring DB from latest backup (if any)..." -ForegroundColor Yellow
Start-Sleep -Seconds 3  # allow MySQL to accept connections
& "$BackendRoot\restore-db.ps1"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Restore skipped or failed (no backup file or DB not ready)." -ForegroundColor Gray
}

Write-Host ""
Write-Host "Done. All services running in Docker:" -ForegroundColor Green
Write-Host "  MySQL (3306), Redis (6379), Core Operations (50051), GraphQL (8080)" -ForegroundColor Gray
Write-Host "  Stop with: docker-compose down" -ForegroundColor Gray
