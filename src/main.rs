use async_trait::async_trait;
use aws_sdk_dynamodb::model::AttributeValue;
use aws_sdk_dynamodb::Client;
use http::Method;
use lambda_http::{run, service_fn, Body, Error, Request, Response};
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let config = aws_config::load_from_env().await;
    let ddb_client = aws_sdk_dynamodb::Client::new(&config);

    let table_name =
        env::var("DDB_TABLE_NAME").expect("DDB_TABLE_NAME environment variable is not defined");

    match *event.method() {
        Method::POST => post_client_id(event, ddb_client, &table_name).await,
        Method::GET => get_client_id(event, ddb_client, &table_name).await,
        _ => panic!("Method not supported"),
    }
}

#[async_trait]
trait DynamoClient {
    async fn put_client_id(&self, table: &str, client_id: &str, payload: &str)
        -> Result<(), Error>;
    async fn get_client_id(&self, table: &str, client_id: &str) -> Result<String, Error>;
}

#[async_trait]
impl DynamoClient for aws_sdk_dynamodb::Client {
    async fn put_client_id(
        &self,
        table: &str,
        client_id: &str,
        payload: &str,
    ) -> Result<(), Error> {
        put_item(self, table, client_id, payload).await
    }
    async fn get_client_id(&self, table: &str, client_id: &str) -> Result<String, Error> {
        get_item(self, table, client_id).await
    }
}

async fn post_client_id(
    event: Request,
    ddb_client: impl DynamoClient,
    table_name: &str,
) -> Result<Response<Body>, Error> {
    let path = event.uri().path();
    let client_id = &path[9..path.len()];
    let body = event.body();
    let payload = std::str::from_utf8(body).unwrap_or("{}");
    info!("Posting record for client_id: {}", client_id);
    ddb_client
        .put_client_id(table_name, &client_id, payload)
        .await?;
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/json")
        .body("{\"result\": \"posted result on DDB\"}".into())
        .map_err(Box::new)?;
    Ok(resp)
}

async fn get_client_id(
    event: Request,
    ddb_client: impl DynamoClient,
    table_name: &str,
) -> Result<Response<Body>, Error> {
    let path = event.uri().path();
    let client_id = &path[1..path.len()];
    info!("Getting record for client_id: {}", client_id);
    match ddb_client
        .get_client_id(table_name, client_id.to_string().as_str())
        .await
    {
        Ok(payload) => {
            let body = format!("{{\"client_id\": \"exists\", \"payload\": {}}}", payload);

            let resp = Response::builder()
                .status(200)
                .header("content-type", "text/json")
                .body(body.into())
                .map_err(Box::new)?;
            Ok(resp)
        }
        Err(_) => Ok(Response::builder()
            .status(404)
            .header("content-type", "text/json")
            .body("{\"client_id\": \"doesn't exist\"}".into())
            .map_err(Box::new)?),
    }
}

async fn put_item(
    client: &Client,
    table: &str,
    client_id: &str,
    payload: &str,
) -> Result<(), Error> {
    let sys_time = SystemTime::now();
    let since_the_epoch = sys_time.duration_since(UNIX_EPOCH).unwrap();
    let expiry_in_10_min = since_the_epoch.as_secs() + 600;
    let client_id_av = AttributeValue::S(client_id.into());
    let payload_av = AttributeValue::S(payload.into());
    let exp_av = AttributeValue::N(expiry_in_10_min.to_string());

    let request = client
        .put_item()
        .table_name(table)
        .item("client_id", client_id_av)
        .item("payload", payload_av)
        .item("expiry", exp_av);

    request.send().await?;

    Ok(())
}

async fn get_item(client: &Client, table: &str, client_id: &str) -> Result<String, Error> {
    let client_id_av = AttributeValue::S(client_id.into());

    let request = client
        .get_item()
        .table_name(table)
        .key("client_id", client_id_av);

    let res = request.send().await?;

    if let Some(item) = res.item {
        if let Some(payload) = item.get("payload") {
            let default = "{}".to_string();
            Ok(payload.as_s().unwrap_or(&default).clone())
        } else {
            Err(Error::from("Missing payload"))
        }
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
