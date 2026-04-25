use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::client::GraphQLClient;
use crate::format;

const ACTIVE_GRANTS_QUERY: &str = r#"
query ActiveDelegationGrants {
    activeDelegationGrants {
        id
        scope
        sessionId
        workspaceRef
        agentId
        permissions
        source
        expiresAt
        revokedAt
        expiredAt
        insertedAt
    }
}
"#;

const GRANTS_QUERY: &str = r#"
query DelegationGrants($filter: DelegationGrantFilterInput) {
    delegationGrants(filter: $filter) {
        id
        scope
        sessionId
        workspaceRef
        agentId
        permissions
        source
        expiresAt
        revokedAt
        expiredAt
        insertedAt
    }
}
"#;

const CREATE_GRANT_MUTATION: &str = r#"
mutation CreateDelegationGrant($input: DelegationGrantInput!) {
    createDelegationGrant(input: $input) {
        id
        scope
        sessionId
        workspaceRef
        agentId
        permissions
        source
        expiresAt
        revokedAt
        expiredAt
        insertedAt
    }
}
"#;

const REVOKE_GRANT_MUTATION: &str = r#"
mutation RevokeDelegationGrant($id: ID!) {
    revokeDelegationGrant(id: $id) {
        id
        scope
        expiresAt
        revokedAt
    }
}
"#;

const REVOKE_MANY_MUTATION: &str = r#"
mutation RevokeDelegationGrants($filter: DelegationGrantFilterInput) {
    revokeDelegationGrants(filter: $filter) {
        revokedCount
    }
}
"#;

#[derive(Clone, Debug, Default)]
pub struct GrantsFilterArgs {
    pub active: Option<bool>,
    pub scope: Option<String>,
    pub session_id: Option<String>,
    pub workspace_ref: Option<String>,
    pub agent_id: Option<String>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct CreateGrantArgs {
    pub scope: Option<String>,
    pub session_id: Option<String>,
    pub workspace_ref: Option<String>,
    pub agent_id: Option<String>,
    pub permissions: Vec<String>,
    pub duration_minutes: Option<i32>,
    pub expires_at: Option<String>,
    pub source: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct DelegationGrant {
    id: String,
    scope: String,
    #[serde(rename = "expiresAt")]
    expires_at: Option<String>,
    permissions: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ActiveGrantsResponse {
    #[serde(rename = "activeDelegationGrants")]
    active_delegation_grants: Vec<DelegationGrant>,
}

#[derive(Deserialize)]
struct GrantsResponse {
    #[serde(rename = "delegationGrants")]
    delegation_grants: Vec<DelegationGrant>,
}

#[derive(Deserialize)]
struct CreateGrantResponse {
    #[serde(rename = "createDelegationGrant")]
    create_delegation_grant: DelegationGrant,
}

#[derive(Deserialize, Serialize)]
struct RevokedGrant {
    id: String,
    scope: String,
}

#[derive(Deserialize)]
struct RevokeGrantResponse {
    #[serde(rename = "revokeDelegationGrant")]
    revoke_delegation_grant: RevokedGrant,
}

#[derive(Deserialize, Serialize)]
struct RevokeManyResult {
    #[serde(rename = "revokedCount")]
    revoked_count: i64,
}

#[derive(Deserialize)]
struct RevokeManyResponse {
    #[serde(rename = "revokeDelegationGrants")]
    revoke_delegation_grants: RevokeManyResult,
}

pub async fn list_active(api_url: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data: ActiveGrantsResponse = client.execute_typed(ACTIVE_GRANTS_QUERY, None).await?;
    output_grants(&data.active_delegation_grants, json)
}

pub async fn list(api_url: &str, filter: GrantsFilterArgs, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({
        "filter": {
            "active": filter.active,
            "scope": filter.scope.map(|v| v.to_uppercase()),
            "sessionId": filter.session_id,
            "workspaceRef": filter.workspace_ref,
            "agentId": filter.agent_id,
            "source": filter.source,
        }
    });

    let data: GrantsResponse = client.execute_typed(GRANTS_QUERY, Some(variables)).await?;
    output_grants(&data.delegation_grants, json)
}

pub async fn create(api_url: &str, args: CreateGrantArgs, json: bool) -> Result<()> {
    let scope = args
        .scope
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("--scope is required for grants create"))?
        .to_uppercase();

    if args.permissions.is_empty() {
        bail!("--permissions is required for grants create");
    }

    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({
        "input": {
            "scope": scope,
            "sessionId": args.session_id,
            "workspaceRef": args.workspace_ref,
            "agentId": args.agent_id,
            "permissions": args.permissions.into_iter().map(|v| v.to_uppercase()).collect::<Vec<String>>(),
            "durationMinutes": args.duration_minutes,
            "expiresAt": args.expires_at,
            "source": args.source.unwrap_or_else(|| "hd".to_string()),
        }
    });

    let data: CreateGrantResponse = client
        .execute_typed(CREATE_GRANT_MUTATION, Some(variables))
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data.create_delegation_grant)?
        );
        return Ok(());
    }

    println!();
    println!("  {} Grant created", format::styled_green_bold("✓"));
    println!(
        "  {} {}",
        format::styled_dimmed("ID:"),
        data.create_delegation_grant.id
    );
    println!(
        "  {} {}",
        format::styled_dimmed("Scope:"),
        data.create_delegation_grant.scope
    );
    println!();
    Ok(())
}

