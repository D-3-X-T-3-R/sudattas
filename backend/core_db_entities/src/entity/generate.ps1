# Generate SeaORM entities
# PowerShell equivalent of generate.sh

$ErrorActionPreference = "Stop"
$SeaOrmCliVersion = "1.1.20"

# Remove old entities
Remove-Item *.rs -ErrorAction SilentlyContinue

# Ensure a deterministic SeaORM CLI version is installed.
cargo install sea-orm-cli --version $SeaOrmCliVersion --locked
if ($LASTEXITCODE -ne 0) {
    throw "Failed to install/update sea-orm-cli"
}

# Ensure rustfmt exists for the active toolchain; sea-orm-cli runs formatting.
$activeToolchainLine = (& rustup show active-toolchain 2>$null | Select-Object -First 1)
$activeToolchain = if ($activeToolchainLine) { ($activeToolchainLine -split '\s+')[0] } else { $null }
if ($activeToolchain) {
    rustup component add rustfmt --toolchain $activeToolchain
} else {
    rustup component add rustfmt
}
if ($LASTEXITCODE -ne 0) {
    throw "Failed to install rustfmt for the active Rust toolchain."
}

# Generate entities from local Docker MySQL (SUDATTAS schema).
$EnvFile = Join-Path $PSScriptRoot "..\..\..\.env"
if (-not (Test-Path $EnvFile)) {
    throw "ERROR: .env not found at $EnvFile. Cannot resolve DATABASE_URL."
}
$databaseLine = Get-Content $EnvFile | Where-Object { $_ -match '^\s*DATABASE_URL=' } | Select-Object -First 1
if (-not $databaseLine) {
    throw "ERROR: DATABASE_URL not found in $EnvFile."
}
$DatabaseUrl = $databaseLine.Split('=', 2)[1].Trim()

sea-orm-cli generate entity `
  -u $DatabaseUrl `
  -o "." `
  --with-serde both `
  --date-time-crate chrono `
  --max-connections 1
if ($LASTEXITCODE -ne 0) {
    throw "sea-orm-cli entity generation failed."
}

Write-Host ""
Write-Host "Entities regenerated successfully!" -ForegroundColor Green
Write-Host ""

Write-Host "New tables:" -ForegroundColor Cyan
Get-ChildItem *.rs | Where-Object { $_.Name -match "session|payment|shipment|coupon|order_event|webhook" } | ForEach-Object { $_.Name }

Write-Host ""
Write-Host "Total entity files:" -ForegroundColor Cyan
(Get-ChildItem *.rs).Count

# Explicitly clear stale native exit codes for callers that check $LASTEXITCODE.
$global:LASTEXITCODE = 0
