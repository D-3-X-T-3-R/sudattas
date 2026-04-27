<#
Start backend services with DB-first bootstrap and explicit DB mode:
1) Backup current DB (if container is running)
2) Stop existing sudattas* containers
3) Start MySQL + Redis only
4) DB mode:
   - -PreserveData (default): restore latest backup, then apply forward migrations
   - -Fresh: load schema from database/sql_dump/01_schema.sql, then apply forward migrations
5) Regenerate SeaORM entities
6) Build and start Core Operations + GraphQL (skipped with -Orm)

Run from backend/: .\start-services.ps1 [-Fresh | -PreserveData] [-Orm]
#>

param(
    [switch]$Fresh,
    [switch]$PreserveData,
    [switch]$Orm
)

$ErrorActionPreference = "Stop"
$BackendRoot = $PSScriptRoot
$DbContainerName = "sudattas-mysql"
$EnvFile = Join-Path $BackendRoot ".env"
$SchemaFile = Join-Path $BackendRoot "database\sql_dump\01_schema.sql"
$AssertSchemaScript = Join-Path $BackendRoot "assert-required-schema.ps1"

if ($Fresh -and $PreserveData) {
    throw "Use either -Fresh or -PreserveData, not both."
}

$DbMode = if ($Fresh) { "fresh" } else { "preserve-data" }

function Invoke-Compose {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Args
    )

    Push-Location $BackendRoot
    try {
        & docker compose version *> $null
        if ($LASTEXITCODE -eq 0) {
            & docker compose @Args | Out-Host
            return [int]$LASTEXITCODE
        }
        & docker-compose @Args | Out-Host
        return [int]$LASTEXITCODE
    } finally {
        Pop-Location
    }
}

function Get-DbCredentials {
    if (-not (Test-Path $EnvFile)) {
        throw "ERROR: .env not found. Cannot resolve DATABASE_URL."
    }
    $databaseLine = Get-Content $EnvFile | Where-Object { $_ -match '^\s*DATABASE_URL=' } | Select-Object -First 1
    if (-not $databaseLine) {
        throw "ERROR: DATABASE_URL not found in .env."
    }
    $url = $databaseLine.Split('=', 2)[1].Trim()
    if ($url -notmatch '^mysql://([^:]+):([^@]+)@[^/]+/([^?]+)') {
        throw "ERROR: DATABASE_URL format not recognized."
    }

    return @{
        User = $matches[1]
        Password = $matches[2]
    }
}

function Wait-ForMysqlReady {
    param(
        [Parameter(Mandatory = $true)]
        [string]$DbUser,
        [Parameter(Mandatory = $true)]
        [string]$DbPassword
    )

    $maxAttempts = 45
    $attempt = 0
    Write-Host "Waiting for MySQL to accept connections..." -ForegroundColor Yellow
    while ($attempt -lt $maxAttempts) {
        $attempt++
        try {
            $null = docker exec -e MYSQL_PWD=$DbPassword $DbContainerName sh -c "mysql -h127.0.0.1 -u$DbUser -e 'select 1'" 2>&1
        } catch {
        }
        if ($LASTEXITCODE -eq 0) {
            return
        }
        Start-Sleep -Seconds 2
    }
    throw "ERROR: MySQL did not become ready in time."
}

function Load-FreshSchema {
    if (-not (Test-Path -Path $SchemaFile -PathType Leaf)) {
        throw "ERROR: schema file not found: $SchemaFile"
    }

    $creds = Get-DbCredentials
    Wait-ForMysqlReady -DbUser $creds.User -DbPassword $creds.Password

    Write-Host "Loading fresh schema from 01_schema.sql..." -ForegroundColor Yellow
    $null = docker cp $SchemaFile "${DbContainerName}:/tmp/01_schema.sql" 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to copy 01_schema.sql into DB container."
    }

    docker exec -e MYSQL_PWD=$($creds.Password) $DbContainerName sh -c "mysql -h127.0.0.1 -u$($creds.User) < /tmp/01_schema.sql" 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to load fresh schema."
    }
    Write-Host "Fresh schema loaded." -ForegroundColor Green
}

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
$composeStatus = Invoke-Compose -Args @("up", "-d", "mysql", "redis")
if ($composeStatus -ne 0) { throw "docker compose mysql/redis failed" }

