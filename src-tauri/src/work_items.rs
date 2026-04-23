use crate::ado_client::AdoClient;
use crate::types::*;
use log::info;
use tokio::sync::RwLock;
use tauri::State;

#[tauri::command]
pub async fn get_sprint_work_items(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    project: String,
    team: String,
    iteration_path: String,
) -> Result<Vec<WorkItem>, String> {
    info!("Loading work items for sprint '{}'", iteration_path);
    let client = client.read().await;

    let team_area_paths = client
        .get_team_area_paths(&organization, &project, &team)
        .await
        .map_err(|e| e.to_string())?;

    let area_filter = if team_area_paths.is_empty() {
        String::new()
    } else {
        let clauses = team_area_paths
            .into_iter()
            .map(|(path, include_children)| {
                let escaped = path.replace('\'', "''");
                if include_children {
                    format!("[System.AreaPath] UNDER '{}'", escaped)
                } else {
                    format!("[System.AreaPath] = '{}'", escaped)
                }
            })
            .collect::<Vec<_>>()
            .join(" OR ");

        format!(" AND ({})", clauses)
    };

    // Query PBIs, Bugs, and Tasks in the given sprint
    let wiql = format!(
        "SELECT [System.Id] FROM workitems \
         WHERE [System.IterationPath] = '{}' \
         {} \
         AND [System.WorkItemType] IN ('Product Backlog Item', 'Bug', 'Task') \
         AND [System.State] <> 'Removed' \
         ORDER BY [Microsoft.VSTS.Common.BacklogPriority] ASC, [System.Id] ASC",
        iteration_path.replace('\'', "''"),
        area_filter
    );

    let ids = client
        .query_work_items_wiql(&organization, &project, &wiql)
        .await
        .map_err(|e| e.to_string())?;

    client
        .get_work_items_batch(&organization, &project, &ids)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_work_item(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    project: String,
    id: i32,
) -> Result<WorkItem, String> {
    let client = client.read().await;
    client
        .get_work_item(&organization, &project, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_work_item(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    project: String,
    request: CreateWorkItemRequest,
) -> Result<WorkItem, String> {
    let client = client.read().await;
    client
        .create_work_item(&organization, &project, &request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_work_item(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    project: String,
    request: UpdateWorkItemRequest,
) -> Result<WorkItem, String> {
    let client = client.read().await;
    client
        .update_work_item(&organization, &project, &request)
        .await
        .map_err(|e| e.to_string())
}

/// Move a work item to a board column (translates column to ADO state)
#[tauri::command]
pub async fn move_work_item(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    project: String,
    id: i32,
    work_item_type: String,
    target_column: String,
) -> Result<WorkItem, String> {
    let client = client.read().await;
    let ado_state = column_to_ado_state(&work_item_type, &target_column);

    let req = UpdateWorkItemRequest {
        id,
        state: Some(ado_state),
        assigned_to: None,
        title: None,
        description: None,
        priority: None,
        story_points: None,
        remaining_work: None,
        tags: None,
    };

    client
        .update_work_item(&organization, &project, &req)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_projects(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    query: String,
) -> Result<Vec<Project>, String> {
    let mut client = client.write().await;
    client
        .search_projects(&organization, &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_teams(
    client: State<'_, RwLock<AdoClient>>,
    organization: String,
    project: String,
    query: String,
) -> Result<Vec<Team>, String> {
    let mut client = client.write().await;
    client
        .search_teams(&organization, &project, &query)
        .await
        .map_err(|e| e.to_string())
}
