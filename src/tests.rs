use crate::function_handler;

#[tokio::test]
async fn test_lambda() {
  let input = include_str!("test_apigw_proxy_request.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let response = function_handler(request).await.expect("failed to handle request");

  assert_eq!(response.status(), 200);
  assert_eq!(response.into_body(), "{\"result\": \"posted result on SNS\"}".into());
}

#[tokio::test]
async fn test_invalid_input() {
  let input = include_str!("test_apigw_proxy_request_invalid_input.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let response = function_handler(request).await.expect("failed to handle request");

  assert_eq!(response.status(), 400);
}
