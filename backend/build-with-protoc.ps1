# Build script with protoc environment setup
Write-Host "Setting up build environment..." -ForegroundColor Cyan

# Find protoc
$protocPaths = @(
    "C:\ProgramData\protoc\bin\protoc.exe",
    "C:\Program Files\protoc\bin\protoc.exe",
    "$env:LOCALAPPDATA\Microsoft\WinGet\Links\protoc.exe",
    "$env:LOCALAPPDATA\Microsoft\WinGet\Packages\Google.Protobuf_Microsoft.Winget.Source_*\bin\protoc.exe"
)

$protoc = $null
foreach ($path in $protocPaths) {
    $resolved = Get-Item $path -ErrorAction SilentlyContinue
    if ($resolved) {
        $protoc = $resolved.FullName
        Write-Host "Found protoc at: $protoc" -ForegroundColor Green
        break
    }
}

if (-not $protoc) {
    # Try to find it anywhere
    Write-Host "Searching for protoc..." -ForegroundColor Yellow
    $protoc = (Get-Command protoc.exe -ErrorAction SilentlyContinue).Source
}

if ($protoc) {
    $env:PROTOC = $protoc
    Write-Host "Set PROTOC=$protoc" -ForegroundColor Green
} else {
    Write-Host "WARNING: protoc not found. Build may fail." -ForegroundColor Yellow
    Write-Host "Trying to build anyway..." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Building core_operations..." -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Gray

cargo build --package core_operations

$exitCode = $LASTEXITCODE
Write-Host "========================================" -ForegroundColor Gray

if ($exitCode -eq 0) {
    Write-Host "[SUCCESS] Build completed successfully!" -ForegroundColor Green
} else {
    Write-Host "[FAILED] Build failed with exit code: $exitCode" -ForegroundColor Red
}

exit $exitCode
