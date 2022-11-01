# Data Lake API

API Gateway fronted Lambda written in Rust. Grabs data from the data lake.

### Local Development

`cargo test` to run tests and `cargo lambda build --release --arm64 --output-format zip` to build.

Lambda can be deployed via `terraform -chdir=terraform apply  -var-file="vars/dev.tfvars"`.

Then call the API via `curl https://(dev)?.data.walletconnect.com/testProjectId`.
