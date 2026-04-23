use crate::auth::AuthMethod;
use crate::types::*;
use anyhow::Result;
use log::{info, warn};
use reqwest::{Client, Response};
use std::collections::HashMap;

const SEARCH_PREFETCH_PAGE_SIZE: usize = 25;

pub struct AdoClient {
    client: Client,
    auth: AuthMethod,
    projects_cache: HashMap<String, Vec<Project>>,
    teams_cache: HashMap<(String, String), Vec<Team>>,
}

/// Check an HTTP response and return a detailed error if it failed.
/// Captures status code, headers (e.g. Retry-After), and the response body.
async fn check_response(resp: Response, context: &str) -> Result<Response> {
    let status = resp.status();
    if status.is_success() {
        return Ok(resp);
    }
    let retry_after = resp
        .headers()
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .map(|v| format!(" (Retry-After: {}s)", v));
    let url = resp.url().to_string();
    let body = resp.text().await.unwrap_or_default();
    let truncated = if body.len() > 500 { &body[..500] } else { &body };
    let msg = if status == reqwest::StatusCode::UNAUTHORIZED && body.contains("TF400813") {
        format!(
            "{context} — not authorized to access this Azure DevOps organization with the current account. Verify the organization name and that your Azure CLI or PAT identity has access.\nHTTP {status}{retry} — {url}\n{body}",
            context = context,
            status = status,
            retry = retry_after.unwrap_or_default(),
            url = url,
            body = truncated,
        )
    } else {
        format!(
            "{context} — HTTP {status}{retry} — {url}\n{body}",
            context = context,
            status = status,
            retry = retry_after.unwrap_or_default(),
            url = url,
            body = truncated,
        )
    };
    warn!("{}", msg);
    anyhow::bail!(msg)
}

