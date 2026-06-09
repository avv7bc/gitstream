//! Askpass-мост: GitStream выступает helper'ом для `GIT_ASKPASS`/`SSH_ASKPASS`.
//!
//! При сетевой git-операции `run_network_git` указывает git'у на этот же бинарь
//! как на askpass. Когда git'у нужен логин/пароль/passphrase, он запускает нас с
//! текстом запроса в `argv[1]`; мы коннектимся к IPC-серверу родителя (TCP на
//! localhost + nonce), родитель показывает диалог в GUI и возвращает введённое
//! значение, которое мы печатаем git'у в stdout.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

const ENV_PIPE: &str = "GITSTREAM_ASKPASS_PIPE";
const ENV_NONCE: &str = "GITSTREAM_ASKPASS_NONCE";

/// id запроса → канал, по которому GUI вернёт введённое значение (или отмену).
type PendingMap = Arc<Mutex<HashMap<u64, Sender<Option<String>>>>>;

/// Разобранный prompt git'а — чтобы показать подходящий диалог.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptInfo {
    pub kind: PromptKind,
    pub host: Option<String>,
    pub key_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptKind {
    Username,
    Password,
    Passphrase,
    Confirm,
    Generic,
}

impl PromptKind {
    fn as_str(self) -> &'static str {
        match self {
            PromptKind::Username => "username",
            PromptKind::Password => "password",
            PromptKind::Passphrase => "passphrase",
            PromptKind::Confirm => "confirm",
            PromptKind::Generic => "generic",
        }
    }
}

/// Текст в одинарных кавычках: `Username for 'https://github.com': ` → host.
fn extract_quoted(s: &str) -> Option<String> {
    let start = s.find('\'')?;
    let rest = &s[start + 1..];
    let end = rest.find('\'')?;
    Some(rest[..end].to_string())
}

/// Классифицирует prompt git/ssh в тип поля для диалога.
pub fn parse_prompt(prompt: &str) -> PromptInfo {
    let lower = prompt.to_lowercase();
    if lower.contains("passphrase for key") || lower.contains("enter passphrase") {
        PromptInfo {
            kind: PromptKind::Passphrase,
            host: None,
            key_path: extract_quoted(prompt),
        }
    } else if lower.contains("username for") {
        PromptInfo {
            kind: PromptKind::Username,
            host: extract_quoted(prompt),
            key_path: None,
        }
    } else if lower.contains("password for") || lower.starts_with("password") {
        PromptInfo {
            kind: PromptKind::Password,
            host: extract_quoted(prompt),
            key_path: None,
        }
    } else if lower.contains("(yes/no")
        || lower.contains("yes/no/[fingerprint]")
        || lower.contains("are you sure you want to continue connecting")
    {
        PromptInfo {
            kind: PromptKind::Confirm,
            host: None,
            key_path: None,
        }
    } else {
        PromptInfo {
            kind: PromptKind::Generic,
            host: None,
            key_path: None,
        }
    }
}

/// Событие во фронтенд: показать диалог ввода credentials.
#[derive(Serialize, Clone)]
struct AskpassRequest {
    id: u64,
    prompt: String,
    kind: &'static str,
    host: Option<String>,
    key_path: Option<String>,
}

/// Состояние IPC-сервера, живёт в managed-state Tauri.
pub struct AskpassState {
    /// Адрес listener'а — отдаём git'у через env.
    pub addr: String,
    /// Одноразовый секрет — отсекает чужие локальные процессы.
    pub nonce: String,
    /// Число открытых prompt'ов — `run_network_git` ставит таймаут на паузу, пока > 0.
    pub active: Arc<AtomicUsize>,
    pending: PendingMap,
}

impl AskpassState {
    /// Ответ из GUI: `Some(value)` — провести, `None` — отмена.
    pub fn respond(&self, id: u64, value: Option<String>) {
        if let Some(tx) = self.pending.lock().unwrap().remove(&id) {
            let _ = tx.send(value);
        }
    }
}

/// Генерирует hex-nonce из системного CSPRNG.
fn random_hex(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    getrandom::getrandom(&mut buf).expect("getrandom failed");
    buf.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Поднимает IPC-сервер askpass и возвращает состояние для managed-state.
pub fn start(app: AppHandle) -> std::io::Result<AskpassState> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?.to_string();
    let nonce = random_hex(24);
    let pending: PendingMap = Arc::default();
    let active = Arc::new(AtomicUsize::new(0));
    let next_id = Arc::new(AtomicU64::new(1));

    {
        let pending = pending.clone();
        let active = active.clone();
        let nonce = nonce.clone();
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let pending = pending.clone();
                let active = active.clone();
                let nonce = nonce.clone();
                let next_id = next_id.clone();
                let app = app.clone();
                std::thread::spawn(move || {
                    handle_conn(stream, &nonce, &pending, &next_id, &active, &app);
                });
            }
        });
    }

    Ok(AskpassState {
        addr,
        nonce,
        active,
        pending,
    })
}

