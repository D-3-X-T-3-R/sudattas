<#
  Apply forward-only SQL migrations from database/migrations against SUDATTAS.
  Tracks applied files in schema_migrations(file_name, applied_at).

  Usage:
    .\apply-db-migrations.ps1
#>

$ErrorActionPreference = "Stop"
$BackendRoot = $PSScriptRoot
$EnvFile = Join-Path $BackendRoot ".env"
$MigrationsDir = Join-Path $BackendRoot "database\migrations"
$DbContainerName = "sudattas-mysql"
$mysqlHost = "127.0.0.1"

function Invoke-MySqlScript {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SqlContent,
        [switch]$CaptureOutput
    )

    $tempFile = [System.IO.Path]::GetTempFileName()
    $containerSqlPath = "/tmp/db-migrations-query.sql"
    try {
        Set-Content -Path $tempFile -Value $SqlContent -Encoding UTF8
        docker cp $tempFile "${DbContainerName}:${containerSqlPath}" 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to copy SQL script into container."
        }

        if ($CaptureOutput) {
            $result = docker exec -e MYSQL_PWD=$DbPassword $DbContainerName sh -c "mysql -N -s -h$mysqlHost -u$DbUser $DbName < $containerSqlPath" 2>$null
            if ($LASTEXITCODE -ne 0) {
                throw "Failed to execute SQL script."
            }
            return $result
        }

        docker exec -e MYSQL_PWD=$DbPassword $DbContainerName sh -c "mysql -h$mysqlHost -u$DbUser $DbName < $containerSqlPath" 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to execute SQL script."
        }
        return $null
    } finally {
        Remove-Item -LiteralPath $tempFile -Force -ErrorAction SilentlyContinue
    }
}

if (-not (Test-Path -Path $MigrationsDir -PathType Container)) {
    Write-Host "Migrations directory not found: $MigrationsDir" -ForegroundColor Red
    exit 1
}

if (-not (Test-Path $EnvFile)) {
    Write-Host "ERROR: .env not found. Cannot resolve DATABASE_URL." -ForegroundColor Red
    exit 1
}

$databaseLine = Get-Content $EnvFile | Where-Object { $_ -match '^\s*DATABASE_URL=' } | Select-Object -First 1
if (-not $databaseLine) {
    Write-Host "ERROR: DATABASE_URL not found in .env." -ForegroundColor Red
    exit 1
}

$url = $databaseLine.Split('=', 2)[1].Trim()
if ($url -notmatch '^mysql://([^:]+):([^@]+)@[^/]+/([^?]+)') {
    Write-Host "ERROR: DATABASE_URL format not recognized." -ForegroundColor Red
    exit 1
}
$DbUser = $matches[1]
$DbPassword = $matches[2]
$DbName = $matches[3]

$running = docker ps --format "{{.Names}}" 2>$null | Where-Object { $_ -eq $DbContainerName }
if (-not $running) {
    Write-Host "ERROR: DB container '$DbContainerName' is not running." -ForegroundColor Red
    exit 1
}

$maxAttempts = 45
$attempt = 0
$probeOk = $false
Write-Host "Waiting for MySQL to accept connections..." -ForegroundColor Yellow
while ($attempt -lt $maxAttempts) {
    $attempt++
    try {
        $null = docker exec -e MYSQL_PWD=$DbPassword $DbContainerName sh -c "mysql -h$mysqlHost -u$DbUser -e 'select 1'" 2>&1
    } catch {
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

try {
    Invoke-MySqlScript -SqlContent @"
CREATE TABLE IF NOT EXISTS schema_migrations (
  file_name VARCHAR(255) PRIMARY KEY,
  applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
"@
} catch {
    Write-Host "ERROR: Failed to ensure schema_migrations table." -ForegroundColor Red
    exit 1
}

$migrationFiles = Get-ChildItem -Path $MigrationsDir -Filter "*.sql" -File -ErrorAction SilentlyContinue | Sort-Object Name
if (-not $migrationFiles -or $migrationFiles.Count -eq 0) {
    Write-Host "No migration files found in $MigrationsDir. Nothing to apply." -ForegroundColor Gray
    exit 0
}

$appliedCount = 0
$skippedCount = 0
foreach ($file in $migrationFiles) {
    $fileName = $file.Name
    $safeFileName = $fileName.Replace("'", "''")
    $exists = $null
    try {
        $exists = Invoke-MySqlScript -CaptureOutput -SqlContent "SELECT 1 FROM schema_migrations WHERE file_name='${safeFileName}' LIMIT 1;"
    } catch {
        Write-Host "ERROR: Failed to check migration state for: $fileName" -ForegroundColor Red
        exit 1
    }
    if (($exists | Out-String).Trim() -eq "1") {
        Write-Host "Skipping already applied migration: $fileName" -ForegroundColor Gray
        $skippedCount++
        continue
    }

    $containerSql = "/tmp/migration-$fileName"
    docker cp $file.FullName "${DbContainerName}:${containerSql}" 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to copy migration into container: $fileName" -ForegroundColor Red
        exit 1
    }

    Write-Host "Applying migration: $fileName" -ForegroundColor Yellow
    $applyCmd = "mysql -h$mysqlHost -u$DbUser $DbName < $containerSql"
    docker exec -e MYSQL_PWD=$DbPassword $DbContainerName sh -c $applyCmd 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Failed to apply migration: $fileName" -ForegroundColor Red
        exit 1
    }

    try {
        Invoke-MySqlScript -SqlContent "INSERT INTO schema_migrations (file_name) VALUES ('${safeFileName}');"
    } catch {
        Write-Host "ERROR: Failed to record migration: $fileName" -ForegroundColor Red
        exit 1
    }

    $appliedCount++
}

Write-Host "Migration apply complete. Applied: $appliedCount, skipped: $skippedCount." -ForegroundColor Green