impl AdoClient {
    fn normalize_iteration_path(project: &str, path: &str) -> String {
        if path == project || path.starts_with(&format!("{}\\", project)) {
            return path.to_string();
        }

        format!("{}\\{}", project, path)
    }

    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: AuthMethod::AzCli,
            projects_cache: HashMap::new(),
            teams_cache: HashMap::new(),
        }
    }

    pub fn set_auth(&mut self, auth: AuthMethod) {
        info!("Auth method changed to {:?}", auth);
        self.auth = auth;
        self.projects_cache.clear();
        self.teams_cache.clear();
    }

    pub fn auth_method(&self) -> &AuthMethod {
        &self.auth
    }

    fn authed_get(&self, url: &str) -> Result<reqwest::RequestBuilder> {
        let req = self.client.get(url);
        self.auth.apply_auth(req)
    }

    fn authed_post(&self, url: &str) -> Result<reqwest::RequestBuilder> {
        let req = self.client.post(url);
        self.auth.apply_auth(req)
    }

    fn authed_patch(&self, url: &str) -> Result<reqwest::RequestBuilder> {
        let req = self.client.patch(url);
        self.auth.apply_auth(req)
    }

    fn base_url(&self, org: &str) -> String {
        format!("https://dev.azure.com/{}", org)
    }

    // --- Projects ---

    async fn fetch_all_projects(&self, org: &str) -> Result<Vec<Project>> {
        let url = format!("{}/_apis/projects?api-version=7.1&$top=1000", self.base_url(org));

        let raw = self.authed_get(&url)?.send().await?;
        let resp: AdoListResponse<AdoProject> = check_response(raw, "Failed to list projects")
            .await?
            .json()
            .await?;

        let mut projects: Vec<Project> = resp
            .value
            .into_iter()
            .map(|p| Project {
                id: p.id,
                name: p.name,
                description: p.description,
            })
            .collect();
        projects.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(projects)
    }

    pub async fn search_projects(&mut self, org: &str, query: &str) -> Result<Vec<Project>> {
        let key = org.to_string();
        if !self.projects_cache.contains_key(&key) {
            info!("Fetching all projects for org '{}' (cache miss)", org);
            let all = self.fetch_all_projects(org).await?;
            info!("Cached {} projects for org '{}'", all.len(), org);
            self.projects_cache.insert(key.clone(), all);
        }

        let all = self.projects_cache.get(&key).unwrap();
        let q = query.to_lowercase();

        if q.is_empty() {
            return Ok(all.iter().take(SEARCH_PREFETCH_PAGE_SIZE).cloned().collect());
        }

        Ok(all
            .iter()
            .filter(|p| p.name.to_lowercase().contains(&q))
            .cloned()
            .collect())
    }

    // --- Teams ---

    async fn fetch_all_teams(&self, org: &str, project: &str) -> Result<Vec<Team>> {
        let url = format!(
            "{}/_apis/projects/{}/teams?api-version=7.1-preview.3&$mine=true",
            self.base_url(org),
            project,
        );

        let raw = self.authed_get(&url)?.send().await?;
        let resp: AdoListResponse<AdoTeam> = check_response(raw, "Failed to list teams")
            .await?
            .json()
            .await?;

        let mut teams: Vec<Team> = resp
            .value
            .into_iter()
            .map(|t| Team {
                id: t.id,
                name: t.name,
                description: t.description,
            })
            .collect();
        teams.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(teams)
    }

    pub async fn search_teams(&mut self, org: &str, project: &str, query: &str) -> Result<Vec<Team>> {
        let key = (org.to_string(), project.to_string());
        if !self.teams_cache.contains_key(&key) {
            info!("Fetching teams for '{}/{}' (cache miss)", org, project);
            let all = self.fetch_all_teams(org, project).await?;
            info!("Cached {} teams for '{}/{}'", all.len(), org, project);
            self.teams_cache.insert(key.clone(), all);
        }

        let all = self.teams_cache.get(&key).unwrap();
        let q = query.to_lowercase();

        if q.is_empty() {
            return Ok(all.iter().take(SEARCH_PREFETCH_PAGE_SIZE).cloned().collect());
        }

        Ok(all
            .iter()
            .filter(|t| t.name.to_lowercase().contains(&q))
            .cloned()
            .collect())
    }

    pub async fn get_team_area_paths(
        &self,
        org: &str,
        project: &str,
        team: &str,
    ) -> Result<Vec<(String, bool)>> {
        let url = format!(
            "{}/{}/{}/_apis/work/teamsettings/teamfieldvalues?api-version=7.1",
            self.base_url(org),
            project,
            team
        );

        let raw = self.authed_get(&url)?.send().await?;
        let resp: AdoTeamFieldValuesResponse = check_response(raw, "Failed to load team area paths")
            .await?
            .json()
            .await?;

        Ok(resp
            .values
            .into_iter()
            .map(|value| (value.value, value.include_children))
            .collect())
    }

    // --- Iterations / Sprints ---

    pub async fn list_iterations(
        &self,
        org: &str,
        project: &str,
        team: &str,
    ) -> Result<Vec<Sprint>> {
        let url = format!(
            "{}/{}/{}/_apis/work/teamsettings/iterations?api-version=7.1",
            self.base_url(org),
            project,
            team
        );

        let raw = self.authed_get(&url)?.send().await?;
        let resp: AdoListResponse<AdoIteration> = check_response(raw, "Failed to list iterations")
            .await?
            .json()
            .await?;

        Ok(resp
            .value
            .into_iter()
            .map(|i| Sprint {
                id: i.id,
                name: i.name.clone(),
                path: Self::normalize_iteration_path(project, &i.path),
                start_date: i.attributes.as_ref().and_then(|a| a.start_date.clone()),
                finish_date: i.attributes.as_ref().and_then(|a| a.finish_date.clone()),
                time_frame: i.attributes.as_ref().and_then(|a| a.time_frame.clone()),
            })
            .collect())
    }

    pub async fn get_current_iteration(
        &self,
        org: &str,
        project: &str,
        team: &str,
    ) -> Result<Option<Sprint>> {
        let url = format!(
            "{}/{}/{}/_apis/work/teamsettings/iterations?$timeframe=current&api-version=7.1",
            self.base_url(org),
            project,
            team
        );

        let raw = self.authed_get(&url)?.send().await?;
        let resp: AdoListResponse<AdoIteration> = check_response(raw, "Failed to get current iteration")
            .await?
            .json()
            .await?;

        Ok(resp.value.into_iter().next().map(|i| Sprint {
            id: i.id,
            name: i.name.clone(),
            path: Self::normalize_iteration_path(project, &i.path),
            start_date: i.attributes.as_ref().and_then(|a| a.start_date.clone()),
            finish_date: i.attributes.as_ref().and_then(|a| a.finish_date.clone()),
            time_frame: i.attributes.as_ref().and_then(|a| a.time_frame.clone()),
        }))
    }

    // --- Work Items ---

    pub async fn query_work_items_wiql(
        &self,
        org: &str,
        project: &str,
        wiql: &str,
    ) -> Result<Vec<i32>> {
        let url = format!(
            "{}/{}/_apis/wit/wiql?api-version=7.1",
            self.base_url(org),
            project
        );

        let body = serde_json::json!({ "query": wiql });

        let response = self
            .authed_post(&url)?
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body_text = response.text().await.unwrap_or_default();
            anyhow::bail!("WIQL query failed (HTTP {}): {}", status, body_text);
        }

        let resp: WiqlResponse = response.json().await?;

        let ids: Vec<i32> = resp
            .work_items
            .unwrap_or_default()
            .into_iter()
            .map(|wi| wi.id)
            .collect();

        Ok(ids)
    }

    pub async fn get_work_items_batch(
        &self,
        org: &str,
        project: &str,
        ids: &[i32],
    ) -> Result<Vec<WorkItem>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let url = format!(
            "{}/{}/_apis/wit/workitemsbatch?api-version=7.1",
            self.base_url(org),
            project
        );

        // ADO limits batch to 200 items
        let mut all_items = Vec::new();
        for chunk in ids.chunks(200) {
            let body = AdoWorkItemBatchRequest {
                ids: chunk.to_vec(),
                expand: Some("Relations".to_string()),
                fields: None,
            };

            let raw = self.authed_post(&url)?.json(&body).send().await?;
            let resp: AdoListResponse<AdoWorkItem> = check_response(raw, "Batch get work items failed")
                .await?
                .json()
                .await?;

            for raw in &resp.value {
                all_items.push(parse_work_item(raw));
            }
        }

        Ok(all_items)
    }

    pub async fn get_work_item(
        &self,
        org: &str,
        project: &str,
        id: i32,
    ) -> Result<WorkItem> {
        let url = format!(
            "{}/{}/_apis/wit/workitems/{}?$expand=Relations&api-version=7.1",
            self.base_url(org),
            project,
            id
        );

        let raw_resp = self.authed_get(&url)?.send().await?;
        let raw: AdoWorkItem = check_response(raw_resp, "Failed to get work item")
            .await?
            .json()
            .await?;

        Ok(parse_work_item(&raw))
    }

    pub async fn create_work_item(
        &self,
        org: &str,
        project: &str,
        req: &CreateWorkItemRequest,
    ) -> Result<WorkItem> {
        let wi_type = req.work_item_type.replace(" ", "%20");
        let url = format!(
            "{}/{}/_apis/wit/workitems/${}?api-version=7.1",
            self.base_url(org),
            project,
            wi_type
        );

        let mut ops: Vec<AdoPatchOperation> = vec![AdoPatchOperation {
            op: "add".to_string(),
            path: "/fields/System.Title".to_string(),
            value: serde_json::Value::String(req.title.clone()),
        }];

        if let Some(ref desc) = req.description {
            ops.push(AdoPatchOperation {
                op: "add".to_string(),
                path: "/fields/System.Description".to_string(),
                value: serde_json::Value::String(desc.clone()),
            });
        }
        if let Some(ref assigned) = req.assigned_to {
            ops.push(AdoPatchOperation {
                op: "add".to_string(),
                path: "/fields/System.AssignedTo".to_string(),
                value: serde_json::Value::String(assigned.clone()),
            });
        }
        if let Some(ref iter) = req.iteration_path {
            ops.push(AdoPatchOperation {
                op: "add".to_string(),
                path: "/fields/System.IterationPath".to_string(),
                value: serde_json::Value::String(iter.clone()),
            });
        }
        if let Some(ref area) = req.area_path {
            ops.push(AdoPatchOperation {
                op: "add".to_string(),
                path: "/fields/System.AreaPath".to_string(),
                value: serde_json::Value::String(area.clone()),
            });
        }
        if let Some(priority) = req.priority {
            ops.push(AdoPatchOperation {
                op: "add".to_string(),
                path: "/fields/Microsoft.VSTS.Common.Priority".to_string(),
                value: serde_json::json!(priority),
            });
        }
        if let Some(sp) = req.story_points {
            ops.push(AdoPatchOperation {
                op: "add".to_string(),
                path: "/fields/Microsoft.VSTS.Scheduling.StoryPoints".to_string(),
                value: serde_json::json!(sp),
            });
        }
        if let Some(ref tags) = req.tags {
            ops.push(AdoPatchOperation {
                op: "add".to_string(),
                path: "/fields/System.Tags".to_string(),
                value: serde_json::Value::String(tags.clone()),
            });
        }

        // Add parent link if specified
        if let Some(parent_id) = req.parent_id {
            ops.push(AdoPatchOperation {
                op: "add".to_string(),
                path: "/relations/-".to_string(),
                value: serde_json::json!({
                    "rel": "System.LinkTypes.Hierarchy-Reverse",
                    "url": format!("{}/{}/_apis/wit/workitems/{}", self.base_url(org), project, parent_id)
                }),
            });
        }

        let raw_resp = self
            .authed_post(&url)?
            .header("Content-Type", "application/json-patch+json")
            .json(&ops)
            .send()
            .await?;
        let raw: AdoWorkItem = check_response(raw_resp, "Failed to create work item")
            .await?
            .json()
            .await?;

        Ok(parse_work_item(&raw))
    }

    pub async fn update_work_item(
        &self,
        org: &str,
        project: &str,
        req: &UpdateWorkItemRequest,
    ) -> Result<WorkItem> {
        let url = format!(
            "{}/{}/_apis/wit/workitems/{}?api-version=7.1",
            self.base_url(org),
            project,
            req.id
        );

        let mut ops: Vec<AdoPatchOperation> = vec![];

        if let Some(ref state) = req.state {
            ops.push(AdoPatchOperation {
                op: "replace".to_string(),
                path: "/fields/System.State".to_string(),
                value: serde_json::Value::String(state.clone()),
            });
        }
        if let Some(ref assigned) = req.assigned_to {
            ops.push(AdoPatchOperation {
                op: "replace".to_string(),
                path: "/fields/System.AssignedTo".to_string(),
                value: serde_json::Value::String(assigned.clone()),
            });
        }
        if let Some(ref title) = req.title {
            ops.push(AdoPatchOperation {
                op: "replace".to_string(),
                path: "/fields/System.Title".to_string(),
                value: serde_json::Value::String(title.clone()),
            });
        }
        if let Some(ref desc) = req.description {
            ops.push(AdoPatchOperation {
                op: "replace".to_string(),
                path: "/fields/System.Description".to_string(),
                value: serde_json::Value::String(desc.clone()),
            });
        }
        if let Some(priority) = req.priority {
            ops.push(AdoPatchOperation {
                op: "replace".to_string(),
                path: "/fields/Microsoft.VSTS.Common.Priority".to_string(),
                value: serde_json::json!(priority),
            });
        }
        if let Some(sp) = req.story_points {
            ops.push(AdoPatchOperation {
                op: "replace".to_string(),
                path: "/fields/Microsoft.VSTS.Scheduling.StoryPoints".to_string(),
                value: serde_json::json!(sp),
            });
        }
        if let Some(rw) = req.remaining_work {
            ops.push(AdoPatchOperation {
                op: "replace".to_string(),
                path: "/fields/Microsoft.VSTS.Scheduling.RemainingWork".to_string(),
                value: serde_json::json!(rw),
            });
        }
        if let Some(ref tags) = req.tags {
            ops.push(AdoPatchOperation {
                op: "replace".to_string(),
                path: "/fields/System.Tags".to_string(),
                value: serde_json::Value::String(tags.clone()),
            });
        }

        if ops.is_empty() {
            // Nothing to update, just return the current item
            return self.get_work_item(org, project, req.id).await;
        }

        let raw_resp = self
            .authed_patch(&url)?
            .header("Content-Type", "application/json-patch+json")
            .json(&ops)
            .send()
            .await?;
        let raw: AdoWorkItem = check_response(raw_resp, "Failed to update work item")
            .await?
            .json()
            .await?;

        Ok(parse_work_item(&raw))
    }

}
