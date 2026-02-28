# Run E2E tests locally. Requires:
# - MySQL with SUDATTAS schema (e.g. docker-compose up -d from backend/)
# - Redis (optional; set REDIS_URL or leave unset for session-disabled)
# Run from backend/: .\run_e2e_local.ps1

$ErrorActionPreference = "Stop"
$backend = $PSScriptRoot
Set-Location $backend

# Defaults matching docker-compose and CI
$env:DATABASE_URL = if ($env:DATABASE_URL) { $env:DATABASE_URL } else { "mysql://root:12345678@127.0.0.1:3306/SUDATTAS" }
$env:GRPC_SERVER = "0.0.0.0:50051"
$env:GRPC_URL = "http://127.0.0.1:50051"
$env:GRAPHQL_URL = "http://127.0.0.1:8080"
$env:GRAPHQL_SESSION_ID = "ci-e2e-session"
if (-not $env:REDIS_URL) { $env:REDIS_URL = "redis://127.0.0.1:6379" }
if (-not $env:OAUTH_DOMAIN) { $env:OAUTH_DOMAIN = "https://accounts.google.com/" }
if (-not $env:OAUTH_AUDIENCE) { $env:OAUTH_AUDIENCE = "https://www.googleapis.com/oauth2/v3/tokeninfo" }
# High limit so e2e_all_graphql_operations (many requests) don't hit 429
if (-not $env:RATE_LIMIT_PER_MINUTE) { $env:RATE_LIMIT_PER_MINUTE = "1000" }

# Seed Redis session for E2E auth (if redis-cli available)
$redisCli = Get-Command redis-cli -ErrorAction SilentlyContinue
if ($redisCli) {
    redis-cli -h 127.0.0.1 SET session:ci-e2e-session 1 2>$null
}

# Build binaries and test executables (before starting servers, so exe is not locked)
Write-Host "Building core_operations, graphql, and E2E test binaries..."
cargo build -p core_operations -p graphql --all-features --tests
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

# Start gRPC server in background
Write-Host "Starting core_operations (gRPC) on 50051..."
$grpc = Start-Process -FilePath "$backend\target\debug\core_operations.exe" -WorkingDirectory $backend -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 5

# Start GraphQL server in background
Write-Host "Starting graphql on 8080..."
$gql = Start-Process -FilePath "$backend\target\debug\graphql.exe" -WorkingDirectory $backend -PassThru -WindowStyle Hidden

# Wait for GraphQL
$max = 30
for ($i = 0; $i -lt $max; $i++) {
    try {
        $r = Invoke-WebRequest -Uri "http://127.0.0.1:8080/" -UseBasicParsing -TimeoutSec 2
        Write-Host "GraphQL server ready."
        break
    } catch {
        if ($i -eq $max - 1) {
            Write-Host "GraphQL server did not become ready. Stopping processes."
            Stop-Process -Id $grpc.Id -Force -ErrorAction SilentlyContinue
            Stop-Process -Id $gql.Id -Force -ErrorAction SilentlyContinue
            exit 1
        }
        Start-Sleep -Seconds 2
    }
}

try {
    Write-Host "Running E2E tests (using pre-built test binaries to avoid locking graphql.exe)..."
    $e2eTestsExe = Get-ChildItem "$backend\target\debug\deps\e2e_tests*.exe" | Where-Object { $_.Name -notmatch '\.d$' } | Select-Object -First 1
    $e2eAllExe = Get-ChildItem "$backend\target\debug\deps\e2e_all_graphql_operations*.exe" | Where-Object { $_.Name -notmatch '\.d$' } | Select-Object -First 1
    if (-not $e2eTestsExe -or -not $e2eAllExe) {
        Write-Host "Test executables not found. Run: cargo build -p graphql --all-features --tests"
        exit 1
    }
    & $e2eTestsExe.FullName --ignored
    $e2e1 = $LASTEXITCODE
    & $e2eAllExe.FullName --ignored
    $e2e2 = $LASTEXITCODE
    if ($e2e1 -ne 0 -or $e2e2 -ne 0) { exit 1 }
} finally {
    Stop-Process -Id $grpc.Id -Force -ErrorAction SilentlyContinue
    Stop-Process -Id $gql.Id -Force -ErrorAction SilentlyContinue
    Write-Host "Stopped servers."
}
Write-Host "E2E tests passed."
