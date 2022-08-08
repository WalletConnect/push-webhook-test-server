use lambda_http::RequestExt;
use lambda_http::{Error};

use crate::{post_topic, get_topic, DynamoClient};
use async_trait::async_trait;

struct MockDynamoDbClient {}

#[async_trait]
impl DynamoClient for MockDynamoDbClient {
    async fn put_topic(&self, table: &str, topic: &str) -> Result<(), Error> {
        Ok(())
    }
    async fn get_topic(&self, table: &str, topic: &str) -> Result<(), Error> {
      println!("topic {}", topic);
      if topic.eq("topic-not-existing") {
        Err(Error::from("Boom"))
      } else {
        Ok(())
      }
    }
}

#[tokio::test]
async fn test_post_nominal() {
  let input = include_str!("test_apigw_proxy_request.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let mock_ddb_client = MockDynamoDbClient{};
  let response = post_topic(request, mock_ddb_client, "test_table_name".into()).await.expect("failed to handle request");

  assert_eq!(response.status(), 200);
  assert_eq!(response.into_body(), "{\"result\": \"posted result on DDB\"}".into());
}

#[tokio::test]
async fn test_post_invalid_input() {
  let input = include_str!("test_apigw_proxy_request_invalid_input.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let mock_ddb_client = MockDynamoDbClient{};
  let response = post_topic(request, mock_ddb_client, "test_table_name".into()).await.expect("failed to handle request");

  assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn test_get_nominal() {
  let input = include_str!("test_apigw_proxy_request.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let mock_ddb_client = MockDynamoDbClient{};
  let response = get_topic(request, mock_ddb_client, "test_table_name".into()).await.expect("failed to handle request");

  assert_eq!(response.status(), 200);
  assert_eq!(response.into_body(), "{\"topic\": \"exists\"}".into());
}

#[tokio::test]
async fn test_get_topic_does_not_exist() {
  let input = include_str!("test_apigw_proxy_request_invalid_input.json");

  let request = lambda_http::request::from_str(input)
    .expect("failed to create request");

  let mock_ddb_client = MockDynamoDbClient{};
  let response = get_topic(request, mock_ddb_client, "test_table_name".into()).await.expect("failed to handle request");

  assert_eq!(response.status(), 404);
}
