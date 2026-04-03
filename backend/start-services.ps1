# Start backend services with DB-first bootstrap:
# 1) Backup current DB (if container is running)
# 2) Stop existing sudattas* containers
# 3) Start MySQL + Redis only
# 4) Restore DB from latest backup (if any)
# 5) Regenerate SeaORM entities
# 6) Build and start Core Operations + GraphQL
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

Write-Host "Starting database dependencies (MySQL, Redis)..." -ForegroundColor Cyan
Push-Location $BackendRoot
try {
    docker-compose up -d mysql redis
    if ($LASTEXITCODE -ne 0) { throw "docker-compose mysql/redis failed" }
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

# 5) Regenerate SeaORM entities from current DB schema
Write-Host "Regenerating SeaORM entities..." -ForegroundColor Yellow
$entityGenerateScript = Join-Path $BackendRoot "core_db_entities\src\entity\generate.ps1"
if (-not (Test-Path -Path $entityGenerateScript -PathType Leaf)) {
    Write-Host "Entity generation script not found: $entityGenerateScript" -ForegroundColor Gray
} else {
    $seaOrmCli = Get-Command sea-orm-cli -ErrorAction SilentlyContinue
    if (-not $seaOrmCli) {
        Write-Host "sea-orm-cli is not installed; skipping entity regeneration." -ForegroundColor Gray
        Write-Host "Install with: cargo install sea-orm-cli" -ForegroundColor Gray
    } else {
        $entityDir = Split-Path -Parent $entityGenerateScript
        Push-Location $entityDir
        try {
            & $entityGenerateScript
            if ($LASTEXITCODE -ne 0) { throw "generate.ps1 exited with $LASTEXITCODE" }
        } finally {
            Pop-Location
        }
    }
}

# 6) Build and start app services after DB + entities are ready
Write-Host "Building and starting app services (Core Operations, GraphQL)..." -ForegroundColor Cyan
Push-Location $BackendRoot
try {
    docker-compose up -d --build core_operations graphql
    if ($LASTEXITCODE -ne 0) { throw "docker-compose core_operations/graphql failed" }
} finally {
    Pop-Location
}

# Keep only the latest 5 DB dumps under database/db-backups
Write-Host "Pruning old DB backups (keeping latest 5)..." -ForegroundColor Yellow
$BackupDirForPrune = Join-Path $BackendRoot "database\db-backups"
if (Test-Path -Path $BackupDirForPrune -PathType Container) {
    Get-ChildItem -Path $BackupDirForPrune -Filter "db-backup-*.sql" -File -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -Skip 5 |
        ForEach-Object {
            Remove-Item -LiteralPath $_.FullName -Force -ErrorAction SilentlyContinue
            Write-Host "  Removed: $($_.Name)" -ForegroundColor Gray
        }
}

Write-Host ""
Write-Host "Done. All services running in Docker:" -ForegroundColor Green
Write-Host "  MySQL (3306), Redis (6379), Core Operations (50051), GraphQL (8080)" -ForegroundColor Gray
Write-Host "  SeaORM entities refreshed from current DB schema (when sea-orm-cli is available)" -ForegroundColor Gray
Write-Host "  Stop with: docker-compose down" -ForegroundColor Gray
