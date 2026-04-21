use anyhow::{Context, Result};
use std::process::Command;

use crate::ado_client::AdoClient;
use crate::types::UserProfile;
use tokio::sync::Mutex;
use tauri::State;

/// Authentication method for Azure DevOps
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// Azure CLI (`az login`) — uses Bearer token
    AzCli,
    /// Personal Access Token — uses Basic auth with Base64(":pat")
    Pat(String),
}

impl AuthMethod {
    /// Apply authentication to a reqwest RequestBuilder
    pub fn apply_auth(&self, req: reqwest::RequestBuilder) -> Result<reqwest::RequestBuilder> {
        match self {
            AuthMethod::AzCli => {
                let token = get_az_cli_token()?;
                Ok(req.bearer_auth(token))
            }
            AuthMethod::Pat(pat) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(format!(":{}", pat));
                Ok(req.header("Authorization", format!("Basic {}", encoded)))
            }
        }
    }
}

/// Get an Azure DevOps access token by shelling out to `az account get-access-token`.
/// Requires the user to have run `az login` beforehand.
pub fn get_az_cli_token() -> Result<String> {
    let output = Command::new("az")
        .args([
            "account",
            "get-access-token",
            "--resource",
            "499b84ac-1321-427f-aa17-267ca6975798",
            "--query",
            "accessToken",
            "--output",
            "tsv",
        ])
        .output()
        .context("Failed to run 'az' command. Is Azure CLI installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("AADSTS") || stderr.contains("Please run 'az login'") {
            anyhow::bail!("Not logged in. Please run 'az login' in your terminal first.");
        }
        anyhow::bail!("az account get-access-token failed: {}", stderr.trim());
    }

    let token = String::from_utf8(output.stdout)
        .context("Invalid UTF-8 in token")?
        .trim()
        .to_string();

    if token.is_empty() {
        anyhow::bail!("Empty token returned. Please run 'az login' first.");
    }

    Ok(token)
}

/// Check if az CLI is installed and user is logged in
pub fn check_auth() -> Result<bool> {
    let output = Command::new("az")
        .args(["account", "show", "--query", "user.name", "--output", "tsv"])
        .output();

    match output {
        Ok(o) if o.status.success() => Ok(true),
        Ok(_) => Ok(false),
        Err(_) => Ok(false),
    }
}

// --- Tauri Commands ---

#[tauri::command]
pub async fn check_auth_status() -> Result<bool, String> {
    check_auth().map_err(|e| e.to_string())
}

/// Set authentication to use a Personal Access Token
#[tauri::command]
pub async fn set_auth_pat(
    client: State<'_, Mutex<AdoClient>>,
    pat: String,
) -> Result<(), String> {
    let mut client = client.lock().await;
    client.set_auth(AuthMethod::Pat(pat));
    Ok(())
}

/// Set authentication to use Azure CLI
#[tauri::command]
pub async fn set_auth_azcli(
    client: State<'_, Mutex<AdoClient>>,
) -> Result<(), String> {
    let mut client = client.lock().await;
    client.set_auth(AuthMethod::AzCli);
    Ok(())
}

/// Validate a PAT by attempting to connect to ADO
#[tauri::command]
pub async fn validate_pat(
    pat: String,
    organization: String,
) -> Result<bool, String> {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(format!(":{}", pat));
    let http = reqwest::Client::new();

    let resp = http
        .get(format!(
            "https://dev.azure.com/{}/_apis/connectionData",
            organization
        ))
        .header("Authorization", format!("Basic {}", encoded))
        .send()
        .await
        .map_err(|e| format!("Connection failed: {}", e))?;

    Ok(resp.status().is_success())
}

#[tauri::command]
pub async fn get_current_user(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
) -> Result<UserProfile, String> {
    let client = client.lock().await;
    let http = reqwest::Client::new();

    let req = http.get("https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=7.1");
    let req = client.auth_method().apply_auth(req).map_err(|e| e.to_string())?;

    let resp = req
        .send()
        .await
        .map_err(|e| format!("Failed to fetch user profile: {}", e))?;

    if !resp.status().is_success() {
        // Try the org-scoped connection data endpoint instead
        let req2 = http.get(format!(
            "https://dev.azure.com/{}/_apis/connectionData",
            organization
        ));
        let req2 = client.auth_method().apply_auth(req2).map_err(|e| e.to_string())?;

        let resp2 = req2
            .send()
            .await
            .map_err(|e| format!("Failed to fetch connection data: {}", e))?;

        let data: serde_json::Value = resp2
            .json()
            .await
            .map_err(|e| format!("Failed to parse connection data: {}", e))?;

        let display_name = data["authenticatedUser"]["providerDisplayName"]
            .as_str()
            .unwrap_or("Unknown User")
            .to_string();
        let id = data["authenticatedUser"]["id"]
            .as_str()
            .unwrap_or("")
            .to_string();

        return Ok(UserProfile {
            display_name,
            email: None,
            id,
        });
    }

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse profile: {}", e))?;

    Ok(UserProfile {
        display_name: data["displayName"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string(),
        email: data["emailAddress"].as_str().map(|s| s.to_string()),
        id: data["id"].as_str().unwrap_or("").to_string(),
    })
}
