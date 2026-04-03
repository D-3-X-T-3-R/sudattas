param(
    [string]$ReadyUrl = "http://127.0.0.1:8080/ready"
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$Backend = Join-Path $Root "backend"

Write-Host "== Release Gates ==" -ForegroundColor Cyan

Write-Host "Gate: no exposed privileged NEXT_PUBLIC env vars" -ForegroundColor Yellow
& (Join-Path $Root "scripts/check-no-privileged-public-env.ps1")

Write-Host "Gate: frontend/backend validation limits stay aligned" -ForegroundColor Yellow
& (Join-Path $Root "scripts/check-validation-parity.ps1")

Write-Host "Gate: backend health check green ($ReadyUrl)" -ForegroundColor Yellow
try {
    $null = Invoke-WebRequest -Uri $ReadyUrl -Method GET -UseBasicParsing -TimeoutSec 10
    Write-Host "OK   backend readiness healthy" -ForegroundColor Green
} catch {
    Write-Host "FAIL backend readiness endpoint not healthy: $ReadyUrl" -ForegroundColor Red
    exit 1
}

Write-Host "Gate: admin authorization tests green" -ForegroundColor Yellow
Push-Location $Backend
try {
    cargo test -p graphql --test graphql_tests test_admin_mutation_requires_admin_authorization -- --exact
    cargo test -p graphql --test graphql_tests test_search_user_requires_admin_authorization -- --exact

    Write-Host "Gate: payment negative-path tests green" -ForegroundColor Yellow
    cargo test -p core_operations --test handler_payment_intents verify_razorpay_payment_missing_fields_returns_invalid_argument -- --exact
    cargo test -p core_operations --test handler_payment_intents verify_razorpay_payment_not_configured_returns_failed_precondition -- --exact
} finally {
    Pop-Location
}

Write-Host "All configured release gates passed." -ForegroundColor Green
