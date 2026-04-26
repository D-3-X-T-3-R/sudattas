Push-Location $PSScriptRoot
$cargoExit = 0
try {
    $env:TEST_DATABASE_URL = "mysql://root:12345678@127.0.0.1:3306/SUDATTAS_CLONED";
    $env:DATABASE_URL = $env:TEST_DATABASE_URL;
    $env:GRPC_URL = "http://127.0.0.1:50051";
    $env:REDIS_URL = "redis://127.0.0.1:6379";
    $env:INTERNAL_API_SECRET = "4737c5a983839a6a958c1069234b8f87887bc8248531fc6e8a499659833bd84f";
    $env:RUN_LIVE_LOGISTICS_TESTS = 1;
    cargo test --workspace -- --include-ignored --test-threads=1 --nocapture
    $cargoExit = $LASTEXITCODE
}
finally {
    Pop-Location
}
exit $cargoExit
