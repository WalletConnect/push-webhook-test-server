# push-webhook-test-server

API Gateway fronted Lambda written in Rust. Broadcasts messages via SNS for consumers to pick up.

### Local Development

`cargo test` to run tests and `cargo lambda build --release --arm64 --output-format zip` to build.
