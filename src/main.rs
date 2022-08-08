use lambda_http::{run, service_fn, Body, Error, Request, Response, RequestExt};
use serde::{Deserialize, Serialize};
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::{Client};
use async_trait::async_trait;
use tracing::{error, trace, info};
use std::{env};
use http::Method;

#[derive(Serialize, Deserialize)]
struct TopicRequestBody {
    topic: String,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let ddb_client = aws_sdk_dynamodb::Client::new(&config);

    let table_name = env::var("DDB_TABLE_NAME").expect("DDB_TABLE_NAME environment variable is not defined");

    match *event.method() {
        Method::POST => post_topic(event, ddb_client, &table_name).await,
        Method::GET => get_topic(event, ddb_client, &table_name).await,
        _ => panic!("Method not supported")
    }
}

#[async_trait]
trait DynamoClient {
    async fn put_topic(&self, table: &str, topic: &str) -> Result<(), Error>;
    async fn get_topic(&self, table: &str, topic: &str) -> Result<(), Error>;
}

#[async_trait]
impl DynamoClient for aws_sdk_dynamodb::Client {
    async fn put_topic(&self, table: &str, topic: &str) -> Result<(), Error> {
        put_item(self, table, topic).await
    }
    async fn get_topic(&self, table: &str, topic: &str) -> Result<(), Error> {
        get_item(self, table, topic).await
    }
}

async fn post_topic(event: Request, ddb_client: impl DynamoClient, table_name: &str) -> Result<Response<Body>, Error> {
    let invalid_payload_response = Response::builder()
        .status(400)
        .body("Invalid payload".into())
        .expect("failed to render response");
    if let Body::Text(body) = event.body() {
        match serde_json::from_str::<TopicRequestBody>(&body) {
            Ok(topic_body) => {
                ddb_client.put_topic(table_name, &topic_body.topic).await?;
                let _ = topic_body.topic;
                let resp = Response::builder()
                    .status(200)
                    .header("content-type", "text/json")
                    .body("{\"result\": \"posted result on DDB\"}".into())
                    .map_err(Box::new)?;
                Ok(resp)
            }
            Err(_) => Ok(invalid_payload_response)
        }
    } else {
        Ok(invalid_payload_response)
    }
}

async fn get_topic(event: Request, ddb_client: impl DynamoClient, table_name: &str) -> Result<Response<Body>, Error> {
    let path = event.uri().path();
    let topic = &path[1..path.len()];
    match ddb_client.get_topic(table_name, topic.to_string().as_str()).await {
        Ok(_) => {
            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/json")
                .body("{\"topic\": \"exists\"}".into())
                .map_err(Box::new)?;
            Ok(resp)
        }
        Err(_) => Ok(Response::builder()
        .status(404)
        .header("content-type", "text/json")
        .body("{\"topic\": \"doesn't exist\"}".into())
        .map_err(Box::new)?)
    }
}

async fn put_item(
    client: &Client,
    table: &str,
    topic: &str,
  ) -> Result<(), Error> {
    let topic_av = AttributeValue::S(topic.into());

    let request = client
        .put_item()
        .table_name(table)
        .item("topic", topic_av);

    request.send().await?;

    Ok(())
}

async fn get_item(
    client: &Client,
    table: &str,
    topic: &str,
  ) -> Result<(), Error> {
    let topic_av = AttributeValue::S(topic.into());

    let request = client
        .get_item()
        .table_name(table)
        .key("topic", topic_av);

    let res = request.send().await?;

    if res.item.is_some() {
        Ok(())
    } else {
        Err(Error::from("Item does not exist"))
    }
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
