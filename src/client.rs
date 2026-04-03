use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// A thin GraphQL client that talks to the HeadsDown API.
/// Includes automatic retry with exponential backoff for transient failures.
pub struct GraphQLClient {
    client: Client,
    endpoint: String,
    token: String,
}

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    variables: Option<Value>,
}

#[derive(Deserialize)]
struct GraphQLResponse {
    data: Option<Value>,
    errors: Option<Vec<GraphQLError>>,
}

#[derive(Deserialize)]
struct GraphQLError {
    message: String,
}

const MAX_RETRIES: u32 = 3;
const INITIAL_BACKOFF_MS: u64 = 500;

impl GraphQLClient {
    pub fn new(api_url: &str, token: &str) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            endpoint: format!("{}/graphql", api_url),
            token: token.to_string(),
        }
    }

    /// Execute a GraphQL query/mutation and return the data portion.
    /// Retries transient failures (network errors, 5xx) with exponential backoff.
    pub async fn execute(&self, query: &str, variables: Option<Value>) -> Result<Value> {
        let request = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        let mut last_error = None;

        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = Duration::from_millis(INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1));
                tokio::time::sleep(backoff).await;
            }

            match self.send_request(&request).await {
                Ok(value) => return Ok(value),
                Err(e) => {
                    // Don't retry auth failures or GraphQL errors (they won't resolve)
                    let msg = format!("{}", e);
                    if msg.contains("Authentication failed")
                        || msg.contains("GraphQL error")
                        || msg.contains("API request failed (HTTP 4")
                        || msg.contains("No data returned")
                    {
                        return Err(e);
                    }
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Request failed after retries")))
    }

    async fn send_request(&self, request: &GraphQLRequest) -> Result<Value> {
        let response = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .context("Failed to connect to HeadsDown API")?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            bail!("Authentication failed. Your API key may be invalid or expired. Run `hd auth` to re-authenticate.");
        }

        if status.is_server_error() {
            let body = response.text().await.unwrap_or_default();
            bail!("Server error (HTTP {}): {}", status, body);
        }

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            bail!("API request failed (HTTP {}): {}", status, body);
        }

        let gql_response: GraphQLResponse = response
            .json()
            .await
            .context("Failed to parse API response")?;

        if let Some(errors) = gql_response.errors {
            let messages: Vec<String> = errors.into_iter().map(|e| e.message).collect();
            bail!("GraphQL error: {}", messages.join("; "));
        }

        gql_response
            .data
            .ok_or_else(|| anyhow::anyhow!("No data returned from API"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup() -> (MockServer, GraphQLClient) {
        let server = MockServer::start().await;
        let client = GraphQLClient::new(&server.uri(), "hd_test_token");
        (server, client)
    }

    #[tokio::test]
    async fn successful_request_returns_data() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"profile": {"name": "Alice"}}
            })))
            .mount(&server)
            .await;

        let data = client
            .execute("query { profile { name } }", None)
            .await
            .unwrap();
        assert_eq!(data["profile"]["name"], "Alice");
    }

    #[tokio::test]
    async fn retries_on_server_error_then_succeeds() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .up_to_n_times(1)
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": {"ok": true}
            })))
            .expect(1)
            .mount(&server)
            .await;

        let data = client.execute("query { ok }", None).await.unwrap();
        assert_eq!(data["ok"], true);
    }

    #[tokio::test]
    async fn gives_up_after_max_retries() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(500).set_body_string("down"))
            .expect(4) // 1 initial + 3 retries
            .mount(&server)
            .await;

        let err = client.execute("query { fail }", None).await.unwrap_err();
        assert!(err.to_string().contains("Server error"));
    }

    #[tokio::test]
    async fn does_not_retry_on_401() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(401))
            .expect(1)
            .mount(&server)
            .await;

        let err = client.execute("query { fail }", None).await.unwrap_err();
        assert!(err.to_string().contains("Authentication failed"));
    }

    #[tokio::test]
    async fn does_not_retry_on_graphql_error() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "errors": [{"message": "Not found"}]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let err = client.execute("query { fail }", None).await.unwrap_err();
        assert!(err.to_string().contains("GraphQL error"));
    }

    #[tokio::test]
    async fn returns_error_when_no_data() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": null
            })))
            .expect(1)
            .mount(&server)
            .await;

        let err = client.execute("query { fail }", None).await.unwrap_err();
        assert!(err.to_string().contains("No data returned"));
    }
}