pub async fn revoke(api_url: &str, id: &str, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);
    let data: RevokeGrantResponse = client
        .execute_typed(REVOKE_GRANT_MUTATION, Some(serde_json::json!({ "id": id })))
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data.revoke_delegation_grant)?
        );
        return Ok(());
    }

    println!();
    println!("  {} Grant revoked", format::styled_green_bold("✓"));
    println!("  {} {}", format::styled_dimmed("ID:"), id);
    println!();
    Ok(())
}

pub async fn revoke_many(api_url: &str, filter: GrantsFilterArgs, json: bool) -> Result<()> {
    let token = auth::require_token()?;
    let client = GraphQLClient::new(api_url, &token);

    let variables = serde_json::json!({
        "filter": {
            "active": filter.active,
            "scope": filter.scope.map(|v| v.to_uppercase()),
            "sessionId": filter.session_id,
            "workspaceRef": filter.workspace_ref,
            "agentId": filter.agent_id,
            "source": filter.source,
        }
    });

    let data: RevokeManyResponse = client
        .execute_typed(REVOKE_MANY_MUTATION, Some(variables))
        .await?;

    if json {
        println!(
            "{}",
            serde_json::to_string_pretty(&data.revoke_delegation_grants)?
        );
        return Ok(());
    }

    println!();
    println!(
        "  {} Revoked {} grants",
        format::styled_green_bold("✓"),
        data.revoke_delegation_grants.revoked_count
    );
    println!();
    Ok(())
}

fn output_grants(grants: &[DelegationGrant], json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(grants)?);
        return Ok(());
    }

    println!();
    println!("  {}", format::styled_bold("Delegation Grants"));
    println!();

    if grants.is_empty() {
        println!("  {}", format::styled_dimmed("No grants found."));
        println!();
        return Ok(());
    }

    for grant in grants {
        println!(
            "  {} {}",
            format::styled_dimmed("•"),
            format::styled_bold(&grant.id)
        );
        println!("    {} {}", format::styled_dimmed("Scope:"), grant.scope);
        println!(
            "    {} {}",
            format::styled_dimmed("Expires:"),
            grant.expires_at.clone().unwrap_or_else(|| "-".to_string())
        );

        if let Some(perms) = &grant.permissions {
            let joined = perms.join(", ");
            if !joined.is_empty() {
                println!("    {} {}", format::styled_dimmed("Permissions:"), joined);
            }
        }

        println!();
    }

    Ok(())
}
