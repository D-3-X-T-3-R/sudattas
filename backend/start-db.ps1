# Start database, apply migrations, validate schema, and regenerate entities.
# 1. Stop any running container using an image whose name starts with sudattas_
# 2. Run run_database_windows.bat (build and start container)
# 3. Apply forward migrations
# 4. Validate required migration-created tables
# 5. Run generate.ps1 to regenerate SeaORM entities

$ErrorActionPreference = "Stop"

$BackendRoot = $PSScriptRoot
$DatabaseDir = Join-Path $BackendRoot "database"
$EntityDir = Join-Path $BackendRoot "core_db_entities\src\entity"
$ApplyMigrationsScript = Join-Path $BackendRoot "apply-db-migrations.ps1"
$AssertSchemaScript = Join-Path $BackendRoot "assert-required-schema.ps1"

# 1. Check for running containers with image name matching sudattas_* and stop them
$running = docker ps --format "{{.ID}} {{.Image}}" 2>$null
if ($LASTEXITCODE -eq 0 -and $running) {
    foreach ($line in $running -split "`n") {
        $line = $line.Trim()
        if (-not $line) { continue }
        $id = ($line -split "\s+", 2)[0]
        $image = ($line -split "\s+", 2)[1]
        if ($image -like "sudattas_*") {
            Write-Host "Stopping container $id (image: $image)..." -ForegroundColor Yellow
            docker stop $id 2>$null
        }
    }
}

# 2. Execute run_database_windows.bat (must run from database directory)
$batPath = Join-Path $DatabaseDir "run_database_windows.bat"
if (-not (Test-Path $batPath)) {
    throw "Batch file not found: $batPath"
}
Write-Host "Starting database..." -ForegroundColor Cyan
Push-Location $DatabaseDir
try {
    & cmd /c $batPath
    if ($LASTEXITCODE -ne 0) { throw "run_database_windows.bat exited with $LASTEXITCODE" }
} finally {
    Pop-Location
}

# Give MySQL time to run init scripts and accept connections
Write-Host "Waiting for MySQL to be ready..." -ForegroundColor Cyan
Start-Sleep -Seconds 15

# 3. Apply forward DB migrations
if (-not (Test-Path -Path $ApplyMigrationsScript -PathType Leaf)) {
    throw "Migration script not found: $ApplyMigrationsScript"
}
Write-Host "Applying forward DB migrations..." -ForegroundColor Cyan
& $ApplyMigrationsScript
if ($LASTEXITCODE -ne 0) {
    throw "apply-db-migrations.ps1 exited with $LASTEXITCODE"
}

# 4. Validate required migration-created schema
if (-not (Test-Path -Path $AssertSchemaScript -PathType Leaf)) {
    throw "Schema validation script not found: $AssertSchemaScript"
}
Write-Host "Validating required DB schema..." -ForegroundColor Cyan
& $AssertSchemaScript
if ($LASTEXITCODE -ne 0) {
    throw "assert-required-schema.ps1 exited with $LASTEXITCODE"
}

# 5. Execute generate.ps1 (must run from entity directory)
$generatePath = Join-Path $EntityDir "generate.ps1"
if (-not (Test-Path $generatePath)) {
    throw "Generate script not found: $generatePath"
}
Write-Host "Regenerating entities..." -ForegroundColor Cyan
Push-Location $EntityDir
try {
    & $generatePath
    if ($LASTEXITCODE -ne 0) { throw "generate.ps1 exited with $LASTEXITCODE" }
} finally {
    Pop-Location
}

Write-Host ""
Write-Host "Done. Database is running, migrations were applied, schema validated, and entities were regenerated." -ForegroundColor Green
