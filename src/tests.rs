use lambda_http::Error;

use crate::{get_client_id, post_client_id, DynamoClient};
use async_trait::async_trait;

struct MockDynamoDbClient {}

#[async_trait]
impl DynamoClient for MockDynamoDbClient {
    async fn put_client_id(
        &self,
        _table: &str,
        client_id: &str,
        _payload: &str,
    ) -> Result<(), Error> {
        println!("client_id {}", client_id);
        if client_id.eq("client_id-not-existing") {
            Err(Error::from("Boom"))
        } else {
            Ok(())
        }
    }
    async fn get_client_id(&self, _table: &str, client_id: &str) -> Result<String, Error> {
        println!("client_id {}", client_id);
        if client_id.eq("client_id-not-existing") {
            Err(Error::from("Boom"))
        } else {
            Ok(String::new())
        }
    }
}

#[tokio::test]
async fn test_post_nominal() {
    let input = include_str!("test_apigw_proxy_request.json");

    let request = lambda_http::request::from_str(input).expect("failed to create request");

    let mock_ddb_client = MockDynamoDbClient {};
    let response = post_client_id(request, mock_ddb_client, "test_table_name".into())
        .await
        .expect("failed to handle request");

    assert_eq!(response.status(), 200);
    assert_eq!(
        response.into_body(),
        "{\"result\": \"posted result on DDB\"}".into()
    );
}

#[tokio::test]
async fn test_get_nominal() {
    let input = include_str!("test_apigw_proxy_request.json");

    let request = lambda_http::request::from_str(input).expect("failed to create request");

    let mock_ddb_client = MockDynamoDbClient {};
    let response = get_client_id(request, mock_ddb_client, "test_table_name".into())
        .await
        .expect("failed to handle request");

    assert_eq!(response.status(), 200);
    assert_eq!(response.into_body(), "{\"client_id\": \"exists\"}".into());
}

#[tokio::test]
async fn test_get_client_id_does_not_exist() {
    let input = include_str!("test_apigw_proxy_request_invalid_input.json");

    let request = lambda_http::request::from_str(input).expect("failed to create request");

    let mock_ddb_client = MockDynamoDbClient {};
    let response = get_client_id(request, mock_ddb_client, "test_table_name".into())
        .await
        .expect("failed to handle request");

    assert_eq!(response.status(), 404);
}
