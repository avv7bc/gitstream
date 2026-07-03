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

// Query-команды (log, diff, blame) могут отдавать десятки тысяч строк, а blob'ы —
// бинарь. В Git output пишем усечённый ответ, чтобы панель оставалась читаемой.
const MAX_OUTPUT_LINES: usize = 40;
const MAX_OUTPUT_CHARS: usize = 4000;

// Ответ одной команды — это ОДНО событие и должен занимать одну строку в
// панели (перенос строки = новое событие). Многие --format/-z команды содержат
// управляющие символы (%x1e RS между записями, %x00 NUL между полями) и
// собственные переводы строк (тело коммита, список remote). Все они сводятся к
// пробелу с схлопыванием повторов — вывод остаётся одной строкой, а по ширине
// панель переносит его мягко (pre-wrap), не создавая псевдо-событий.
fn sanitize(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_control() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out
}

fn truncate_output(s: &str) -> String {
    let sanitized = sanitize(s);
    let s = sanitized.trim();
    let total_lines = s.lines().count();
    let mut out: String = if total_lines > MAX_OUTPUT_LINES {
        let kept: Vec<&str> = s.lines().take(MAX_OUTPUT_LINES).collect();
        format!("{}\n… (+{} строк)", kept.join("\n"), total_lines - MAX_OUTPUT_LINES)
    } else {
        s.to_string()
    };
    if out.chars().count() > MAX_OUTPUT_CHARS {
        out = out.chars().take(MAX_OUTPUT_CHARS).collect::<String>() + "\n… (ответ усечён)";
    }
    out
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
            output: truncate_output(output),
            success,
        },
    );
}
