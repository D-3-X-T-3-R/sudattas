# Script to build Rust project with Visual Studio Build Tools environment
# This script sets up the MSVC environment and builds the project

Write-Host "Setting up Visual Studio Build Tools environment..." -ForegroundColor Cyan

# Try to find VS installation using common paths
$vsPaths = @(
    "C:\Program Files\Microsoft Visual Studio\2022\BuildTools",
    "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools",
    "C:\Program Files\Microsoft Visual Studio\2022\Community",
    "C:\Program Files\Microsoft Visual Studio\2022\Professional",
    "C:\Program Files\Microsoft Visual Studio\2022\Enterprise"
)

$vsPath = $null
foreach ($path in $vsPaths) {
    if (Test-Path $path) {
        $vsPath = $path
        Write-Host "Found Visual Studio at: $vsPath" -ForegroundColor Green
        break
    }
}

if (-not $vsPath) {
    Write-Host "ERROR: Visual Studio Build Tools not found!" -ForegroundColor Red
    Write-Host "Please run: winget install Microsoft.VisualStudio.2022.BuildTools" -ForegroundColor Yellow
    exit 1
}

# Import VS Developer environment
$vsDevCmd = Join-Path $vsPath "Common7\Tools\VsDevCmd.bat"
if (Test-Path $vsDevCmd) {
    Write-Host "Loading VS Developer environment..." -ForegroundColor Cyan
    
    # Run VsDevCmd.bat and capture environment variables
    $tempFile = [System.IO.Path]::GetTempFileName()
    cmd /c "`"$vsDevCmd`" > nul && set" | Out-File $tempFile
    
    Get-Content $tempFile | ForEach-Object {
        if ($_ -match "^(.*?)=(.*)$") {
            Set-Item -Path "env:$($matches[1])" -Value $matches[2]
        }
    }
    
    Remove-Item $tempFile
    
    Write-Host "VS environment loaded successfully!" -ForegroundColor Green
    Write-Host ""
    
    # Verify link.exe is now in PATH
    $linkPath = (Get-Command link.exe -ErrorAction SilentlyContinue).Source
    if ($linkPath) {
        Write-Host "MSVC linker found at: $linkPath" -ForegroundColor Green
    } else {
        Write-Host "WARNING: link.exe still not found in PATH" -ForegroundColor Yellow
    }
    
    Write-Host ""
    Write-Host "Building Rust project..." -ForegroundColor Cyan
    Write-Host "----------------------------------------" -ForegroundColor Gray
    
    # Build the project
    cargo build --package core_operations
    
    $exitCode = $LASTEXITCODE
    Write-Host "----------------------------------------" -ForegroundColor Gray
    
    if ($exitCode -eq 0) {
        Write-Host "Build completed successfully!" -ForegroundColor Green
    } else {
        Write-Host "Build failed with exit code: $exitCode" -ForegroundColor Red
    }
    
    exit $exitCode
} else {
    Write-Host "ERROR: VsDevCmd.bat not found at: $vsDevCmd" -ForegroundColor Red
    exit 1
}
