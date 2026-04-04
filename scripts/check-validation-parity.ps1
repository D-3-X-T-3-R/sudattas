$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$backendFile = Join-Path $Root "backend/graphql/src/validation.rs"
$frontendFile = Join-Path $Root "frontend/src/lib/validation-schemas.ts"

function Get-BackendConst([string]$name) {
    $line = Select-String -Path $backendFile -Pattern "pub const ${name}:"
    if (-not $line) { throw "Failed to parse backend constant: $name" }
    if ($line.Line -match "= ([0-9]+)") { return [int]$Matches[1] }
    throw "Failed to parse backend constant value: $name"
}

function Get-FrontendConst([string]$name) {
    $line = Select-String -Path $frontendFile -Pattern "export const ${name} ="
    if (-not $line) { throw "Failed to parse frontend constant: $name" }
    if ($line.Line -match "= ([0-9]+)") { return [int]$Matches[1] }
    throw "Failed to parse frontend constant value: $name"
}

$backendSku = Get-BackendConst "MAX_SKU_SLUG_LEN"
$backendQty = Get-BackendConst "MAX_QUANTITY_PER_ITEM"
$backendAddr = Get-BackendConst "MAX_ADDRESS_LINE_LEN"

$frontendSku = Get-FrontendConst "BACKEND_MAX_SKU_SLUG_LEN"
$frontendQty = Get-FrontendConst "BACKEND_MAX_QUANTITY_PER_ITEM"
$frontendAddr = Get-FrontendConst "BACKEND_MAX_ADDRESS_LINE_LEN"
$frontendPostal = Get-FrontendConst "BACKEND_POSTAL_CODE_LEN"

$failed = $false

if ($backendSku -ne $frontendSku) {
    Write-Host "Mismatch: SKU/slug length backend=$backendSku frontend=$frontendSku" -ForegroundColor Red
    $failed = $true
}
if ($backendQty -ne $frontendQty) {
    Write-Host "Mismatch: quantity limit backend=$backendQty frontend=$frontendQty" -ForegroundColor Red
    $failed = $true
}
if ($backendAddr -ne $frontendAddr) {
    Write-Host "Mismatch: address line length backend=$backendAddr frontend=$frontendAddr" -ForegroundColor Red
    $failed = $true
}
if ($frontendPostal -ne 6) {
    Write-Host "Mismatch: postal code length frontend=$frontendPostal expected=6" -ForegroundColor Red
    $failed = $true
}

if ($failed) {
    throw "Validation parity check failed."
}

Write-Host "OK validation parity: frontend and backend limits are aligned." -ForegroundColor Green
