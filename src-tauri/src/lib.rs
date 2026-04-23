mod ado_client;
mod auth;
mod sprints;
mod types;
mod work_items;

use ado_client::AdoClient;
use tauri_plugin_log::{Target, TargetKind};
use tokio::sync::RwLock;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::Webview),
                    Target::new(TargetKind::LogDir {
                        file_name: Some("adovibes".to_string()),
                    }),
                ])
                .level(log::LevelFilter::Info)
                .timezone_strategy(tauri_plugin_log::TimezoneStrategy::UseLocal)
                .build(),
        )
        .manage(RwLock::new(AdoClient::new()))
        .invoke_handler(tauri::generate_handler![
            auth::check_auth_status,
            auth::validate_azcli,
            auth::get_auth_header,
            auth::get_current_user,
            auth::set_auth_pat,
            auth::set_auth_azcli,
            auth::validate_pat,
            sprints::list_iterations,
            sprints::get_current_iteration,
            work_items::search_projects,
            work_items::search_teams,
            work_items::get_sprint_work_items,
            work_items::get_work_item,
            work_items::create_work_item,
            work_items::update_work_item,
            work_items::move_work_item,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
