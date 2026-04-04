$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
node (Join-Path $Root "scripts/check-frontend-performance-budgets.mjs")
