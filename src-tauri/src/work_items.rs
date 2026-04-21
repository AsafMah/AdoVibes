use crate::ado_client::AdoClient;
use crate::types::*;
use tokio::sync::Mutex;
use tauri::State;

#[tauri::command]
pub async fn get_sprint_work_items(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    project: String,
    iteration_path: String,
) -> Result<Vec<WorkItem>, String> {
    let client = client.lock().await;

    // Query PBIs, Bugs, and Tasks in the given sprint
    let wiql = format!(
        "SELECT [System.Id] FROM workitems \
         WHERE [System.IterationPath] = '{}' \
         AND [System.WorkItemType] IN ('Product Backlog Item', 'Bug', 'Task') \
         AND [System.State] <> 'Removed' \
         ORDER BY [Microsoft.VSTS.Common.BacklogPriority] ASC, [System.Id] ASC",
        iteration_path.replace('\'', "''")
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
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    project: String,
    id: i32,
) -> Result<WorkItem, String> {
    let client = client.lock().await;
    client
        .get_work_item(&organization, &project, id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_work_item(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    project: String,
    request: CreateWorkItemRequest,
) -> Result<WorkItem, String> {
    let client = client.lock().await;
    client
        .create_work_item(&organization, &project, &request)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_work_item(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    project: String,
    request: UpdateWorkItemRequest,
) -> Result<WorkItem, String> {
    let client = client.lock().await;
    client
        .update_work_item(&organization, &project, &request)
        .await
        .map_err(|e| e.to_string())
}

/// Move a work item to a board column (translates column to ADO state)
#[tauri::command]
pub async fn move_work_item(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    project: String,
    id: i32,
    work_item_type: String,
    target_column: String,
) -> Result<WorkItem, String> {
    let client = client.lock().await;
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

/// Mark a PBI/Bug as done and cascade to all child tasks
#[tauri::command]
pub async fn mark_item_done_cascade(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    project: String,
    id: i32,
    work_item_type: String,
) -> Result<Vec<WorkItem>, String> {
    let client = client.lock().await;
    let mut updated = Vec::new();

    // First, update the parent item
    let parent_req = UpdateWorkItemRequest {
        id,
        state: Some("Done".to_string()),
        assigned_to: None,
        title: None,
        description: None,
        priority: None,
        story_points: None,
        remaining_work: None,
        tags: None,
    };
    let parent = client
        .update_work_item(&organization, &project, &parent_req)
        .await
        .map_err(|e| e.to_string())?;
    updated.push(parent);

    // If it's a PBI or Bug, also mark all child tasks as done
    if work_item_type == "Product Backlog Item" || work_item_type == "Bug" {
        let child_ids = client
            .get_child_ids(&organization, &project, id)
            .await
            .map_err(|e| e.to_string())?;

        for child_id in child_ids {
            let child_req = UpdateWorkItemRequest {
                id: child_id,
                state: Some("Done".to_string()),
                assigned_to: None,
                title: None,
                description: None,
                priority: None,
                story_points: None,
                remaining_work: None,
                tags: None,
            };

            match client
                .update_work_item(&organization, &project, &child_req)
                .await
            {
                Ok(child) => updated.push(child),
                Err(e) => eprintln!("Failed to update child {}: {}", child_id, e),
            }
        }
    }

    Ok(updated)
}

/// Check if all sibling tasks of a task are done, and if so, mark the parent PBI as done
#[tauri::command]
pub async fn check_and_complete_parent(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    project: String,
    parent_id: i32,
) -> Result<Option<WorkItem>, String> {
    let client = client.lock().await;

    let child_ids = client
        .get_child_ids(&organization, &project, parent_id)
        .await
        .map_err(|e| e.to_string())?;

    if child_ids.is_empty() {
        return Ok(None);
    }

    let children = client
        .get_work_items_batch(&organization, &project, &child_ids)
        .await
        .map_err(|e| e.to_string())?;

    let all_done = children.iter().all(|c| c.board_column == "done");

    if all_done {
        let req = UpdateWorkItemRequest {
            id: parent_id,
            state: Some("Done".to_string()),
            assigned_to: None,
            title: None,
            description: None,
            priority: None,
            story_points: None,
            remaining_work: None,
            tags: None,
        };

        let updated = client
            .update_work_item(&organization, &project, &req)
            .await
            .map_err(|e| e.to_string())?;

        Ok(Some(updated))
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn search_projects(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    query: String,
) -> Result<Vec<Project>, String> {
    let mut client = client.lock().await;
    client
        .search_projects(&organization, &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_teams(
    client: State<'_, Mutex<AdoClient>>,
    organization: String,
    project: String,
    query: String,
) -> Result<Vec<Team>, String> {
    let mut client = client.lock().await;
    client
        .search_teams(&organization, &project, &query)
        .await
        .map_err(|e| e.to_string())
}
