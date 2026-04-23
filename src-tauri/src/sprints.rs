use crate::ado_client::AdoClient;
use crate::types::*;
use tokio::sync::RwLock;
use tauri::State;

#[tauri::command]
pub async fn list_iterations(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    project: String,
    team: String,
) -> Result<Vec<Sprint>, String> {
    let client = client.read().await;
    client
        .list_iterations(&organization, &project, &team)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_current_iteration(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    project: String,
    team: String,
) -> Result<Option<Sprint>, String> {
    let client = client.read().await;
    client
        .get_current_iteration(&organization, &project, &team)
        .await
        .map_err(|e| e.to_string())
}
