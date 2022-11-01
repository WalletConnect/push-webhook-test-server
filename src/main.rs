use lambda_http::{run, service_fn, Body, Error, Request, Response, RequestExt};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use tracing::{error, trace, info};
use std::{env};
use http::Method;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize)]
struct StatsRequestBody {
    stats: String,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let _config = aws_config::load_from_env().await;

    match *event.method() {
        Method::GET => get_stats(event).await,
        _ => panic!("Method not supported")
    }
}

async fn get_stats(event: Request) -> Result<Response<Body>, Error> {
    let path = event.uri().path();
    let _stats = &path[1..path.len()];
    
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/json")
        .body("{\"stats\": \"exists\"}".into())
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
