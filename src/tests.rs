use lambda_http::RequestExt;

use crate::{function_handler, function_handler_helper};

#[tokio::test]
async fn test_lambda() {
  let input = include_str!("test_apigw_proxy_request.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let mock_ddb_client = MockClient::new();
  let response = function_handler_helper(request, mock_ddb_client, "test_table_name".into()).await.expect("failed to handle request");

  assert_eq!(response.status(), 200);
  assert_eq!(response.into_body(), "{\"result\": \"posted result on SNS\"}".into());
}

#[tokio::test]
async fn test_invalid_input() {
  let input = include_str!("test_apigw_proxy_request_invalid_input.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let mock_ddb_client = MockClient::new();
  let response = function_handler_helper(request, mock_ddb_client, "test_table_name".into()).await.expect("failed to handle request");

  assert_eq!(response.status(), 400);
}
