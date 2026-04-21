use serde::{Deserialize, Serialize};

// --- ADO API Response Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoListResponse<T> {
    pub count: Option<i32>,
    pub value: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdoProject {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdoTeam {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdoIteration {
    pub id: String,
    pub name: String,
    pub path: String,
    pub attributes: Option<IterationAttributes>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IterationAttributes {
    pub start_date: Option<String>,
    pub finish_date: Option<String>,
    pub time_frame: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdoWorkItemRef {
    pub id: i32,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoWorkItem {
    pub id: i32,
    pub fields: serde_json::Value,
    pub relations: Option<Vec<AdoRelation>>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoRelation {
    pub rel: String,
    pub url: String,
    pub attributes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WiqlResponse {
    pub work_items: Option<Vec<AdoWorkItemRef>>,
    pub work_item_relations: Option<Vec<WiqlRelation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiqlRelation {
    pub source: Option<AdoWorkItemRef>,
    pub target: Option<AdoWorkItemRef>,
    pub rel: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdoWorkItemBatchRequest {
    pub ids: Vec<i32>,
    #[serde(rename = "$expand")]
    pub expand: Option<String>,
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdoPatchOperation {
    pub op: String,
    pub path: String,
    pub value: serde_json::Value,
}

// --- Frontend-facing types (clean, serialized to frontend) ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Sprint {
    pub id: String,
    pub name: String,
    pub path: String,
    pub start_date: Option<String>,
    pub finish_date: Option<String>,
    pub time_frame: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkItem {
    pub id: i32,
    pub title: String,
    pub state: String,
    pub work_item_type: String,
    pub assigned_to: Option<String>,
    pub iteration_path: Option<String>,
    pub area_path: Option<String>,
    pub priority: Option<i32>,
    pub story_points: Option<f64>,
    pub remaining_work: Option<f64>,
    pub description: Option<String>,
    pub tags: Option<String>,
    pub parent_id: Option<i32>,
    pub board_column: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkItemRequest {
    pub work_item_type: String,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<String>,
    pub iteration_path: Option<String>,
    pub area_path: Option<String>,
    pub priority: Option<i32>,
    pub story_points: Option<f64>,
    pub parent_id: Option<i32>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkItemRequest {
    pub id: i32,
    pub state: Option<String>,
    pub assigned_to: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub priority: Option<i32>,
    pub story_points: Option<f64>,
    pub remaining_work: Option<f64>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub display_name: String,
    pub email: Option<String>,
    pub id: String,
}

// Helper to extract a string field from ADO work item fields JSON
pub fn get_field_str(fields: &serde_json::Value, key: &str) -> Option<String> {
    fields.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

pub fn get_field_f64(fields: &serde_json::Value, key: &str) -> Option<f64> {
    fields.get(key).and_then(|v| v.as_f64())
}

pub fn get_field_i32(fields: &serde_json::Value, key: &str) -> Option<i32> {
    fields.get(key).and_then(|v| v.as_i64()).map(|v| v as i32)
}

/// Map ADO work item state to simplified board column: "new", "active", or "done"
pub fn map_state_to_column(work_item_type: &str, state: &str) -> String {
    let state_lower = state.to_lowercase();
    match work_item_type {
        "Task" => match state_lower.as_str() {
            "to do" | "new" => "new".to_string(),
            "done" | "closed" | "removed" => "done".to_string(),
            _ => "active".to_string(), // "In Progress" and any other state
        },
        "Product Backlog Item" => match state_lower.as_str() {
            "new" => "new".to_string(),
            "done" | "removed" => "done".to_string(),
            _ => "active".to_string(), // "Approved", "Committed"
        },
        "Bug" => match state_lower.as_str() {
            "new" => "new".to_string(),
            "done" | "closed" | "removed" => "done".to_string(),
            _ => "active".to_string(), // "Approved", "Committed"
        },
        "Epic" | "Feature" => match state_lower.as_str() {
            "new" => "new".to_string(),
            "done" | "closed" | "removed" => "done".to_string(),
            _ => "active".to_string(),
        },
        _ => match state_lower.as_str() {
            "new" | "to do" => "new".to_string(),
            "done" | "closed" | "removed" => "done".to_string(),
            _ => "active".to_string(),
        },
    }
}

/// Map board column + work item type to the actual ADO state to set
pub fn column_to_ado_state(work_item_type: &str, column: &str) -> String {
    match column {
        "new" => match work_item_type {
            "Task" => "To Do".to_string(),
            _ => "New".to_string(),
        },
        "active" => match work_item_type {
            "Task" => "In Progress".to_string(),
            "Product Backlog Item" | "Bug" => "Committed".to_string(),
            _ => "Active".to_string(),
        },
        "done" => "Done".to_string(),
        _ => "New".to_string(),
    }
}

/// Convert raw ADO work item to our clean WorkItem type
pub fn parse_work_item(raw: &AdoWorkItem) -> WorkItem {
    let fields = &raw.fields;
    let title = get_field_str(fields, "System.Title").unwrap_or_default();
    let state = get_field_str(fields, "System.State").unwrap_or_default();
    let work_item_type = get_field_str(fields, "System.WorkItemType").unwrap_or_default();

    let assigned_to = fields
        .get("System.AssignedTo")
        .and_then(|v| v.get("displayName"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let parent_id = raw.relations.as_ref().and_then(|rels| {
        rels.iter()
            .find(|r| r.rel == "System.LinkTypes.Hierarchy-Reverse")
            .and_then(|r| {
                r.url
                    .rsplit('/')
                    .next()
                    .and_then(|id| id.parse::<i32>().ok())
            })
    });

    let board_column = map_state_to_column(&work_item_type, &state);

    WorkItem {
        id: raw.id,
        title,
        state,
        work_item_type,
        assigned_to,
        iteration_path: get_field_str(fields, "System.IterationPath"),
        area_path: get_field_str(fields, "System.AreaPath"),
        priority: get_field_i32(fields, "Microsoft.VSTS.Common.Priority"),
        story_points: get_field_f64(fields, "Microsoft.VSTS.Scheduling.StoryPoints"),
        remaining_work: get_field_f64(fields, "Microsoft.VSTS.Scheduling.RemainingWork"),
        description: get_field_str(fields, "System.Description"),
        tags: get_field_str(fields, "System.Tags"),
        parent_id,
        board_column,
    }
}
