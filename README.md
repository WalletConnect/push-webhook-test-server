# push-webhook-test-server

API Gateway fronted Lambda written in Rust. Broadcasts messages via SNS for consumers to pick up.

### Local Development

`cargo test` to run tests and `cargo lambda build --release --arm64 --output-format zip` to build.

Lambda can be deployed via `terraform -chdir=terraform apply  -var-file="vars/dev.tfvars"`.

Then call the API via `curl -H "Content-Type: application/json" -X POST https://<APIG instance>.execute-api.eu-central-1.amazonaws.com/topic -d '{"topic":"test-topic"}'` and then `curl https://<APIG instance>.execute-api.eu-central-1.amazonaws.com/test-topic`.
