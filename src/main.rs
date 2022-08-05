use lambda_http::{run, service_fn, Body, Error, Request, Response, RequestExt};
use serde::{Deserialize, Serialize};
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::{Client};

#[derive(Serialize, Deserialize)]
struct TopicRequestBody {
    topic: String,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let ddb_client = aws_sdk_dynamodb::Client::new(&config);

    let table_name = option_env!("DDB_TABLE_NAME").unwrap();

    function_handler_helper(event, ddb_client, table_name).await
}

async fn function_handler_helper(event: Request, ddb_client: Client, table_name: &str) -> Result<Response<Body>, Error> {
    let invalid_payload_response = Response::builder()
        .status(400)
        .body("Invalid payload".into())
        .expect("failed to render response");
    if let Body::Text(body) = event.body() {
        match serde_json::from_str::<TopicRequestBody>(&body) {
            Ok(topic_body) => {
                record_call(&ddb_client, table_name, &topic_body.topic).await?;
                let _ = topic_body.topic;
                let resp = Response::builder()
                    .status(200)
                    .header("content-type", "text/json")
                    .body("{\"result\": \"posted result on SNS\"}".into())
                    .map_err(Box::new)?;
                Ok(resp)
            }
            Err(_) => Ok(invalid_payload_response)
        }
    } else {
        Ok(invalid_payload_response)
    }
}

async fn record_call(
    client: &Client,
    table: &str,
    topic: &str,
  ) -> Result<(), Error> {
    let topic_av = AttributeValue::S(topic.into());
  
    let request = client
        .put_item()
        .table_name(table)
        .item("topic", topic_av);
  
    println!("Executing request [{:?}] to add item...", request);
  
    request.send().await?;
  
    Ok(())
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
