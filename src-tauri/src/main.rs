#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod git;
mod settings;

fn main() {
    #[cfg(target_os = "linux")]
    {
        // WebKitGTK 2.44+ DMA-buf renderer fails to init EGL via GBM on some systems
        if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none() {
            unsafe { std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1") };
        }
    }

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
            commands::get_repo_state,
            commands::get_status,
            commands::get_log,
            commands::get_branches,
            commands::get_tags,
            commands::get_stashes,
            commands::get_remotes,
            commands::get_diff_file,
            commands::get_diff_commit,
            commands::get_diff_commit_file,
            commands::stage_files,
            commands::unstage_files,
            commands::discard_files,
            commands::remove_files,
            commands::delete_files,
            commands::stage_hunk,
            commands::unstage_hunk,
            commands::discard_hunk,
            commands::apply_lines,
            commands::do_commit,
            commands::do_rebase,
            commands::do_reset,
            commands::do_revert,
            commands::do_cherry_pick,
            commands::do_squash,
            commands::do_reword_commit,
            commands::do_accept_ours,
            commands::do_accept_theirs,
            commands::do_op_abort,
            commands::do_op_continue,
            commands::do_create_branch,
            commands::do_stash_save,
            commands::do_stash_apply,
            commands::do_stash_pop,
            commands::do_stash_drop,
            commands::do_checkout,
            commands::do_checkout_remote,
            commands::do_fetch,
            commands::do_pull,
            commands::do_push,
            commands::do_push_branch,
            commands::do_merge,
            commands::do_rename_branch,
            commands::do_delete_branch,
            commands::do_create_tag,
            commands::do_delete_tag,
            commands::do_push_tag,
            settings::get_settings,
            settings::set_settings,
            commands::check_repo_path,
            commands::get_repo_stats,
            commands::check_for_update,
            commands::open_url,
            commands::list_system_fonts,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
