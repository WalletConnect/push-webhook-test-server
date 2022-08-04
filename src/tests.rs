use crate::function_handler;

#[tokio::test]
async fn test_lambda() {
  let input = include_str!("test_apigw_proxy_request.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let response = function_handler(request).await.expect("failed to handle request");

  assert_eq!(response.status(), 200);
  assert_eq!(response.into_body(), "{\"hello\": \"world\"}".into());
}
