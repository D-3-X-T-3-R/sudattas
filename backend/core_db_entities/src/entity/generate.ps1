# Generate SeaORM entities
# PowerShell equivalent of generate.sh

$ErrorActionPreference = "Stop"

# Remove old entities
Remove-Item *.rs -ErrorAction SilentlyContinue

# Generate entities from local Docker MySQL
sea-orm-cli generate entity `
  -u "mysql://root:12345678@localhost:3306/SUDATTAS" `
  --with-serde both `
  --date-time-crate chrono `
  --max-connections 1

Write-Host ""
Write-Host "Entities regenerated successfully!" -ForegroundColor Green
Write-Host ""

Write-Host "New tables:" -ForegroundColor Cyan
Get-ChildItem *.rs | Where-Object { $_.Name -match "session|payment|shipment|coupon|order_event|webhook" } | ForEach-Object { $_.Name }

Write-Host ""
Write-Host "Total entity files:" -ForegroundColor Cyan
(Get-ChildItem *.rs).Count
