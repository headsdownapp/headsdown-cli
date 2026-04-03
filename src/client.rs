use anyhow::{bail, Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A thin GraphQL client that talks to the HeadsDown API.
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

impl GraphQLClient {
    pub fn new(api_url: &str, token: &str) -> Self {
        Self {
            client: Client::new(),
            endpoint: format!("{}/graphql", api_url),
            token: token.to_string(),
        }
    }

    /// Execute a GraphQL query/mutation and return the data portion.
    pub async fn execute(&self, query: &str, variables: Option<Value>) -> Result<Value> {
        let request = GraphQLRequest {
            query: query.to_string(),
            variables,
        };

        let response = self
            .client
            .post(&self.endpoint)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to connect to HeadsDown API")?;

        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED {
            bail!("Authentication failed. Your API key may be invalid or expired. Run `hd auth` to re-authenticate.");
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
