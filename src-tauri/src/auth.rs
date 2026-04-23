use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Output};

use crate::ado_client::AdoClient;
use crate::types::UserProfile;
use tokio::sync::RwLock;
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

    pub fn auth_header_value(&self) -> Result<String> {
        match self {
            AuthMethod::AzCli => Ok(format!("Bearer {}", get_az_cli_token()?)),
            AuthMethod::Pat(pat) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(format!(":{}", pat));
                Ok(format!("Basic {}", encoded))
            }
        }
    }
}

/// Get an Azure DevOps access token by shelling out to `az account get-access-token`.
/// Requires the user to have run `az login` beforehand.
fn resolve_az_cli() -> Result<PathBuf> {
    #[cfg(windows)]
    {
        let mut candidates = Vec::new();

        if let Some(path) = env::var_os("PATH") {
            for dir in env::split_paths(&path) {
                candidates.push(dir.join("az.cmd"));
                candidates.push(dir.join("az.exe"));
                candidates.push(dir.join("az.bat"));
            }
        }

        candidates.push(PathBuf::from(
            r"C:\Program Files\Microsoft SDKs\Azure\CLI2\wbin\az.cmd",
        ));
        candidates.push(PathBuf::from(
            r"C:\Program Files (x86)\Microsoft SDKs\Azure\CLI2\wbin\az.cmd",
        ));

        if let Some(path) = candidates.into_iter().find(|candidate| candidate.is_file()) {
            return Ok(path);
        }

        anyhow::bail!(
            "Failed to find Azure CLI. Ensure `az` is installed and available in PATH."
        );
    }

    #[cfg(not(windows))]
    {
        Ok(PathBuf::from("az"))
    }
}

fn run_az(args: &[&str]) -> Result<Output> {
    let az = resolve_az_cli()?;
    Command::new(&az)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run Azure CLI at '{}'.", az.display()))
}

fn is_not_logged_in(stderr: &str) -> bool {
    stderr.contains("AADSTS")
        || stderr.contains("Please run 'az login'")
        || stderr.contains("az login")
        || stderr.contains("No subscriptions found")
}

fn current_az_cli_user() -> Option<String> {
    let output = run_az(&["account", "show", "--query", "user.name", "--output", "tsv"]).ok()?;
    if !output.status.success() {
        return None;
    }

    let user = String::from_utf8(output.stdout).ok()?.trim().to_string();
    if user.is_empty() {
        None
    } else {
        Some(user)
    }
}

async fn validate_auth_for_organization(
    auth: &AuthMethod,
    organization: &str,
) -> Result<()> {
    let http = reqwest::Client::new();
    let req = http.get(format!(
        "https://dev.azure.com/{}/_apis/connectionData",
        organization
    ));
    let req = auth.apply_auth(req)?;
    let resp = req.send().await?;
    let status = resp.status();

    if status.is_success() {
        return Ok(());
    }

    let body = resp.text().await.unwrap_or_default();
    if status == reqwest::StatusCode::UNAUTHORIZED {
        let identity = current_az_cli_user();
        if body.contains("TF400813") {
            if let Some(identity) = identity {
                anyhow::bail!(
                    "Azure CLI is logged in as '{}', but that account is not authorized for Azure DevOps organization '{}'.",
                    identity,
                    organization
                );
            }

            anyhow::bail!(
                "Azure CLI is logged in, but the current account is not authorized for Azure DevOps organization '{}'.",
                organization
            );
        }
    }

    anyhow::bail!(
        "Failed to validate Azure DevOps access for organization '{}': HTTP {} — {}",
        organization,
        status,
        body.trim()
    );
}

pub fn get_az_cli_token() -> Result<String> {
    let output = run_az(&[
        "account",
        "get-access-token",
        "--resource",
        "499b84ac-1321-427f-aa17-267ca6975798",
        "--query",
        "accessToken",
        "--output",
        "tsv",
    ])?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if is_not_logged_in(&stderr) {
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
    match get_az_cli_token() {
        Ok(_) => Ok(true),
        Err(err) => {
            let message = err.to_string();
            if message.contains("Not logged in") {
                Ok(false)
            } else {
                Err(err)
            }
        }
    }
}

// --- Tauri Commands ---

#[tauri::command]
pub async fn check_auth_status() -> Result<bool, String> {
    check_auth().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn validate_azcli(organization: String) -> Result<(), String> {
    validate_auth_for_organization(&AuthMethod::AzCli, &organization)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_auth_header(
    client: State<'_, RwLock<AdoClient>>,
) -> Result<String, String> {
    let client = client.read().await;
    client
        .auth_method()
        .auth_header_value()
        .map_err(|e| e.to_string())
}

/// Set authentication to use a Personal Access Token
#[tauri::command]
pub async fn set_auth_pat(
    client: State<'_, RwLock<AdoClient>>,
    pat: String,
) -> Result<(), String> {
    let mut client = client.write().await;
    client.set_auth(AuthMethod::Pat(pat));
    Ok(())
}

/// Set authentication to use Azure CLI
#[tauri::command]
pub async fn set_auth_azcli(
    client: State<'_, RwLock<AdoClient>>,
) -> Result<(), String> {
    let mut client = client.write().await;
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
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
) -> Result<UserProfile, String> {
    let client = client.read().await;
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

        if !resp2.status().is_success() {
            return Err(format!(
                "Failed to fetch connection data: HTTP {}",
                resp2.status()
            ));
        }

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
