$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
$blockedPatterns = @(
    "NEXT_PUBLIC_ADMIN_API_KEY",
    "NEXT_PUBLIC_GRPC_AUTH_TOKEN",
    "NEXT_PUBLIC_RAZORPAY_KEY_SECRET",
    "NEXT_PUBLIC_GOOGLE_CLIENT_SECRET",
    "NEXT_PUBLIC_AUTH_SECRET"
)
$searchPaths = @(
    (Join-Path $Root "frontend/src"),
    (Join-Path $Root "frontend"),
    (Join-Path $Root "backend")
)

$failures = 0

function Find-PatternHits {
    param(
        [string]$Pattern,
        [string[]]$Paths
    )

    $tracked = git -C $Root ls-files frontend backend 2>$null |
        Where-Object { $_ -match '(\.ts|\.tsx|\.js|\.mjs|\.cjs|/\.env|\.env\.)' }
    if (-not $tracked) {
        return @()
    }

    if (Get-Command rg -ErrorAction SilentlyContinue) {
        return rg -n $Pattern $tracked 2>$null
    }

    $items = $tracked | ForEach-Object { Join-Path $Root $_ } | Where-Object { Test-Path $_ }
    if (-not $items) { return @() }

    $hits = @()
    foreach ($item in $items) {
        $matches = Select-String -Path $item.FullName -Pattern $Pattern -SimpleMatch -ErrorAction SilentlyContinue
        foreach ($m in $matches) {
            $hits += "$($m.Path):$($m.LineNumber):$($m.Line.Trim())"
        }
    }
    return $hits
}

foreach ($pattern in $blockedPatterns) {
    $hits = Find-PatternHits -Pattern $pattern -Paths $searchPaths
    if ($hits -and $hits.Count -gt 0) {
        Write-Host "FAIL blocked public env pattern found: $pattern" -ForegroundColor Red
        $hits | ForEach-Object { Write-Host $_ }
        $failures++
    } else {
        Write-Host "OK   pattern not found: $pattern" -ForegroundColor Green
    }
}

if ($failures -gt 0) {
    Write-Host ""
    Write-Host "Blocked NEXT_PUBLIC privileged env variables detected." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "No blocked public env patterns detected." -ForegroundColor Green
