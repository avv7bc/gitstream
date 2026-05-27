use std::sync::OnceLock;
use tauri::Emitter;

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

#[derive(serde::Serialize, Clone)]
struct GitCommandEvent {
    cmd: String,
    output: String,
    success: bool,
}

pub fn init(handle: tauri::AppHandle) {
    let _ = APP_HANDLE.set(handle);
}

pub fn log_git(args: &[&str], output: &str, success: bool) {
    let Some(handle) = APP_HANDLE.get() else {
        return;
    };
    let cmd = format!("$ git {}", args.join(" "));
    let _ = handle.emit(
        "git_command",
        GitCommandEvent {
            cmd,
            output: output.trim().to_string(),
            success,
        },
    );
}
