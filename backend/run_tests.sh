export TEST_DATABASE_URL = "mysql://root:12345678@127.0.0.1:3306/SUDATTAS_CLONED";
export DATABASE_URL = $env:TEST_DATABASE_URL;
export GRPC_URL = "http://127.0.0.1:50051";
export REDIS_URL = "redis://127.0.0.1:6379";
export INTERNAL_API_SECRET = "4737c5a983839a6a958c1069234b8f87887bc8248531fc6e8a499659833bd84f";
export RUN_LIVE_LOGISTICS_TESTS =1;
cargo test --workspace -- --include-ignored --test-threads=1 --nocapture