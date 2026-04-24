#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod git;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            use tauri::Manager;
            if let Some(window) = app.get_webview_window("main") {
                let icon_bytes = include_bytes!("../icons/icon.png");
                if let Ok(image) = tauri::image::Image::from_bytes(icon_bytes) {
                    let _ = window.set_icon(image);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_repo_info,
            commands::get_status,
            commands::get_log,
            commands::get_branches,
            commands::get_tags,
            commands::get_stashes,
            commands::get_remotes,
            commands::get_diff_file,
            commands::get_diff_commit,
            commands::stage_files,
            commands::unstage_files,
            commands::discard_files,
            commands::do_commit,
            commands::do_checkout,
            commands::do_checkout_remote,
            commands::do_fetch,
            commands::do_pull,
            commands::do_push,
            commands::do_push_branch,
            commands::do_merge,
            commands::do_rename_branch,
            commands::do_delete_branch,
            commands::do_clone,
            commands::check_repo_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
