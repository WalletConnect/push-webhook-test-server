use lambda_http::RequestExt;
use lambda_http::{Error};

use crate::{get_topic};
use async_trait::async_trait;

#[tokio::test]
async fn test_get_nominal() {
  let input = include_str!("test_apigw_proxy_request.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let response = get_topic(request).await.expect("failed to handle request");

  assert_eq!(response.status(), 200);
  assert_eq!(response.into_body(), "{\"topic\": \"exists\"}".into());
}
