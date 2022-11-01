use crate::{query, RdsClient};
use async_trait::async_trait;
use lambda_http::Error;

struct MockRdsClient {}

#[async_trait]
impl RdsClient for MockRdsClient {
    async fn query(&self, _query: &str) -> Result<usize, Error> {
        Ok(5)
    }
}

#[tokio::test]
async fn test_get_nominal() {
    let input = include_str!("test_apigw_proxy_request.json");

    let request = lambda_http::request::from_str(input).expect("failed to create request");

    let mock_rds_client = MockRdsClient {};
    let response = query(request, mock_rds_client)
        .await
        .expect("failed to handle request");

    assert_eq!(response.status(), 200);
    assert_eq!(response.into_body(), "{\"stats\": \"5\"}".into());
}
