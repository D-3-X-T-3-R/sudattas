param(
    [string]$BackendStaging = "",
    [string]$BackendProduction = "",
    [string]$FrontendStaging = "",
    [string]$FrontendProduction = ""
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($BackendStaging)) {
    $BackendStaging = Join-Path $Root "backend/.env.staging"
}
if ([string]::IsNullOrWhiteSpace($BackendProduction)) {
    $BackendProduction = Join-Path $Root "backend/.env.production"
}
if ([string]::IsNullOrWhiteSpace($FrontendStaging)) {
    $FrontendStaging = Join-Path $Root "frontend/.env.staging"
}
if ([string]::IsNullOrWhiteSpace($FrontendProduction)) {
    $FrontendProduction = Join-Path $Root "frontend/.env.production"
}

function Read-EnvValue {
    param(
        [string]$FilePath,
        [string]$Key
    )

    if (-not (Test-Path $FilePath)) {
        return "__MISSING_FILE__"
    }
    $line = Get-Content $FilePath | Where-Object { $_ -match "^\s*(export\s+)?$([Regex]::Escape($Key))=" } | Select-Object -Last 1
    if (-not $line) {
        return "__MISSING_KEY__"
    }
    $value = $line -replace '^\s*(export\s+)?[^=]+=', ''
    $value = $value.Trim()
    $value = $value.Trim('"').Trim("'")
    return $value
}

function Check-EqualKey {
    param(
        [string]$StagingFile,
        [string]$ProductionFile,
        [string]$Key
    )
    $sv = Read-EnvValue -FilePath $StagingFile -Key $Key
    $pv = Read-EnvValue -FilePath $ProductionFile -Key $Key
    if ($sv -eq "__MISSING_FILE__" -or $pv -eq "__MISSING_FILE__") {
        Write-Host "FAIL ${Key}: missing env file (staging='$StagingFile' prod='$ProductionFile')" -ForegroundColor Red
        return $false
    }
    if ($sv -eq "__MISSING_KEY__" -or $pv -eq "__MISSING_KEY__") {
        Write-Host "FAIL ${Key}: key missing in one env file" -ForegroundColor Red
        return $false
    }
    if ($sv -ne $pv) {
        Write-Host "FAIL ${Key}: staging != production" -ForegroundColor Red
        return $false
    }
    Write-Host "OK   $Key" -ForegroundColor Green
    return $true
}

function Check-PresenceKey {
    param(
        [string]$StagingFile,
        [string]$ProductionFile,
        [string]$Key
    )
    $sv = Read-EnvValue -FilePath $StagingFile -Key $Key
    $pv = Read-EnvValue -FilePath $ProductionFile -Key $Key
    if ($sv -eq "__MISSING_FILE__" -or $pv -eq "__MISSING_FILE__") {
        Write-Host "FAIL ${Key}: missing env file (staging='$StagingFile' prod='$ProductionFile')" -ForegroundColor Red
        return $false
    }
    if ($sv -eq "__MISSING_KEY__" -or $pv -eq "__MISSING_KEY__") {
        Write-Host "FAIL ${Key}: key missing in one env file" -ForegroundColor Red
        return $false
    }
    if ([string]::IsNullOrWhiteSpace($sv) -or [string]::IsNullOrWhiteSpace($pv)) {
        Write-Host "FAIL ${Key}: key present but empty" -ForegroundColor Red
        return $false
    }
    Write-Host "OK   $Key (present in both)" -ForegroundColor Green
    return $true
}

$failures = 0

Write-Host "Checking frontend parity..." -ForegroundColor Cyan
foreach ($k in @("GRAPHQL_URL", "STOREFRONT_ORIGIN", "ADMIN_ALLOWED_EMAILS")) {
    if (-not (Check-EqualKey -StagingFile $FrontendStaging -ProductionFile $FrontendProduction -Key $k)) {
        $failures++
    }
}

Write-Host "Checking backend GraphQL parity..." -ForegroundColor Cyan
foreach ($k in @("ALLOWED_ORIGINS", "RATE_LIMIT_TRUST_PROXY_HEADERS", "RATE_LIMIT_PER_MINUTE", "RATE_LIMIT_WEBHOOK_PER_MINUTE", "GRAPHQL_MAX_QUERY_DEPTH", "GRAPHQL_MAX_QUERY_COMPLEXITY")) {
    if (-not (Check-EqualKey -StagingFile $BackendStaging -ProductionFile $BackendProduction -Key $k)) {
        $failures++
    }
}

Write-Host "Checking backend core parity..." -ForegroundColor Cyan
foreach ($k in @("GRPC_AUTH_TOKEN")) {
    if (-not (Check-PresenceKey -StagingFile $BackendStaging -ProductionFile $BackendProduction -Key $k)) {
        $failures++
    }
}

Write-Host "Checking required secret presence..." -ForegroundColor Cyan
foreach ($k in @("AUTH_SECRET", "GOOGLE_CLIENT_ID", "GOOGLE_CLIENT_SECRET", "RAZORPAY_KEY_ID", "RAZORPAY_KEY_SECRET", "RAZORPAY_WEBHOOK_SECRET")) {
    if (-not (Check-PresenceKey -StagingFile $BackendStaging -ProductionFile $BackendProduction -Key $k)) {
        $failures++
    }
}

if ($failures -gt 0) {
    Write-Host ""
    Write-Host "Parity check failed with $failures issue(s)." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Parity check passed." -ForegroundColor Green