fn handle_conn(
    stream: TcpStream,
    nonce: &str,
    pending: &PendingMap,
    next_id: &AtomicU64,
    active: &AtomicUsize,
    app: &AppHandle,
) {
    let mut writer = match stream.try_clone() {
        Ok(w) => w,
        Err(_) => return,
    };
    let mut reader = BufReader::new(stream);

    let mut got_nonce = String::new();
    if reader.read_line(&mut got_nonce).is_err() || got_nonce.trim() != nonce {
        let _ = writeln!(writer, "CANCEL");
        return;
    }
    let mut prompt = String::new();
    let _ = reader.read_line(&mut prompt);
    let prompt = prompt.trim_end_matches(['\r', '\n']).to_string();

    let info = parse_prompt(&prompt);
    let id = next_id.fetch_add(1, Ordering::SeqCst);
    let (tx, rx) = channel::<Option<String>>();
    pending.lock().unwrap().insert(id, tx);
    active.fetch_add(1, Ordering::SeqCst);

    let _ = app.emit(
        "askpass_request",
        AskpassRequest {
            id,
            prompt,
            kind: info.kind.as_str(),
            host: info.host,
            key_path: info.key_path,
        },
    );

    // Блокируемся, пока GUI не ответит через команду askpass_respond.
    let reply = rx.recv().unwrap_or(None);
    active.fetch_sub(1, Ordering::SeqCst);
    pending.lock().unwrap().remove(&id);

    match reply {
        Some(val) => {
            let _ = writeln!(writer, "OK");
            let _ = writeln!(writer, "{}", val);
        }
        None => {
            let _ = writeln!(writer, "CANCEL");
        }
    }
}

/// Если процесс запущен как askpass-helper (выставлен `GITSTREAM_ASKPASS_PIPE`) —
/// обрабатывает запрос и завершает процесс. Возвращает `true`, если это был
/// askpass-вызов (тогда Tauri поднимать не нужно).
pub fn maybe_run_askpass() -> bool {
    let Ok(addr) = std::env::var(ENV_PIPE) else {
        return false;
    };
    let nonce = std::env::var(ENV_NONCE).unwrap_or_default();
    let prompt = std::env::args().nth(1).unwrap_or_default();

    let code = match ask_parent(&addr, &nonce, &prompt) {
        Ok(Some(value)) => {
            let _ = writeln!(std::io::stdout(), "{}", value);
            0
        }
        _ => 1, // отмена/ошибка → git прерывает операцию
    };
    std::process::exit(code);
}

fn ask_parent(addr: &str, nonce: &str, prompt: &str) -> std::io::Result<Option<String>> {
    let stream = TcpStream::connect(addr)?;
    let mut writer = stream.try_clone()?;
    writeln!(writer, "{}", nonce)?;
    writeln!(writer, "{}", prompt)?;
    writer.flush()?;

    let mut reader = BufReader::new(stream);
    let mut status = String::new();
    reader.read_line(&mut status)?;
    if status.trim() != "OK" {
        return Ok(None);
    }
    let mut value = String::new();
    reader.read_line(&mut value)?;
    Ok(Some(value.trim_end_matches(['\r', '\n']).to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_username() {
        let info = parse_prompt("Username for 'https://github.com': ");
        assert_eq!(info.kind, PromptKind::Username);
        assert_eq!(info.host.as_deref(), Some("https://github.com"));
    }

    #[test]
    fn parses_password() {
        let info = parse_prompt("Password for 'https://avv@github.com': ");
        assert_eq!(info.kind, PromptKind::Password);
        assert_eq!(info.host.as_deref(), Some("https://avv@github.com"));
    }

    #[test]
    fn parses_passphrase_with_key_path() {
        let info = parse_prompt("Enter passphrase for key '/home/avv/.ssh/id_ed25519': ");
        assert_eq!(info.kind, PromptKind::Passphrase);
        assert_eq!(info.key_path.as_deref(), Some("/home/avv/.ssh/id_ed25519"));
    }

    #[test]
    fn parses_host_key_confirm() {
        let info = parse_prompt(
            "Are you sure you want to continue connecting (yes/no/[fingerprint])? ",
        );
        assert_eq!(info.kind, PromptKind::Confirm);
    }

    #[test]
    fn falls_back_to_generic() {
        assert_eq!(parse_prompt("Some odd prompt").kind, PromptKind::Generic);
    }
}
