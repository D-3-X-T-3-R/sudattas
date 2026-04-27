<#
  Validate that migration-created critical tables are present in the active DB.

  Usage:
    .\assert-required-schema.ps1
#>

param(
    [string[]]$RequiredTables = @(
        "RefundAttempts",
        "ReturnRequests",
        "ReturnRequestItems",
        "OrderInventoryRestores",
        "OrderInventoryRestoreItems",
        "Invoices",
        "SchemaMigrations"
    )
)

$ErrorActionPreference = "Stop"
$BackendRoot = $PSScriptRoot
$EnvFile = Join-Path $BackendRoot ".env"
$DbContainerName = "sudattas-mysql"
$mysqlHost = "127.0.0.1"

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
        Database = $matches[3]
    }
}

function Invoke-MySqlQuery {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Sql
    )

    $tempFile = [System.IO.Path]::GetTempFileName()
    $containerSqlPath = "/tmp/schema-assert.sql"
    try {
        Set-Content -Path $tempFile -Value $Sql -Encoding UTF8
        docker cp $tempFile "${DbContainerName}:${containerSqlPath}" 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to copy SQL into DB container."
        }
        $rows = docker exec -e MYSQL_PWD=$DbPassword $DbContainerName sh -c "mysql -N -s -h$mysqlHost -u$DbUser $DbName < $containerSqlPath" 2>$null
        if ($LASTEXITCODE -ne 0) {
            throw "Failed to execute schema validation query."
        }
        return $rows
    } finally {
        Remove-Item -LiteralPath $tempFile -Force -ErrorAction SilentlyContinue
    }
}

function Normalize-TableName {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name
    )

    return (($Name.ToLowerInvariant() -replace '[^a-z0-9]', '')).Trim()
}

if (-not $RequiredTables -or $RequiredTables.Count -eq 0) {
    throw "RequiredTables cannot be empty."
}

$running = docker ps --format "{{.Names}}" 2>$null | Where-Object { $_ -eq $DbContainerName }
if (-not $running) {
    throw "ERROR: DB container '$DbContainerName' is not running."
}

$creds = Get-DbCredentials
$DbUser = $creds.User
$DbPassword = $creds.Password
$DbName = $creds.Database

$maxAttempts = 45
$attempt = 0
$probeOk = $false
Write-Host "Validating DB connectivity before schema assertion..." -ForegroundColor Yellow
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
    throw "ERROR: MySQL did not become ready in time."
}

$sql = @"
SELECT table_name
FROM information_schema.tables
WHERE LOWER(table_schema) = LOWER('$($DbName -replace "'", "''")');
"@

$foundRows = Invoke-MySqlQuery -Sql $sql
$found = @{}
foreach ($row in $foundRows) {
    $normalized = Normalize-TableName "$row"
    if ($normalized) { $found[$normalized] = $true }
}

$missing = @()
foreach ($table in $RequiredTables) {
    $normalizedRequired = Normalize-TableName $table
    if (-not $found.ContainsKey($normalizedRequired)) {
        $missing += $table
    }
}

if ($missing.Count -gt 0) {
    throw "ERROR: Missing required migration table(s): $($missing -join ', '). Database is NOT ready."
}

Write-Host "Schema validation passed. Required migration tables are present." -ForegroundColor Green
