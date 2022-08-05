use lambda_http::{run, service_fn, Body, Error, Request, RequestExt, Response};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TopicRequestBody {
    topic: String,
}

async fn function_handler(event: Request) -> Result<Response<Body>, Error> {
    let invalid_payload_response = Response::builder()
        .status(400)
        .body("Invalid payload".into())
        .expect("failed to render response");
    if let Body::Text(body) = event.body() {
        match serde_json::from_str::<TopicRequestBody>(&body) {
            Ok(topic_body) => {
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