if ($DbMode -eq "fresh") {
    Load-FreshSchema
} else {
    Write-Host "Restoring DB from latest backup (if any)..." -ForegroundColor Yellow
    Start-Sleep -Seconds 3
    & "$BackendRoot\restore-db.ps1"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Restore skipped or failed (no backup file or DB not ready)." -ForegroundColor Gray
    }
}

Write-Host "Applying forward DB migrations..." -ForegroundColor Yellow
& "$BackendRoot\apply-db-migrations.ps1"
if ($LASTEXITCODE -ne 0) { throw "apply-db-migrations.ps1 failed" }

Write-Host "Validating required DB schema..." -ForegroundColor Yellow
if (-not (Test-Path -Path $AssertSchemaScript -PathType Leaf)) {
    throw "Required schema validator not found: $AssertSchemaScript"
}
& $AssertSchemaScript
if ($LASTEXITCODE -ne 0) { throw "assert-required-schema.ps1 failed" }

Write-Host "Regenerating SeaORM entities..." -ForegroundColor Yellow
$entityGenerateScript = Join-Path $BackendRoot "core_db_entities\src\entity\generate.ps1"
if (-not (Test-Path -Path $entityGenerateScript -PathType Leaf)) {
    throw "Entity generation script not found: $entityGenerateScript"
}
$entityDir = Split-Path -Parent $entityGenerateScript
Push-Location $entityDir
try {
    & $entityGenerateScript
    if ($LASTEXITCODE -ne 0) { throw "generate.ps1 exited with $LASTEXITCODE" }
} finally {
    Pop-Location
}

docker exec -i $DbContainerName sh -c "mysql -u root -p12345678 -e 'DROP DATABASE IF EXISTS SUDATTAS_CLONED; CREATE DATABASE SUDATTAS_CLONED;' && mysqldump --set-gtid-purged=OFF -u root -p12345678 SUDATTAS | mysql -u root -p12345678 SUDATTAS_CLONED"
if ($Orm) {
    Write-Host "ORM-only mode (-Orm): skipping Core Operations and GraphQL." -ForegroundColor Yellow
} else {
    Write-Host "Building and starting app services (Core Operations, GraphQL)..." -ForegroundColor Cyan
    $composeStatus = Invoke-Compose -Args @("up", "-d", "--build", "core_operations", "graphql")
    if ($composeStatus -ne 0) { throw "docker compose core_operations/graphql failed" }
}

Write-Host "Pruning old DB backups (keeping latest 50)..." -ForegroundColor Yellow
$BackupDirForPrune = Join-Path $BackendRoot "database\db-backups"
if (Test-Path -Path $BackupDirForPrune -PathType Container) {
    Get-ChildItem -Path $BackupDirForPrune -Filter "db-backup-*.sql" -File -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending |
        Select-Object -Skip 50 |
        ForEach-Object {
            Remove-Item -LiteralPath $_.FullName -Force -ErrorAction SilentlyContinue
            Write-Host "  Removed: $($_.Name)" -ForegroundColor Gray
        }
}

Write-Host ""
if ($Orm) {
    Write-Host "Done (ORM-only). MySQL + Redis are running; SeaORM entities were regenerated." -ForegroundColor Green
    Write-Host "  MySQL (3306), Redis (6379) - Core Operations / GraphQL were not started." -ForegroundColor Gray
} else {
    Write-Host "Done. All services running in Docker:" -ForegroundColor Green
    Write-Host "  MySQL (3306), Redis (6379), Core Operations (50051), GraphQL (8080)" -ForegroundColor Gray
}
Write-Host "  DB mode: $DbMode" -ForegroundColor Gray
Write-Host "  SeaORM entities refreshed from current DB schema" -ForegroundColor Gray
Write-Host "  Stop with: docker compose down (or docker-compose down)" -ForegroundColor Gray
