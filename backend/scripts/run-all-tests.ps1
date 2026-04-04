<#
Run all backend tests:
1) non-ignored workspace tests + doc tests
2) ignored core_operations DB/integration tests
3) ignored graphql e2e tests

Usage:
  .\scripts\run-all-tests.ps1
#>

$ErrorActionPreference = "Stop"
$BackendRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $BackendRoot

if (-not $env:TEST_DATABASE_URL) { $env:TEST_DATABASE_URL = "mysql://root:12345678@127.0.0.1:3306/SUDATTAS" }
if (-not $env:GRAPHQL_URL) { $env:GRAPHQL_URL = "http://127.0.0.1:8080" }
if (-not $env:GRAPHQL_SESSION_ID) { $env:GRAPHQL_SESSION_ID = "ci-e2e-session" }
if (-not $env:ALLOWED_ORIGINS) { $env:ALLOWED_ORIGINS = "http://127.0.0.1:3000" }
if (-not $env:INTERNAL_API_SECRET) { $env:INTERNAL_API_SECRET = "ci-e2e-internal-secret" }
if (-not $env:GRAPHQL_E2E_CUSTOMER_USER_ID) { $env:GRAPHQL_E2E_CUSTOMER_USER_ID = "2" }
if (-not $env:GRAPHQL_E2E_ADMIN_USER_ID) { $env:GRAPHQL_E2E_ADMIN_USER_ID = "1" }
if (-not $env:ADMIN_ALLOWED_USER_IDS) { $env:ADMIN_ALLOWED_USER_IDS = "1" }

Write-Host "== Environment ==" -ForegroundColor Cyan
Write-Host "TEST_DATABASE_URL=$($env:TEST_DATABASE_URL)" -ForegroundColor Gray
Write-Host "GRAPHQL_URL=$($env:GRAPHQL_URL)" -ForegroundColor Gray
Write-Host "ALLOWED_ORIGINS=$($env:ALLOWED_ORIGINS)" -ForegroundColor Gray

Write-Host "== Readiness checks ==" -ForegroundColor Cyan
$readyUrl = "$($env:GRAPHQL_URL)/ready"
$readyOk = $false
for ($i = 0; $i -lt 90; $i++) {
    try {
        $res = Invoke-WebRequest -UseBasicParsing -Uri $readyUrl -TimeoutSec 3
        if ($res.StatusCode -ge 200 -and $res.StatusCode -lt 300) {
            $readyOk = $true
            break
        }
    } catch {
    }
    Start-Sleep -Seconds 2
}
if (-not $readyOk) { throw "GraphQL readiness check failed." }

Write-Host "== 1/3 Non-ignored workspace tests ==" -ForegroundColor Cyan
cargo test --all-features --workspace --no-fail-fast -- --skip ignored
if ($LASTEXITCODE -ne 0) { throw "Non-ignored workspace tests failed." }

cargo test --doc --all-features --workspace
if ($LASTEXITCODE -ne 0) { throw "Workspace doc tests failed." }

Write-Host "== 2/3 Ignored core_operations DB/integration tests ==" -ForegroundColor Cyan
$integrationTests = @(
    "integration_abandoned_cart_outbox",
    "integration_cart",
    "integration_coupons",
    "integration_order_state",
    "integration_payments",
    "integration_products",
    "integration_refunds",
    "integration_reviews",
    "integration_shipping",
    "integration_users",
    "integration_users_carts_orders_products",
    "integration_webhooks",
    "integration_wishlist"
)
foreach ($test in $integrationTests) {
    cargo test -p core_operations --test $test --all-features --no-fail-fast -- --ignored --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw "Ignored integration test failed: $test" }
}

$ignoredHandlerTests = @(
    "handler_outbox",
    "handler_p2",
    "handler_refunds_resolve",
    "handler_security"
)
foreach ($test in $ignoredHandlerTests) {
    cargo test -p core_operations --test $test --all-features --no-fail-fast -- --ignored --test-threads=1
    if ($LASTEXITCODE -ne 0) { throw "Ignored handler test failed: $test" }
}

Write-Host "== 3/3 Ignored graphql e2e tests ==" -ForegroundColor Cyan
cargo test -p graphql --test e2e_tests --all-features -- --ignored
if ($LASTEXITCODE -ne 0) { throw "Ignored graphql test failed: e2e_tests" }

cargo test -p graphql --test e2e_all_graphql_operations --all-features -- --ignored
if ($LASTEXITCODE -ne 0) { throw "Ignored graphql test failed: e2e_all_graphql_operations" }

cargo test -p graphql --test e2e_business_flows --all-features -- --ignored
if ($LASTEXITCODE -ne 0) { throw "Ignored graphql test failed: e2e_business_flows" }

Write-Host "All backend tests completed." -ForegroundColor Green
