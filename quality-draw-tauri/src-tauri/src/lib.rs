//! 宁夏特检院质量监督检查抽签程序 - Tauri 后端

mod models;
mod storage;
mod logic;
mod commands;

pub use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_departments,
            get_records,
            clear_records,
            start_new_round,
            get_current_round_status,
            get_candidate_departments,
            execute_draw,
            export_to_excel,
            export_to_pdf,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
