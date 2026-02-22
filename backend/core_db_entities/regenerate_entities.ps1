# Regenerate SeaORM Entities Script
# Run this in PowerShell: .\regenerate_entities.ps1

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "SeaORM Entity Regeneration Script" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host ""

# Step 1: Check if Docker is running
Write-Host "[1/6] Checking Docker..." -ForegroundColor Yellow
try {
    $dockerVersion = docker version 2>$null
    if ($LASTEXITCODE -ne 0) {
        throw "Docker not found"
    }
    Write-Host "[OK] Docker is available" -ForegroundColor Green
} catch {
    Write-Host "[FAIL] Docker not found. Please start Docker Desktop." -ForegroundColor Red
    exit 1
}

# Step 2: Start Docker services
Write-Host ""
Write-Host "[2/6] Starting Docker services..." -ForegroundColor Yellow
Set-Location D:\personal\sudattas
docker-compose up -d

if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] Failed to start Docker services" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] Docker services started" -ForegroundColor Green

# Step 3: Wait for MySQL to be ready
Write-Host ""
Write-Host "[3/6] Waiting for MySQL to be ready (30 seconds)..." -ForegroundColor Yellow
Start-Sleep -Seconds 30

# Test MySQL connection
$maxRetries = 5
$retry = 0
$connected = $false

while ($retry -lt $maxRetries -and -not $connected) {
    try {
        $result = docker exec sudattas-mysql mysql -u sudattas_user -psudattas_pass_2024 SUDATTAS -e "SELECT 1;" 2>$null
        if ($LASTEXITCODE -eq 0) {
            $connected = $true
            Write-Host "[OK] MySQL is ready" -ForegroundColor Green
        }
    } catch {
        $retry++
        Write-Host "  Retry $retry/$maxRetries..." -ForegroundColor Yellow
        Start-Sleep -Seconds 5
    }
}

if (-not $connected) {
    Write-Host "[FAIL] Could not connect to MySQL" -ForegroundColor Red
    Write-Host "  Try: docker logs sudattas-mysql" -ForegroundColor Yellow
    exit 1
}

# Step 4: Verify database schema
Write-Host ""
Write-Host "[4/6] Checking database tables..." -ForegroundColor Yellow
$tableOutput = docker exec sudattas-mysql mysql -u sudattas_user -psudattas_pass_2024 SUDATTAS -e "SHOW TABLES;" 2>$null
$tableCount = ($tableOutput -split "`n").Count - 1
Write-Host "[OK] Found $tableCount tables" -ForegroundColor Green

# Check for Phase 1 tables
$phase1Tables = @("sessions", "payment_intents", "shipments", "coupons", "order_events", "webhook_events")
foreach ($table in $phase1Tables) {
    $exists = docker exec sudattas-mysql mysql -u sudattas_user -psudattas_pass_2024 SUDATTAS -e "SHOW TABLES LIKE '$table';" 2>$null
    if ($exists -match $table) {
        Write-Host "  [OK] $table exists" -ForegroundColor Green
    } else {
        Write-Host "  [WARN] $table NOT found" -ForegroundColor Yellow
    }
}

# Step 5: Remove old entity files
Write-Host ""
Write-Host "[5/6] Removing old entity files..." -ForegroundColor Yellow
Set-Location D:\personal\sudattas\backend\core_db_entities\src\entity
$oldFiles = Get-ChildItem *.rs -ErrorAction SilentlyContinue
if ($oldFiles) {
    $oldFiles | Remove-Item
    Write-Host "[OK] Removed $($oldFiles.Count) old entity files" -ForegroundColor Green
} else {
    Write-Host "[OK] No old files to remove" -ForegroundColor Green
}

# Step 6: Generate new entities
Write-Host ""
Write-Host "[6/6] Generating new entity files..." -ForegroundColor Yellow
Set-Location D:\personal\sudattas\backend\core_db_entities

$connectionString = "mysql://sudattas_user:sudattas_pass_2024@localhost:3306/SUDATTAS"
sea-orm-cli generate entity -u $connectionString -o src\entity --with-serde both --date-time-crate chrono --max-connections 1

if ($LASTEXITCODE -ne 0) {
    Write-Host "[FAIL] Entity generation failed" -ForegroundColor Red
    Write-Host ""
    Write-Host "Troubleshooting:" -ForegroundColor Yellow
    Write-Host "  1. Make sure sea-orm-cli is installed: cargo install sea-orm-cli" -ForegroundColor Gray
    Write-Host "  2. Check MySQL is accessible: docker exec sudattas-mysql mysql -u sudattas_user -psudattas_pass_2024 -e 'SELECT 1;'" -ForegroundColor Gray
    exit 1
}

Write-Host "[OK] Entity generation complete" -ForegroundColor Green

# Verify new files
Write-Host ""
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Summary" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan

$newFiles = Get-ChildItem src\entity\*.rs -ErrorAction SilentlyContinue
Write-Host "Total entity files: $($newFiles.Count)" -ForegroundColor White

Write-Host ""
Write-Host "Phase 1 entity files:" -ForegroundColor White
$phase1Entities = $newFiles | Where-Object { $_.Name -match "session|payment|shipment|coupon|order_event|webhook" }
if ($phase1Entities) {
    foreach ($file in $phase1Entities) {
        Write-Host "  [OK] $($file.Name)" -ForegroundColor Green
    }
} else {
    Write-Host "  [WARN] No Phase 1 entities found" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Sample of all entity files:" -ForegroundColor White
$newFiles | Select-Object -First 10 | ForEach-Object {
    Write-Host "  - $($_.Name)" -ForegroundColor Gray
}
if ($newFiles.Count -gt 10) {
    Write-Host "  ... and $($newFiles.Count - 10) more" -ForegroundColor Gray
}

Write-Host ""
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "[SUCCESS] Entity regeneration complete!" -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Build project: cd D:\personal\sudattas\backend\core_operations && cargo build" -ForegroundColor Gray
Write-Host "  2. Run tests: cargo test --lib" -ForegroundColor Gray
Write-Host ""
