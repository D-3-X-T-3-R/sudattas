<# 
  Backup the MySQL database running in the sudattas-mysql container.

  Usage (from backend/):
    .\backup-db.ps1

  - Reads DATABASE_URL from .env to get DB name/user/password.
  - Uses docker exec + mysqldump to create a timestamped .sql file
    under backend/db-backups/.
#>

$ErrorActionPreference = "Stop"
$BackendRoot = $PSScriptRoot

$DbContainerName = "sudattas-mysql"
# Store backups under backend/database/db-backups
$DatabaseDir = Join-Path $BackendRoot "database"
$BackupDir = Join-Path $DatabaseDir "db-backups"

# Derive DB credentials strictly from backend/.env → DATABASE_URL=mysql://user:pass@host:port/DBNAME
$envFile = Join-Path $BackendRoot ".env"
if (-not (Test-Path $envFile)) {
    Write-Host "ERROR: .env file not found at '$envFile'. Cannot resolve DATABASE_URL." -ForegroundColor Red
    exit 1
}

$databaseLine = Get-Content $envFile | Where-Object { $_ -match '^\s*DATABASE_URL=' } | Select-Object -First 1
if (-not $databaseLine) {
    Write-Host "ERROR: DATABASE_URL not found in .env. Cannot determine DB credentials." -ForegroundColor Red
    exit 1
}

$url = $databaseLine.Split('=', 2)[1].Trim()
# Basic parse: mysql://user:password@host:port/dbname
if ($url -notmatch '^mysql://([^:]+):([^@]+)@[^/]+/([^?]+)') {
    Write-Host "ERROR: DATABASE_URL is not in expected mysql://user:pass@host:port/db format." -ForegroundColor Red
    exit 1
}

$DbUser     = $matches[1]
$DbPassword = $matches[2]
$DbName     = $matches[3]

Write-Host "Using DB credentials from DATABASE_URL: user='$DbUser', db='$DbName'." -ForegroundColor Gray

# Ensure backup directory exists
if (-not (Test-Path $BackupDir)) {
    New-Item -ItemType Directory -Path $BackupDir | Out-Null
}

# Check that the DB container is running
Write-Host "Checking if DB container '$DbContainerName' is running..." -ForegroundColor Yellow
$running = docker ps --format "{{.Names}}" 2>$null | Where-Object { $_ -eq $DbContainerName }
if (-not $running) {
    Write-Host "ERROR: DB container '$DbContainerName' is not running. Start services first." -ForegroundColor Red
    exit 1
}

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$backupFile = Join-Path $BackupDir ("db-backup-{0}.sql" -f $timestamp)

Write-Host "Creating backup from container '$DbContainerName' into '$backupFile'..." -ForegroundColor Cyan

try {
    # Use mysqldump inside the container and stream to a local file.
    # --set-gtid-purged=OFF avoids GTID warning on single-DB dumps and keeps the backup portable.
    $mysqlHost = "127.0.0.1"
    docker exec -e MYSQL_PWD=$DbPassword $DbContainerName mysqldump "--set-gtid-purged=OFF" "-h$mysqlHost" "-u$DbUser" $DbName 2>&1 |
        Out-File -FilePath $backupFile -Encoding utf8

    $fileInfo = Get-Item -Path $backupFile -ErrorAction SilentlyContinue
    if ($null -ne $fileInfo -and $fileInfo.Length -gt 0) {
        Write-Host "Backup created successfully: '$backupFile'." -ForegroundColor Green
        exit 0
    } else {
        throw "Backup file '$backupFile' is empty or missing."
    }
} catch {
    Write-Host "ERROR: Failed to create DB backup: $_" -ForegroundColor Red
    exit 1
}

