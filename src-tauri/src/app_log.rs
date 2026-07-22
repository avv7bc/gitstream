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

// Hooks и прогресс git'а раскрашивают вывод ANSI-последовательностями
// (ESC[38;2;R;G;Bm и т.п.). Управляющий ESC съедается общим схлопыванием,
// но печатаемый хвост "[38;2;…m" остался бы мусором в панели — вырезаем
// последовательность целиком: CSI (ESC[…финальный байт), OSC (ESC]…BEL/ST)
// и двухсимвольные ESC-коды.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.peek() {
            Some('[') => {
                chars.next();
                // параметры/промежуточные байты — до финального 0x40..=0x7e
                for c2 in chars.by_ref() {
                    if ('\x40'..='\x7e').contains(&c2) {
                        break;
                    }
                }
            }
            Some(']') => {
                chars.next();
                while let Some(c2) = chars.next() {
                    if c2 == '\x07' {
                        break;
                    }
                    if c2 == '\x1b' {
                        if chars.peek() == Some(&'\\') {
                            chars.next();
                        }
                        break;
                    }
                }
            }
            Some(_) => {
                // прочие ESC-коды: промежуточные байты 0x20..=0x2f, затем финальный
                while let Some(&c2) = chars.peek() {
                    chars.next();
                    if !('\x20'..='\x2f').contains(&c2) {
                        break;
                    }
                }
            }
            None => {}
        }
    }
    out
}

// Псевдографика, которой hooks (lefthook) рисуют декоративные рамки и
// анимации: box drawing (U+2500..U+257F) и block elements (U+2580..U+259F).
// В логе это шум — сводим к пробелу, как управляющие символы.
fn is_decor(c: char) -> bool {
    matches!(c, '\u{2500}'..='\u{259F}')
}

// Ответ одной команды — одно событие (один div панели, pre-wrap + monospace),
// поэтому переводы строк СОХРАНЯЮТСЯ: diffstat после pull, списки файлов и
// прочий табличный вывод остаются выровненными, как в терминале. Внутри строки
// управляющие символы (%x1e RS, %x00 NUL у --format/-z команд) и псевдографика
// рамок hook'ов сводятся к пробелу со схлопыванием повторов; строки, состоящие
// из одной декорации, выбрасываются целиком.
fn sanitize(s: &str) -> String {
    let stripped = strip_ansi(s);
    let mut out = String::with_capacity(stripped.len());
    for raw_line in stripped.split('\n') {
        // Спиннеры и счётчики прогресса перерисовывают строку через \r — в лог
        // берём только финальное состояние (текст после последнего \r), иначе
        // все кадры анимации склеиваются в простыню. CRLF-хвост не считается.
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let line = line.rsplit('\r').next().unwrap_or(line);
        let mut cleaned = String::with_capacity(line.len());
        let mut prev_space = false;
        for c in line.chars() {
            if c.is_control() || is_decor(c) {
                if !prev_space {
                    cleaned.push(' ');
                    prev_space = true;
                }
            } else {
                cleaned.push(c);
                prev_space = false;
            }
        }
        if cleaned.trim().is_empty() {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(cleaned.trim_end());
    }
    out
}

fn truncate_output(s: &str) -> String {
    // sanitize уже выбросил пустые строки и хвостовые пробелы; ведущие пробелы
    // строк не трогаем — это выравнивание табличного вывода (diffstat).
    let sanitized = sanitize(s);
    let s = sanitized.as_str();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_true_color_sequences_are_stripped() {
        // фрагмент реального вывода lefthook при push
        let raw = "\x1b[38;2;0;0;0m\x1b[38;2;6;6;6m лefthook won't run\x1b[0m";
        assert_eq!(sanitize(raw), " лefthook won't run");
    }

    #[test]
    fn spinner_redraw_keeps_only_final_frame() {
        let raw = "\x1b[38;2;12;12;12m—\r\x1b[38;2;24;24;24m—\r🥊 lefthook v2.1.9  hook: pre-push";
        assert_eq!(sanitize(raw), "🥊 lefthook v2.1.9  hook: pre-push");
    }

    #[test]
    fn crlf_line_endings_do_not_lose_text() {
        assert_eq!(sanitize("first\r\nsecond"), "first\nsecond");
    }

    #[test]
    fn lefthook_box_frame_is_removed() {
        let raw = "╭─────────────╮\n│ 🥊 lefthook v2.1.9  hook: pre-push │\n╰─────────────╯\nbranch 'master' set up to track 'origin/master'.";
        assert_eq!(
            sanitize(raw),
            "  🥊 lefthook v2.1.9  hook: pre-push\nbranch 'master' set up to track 'origin/master'."
        );
    }

    #[test]
    fn diffstat_lines_and_alignment_are_preserved() {
        let raw = "Fast-forward\n a/b.rs  | 16 +-\n c/d.lua |  5 +\n";
        assert_eq!(sanitize(raw), "Fast-forward\n a/b.rs  | 16 +-\n c/d.lua |  5 +");
    }

    #[test]
    fn osc_and_two_char_escapes_are_stripped() {
        assert_eq!(strip_ansi("\x1b]0;title\x07text\x1b(Bmore"), "textmore");
    }

    #[test]
    fn plain_output_is_untouched() {
        assert_eq!(sanitize("branch 'master' set up to track 'origin/master'."),
                   "branch 'master' set up to track 'origin/master'.");
    }
}
