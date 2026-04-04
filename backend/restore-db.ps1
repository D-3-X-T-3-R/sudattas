<#
  Restore the MySQL database from a backup file.

  Usage (from backend/):
    .\restore-db.ps1                    # restore from latest backup in database/db-backups/
    .\restore-db.ps1 -BackupFile path   # restore from specific .sql file

  - Reads DATABASE_URL from .env (no creds in script).
  - Copies the backup into the container and runs mysql to import.
#>

param(
    [string]$BackupFile
)

$ErrorActionPreference = "Stop"
$BackendRoot = $PSScriptRoot

$DbContainerName = "sudattas-mysql"
$DatabaseDir = Join-Path $BackendRoot "database"
$BackupDir = Join-Path $DatabaseDir "db-backups"

# Resolve backup file: explicit path or latest in db-backups
if ($BackupFile) {
    if (-not (Test-Path $BackupFile)) {
        Write-Host "ERROR: Backup file not found: '$BackupFile'." -ForegroundColor Red
        exit 1
    }
    $backupPath = (Resolve-Path $BackupFile).Path
} else {
    if (-not (Test-Path $BackupDir)) {
        Write-Host "ERROR: Backup directory not found: '$BackupDir'. Run backup-db.ps1 first or pass -BackupFile." -ForegroundColor Red
        exit 1
    }
    $latest = Get-ChildItem -Path $BackupDir -Filter "db-backup-*.sql" -File -ErrorAction SilentlyContinue |
        Sort-Object LastWriteTime -Descending | Select-Object -First 1
    if (-not $latest) {
        Write-Host "ERROR: No db-backup-*.sql files in '$BackupDir'. Run backup-db.ps1 first or pass -BackupFile." -ForegroundColor Red
        exit 1
    }
    $backupPath = $latest.FullName
}

Write-Host "Using backup file: $backupPath" -ForegroundColor Gray

# Credentials from .env
$envFile = Join-Path $BackendRoot ".env"
if (-not (Test-Path $envFile)) {
    Write-Host "ERROR: .env not found. Cannot resolve DATABASE_URL." -ForegroundColor Red
    exit 1
}
$databaseLine = Get-Content $envFile | Where-Object { $_ -match '^\s*DATABASE_URL=' } | Select-Object -First 1
if (-not $databaseLine) {
    Write-Host "ERROR: DATABASE_URL not found in .env." -ForegroundColor Red
    exit 1
}
$url = $databaseLine.Split('=', 2)[1].Trim()
if ($url -notmatch '^mysql://([^:]+):([^@]+)@[^/]+/([^?]+)') {
    Write-Host "ERROR: DATABASE_URL format not recognized." -ForegroundColor Red
    exit 1
}
$DbUser     = $matches[1]
$DbPassword = $matches[2]
$DbName     = $matches[3]

# Container must be running
$running = docker ps --format "{{.Names}}" 2>$null | Where-Object { $_ -eq $DbContainerName }
if (-not $running) {
    Write-Host "ERROR: DB container '$DbContainerName' is not running. Run start-services.ps1 first." -ForegroundColor Red
    exit 1
}

# Wait for MySQL to accept TCP connections (container can be "healthy" before server is ready)
$mysqlHost = "127.0.0.1"
$probeCmd = "mysql -h$mysqlHost -u$DbUser -e 'select 1'"
$maxAttempts = 45
$attempt = 0
Write-Host "Waiting for MySQL to accept connections..." -ForegroundColor Yellow
$probeOk = $false
while ($attempt -lt $maxAttempts) {
    $attempt++
    try {
        $null = docker exec -e MYSQL_PWD=$DbPassword $DbContainerName sh -c $probeCmd 2>&1
    } catch {
        # docker exec writes stderr -> PowerShell can throw; ignore and retry
    }
    if ($LASTEXITCODE -eq 0) {
        $probeOk = $true
        break
    }
    Start-Sleep -Seconds 2
}
if (-not $probeOk) {
    Write-Host "ERROR: MySQL did not become ready in time." -ForegroundColor Red
    exit 1
}

$containerPath = "/tmp/db-restore.sql"
$dockerCpDest = "${DbContainerName}:${containerPath}"

Write-Host "Copying backup into container..." -ForegroundColor Yellow
$null = docker cp $backupPath $dockerCpDest 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "ERROR: Failed to copy backup into container." -ForegroundColor Red
    exit 1
}

Write-Host "Restoring into database '$DbName'..." -ForegroundColor Yellow
$restoreCmd = "mysql -h$mysqlHost -u$DbUser $DbName < $containerPath"
docker exec -e MYSQL_PWD=$DbPassword $DbContainerName sh -c $restoreCmd 2>&1 | Out-Null

if ($LASTEXITCODE -eq 0) {
    Write-Host "Restore completed successfully." -ForegroundColor Green
    exit 0
} else {
    Write-Host "ERROR: Restore failed (mysql exit code $LASTEXITCODE)." -ForegroundColor Red
    exit 1
}
