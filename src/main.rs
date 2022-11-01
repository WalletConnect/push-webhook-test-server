use async_trait::async_trait;
use aws_sdk_rdsdata::Client;
use http::Method;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::info;

#[derive(Serialize, Deserialize)]
struct StatsRequestBody {
    stats: String,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);

    match *event.method() {
        Method::GET => query(event, &client).await,
        _ => panic!("Method not supported"),
    }
}

#[async_trait]
trait RdsClient {
    async fn query(&self, query: &str) -> Result<usize, Error>;
}

#[async_trait]
impl RdsClient for &aws_sdk_rdsdata::Client {
    async fn query(&self, query: &str) -> Result<usize, Error> {
        let cluster_arn = env::var("RDS_CLUSTER_ARN")
            .expect("RDS_CLUSTER_ARN environment variable is not defined");
        let secret_arn =
            env::var("RDS_SECRET_ARN").expect("RDS_SECRET_ARN environment variable is not defined");

        let st = self
            .execute_statement()
            .resource_arn(cluster_arn)
            .database("postgres") // Do not confuse this with db instance name
            .sql(query)
            .secret_arn(secret_arn);

        let res = st.send().await.unwrap();

        Ok(res.records().unwrap().len())
    }
}

async fn query(event: Request, client: impl RdsClient) -> Result<Response<Body>, Error> {
    let path = event.uri().path();
    let project_id = &path[1..path.len()];

    let query = format!(
        "SELECT * FROM project_data WHERE projectid = '{}' LIMIT 5;",
        project_id
    );

    let result_count = client.query(&query).await.unwrap();
    let response_body = format!("{{\"stats\": \"{}\"}}", result_count);

    info!("Result: {:?}", result_count);

    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/json")
        .body(response_body.into())
        .map_err(Box::new)?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    run(service_fn(function_handler)).await
}

#[cfg(test)]
mod tests;
