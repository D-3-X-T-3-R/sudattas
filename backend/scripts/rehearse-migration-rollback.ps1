<#
  Rehearse schema migration + rollback on local Docker MySQL.

  Usage (from backend/):
    .\scripts\rehearse-migration-rollback.ps1 -ApplySchema
    .\scripts\rehearse-migration-rollback.ps1 -ApplySchema -SchemaFile .\database\sql_dump\01_schema.sql

  What it does:
    1) Creates a backup (backup-db.ps1)
    2) Captures pre-change table snapshot (name + row estimate)
    3) Optionally applies schema SQL
    4) Runs basic smoke checks
    5) Restores backup (restore-db.ps1)
    6) Captures post-restore snapshot and compares with pre-change snapshot
#>

param(
    [switch]$ApplySchema,
    [string]$SchemaFile = ""
)

$ErrorActionPreference = "Stop"
$BackendRoot = Split-Path -Parent $PSScriptRoot
$DbContainerName = "sudattas-mysql"
$DefaultSchema = Join-Path $BackendRoot "database\sql_dump\01_schema.sql"
$SchemaPath = if ($SchemaFile) { $SchemaFile } else { $DefaultSchema }

function Parse-DatabaseUrl {
    param([string]$EnvFilePath)
    if (-not (Test-Path $EnvFilePath)) {
        throw ".env file not found at '$EnvFilePath'"
    }
    $databaseLine = Get-Content $EnvFilePath | Where-Object { $_ -match '^\s*DATABASE_URL=' } | Select-Object -First 1
    if (-not $databaseLine) {
        throw "DATABASE_URL not found in $EnvFilePath"
    }
    $url = $databaseLine.Split('=', 2)[1].Trim()
    if ($url -notmatch '^mysql://([^:]+):([^@]+)@[^/]+/([^?]+)') {
        throw "DATABASE_URL format invalid (expected mysql://user:pass@host:port/db)"
    }
    return @{
        User = $matches[1]
        Password = $matches[2]
        Database = $matches[3]
    }
}

function Ensure-DbContainerRunning {
    $running = docker ps --format "{{.Names}}" 2>$null | Where-Object { $_ -eq $DbContainerName }
    if (-not $running) {
        throw "DB container '$DbContainerName' is not running."
    }
}

function Get-TableSnapshot {
    param(
        [string]$DbUser,
        [string]$DbPassword,
        [string]$DbName
    )
    $mysqlHost = "127.0.0.1"
    $query = @"
SELECT TABLE_NAME, TABLE_ROWS
FROM information_schema.tables
WHERE table_schema='${DbName}'
ORDER BY TABLE_NAME;
"@
    return docker exec -e MYSQL_PWD=$DbPassword $DbContainerName mysql "-h$mysqlHost" "-u$DbUser" "-N" "-e" $query
}

function Run-SmokeChecks {
    param(
        [string]$DbUser,
        [string]$DbPassword,
        [string]$DbName
    )
    $mysqlHost = "127.0.0.1"
    $requiredTables = @("users", "products", "orders", "order_details", "payment_intents")
    foreach ($t in $requiredTables) {
        $query = "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema='${DbName}' AND table_name='${t}';"
        $count = docker exec -e MYSQL_PWD=$DbPassword $DbContainerName mysql "-h$mysqlHost" "-u$DbUser" "-N" "-e" $query
        if ([int]$count -lt 1) {
            throw "Smoke check failed: missing required table '$t'"
        }
    }
}

Write-Host "== Migration + Rollback Rehearsal ==" -ForegroundColor Cyan
Ensure-DbContainerRunning

$envFile = Join-Path $BackendRoot ".env"
$db = Parse-DatabaseUrl -EnvFilePath $envFile
$backupScript = Join-Path $BackendRoot "backup-db.ps1"
$restoreScript = Join-Path $BackendRoot "restore-db.ps1"

Write-Host "Step 1/6: creating backup..." -ForegroundColor Yellow
& $backupScript

$backupDir = Join-Path $BackendRoot "database\db-backups"
$backupPath = (Get-ChildItem -Path $backupDir -Filter "db-backup-*.sql" -File | Sort-Object LastWriteTime -Descending | Select-Object -First 1).FullName
if (-not $backupPath) {
    throw "Could not locate backup file after backup step."
}
Write-Host "Backup: $backupPath" -ForegroundColor Gray

$timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$rehearsalDir = Join-Path $backupDir "rehearsal"
if (-not (Test-Path $rehearsalDir)) {
    New-Item -ItemType Directory -Path $rehearsalDir | Out-Null
}
$preSnapshotPath = Join-Path $rehearsalDir "snapshot-pre-$timestamp.txt"
$postSnapshotPath = Join-Path $rehearsalDir "snapshot-post-$timestamp.txt"

Write-Host "Step 2/6: capturing pre-change snapshot..." -ForegroundColor Yellow
$pre = Get-TableSnapshot -DbUser $db.User -DbPassword $db.Password -DbName $db.Database
$pre | Out-File -FilePath $preSnapshotPath -Encoding utf8

if ($ApplySchema) {
    if (-not (Test-Path $SchemaPath)) {
        throw "Schema file not found: $SchemaPath"
    }
    Write-Host "Step 3/6: applying schema from '$SchemaPath'..." -ForegroundColor Yellow
    $mysqlHost = "127.0.0.1"
    $containerSchema = "/tmp/rehearsal-schema.sql"
    docker cp $SchemaPath "${DbContainerName}:$containerSchema" | Out-Null
    docker exec -e MYSQL_PWD=$db.Password $DbContainerName sh -c "mysql -h$mysqlHost -u$db.User $db.Database < $containerSchema" | Out-Null
} else {
    Write-Host "Step 3/6: schema apply skipped (pass -ApplySchema to run full rehearsal)." -ForegroundColor DarkYellow
}

Write-Host "Step 4/6: running smoke checks..." -ForegroundColor Yellow
Run-SmokeChecks -DbUser $db.User -DbPassword $db.Password -DbName $db.Database

Write-Host "Step 5/6: restoring backup..." -ForegroundColor Yellow
& $restoreScript -BackupFile $backupPath

Write-Host "Step 6/6: capturing post-restore snapshot..." -ForegroundColor Yellow
$post = Get-TableSnapshot -DbUser $db.User -DbPassword $db.Password -DbName $db.Database
$post | Out-File -FilePath $postSnapshotPath -Encoding utf8

$diff = Compare-Object -ReferenceObject $pre -DifferenceObject $post
if ($diff) {
    Write-Host "Rollback rehearsal completed with snapshot differences." -ForegroundColor Yellow
    Write-Host "Review:"
    Write-Host "  $preSnapshotPath"
    Write-Host "  $postSnapshotPath"
    exit 2
}

Write-Host "Rollback rehearsal completed successfully (pre/post snapshots match)." -ForegroundColor Green
Write-Host "Artifacts:"
Write-Host "  $backupPath"
Write-Host "  $preSnapshotPath"
Write-Host "  $postSnapshotPath"

