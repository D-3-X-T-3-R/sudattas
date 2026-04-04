$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
node (Join-Path $Root "scripts/check-contract-discipline.mjs")
