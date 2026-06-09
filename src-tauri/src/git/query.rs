use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Command;

use base64::{engine::general_purpose::STANDARD as B64, Engine as _};

use super::error::{classify_git_error, GitError};
use super::types::*;

pub fn run_git(repo_path: &Path, args: &[&str]) -> Result<String, GitError> {
    // core.quotePath=false: иначе git экранирует не-ASCII пути (кириллицу)
    // октальными escape'ами вида "\320\277...", и они не совпадают с чистым
    // UTF-8 из `ls-files -z` — ломается список файлов и дерево папок.
    let output = Command::new("git")
        .args(["-c", "core.quotePath=false", "-C"])
        .arg(repo_path)
        .args(args)
        .output()
        .map_err(|e| GitError::CommandFailed {
            message: format!("Failed to run git: {}", e),
            hint: Some("Is git installed and in PATH?".into()),
        })?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(classify_git_error(&stderr))
    }
}

/// Запускает git, возвращая stdout независимо от кода возврата.
/// Нужно для `diff --no-index`, который при наличии различий выходит с кодом 1.
fn run_git_lenient(repo_path: &Path, args: &[&str]) -> String {
    Command::new("git")
        .args(["-c", "core.quotePath=false", "-C"])
        .arg(repo_path)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_default()
}

/// Запускает git, возвращая «сырые» байты stdout без потерь UTF-8.
/// Нужно для бинарных blob'ов (`git show <rev>:<path>`).
fn run_git_bytes(repo_path: &Path, args: &[&str]) -> Result<Vec<u8>, GitError> {
    let output = Command::new("git")
        .args(["-c", "core.quotePath=false", "-C"])
        .arg(repo_path)
        .args(args)
        .output()
        .map_err(|e| GitError::CommandFailed {
            message: format!("Failed to run git: {}", e),
            hint: Some("Is git installed and in PATH?".into()),
        })?;
    if output.status.success() {
        Ok(output.stdout)
    } else {
        Err(classify_git_error(&String::from_utf8_lossy(&output.stderr)))
    }
}

/// Файл не отслеживается git (нет ни в индексе, ни в HEAD).
fn is_untracked(repo_path: &Path, file: &str) -> bool {
    run_git(repo_path, &["ls-files", "--error-unmatch", "--", file]).is_err()
}

/// Источник содержимого файла для предпросмотра бинарных файлов.
enum BlobSrc<'a> {
    /// Файл в рабочем дереве (читается с диска).
    Disk,
    /// git-ревизия: `Rev("HEAD")` → `HEAD:path`, `Rev("")` → `:path` (индекс).
    Rev(&'a str),
}

/// Расширение пути — растровое изображение, показываемое в `<img>`.
fn is_image_path(path: &str) -> bool {
    matches!(
        path.rsplit('.').next().map(str::to_ascii_lowercase).as_deref(),
        Some("png" | "jpg" | "jpeg" | "gif" | "webp" | "bmp" | "ico" | "avif")
    )
}

/// base64 содержимого файла из указанного источника (None — источника нет).
fn image_of(repo_path: &Path, src: &BlobSrc, path: &str) -> Option<String> {
    let bytes = match src {
        BlobSrc::Disk => std::fs::read(repo_path.join(path)).ok()?,
        BlobSrc::Rev(rev) => {
            run_git_bytes(repo_path, &["show", &format!("{}:{}", rev, path)]).ok()?
        }
    };
    Some(B64.encode(bytes))
}

/// Размер файла в байтах из указанного источника.
fn size_of(repo_path: &Path, src: &BlobSrc, path: &str) -> Option<u64> {
    match src {
        BlobSrc::Disk => std::fs::metadata(repo_path.join(path)).ok().map(|m| m.len()),
        BlobSrc::Rev(rev) => run_git(repo_path, &["cat-file", "-s", &format!("{}:{}", rev, path)])
            .ok()
            .and_then(|s| s.trim().parse().ok()),
    }
}

/// Дополняет дифф бинарного файла размером и (для изображений) base64
/// старой/новой версии — иначе панель Changes для бинарного файла пуста.
fn fill_binary(repo_path: &Path, diff: &mut FileDiff, old: Option<BlobSrc>, new: BlobSrc) {
    if !diff.binary {
        return;
    }
    let path = diff.path.clone();
    diff.byte_size = size_of(repo_path, &new, &path)
        .or_else(|| old.as_ref().and_then(|s| size_of(repo_path, s, &path)));
    if is_image_path(&path) {
        diff.new_image = image_of(repo_path, &new, &path);
        diff.old_image = old.as_ref().and_then(|s| image_of(repo_path, s, &path));
    }
}

pub fn status(repo_path: &Path) -> Result<Vec<FileStatus>, GitError> {
    // --untracked-files=all: иначе git схлопывает untracked-каталог в одну
    // запись (`back/ws_server/`); UI должен показывать каждый файл отдельно.
    let output = run_git(
        repo_path,
        &["status", "--porcelain=v2", "--untracked-files=all"],
    )?;
    let mut files = Vec::new();
    for line in output.lines() {
        if line.starts_with('1') || line.starts_with('2') {
            let parts: Vec<&str> = line.splitn(9, ' ').collect();
            if parts.len() < 9 {
                continue;
            }
            let xy = parts[1];
            let bytes = xy.as_bytes();
            if bytes.len() < 2 {
                continue;
            }
            let x = bytes[0] as char;
            let y = bytes[1] as char;
            let path = if line.starts_with('2') {
                parts[8].split('\t').nth(1).unwrap_or(parts[8]).to_string()
            } else {
                parts[8].to_string()
            };
            let (state, staged) = match (x, y) {
                ('M', '.') => ("modified", "staged"),
                ('.', 'M') => ("modified", "unstaged"),
                ('M', 'M') => ("modified", "partial"),
                ('A', '.') => ("added", "staged"),
                ('.', 'A') => ("added", "unstaged"),
                ('D', '.') => ("deleted", "staged"),
                ('.', 'D') => ("deleted", "unstaged"),
                ('R', '.') => ("renamed", "staged"),
                ('R', 'M') => ("renamed", "partial"),
                _ if xy.contains('U') || xy == "AA" || xy == "DD" => ("conflicted", "unstaged"),
                _ => ("modified", "unstaged"),
            };
            files.push(FileStatus {
                path,
                state: state.to_string(),
                staged: staged.to_string(),
            });
        } else if line.starts_with('?') {
            let path = line[2..].to_string();
            files.push(FileStatus {
                path,
                state: "untracked".to_string(),
                staged: "unstaged".to_string(),
            });
        }
    }
    Ok(files)
}

/// Все отслеживаемые файлы рабочей копии (`git ls-files`). Untracked-файлы
/// приходят отдельно через `status` — здесь только tracked, чтобы UI мог
/// показать неизменённые файлы поверх списка изменений ("Show all files").
pub fn list_all_files(repo_path: &Path) -> Result<Vec<String>, GitError> {
    // Вся рабочая копия как в браузере Files SmartGit: tracked (--cached) +
    // untracked (--others), но без игнорируемых (--exclude-standard). Без
    // --others не видны untracked-файлы внутри untracked-папок — git status
    // схлопывает такую папку в одну строку `dir/`, и её содержимое в дерево
    // не попадает, поэтому полная структура папок не строится.
    let output = run_git(
        repo_path,
        &["ls-files", "--cached", "--others", "--exclude-standard", "-z"],
    )?;
    let files = output
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    Ok(files)
}

/// Все файлы дерева конкретного коммита (`git ls-tree -r`). Нужно для тоггла
/// "Show all files" при выбранном коммите: к изменённым файлам коммита
/// добавляются неизменённые (присутствовавшие в дереве, но не затронутые им) —
/// симметрично list_all_files для рабочей копии.
pub fn list_files_at(repo_path: &Path, oid: &str) -> Result<Vec<String>, GitError> {
    let output = run_git(
        repo_path,
        &[
            "-c",
            "core.quotepath=false",
            "ls-tree",
            "-r",
            "--name-only",
            "-z",
            oid,
        ],
    )?;
    let files = output
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    Ok(files)
}

// \x1e разделяет записи коммитов; %B — полное сообщение (может содержать \n).
const LOG_FORMAT: &str = "%x1e%H%x00%h%x00%an%x00%ae%x00%aI%x00%P%x00%D%x00%B";

// Разбирает одну запись формата LOG_FORMAT в CommitInfo. Общий для log/file_log.
fn parse_commit_record(
    record: &str,
    remotes_list: &[String],
    unpushed: &std::collections::HashSet<String>,
) -> Option<CommitInfo> {
    if record.is_empty() {
        return None;
    }
    let parts: Vec<&str> = record.splitn(8, '\0').collect();
    if parts.len() < 8 {
        return None;
    }
    let refs = parse_ref_labels(parts[6], remotes_list);
    let parents: Vec<String> = parts[5].split_whitespace().map(|s| s.to_string()).collect();
    let message = parts[7].trim_end_matches('\n').to_string();
    Some(CommitInfo {
        oid: parts[0].to_string(),
        short_oid: parts[1].to_string(),
        message,
        author: parts[2].to_string(),
        author_email: parts[3].to_string(),
        date: parts[4].to_string(),
        parents,
        refs,
        column: 0,
        lines: Vec::new(),
        unpushed: unpushed.contains(parts[0]),
    })
}

// Незапушенные («исходящие») коммиты: достижимы из локальных веток, но не из
// какого-либо remote-tracking ref-а. Нет remote'ов → не запушено всё.
fn unpushed_set(repo_path: &Path) -> std::collections::HashSet<String> {
    run_git(repo_path, &["rev-list", "--branches", "--not", "--remotes"])
        .map(|o| o.split_whitespace().map(|s| s.to_string()).collect())
        .unwrap_or_default()
}

pub fn log(repo_path: &Path, limit: usize) -> Result<Vec<CommitInfo>, GitError> {
    // Пустой репозиторий (`git init` без коммитов): HEAD ещё не существует,
    // `git log` падает с кодом 128. Это не ошибка — лог просто пуст.
    // Проверка через rev-parse не зависит от локали git.
    if run_git(repo_path, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_err() {
        return Ok(Vec::new());
    }
    let limit_str = format!("-{}", limit);
    // --branches --remotes: видны все локальные и remote-ветки (после fetch
    // тоже). Теги намеренно НЕ включаем (в отличие от --all), иначе коммиты,
    // живые лишь из-за тега (после squash/amend в ветке), рисуются паразитной
    // отдельной веткой. Теги на достижимых коммитах всё равно показываются
    // как ref-лейблы. HEAD добавляем явно ради detached-режима.
    // --topo-order: коммиты одной ветки идут подряд, не перемешиваясь по дате
    // с коммитами других веток (как --date-order). Это даёт чистый граф в
    // стиле SmartGit/gitk: лейн смерженной ветки не «зигзагует» через mainline,
    // а тянется сплошным столбцом справа до своего мержа.
    let output = run_git(
        repo_path,
        &[
            "log",
            "--branches",
            "--remotes",
            "HEAD",
            "--topo-order",
            &format!("--format={}", LOG_FORMAT),
            &limit_str,
        ],
    )?;
    // Список remote'ов нужен parse_ref_labels: иначе локальные ветки
    // вида `feature/auth` неотличимы от `origin/main` (в %D обе со слешем).
    let remotes_list = remotes(repo_path).unwrap_or_default();
    let unpushed = unpushed_set(repo_path);
    let mut commits: Vec<CommitInfo> = output
        .split('\x1e')
        .filter_map(|r| parse_commit_record(r, &remotes_list, &unpushed))
        .collect();
    super::graph::assign_lanes(&mut commits);
    Ok(commits)
}

// История одного файла: `git log --follow -- <path>` (отслеживает переименования).
// Плоский список (без lane-графа). Пустой репозиторий / файл без истории → пусто.
pub fn file_log(repo_path: &Path, path: &str, limit: usize) -> Result<Vec<CommitInfo>, GitError> {
    if run_git(repo_path, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_err() {
        return Ok(Vec::new());
    }
    let limit_str = format!("-{}", limit);
    let output = run_git(
        repo_path,
        &[
            "log",
            "--follow",
            "--topo-order",
            &format!("--format={}", LOG_FORMAT),
            &limit_str,
            "--",
            path,
        ],
    )?;
    let remotes_list = remotes(repo_path).unwrap_or_default();
    let unpushed = unpushed_set(repo_path);
    let commits = output
        .split('\x1e')
        .filter_map(|r| parse_commit_record(r, &remotes_list, &unpushed))
        .collect();
    Ok(commits)
}

// Blame: автор каждой строки файла. `--porcelain` отдаёт по группе строк
// заголовок `<sha> <orig> <final> [n]`, затем метаданные коммита (один раз на
// sha) и строки контента с ведущим TAB. Повторный коммит — только заголовок.
pub fn blame(repo_path: &Path, path: &str, rev: Option<&str>) -> Result<Vec<BlameLine>, GitError> {
    let mut args = vec!["blame", "--porcelain"];
    if let Some(r) = rev {
        args.push(r);
    }
    args.push("--");
    args.push(path);
    let output = run_git(repo_path, &args)?;
    Ok(parse_blame_porcelain(&output))
}

fn parse_blame_porcelain(output: &str) -> Vec<BlameLine> {
    // sha → (author, author_time, summary): метаданные приходят раз на коммит.
    let mut cache: HashMap<String, (String, i64, String)> = HashMap::new();
    let mut out = Vec::new();

    let mut oid = String::new();
    let mut final_line: u32 = 0;
    let mut author: Option<String> = None;
    let mut time: Option<i64> = None;
    let mut summary: Option<String> = None;

    for raw in output.split('\n') {
        if let Some(content) = raw.strip_prefix('\t') {
            // Строка контента завершает запись. Если по ходу собрали метаданные
            // — кэшируем их под текущим sha.
            if author.is_some() || time.is_some() || summary.is_some() {
                cache.insert(
                    oid.clone(),
                    (
                        author.take().unwrap_or_default(),
                        time.take().unwrap_or_default(),
                        summary.take().unwrap_or_default(),
                    ),
                );
            }
            let (a, t, s) = cache.get(&oid).cloned().unwrap_or_default();
            out.push(BlameLine {
                short_oid: oid.chars().take(8).collect(),
                oid: oid.clone(),
                author: a,
                author_time: t,
                summary: s,
                line_no: final_line,
                content: content.to_string(),
            });
        } else {
            let mut parts = raw.splitn(2, ' ');
            let key = parts.next().unwrap_or("");
            let val = parts.next().unwrap_or("");
            if key.len() == 40 && key.bytes().all(|b| b.is_ascii_hexdigit()) {
                // Заголовок группы: `<sha> <orig> <final> [n]`.
                oid = key.to_string();
                final_line = val
                    .split(' ')
                    .nth(1)
                    .and_then(|n| n.parse().ok())
                    .unwrap_or(0);
                author = None;
                time = None;
                summary = None;
            } else {
                match key {
                    "author" => author = Some(val.to_string()),
                    "author-time" => time = val.parse().ok(),
                    "summary" => summary = Some(val.to_string()),
                    _ => {}
                }
            }
        }
    }
    out
}

fn parse_ref_labels(raw: &str, remotes: &[String]) -> Vec<RefLabel> {
    if raw.trim().is_empty() {
        return Vec::new();
    }
    // Ветка считается remote только если её первый сегмент совпадает с
    // именем настоящего remote'а — иначе локальная ветка `feature/auth`
    // ошибочно классифицировалась бы как remote (просто потому что в имени
    // есть слеш).
    let is_remote_name =
        |name: &str| remotes.iter().any(|rem| name.starts_with(&format!("{}/", rem)));
    raw.split(", ")
        .filter_map(|r| {
            let r = r.trim();
            if r.is_empty() {
                return None;
            }
            if r == "HEAD" {
                return Some(RefLabel {
                    name: "HEAD".to_string(),
                    kind: "head".to_string(),
                });
            }
            if let Some(rest) = r.strip_prefix("HEAD -> ") {
                return Some(RefLabel {
                    name: rest.to_string(),
                    kind: "current-branch".to_string(),
                });
            }
            if let Some(t) = r.strip_prefix("tag: ") {
                return Some(RefLabel {
                    name: t.to_string(),
                    kind: "tag".to_string(),
                });
            }
            if is_remote_name(r) {
                // Symbolic `<remote>/HEAD` — alias на дефолтную ветку, дублирует
                // её ref. Не рисуем: SmartGit/GitKraken тоже прячут.
                if r.ends_with("/HEAD") {
                    return None;
                }
                Some(RefLabel {
                    name: r.to_string(),
                    kind: "remote-branch".to_string(),
                })
            } else {
                Some(RefLabel {
                    name: r.to_string(),
                    kind: "local-branch".to_string(),
                })
            }
        })
        .collect()
}

pub fn branches(repo_path: &Path) -> Result<Vec<BranchInfo>, GitError> {
    // %(refname) — полный путь (refs/heads/... vs refs/remotes/...) для
    // надёжной классификации local/remote (имя со слешем вроде `feature/auth`
    // — локальная ветка, не remote).
    // %(symref) — для symbolic ref'ов (например `refs/remotes/origin/HEAD`)
    // даёт целевой ref; такие алиасы фильтруем, иначе они дублируют
    // настоящую ветку и засоряют список (а %(refname:short) у них к тому
    // же возвращает просто `origin`, что коллизит с реальной локальной
    // веткой по имени `origin`).
    // %(authorname) — автор коммита, на который указывает ветка. Git не хранит
    // «кто создал ветку», поэтому автора tip-коммита используем как ближайшее
    // приближение (так же показывают SmartGit/GitKraken).
    let format = "%(refname)%00%(refname:short)%00%(symref)%00%(upstream:short)%00%(upstream:track,nobracket)%00%(HEAD)%00%(authorname)";
    // LC_ALL=C: `%(upstream:track)` git локализует ("впереди 1" в ru) —
    // парсер ahead/behind понимает только английский, иначе индикатор
    // незапушенных коммитов всегда показывает 0.
    let output = Command::new("git")
        .env("LC_ALL", "C")
        .args(["-c", "core.quotePath=false", "-C"])
        .arg(repo_path)
        .args(["branch", "-a", &format!("--format={}", format)])
        .output()
        .map_err(|e| GitError::CommandFailed {
            message: format!("Failed to run git: {}", e),
            hint: Some("Is git installed and in PATH?".into()),
        })?;
    if !output.status.success() {
        return Err(classify_git_error(&String::from_utf8_lossy(&output.stderr)));
    }
    let output = String::from_utf8_lossy(&output.stdout).to_string();
    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(7, '\0').collect();
        if parts.len() < 7 {
            continue;
        }
        let full_ref = parts[0];
        let symref = parts[2];
        // Любой symbolic ref пропускаем — это alias, не самостоятельная ветка.
        if !symref.is_empty() {
            continue;
        }
        let is_remote = full_ref.starts_with("refs/remotes/");
        // Имя берём из полного refname, а не из %(refname:short): для ветки,
        // чьё имя коллизит с remote (например локальная `origin`), git
        // дизамбигуирует short-форму в `heads/origin`. Отрезаем известный
        // префикс сами — так локальная `origin` показывается как `origin`
        // (поведение SmartGit), а у обычных веток имя не меняется.
        let name = full_ref
            .strip_prefix("refs/heads/")
            .or_else(|| full_ref.strip_prefix("refs/remotes/"))
            .unwrap_or(parts[1])
            .to_string();
        // Старая фильтрация на случай вывода `git branch -a` со стрелкой.
        if name.contains("HEAD") && name.contains("->") {
            continue;
        }
        let upstream = if parts[3].is_empty() {
            None
        } else {
            Some(parts[3].to_string())
        };
        let (ahead, behind) = parse_track(parts[4]);
        let is_current = parts[5].trim() == "*";
        let author = if parts[6].is_empty() {
            None
        } else {
            Some(parts[6].to_string())
        };
        result.push(BranchInfo {
            name,
            is_remote,
            upstream,
            ahead,
            behind,
            is_current,
            author,
        });
    }
    Ok(result)
}

fn parse_track(track: &str) -> (u32, u32) {
    let mut ahead = 0u32;
    let mut behind = 0u32;
    for part in track.split(", ") {
        let part = part.trim();
        if let Some(n) = part.strip_prefix("ahead ") {
            ahead = n.parse().unwrap_or(0);
        } else if let Some(n) = part.strip_prefix("behind ") {
            behind = n.parse().unwrap_or(0);
        }
    }
    (ahead, behind)
}

pub fn tags(repo_path: &Path) -> Result<Vec<TagInfo>, GitError> {
    let format = "%(refname:short)%00%(*objectname:short)%00%(contents:subject)";
    // --sort=-version:refname — semver-сортировка по убыванию (v0.7.13 идёт
    // после v0.7.9, а не между v0.7.0 и v0.7.2 как при алфавитной по умолчанию).
    // Как в SmartGit: новые версии сверху.
    let output = run_git(
        repo_path,
        &[
            "tag",
            "-l",
            "--sort=-version:refname",
            &format!("--format={}", format),
        ],
    )?;
    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '\0').collect();
        let name = parts.first().unwrap_or(&"").to_string();
        let oid = parts.get(1).unwrap_or(&"").to_string();
        let message = parts
            .get(2)
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty());
        result.push(TagInfo { name, oid, message });
    }
    Ok(result)
}

pub fn stashes(repo_path: &Path) -> Result<Vec<StashEntry>, GitError> {
    let output = match run_git(repo_path, &["stash", "list", "--format=%gd%x00%gs%x00%ar"]) {
        Ok(o) => o,
        Err(_) => return Ok(Vec::new()),
    };
    let mut result = Vec::new();
    for line in output.lines() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.splitn(3, '\0').collect();
        let index_str = parts.first().unwrap_or(&"");
        let index = index_str
            .strip_prefix("stash@{")
            .and_then(|s| s.strip_suffix('}'))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(0);
        let message = parts.get(1).unwrap_or(&"").to_string();
        let date = parts.get(2).unwrap_or(&"").to_string();
        result.push(StashEntry {
            index,
            message,
            date,
        });
    }
    Ok(result)
}

pub fn remotes(repo_path: &Path) -> Result<Vec<String>, GitError> {
    let output = run_git(repo_path, &["remote"])?;
    Ok(output
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect())
}

/// Список remote'ов с fetch-URL. Парсит `git remote -v`, берёт строки `(fetch)`.
pub fn remote_urls(repo_path: &Path) -> Result<Vec<RemoteInfo>, GitError> {
    let output = run_git(repo_path, &["remote", "-v"])?;
    let mut result = Vec::new();
    for line in output.lines() {
        if !line.ends_with("(fetch)") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let name = match parts.next() {
            Some(n) => n.to_string(),
            None => continue,
        };
        let url = match parts.next() {
            Some(u) => u.to_string(),
            None => continue,
        };
        result.push(RemoteInfo { name, url });
    }
    Ok(result)
}

/// Состояние репозитория: незавершённая merge/rebase/cherry-pick/revert.
/// Возвращает "clean" | "merging" | "rebasing" | "cherry-picking" | "reverting".
pub fn repo_state(repo_path: &Path) -> Result<String, GitError> {
    let git_dir_raw = run_git(repo_path, &["rev-parse", "--git-dir"])?;
    let git_dir = git_dir_raw.trim();
    let base = Path::new(repo_path).join(git_dir);
    let exists = |p: &str| base.join(p).exists();

    let state = if exists("rebase-merge") || exists("rebase-apply") {
        "rebasing"
    } else if exists("CHERRY_PICK_HEAD") {
        "cherry-picking"
    } else if exists("REVERT_HEAD") {
        "reverting"
    } else if exists("MERGE_HEAD") {
        "merging"
    } else {
        "clean"
    };
    Ok(state.to_string())
}

pub fn current_branch_name(repo_path: &Path) -> Result<String, GitError> {
    let out = run_git(repo_path, &["branch", "--show-current"])?;
    Ok(out.trim().to_string())
}

pub fn repo_info(repo_path: &Path) -> Result<RepoInfo, GitError> {
    let path = run_git(repo_path, &["rev-parse", "--show-toplevel"])?;
    let branch = run_git(repo_path, &["branch", "--show-current"])?;
    let head = run_git(repo_path, &["rev-parse", "HEAD"]).unwrap_or_default();
    Ok(RepoInfo {
        path: path.trim().to_string(),
        current_branch: branch.trim().to_string(),
        head_oid: head.trim().to_string(),
    })
}

pub fn diff_file(
    repo_path: &Path,
    file: &str,
    staged: bool,
    context: Option<u32>,
) -> Result<FileDiff, GitError> {
    // `-U<n>` управляет числом строк контекста: File Compare запрашивает весь
    // файл (большое n), основная diff-панель — обычные хунки (None).
    let ctx = context.map(|n| format!("-U{}", n));
    // Untracked-файл отсутствует в индексе/HEAD — обычный `git diff` пуст.
    // Синтезируем дифф «всё добавлено» сравнением с /dev/null.
    if !staged && is_untracked(repo_path, file) {
        let mut args = vec!["-c", "core.quotepath=false", "diff", "--no-index"];
        if let Some(c) = &ctx {
            args.push(c);
        }
        args.extend(["--", "/dev/null", file]);
        let output = run_git_lenient(repo_path, &args);
        let mut diff = parse_diff_single(&output, file);
        fill_binary(repo_path, &mut diff, None, BlobSrc::Disk);
        return Ok(diff);
    }
    let mut args = vec!["-c", "core.quotepath=false", "diff"];
    if staged {
        args.push("--cached");
    }
    if let Some(c) = &ctx {
        args.push(c);
    }
    args.extend(["--", file]);
    let output = run_git(repo_path, &args).unwrap_or_default();
    let mut diff = parse_diff_single(&output, file);
    // staged: новая версия — индекс (`:path`); unstaged — рабочее дерево.
    let new = if staged { BlobSrc::Rev("") } else { BlobSrc::Disk };
    fill_binary(repo_path, &mut diff, Some(BlobSrc::Rev("HEAD")), new);
    Ok(diff)
}

pub fn diff_commit(repo_path: &Path, oid: &str) -> Result<Vec<FileDiff>, GitError> {
    // Корневой коммит не имеет parent — `oid^..oid` падает с «unknown
    // revision». Для остальных оставляем диапазон: на merge-коммитах он
    // отдаёт diff против первого родителя (`oid^` ≡ `oid^1`), а `git show`/
    // `git log -p` вернули бы пустой combined-diff и панель файлов осталась
    // бы пуста.
    let has_parent = run_git(
        repo_path,
        &["rev-parse", "--verify", "--quiet", &format!("{}^", oid)],
    )
    .is_ok();
    let output = if has_parent {
        let range = format!("{}^..{}", oid, oid);
        run_git(repo_path, &["-c", "core.quotepath=false", "diff", &range])?
    } else {
        run_git(
            repo_path,
            &["-c", "core.quotepath=false", "show", oid, "--format="],
        )?
    };
    Ok(parse_diff_multi(&output))
}

/// Дифф одного файла внутри коммита.
///
/// Запрашивается на каждый клик по файлу коммита — это дешевле, чем парсить
/// дифф всего коммита.
///
/// Для обычных и merge-коммитов используем `git diff oid^..oid -- file`
/// (diff vs первого родителя), что совпадает с логикой `diff_commit`.
/// `git show oid -- file` использует combined diff: для merge-коммитов с
/// чистым слиянием он возвращает пустой вывод, хотя файл реально изменился
/// (виден в списке файлов через `diff_commit`).
/// Для корневого коммита (нет родителя) `oid^` не существует — в этом случае
/// откатываемся к `git show`.
/// Пайспек `-- file` убирает детектирование переименований, поэтому
/// переименованный файл всегда показывается как полное добавление.
pub fn diff_commit_file(
    repo_path: &Path,
    oid: &str,
    file: &str,
    context: Option<u32>,
) -> Result<FileDiff, GitError> {
    let ctx = context.map(|n| format!("-U{}", n));
    let has_parent = run_git(
        repo_path,
        &["rev-parse", "--verify", "--quiet", &format!("{}^", oid)],
    )
    .is_ok();

    let range = format!("{}^..{}", oid, oid);
    let output = if has_parent {
        let mut args = vec!["-c", "core.quotepath=false", "diff", &range];
        if let Some(c) = &ctx {
            args.push(c);
        }
        args.extend(["--", file]);
        run_git(repo_path, &args)?
    } else {
        let mut args = vec!["-c", "core.quotepath=false", "show", oid, "--format="];
        if let Some(c) = &ctx {
            args.push(c);
        }
        args.extend(["--", file]);
        run_git(repo_path, &args)?
    };

    let mut diff = parse_diff_single(&output, file);
    let parent = format!("{}^", oid);
    fill_binary(
        repo_path,
        &mut diff,
        if has_parent {
            Some(BlobSrc::Rev(&parent))
        } else {
            None
        },
        BlobSrc::Rev(oid),
    );
    Ok(diff)
}

fn parse_diff_single(diff_text: &str, fallback_path: &str) -> FileDiff {
    let mut hunks = Vec::new();
    let mut current_lines: Vec<DiffLine> = Vec::new();
    let mut current_header = String::new();
    let mut current_raw = String::new();
    let mut patch_header = String::new();
    let mut seen_hunk = false;
    let mut insertions = 0u32;
    let mut deletions = 0u32;
    let mut old_line = 0u32;
    let mut new_line = 0u32;
    let mut path = fallback_path.to_string();
    let mut binary = false;

    let push_hunk =
        |hunks: &mut Vec<DiffHunk>, header: &str, lines: &mut Vec<DiffLine>, raw: &mut String| {
            hunks.push(DiffHunk {
                header: header.to_string(),
                lines: std::mem::take(lines),
                raw: std::mem::take(raw),
            });
        };

    for line in diff_text.lines() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            // git дописывает хвостовой TAB, если имя содержит пробелы/спецсимволы.
            path = rest.split('\t').next().unwrap_or("").to_string();
        } else if let Some(rest) = line.strip_prefix("rename to ") {
            // У чистого переименования нет строки `+++` — берём путь отсюда.
            path = rest.split('\t').next().unwrap_or("").to_string();
        } else if let Some(rest) = line.strip_prefix("copy to ") {
            path = rest.split('\t').next().unwrap_or("").to_string();
        } else if line.starts_with("Binary files ") {
            binary = true;
        }
        if line.starts_with("@@ ") {
            if !current_header.is_empty() {
                push_hunk(
                    &mut hunks,
                    &current_header,
                    &mut current_lines,
                    &mut current_raw,
                );
            }
            seen_hunk = true;
            current_header = line.to_string();
            current_raw.push_str(line);
            current_raw.push('\n');
            if let Some(nums) = line.strip_prefix("@@ ") {
                let parts: Vec<&str> = nums.split(' ').collect();
                if parts.len() >= 2 {
                    old_line = parts[0]
                        .trim_start_matches('-')
                        .split(',')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                    new_line = parts[1]
                        .trim_start_matches('+')
                        .split(',')
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(1);
                }
            }
        } else if !seen_hunk {
            patch_header.push_str(line);
            patch_header.push('\n');
        } else if line.starts_with('+') && !line.starts_with("+++") {
            insertions += 1;
            current_lines.push(DiffLine {
                kind: "added".to_string(),
                old_lineno: None,
                new_lineno: Some(new_line),
                content: line[1..].to_string(),
            });
            new_line += 1;
            current_raw.push_str(line);
            current_raw.push('\n');
        } else if line.starts_with('-') && !line.starts_with("---") {
            deletions += 1;
            current_lines.push(DiffLine {
                kind: "removed".to_string(),
                old_lineno: Some(old_line),
                new_lineno: None,
                content: line[1..].to_string(),
            });
            old_line += 1;
            current_raw.push_str(line);
            current_raw.push('\n');
        } else if let Some(rest) = line.strip_prefix(' ') {
            current_lines.push(DiffLine {
                kind: "context".to_string(),
                old_lineno: Some(old_line),
                new_lineno: Some(new_line),
                content: rest.to_string(),
            });
            old_line += 1;
            new_line += 1;
            current_raw.push_str(line);
            current_raw.push('\n');
        } else if line.starts_with('\\') {
            // "\ No newline at end of file" — часть тела хунка
            current_raw.push_str(line);
            current_raw.push('\n');
        }
    }
    if !current_header.is_empty() {
        push_hunk(
            &mut hunks,
            &current_header,
            &mut current_lines,
            &mut current_raw,
        );
    }
    FileDiff {
        path,
        hunks,
        insertions,
        deletions,
        header: patch_header,
        binary,
        old_image: None,
        new_image: None,
        byte_size: None,
    }
}

fn parse_diff_multi(diff_text: &str) -> Vec<FileDiff> {
    let mut diffs = Vec::new();
    let mut current_chunk = String::new();
    let mut current_path = String::new();
    for line in diff_text.lines() {
        if line.starts_with("diff --git") {
            if !current_chunk.is_empty() {
                diffs.push(parse_diff_single(&current_chunk, &current_path));
            }
            current_chunk = String::new();
            current_path = line.split(" b/").last().unwrap_or("").to_string();
        }
        current_chunk.push_str(line);
        current_chunk.push('\n');
    }
    if !current_chunk.is_empty() {
        diffs.push(parse_diff_single(&current_chunk, &current_path));
    }
    diffs
}

pub fn repo_stats(repo_path: &Path, since_days: Option<u32>) -> Result<RepoStats, GitError> {
    let since_arg;
    let mut args = vec![
        "log",
        "--format=COMMIT%x01%ae%x01%an%x01%ad",
        "--date=format:%Y-%m-%d %H %w",
        "--numstat",
    ];
    if let Some(days) = since_days {
        since_arg = format!("--after={} days ago", days);
        args.push(&since_arg);
    }

    let output = run_git(repo_path, &args)?;

    let mut author_map: HashMap<String, AuthorStat> = HashMap::new();
    let mut author_days: HashMap<String, HashSet<String>> = HashMap::new();
    let mut global_days: HashSet<String> = HashSet::new();
    let mut by_weekday = vec![0u32; 7];
    let mut by_hour = vec![0u32; 24];
    let mut by_month: HashMap<String, u32> = HashMap::new();
    let mut day_counts: HashMap<String, u32> = HashMap::new();

    let mut cur_email = String::new();

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("COMMIT\x01") {
            let parts: Vec<&str> = rest.splitn(3, '\x01').collect();
            if parts.len() < 3 {
                continue;
            }
            let email = parts[0];
            let name = parts[1];
            let date_str = parts[2]; // "YYYY-MM-DD HH W"

            let dparts: Vec<&str> = date_str.split(' ').collect();
            if dparts.len() < 3 {
                continue;
            }
            let ymd = dparts[0];
            let hour: usize = dparts[1].parse::<usize>().unwrap_or(0).min(23);
            let wday: usize = dparts[2].parse::<usize>().unwrap_or(0);
            // git %w: 0=Sun → index 6, 1=Mon → 0, ..., 6=Sat → 5
            let wday_idx = (wday + 6) % 7;

            global_days.insert(ymd.to_string());
            by_weekday[wday_idx] += 1;
            by_hour[hour] += 1;
            if ymd.len() >= 7 {
                *by_month.entry(ymd[..7].to_string()).or_default() += 1;
            }
            *day_counts.entry(ymd.to_string()).or_default() += 1;

            cur_email = email.to_string();
            let entry = author_map
                .entry(email.to_string())
                .or_insert_with(|| AuthorStat {
                    name: name.to_string(),
                    email: email.to_string(),
                    commits: 0,
                    insertions: 0,
                    deletions: 0,
                    active_days: 0,
                    first_date: ymd.to_string(),
                    last_date: ymd.to_string(),
                });
            entry.commits += 1;
            entry.name = name.to_string();
            if ymd < entry.first_date.as_str() {
                entry.first_date = ymd.to_string();
            }
            if ymd > entry.last_date.as_str() {
                entry.last_date = ymd.to_string();
            }

            author_days
                .entry(email.to_string())
                .or_default()
                .insert(ymd.to_string());
        } else {
            // numstat line: "<ins>\t<del>\t<file>", binary shows "-\t-\t..."
            let parts: Vec<&str> = line.splitn(3, '\t').collect();
            if parts.len() >= 2 {
                if let (Ok(ins), Ok(del)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    if let Some(author) = author_map.get_mut(&cur_email) {
                        author.insertions += ins;
                        author.deletions += del;
                    }
                }
            }
        }
    }

    for (email, days) in &author_days {
        if let Some(author) = author_map.get_mut(email) {
            author.active_days = days.len() as u32;
        }
    }

    let mut authors: Vec<AuthorStat> = author_map.into_values().collect();
    authors.sort_by_key(|a| std::cmp::Reverse(a.commits));

    let mut months: Vec<MonthEntry> = by_month
        .into_iter()
        .map(|(month, commits)| MonthEntry { month, commits })
        .collect();
    months.sort_by(|a, b| a.month.cmp(&b.month));

    let mut by_day: Vec<DayEntry> = day_counts
        .into_iter()
        .map(|(date, count)| DayEntry { date, count })
        .collect();
    by_day.sort_by(|a, b| a.date.cmp(&b.date));

    let total_commits: u32 = authors.iter().map(|a| a.commits).sum();
    let total_insertions: u32 = authors.iter().map(|a| a.insertions).sum();
    let total_deletions: u32 = authors.iter().map(|a| a.deletions).sum();
    let first_commit_date = authors
        .iter()
        .map(|a| a.first_date.as_str())
        .min()
        .unwrap_or("")
        .to_string();
    let last_commit_date = authors
        .iter()
        .map(|a| a.last_date.as_str())
        .max()
        .unwrap_or("")
        .to_string();
    let total_authors = authors.len() as u32;

    Ok(RepoStats {
        total_commits,
        total_insertions,
        total_deletions,
        first_commit_date,
        last_commit_date,
        active_days: global_days.len() as u32,
        total_authors,
        authors,
        by_weekday,
        by_hour,
        by_month: months,
        by_day,
    })
}

#[cfg(test)]
mod ref_label_tests {
    use super::*;

    fn kinds(raw: &str) -> Vec<(String, String)> {
        kinds_with_remotes(raw, &["origin".to_string()])
    }

    fn kinds_with_remotes(raw: &str, remotes: &[String]) -> Vec<(String, String)> {
        parse_ref_labels(raw, remotes)
            .into_iter()
            .map(|r| (r.name, r.kind))
            .collect()
    }

    #[test]
    fn current_branch_from_head_arrow() {
        assert_eq!(
            kinds("HEAD -> main"),
            vec![("main".to_string(), "current-branch".to_string())]
        );
    }

    #[test]
    fn standalone_head_is_head() {
        assert_eq!(
            kinds("HEAD"),
            vec![("HEAD".to_string(), "head".to_string())]
        );
    }

    #[test]
    fn tag_remote_local_kinds() {
        assert_eq!(
            kinds("tag: v1.0"),
            vec![("v1.0".to_string(), "tag".to_string())]
        );
        assert_eq!(
            kinds("origin/main"),
            vec![("origin/main".to_string(), "remote-branch".to_string())]
        );
        assert_eq!(
            kinds("dev"),
            vec![("dev".to_string(), "local-branch".to_string())]
        );
    }

    #[test]
    fn combined_decoration() {
        assert_eq!(
            kinds("HEAD -> main, tag: v1, origin/main"),
            vec![
                ("main".to_string(), "current-branch".to_string()),
                ("v1".to_string(), "tag".to_string()),
                ("origin/main".to_string(), "remote-branch".to_string()),
            ]
        );
    }

    // Локальная ветка со слэшем (`feature/auth`) — НЕ remote, даже если в
    // имени есть `/`. Прежняя эвристика `contains('/')` ломала классификацию.
    #[test]
    fn local_branch_with_slash_is_not_remote() {
        assert_eq!(
            kinds("feature/auth"),
            vec![("feature/auth".to_string(), "local-branch".to_string())]
        );
    }

    // Symbolic ref `origin/HEAD` не рисуется отдельной пилюлей (дубликат
    // дефолтной ветки), иначе граф захламляется.
    #[test]
    fn remote_head_symbolic_ref_filtered() {
        assert_eq!(kinds("origin/HEAD"), Vec::<(String, String)>::new());
        assert_eq!(
            kinds("origin/main, origin/HEAD"),
            vec![("origin/main".to_string(), "remote-branch".to_string())]
        );
    }

    // Несколько remote'ов: ветка распознаётся как remote, только если её
    // первый сегмент совпадает с настоящим именем remote'а.
    #[test]
    fn multi_remote_distinguishes_local_and_remote() {
        let remotes = vec!["origin".to_string(), "upstream".to_string()];
        assert_eq!(
            kinds_with_remotes("upstream/dev", &remotes),
            vec![("upstream/dev".to_string(), "remote-branch".to_string())]
        );
        // Локальная ветка `feature/upstream` — не remote (нет remote'а
        // с именем "feature").
        assert_eq!(
            kinds_with_remotes("feature/upstream", &remotes),
            vec![("feature/upstream".to_string(), "local-branch".to_string())]
        );
    }
}

#[cfg(test)]
mod diff_untracked_tests {
    use super::*;
    use std::fs;

    fn git(dir: &Path, args: &[&str]) {
        Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
    }

    #[test]
    fn untracked_file_diff_shows_content() {
        let dir = std::env::temp_dir().join(format!("gitstream_untracked_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        fs::write(dir.join("new.txt"), "alpha\nbeta\ngamma\n").unwrap();

        let diff = diff_file(&dir, "new.txt", false, None).unwrap();

        let added: Vec<&str> = diff
            .hunks
            .iter()
            .flat_map(|h| h.lines.iter())
            .filter(|l| l.kind == "added")
            .map(|l| l.content.as_str())
            .collect();
        assert_eq!(added, vec!["alpha", "beta", "gamma"]);
        assert_eq!(diff.path, "new.txt");

        let _ = fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod status_untracked_tests {
    use super::*;
    use std::fs;

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    // git status по умолчанию схлопывает untracked-каталог в одну запись
    // (`back/ws_server/`). UI должен показывать каждый файл отдельно — как
    // SmartGit. Поэтому status() запрашивает --untracked-files=all.
    #[test]
    fn untracked_directory_lists_individual_files() {
        let dir = std::env::temp_dir()
            .join(format!("gitstream_status_untracked_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "t@t.t"]);
        git(&dir, &["config", "user.name", "t"]);
        fs::write(dir.join("seed.txt"), "seed\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "seed"]);

        let nested = dir.join("back").join("ws_server").join("src");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("server.rs"), "fn main() {}\n").unwrap();
        fs::write(nested.join("metrics.rs"), "pub fn m() {}\n").unwrap();

        let files = status(&dir).unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();

        assert!(
            paths.contains(&"back/ws_server/src/server.rs"),
            "ожидался отдельный файл, получено: {:?}",
            paths
        );
        assert!(
            paths.contains(&"back/ws_server/src/metrics.rs"),
            "ожидался отдельный файл, получено: {:?}",
            paths
        );
        assert!(
            files.iter().all(|f| !f.path.ends_with('/')),
            "каталог не должен попадать в список одной записью: {:?}",
            paths
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // git status по умолчанию (core.quotePath=true) экранирует не-ASCII пути
    // октальными escape'ами ("\320\277..."), и они не совпадают с чистым UTF-8
    // из `ls-files -z` — ломается список файлов и дерево папок в репозиториях
    // с кириллицей. status() передаёт core.quotePath=false → путь как есть.
    #[test]
    fn modified_cyrillic_path_is_verbatim_utf8() {
        let dir = std::env::temp_dir()
            .join(format!("gitstream_status_cyr_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "t@t.t"]);
        git(&dir, &["config", "user.name", "t"]);

        let sub = dir.join("папка").join("вложенная");
        fs::create_dir_all(&sub).unwrap();
        let file = sub.join("файл.txt");
        fs::write(&file, "1\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "seed"]);
        fs::write(&file, "1\n2\n").unwrap();

        let files = status(&dir).unwrap();
        let paths: Vec<&str> = files.iter().map(|f| f.path.as_str()).collect();

        assert!(
            paths.contains(&"папка/вложенная/файл.txt"),
            "ожидался чистый UTF-8 путь, получено: {:?}",
            paths
        );
        assert!(
            files.iter().all(|f| !f.path.contains('\\')),
            "путь не должен содержать escape'ов: {:?}",
            paths
        );

        let _ = fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod diff_commit_tests {
    use super::*;
    use std::fs;

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn temp_repo(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("gitstream_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "t@t.t"]);
        git(&dir, &["config", "user.name", "t"]);
        fs::write(dir.join("seed.txt"), "seed\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "seed"]);
        dir
    }

    fn head(dir: &Path) -> String {
        run_git(dir, &["rev-parse", "HEAD"]).unwrap().trim().to_string()
    }

    // Имя файла на кириллице не должно превращаться в мусор: git c
    // core.quotepath=false отдаёт путь дословно, парсер берёт его как есть.
    #[test]
    fn cyrillic_filename_parsed_verbatim() {
        let dir = temp_repo("cyr");
        fs::write(dir.join("файл.txt"), "привет\nмир\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "cyr"]);

        let diffs = diff_commit(&dir, &head(&dir)).unwrap();
        let d = diffs
            .iter()
            .find(|d| d.path == "файл.txt")
            .expect("кириллический путь должен распознаться дословно");
        assert_eq!(d.insertions, 2);

        let _ = fs::remove_dir_all(&dir);
    }

    // Имя с пробелом: git добавляет хвостовой TAB в строку `+++`, парсер
    // обязан его срезать.
    #[test]
    fn spaced_filename_trims_trailing_tab() {
        let dir = temp_repo("space");
        fs::write(dir.join("new file.txt"), "a\nb\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "spaced"]);

        let d = diff_commit_file(&dir, &head(&dir), "new file.txt", None).unwrap();
        assert_eq!(d.path, "new file.txt");
        assert!(!d.hunks.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    // Бинарный файл флагуется и получает размер.
    #[test]
    fn binary_file_flagged_with_size() {
        let dir = temp_repo("bin");
        fs::write(dir.join("blob.bin"), [0u8, 1, 2, 0, 255, 3]).unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "bin"]);

        let d = diff_commit_file(&dir, &head(&dir), "blob.bin", None).unwrap();
        assert!(d.binary, "бинарный файл должен быть помечен");
        assert_eq!(d.byte_size, Some(6));
        assert!(d.hunks.is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    // Перенос файла: обычный дифф коммита пуст (rename без хунков), а
    // diff_commit_file показывает содержимое — пайспек `-- <file>` не даёт
    // git'у определить переименование, файл выводится как добавление.
    #[test]
    fn renamed_file_shows_content() {
        let dir = temp_repo("ren");
        fs::create_dir_all(dir.join("old")).unwrap();
        fs::write(dir.join("old/m.rs"), "fn a() {}\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "add"]);
        git(&dir, &["mv", "old", "new"]);
        git(&dir, &["commit", "-qm", "move"]);

        let oid = head(&dir);
        let listed = diff_commit(&dir, &oid)
            .unwrap()
            .into_iter()
            .find(|d| d.path == "new/m.rs")
            .expect("переименованный файл присутствует в списке");
        assert!(listed.hunks.is_empty(), "rename в `git diff` без хунков");

        // diff_commit_file ограничен пайспеком `-- new/m.rs`: git не видит
        // удалённый old/m.rs и показывает файл как добавление с содержимым.
        let renamed = diff_commit_file(&dir, &oid, "new/m.rs", None).unwrap();
        assert!(
            !renamed.hunks.is_empty(),
            "diff_commit_file показывает содержимое переименованного файла"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // Корневой коммит (первый, без parent): `git diff oid^..oid` падает —
    // надо использовать `git show`, иначе FileList пуст на свежем репозитории
    // с единственным коммитом.
    #[test]
    fn root_commit_files_listed() {
        let dir = std::env::temp_dir().join(format!("gitstream_root_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "t@t.t"]);
        git(&dir, &["config", "user.name", "t"]);
        fs::write(dir.join("a.txt"), "first\n").unwrap();
        fs::write(dir.join("b.txt"), "second\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "initial"]);

        let oid = head(&dir);
        let diffs = diff_commit(&dir, &oid)
            .expect("корневой коммит должен отдавать список файлов, а не ошибку");
        let paths: Vec<_> = diffs.iter().map(|d| d.path.as_str()).collect();
        assert!(paths.contains(&"a.txt"), "a.txt должен быть в списке: {:?}", paths);
        assert!(paths.contains(&"b.txt"), "b.txt должен быть в списке: {:?}", paths);

        let _ = fs::remove_dir_all(&dir);
    }

    // Merge-коммит: diff показывается против первого родителя (как раньше).
    // Регрессия: `git show --format=` отдавал бы пустой combined-diff,
    // и панель файлов сливалась бы для всех merge-коммитов без конфликтов.
    #[test]
    fn merge_commit_diffs_first_parent() {
        let dir = std::env::temp_dir().join(format!("gitstream_merge_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        git(&dir, &["config", "user.email", "t@t.t"]);
        git(&dir, &["config", "user.name", "t"]);
        fs::write(dir.join("a.txt"), "a\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "initial"]);
        git(&dir, &["checkout", "-qb", "feature"]);
        fs::write(dir.join("b.txt"), "b\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "feat"]);
        git(&dir, &["checkout", "-q", "main"]);
        fs::write(dir.join("a.txt"), "c\n").unwrap();
        git(&dir, &["commit", "-qam", "c"]);
        git(&dir, &["merge", "--no-ff", "-q", "feature", "-m", "merge"]);

        let merge_oid = head(&dir);
        let diffs = diff_commit(&dir, &merge_oid).expect("merge не должен падать");
        let paths: Vec<_> = diffs.iter().map(|d| d.path.as_str()).collect();
        // Diff vs первого родителя (main с a=c): только добавление b.txt.
        assert_eq!(paths, vec!["b.txt"], "ожидаем diff vs first parent, получили {:?}", paths);

        // diff_commit_file на merge-коммите должен показывать содержимое файла
        // (diff vs первого родителя), а не пустой combined diff.
        // Регрессия: git show отдавал бы пустой combined-diff для чистых слияний.
        let file_diff = diff_commit_file(&dir, &merge_oid, "b.txt", None)
            .expect("diff_commit_file для merge не должен падать");
        assert!(
            !file_diff.hunks.is_empty(),
            "diff_commit_file на merge-коммите должен показывать хунки, а не пустой combined diff"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // Пустой репозиторий (`git init` без коммитов): `log` отдаёт пустой
    // список, а не ошибку — иначе фронтенд не очистит граф при переключении.
    #[test]
    fn log_empty_repo_returns_empty_list() {
        let dir = std::env::temp_dir().join(format!("gitstream_empty_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);

        let commits = log(&dir, 500).expect("пустой репозиторий не должен давать ошибку");
        assert!(commits.is_empty(), "лог пустого репозитория должен быть пуст");

        let _ = fs::remove_dir_all(&dir);
    }
}

// Граничные состояния репозитория: пустой (без коммитов), detached HEAD, без
// remote. Read-only запросы обязаны отдавать вменяемый результат, а не падать.
#[cfg(test)]
mod edge_case_tests {
    use super::*;
    use std::fs;

    fn git(dir: &Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr)
        );
    }

    fn empty_repo(tag: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("gitstream_edge_{}_{}", tag, std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        git(&dir, &["init", "-q"]);
        git(&dir, &["config", "user.email", "t@t.t"]);
        git(&dir, &["config", "user.name", "t"]);
        dir
    }

    fn with_commits(tag: &str) -> std::path::PathBuf {
        let dir = empty_repo(tag);
        fs::write(dir.join("a.txt"), "1\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "c1"]);
        fs::write(dir.join("a.txt"), "2\n").unwrap();
        git(&dir, &["commit", "-aqm", "c2"]);
        dir
    }

    // Пустой репозиторий: все основные запросы — Ok без паники.
    #[test]
    fn empty_repo_queries_are_graceful() {
        let dir = empty_repo("empty_all");

        assert!(status(&dir).unwrap().is_empty(), "status пуст");
        assert!(log(&dir, 100).unwrap().is_empty(), "log пуст");
        assert!(branches(&dir).unwrap().is_empty(), "веток ещё нет");
        assert!(tags(&dir).unwrap().is_empty(), "тегов нет");
        assert!(stashes(&dir).unwrap().is_empty(), "stash нет");
        assert!(remotes(&dir).unwrap().is_empty(), "remote нет");
        assert_eq!(repo_state(&dir).unwrap(), "clean");

        // repo_info не должен падать: HEAD ещё не существует (unborn-ветка),
        // head_oid пуст, но ветка по умолчанию уже выбрана.
        let info = repo_info(&dir).unwrap();
        assert!(info.head_oid.is_empty(), "у unborn-ветки нет HEAD-oid");
        assert!(!info.current_branch.is_empty(), "имя unborn-ветки доступно");

        let _ = fs::remove_dir_all(&dir);
    }

    // Пустой репозиторий со staged-файлом: status его видит.
    #[test]
    fn empty_repo_with_staged_file_shows_in_status() {
        let dir = empty_repo("empty_staged");
        fs::write(dir.join("new.txt"), "x\n").unwrap();
        git(&dir, &["add", "."]);

        let st = status(&dir).unwrap();
        assert!(
            st.iter().any(|f| f.path == "new.txt"),
            "staged-файл в unborn-ветке должен попасть в status"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    // Detached HEAD: current_branch пуст, но лог и состояние читаются.
    #[test]
    fn detached_head_is_handled() {
        let dir = with_commits("detached");
        let first = run_git(&dir, &["rev-list", "--max-parents=0", "HEAD"])
            .unwrap()
            .trim()
            .to_string();
        git(&dir, &["checkout", "-q", &first]);

        let info = repo_info(&dir).unwrap();
        assert!(
            info.current_branch.is_empty(),
            "в detached HEAD имени текущей ветки нет"
        );
        assert_eq!(info.head_oid, first, "HEAD указывает на выбранный коммит");
        assert!(!log(&dir, 100).unwrap().is_empty(), "лог доступен в detached HEAD");
        assert_eq!(repo_state(&dir).unwrap(), "clean");

        let _ = fs::remove_dir_all(&dir);
    }

    // file_log: история одного файла, --follow находит её и до переименования.
    #[test]
    fn file_log_follows_rename() {
        let dir = empty_repo("file_log");
        fs::write(dir.join("old.txt"), "a\n").unwrap();
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-qm", "add old"]);
        git(&dir, &["mv", "old.txt", "new.txt"]);
        git(&dir, &["commit", "-qm", "rename to new"]);
        fs::write(dir.join("new.txt"), "a\nb\n").unwrap();
        git(&dir, &["commit", "-aqm", "edit new"]);

        // По новому имени с --follow видны все три коммита (включая до rename).
        let commits = file_log(&dir, "new.txt", 100).unwrap();
        assert_eq!(commits.len(), 3, "--follow должен пройти через переименование");
        assert_eq!(commits[0].message, "edit new", "новейший — первым");

        // Несуществующий файл → пустая история, не ошибка.
        assert!(file_log(&dir, "ghost.txt", 100).unwrap().is_empty());

        let _ = fs::remove_dir_all(&dir);
    }

    // Репозиторий без remote: remotes пуст, всё незапушено (лог это отражает).
    #[test]
    fn repo_without_remote_marks_commits_unpushed() {
        let dir = with_commits("no_remote");

        assert!(remotes(&dir).unwrap().is_empty(), "remote не настроен");
        let commits = log(&dir, 100).unwrap();
        assert!(!commits.is_empty());
        assert!(
            commits.iter().all(|c| c.unpushed),
            "без remote все коммиты считаются незапушенными"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}

#[cfg(test)]
mod blame_parser_tests {
    use super::*;

    // Порционный --porcelain: метаданные коммита идут раз, повтор того же sha —
    // только заголовок + контент. Парсер обязан брать автора/время из кэша.
    #[test]
    fn parses_porcelain_with_cached_repeats() {
        let a = "a".repeat(40);
        let b = "b".repeat(40);
        let out = format!(
            "{a} 1 1 1\n\
author Alice\n\
author-mail <a@a>\n\
author-time 1700000000\n\
author-tz +0000\n\
summary first\n\
filename f.txt\n\
\tline one\n\
{b} 2 2 1\n\
author Bob\n\
author-mail <b@b>\n\
author-time 1700000100\n\
author-tz +0000\n\
summary second\n\
filename f.txt\n\
\tline two\n\
{a} 1 3 1\n\
\tline three (same commit as 1)\n",
            a = a,
            b = b
        );

        let lines = parse_blame_porcelain(&out);
        assert_eq!(lines.len(), 3);

        assert_eq!(lines[0].author, "Alice");
        assert_eq!(lines[0].author_time, 1700000000);
        assert_eq!(lines[0].line_no, 1);
        assert_eq!(lines[0].content, "line one");
        assert_eq!(lines[0].short_oid, "aaaaaaaa");
        assert_eq!(lines[0].summary, "first");

        assert_eq!(lines[1].author, "Bob");
        assert_eq!(lines[1].author_time, 1700000100);
        assert_eq!(lines[1].line_no, 2);
        assert_eq!(lines[1].content, "line two");

        // Третья строка — тот же коммит, что и первая: метаданные из кэша.
        assert_eq!(lines[2].oid, a);
        assert_eq!(lines[2].author, "Alice");
        assert_eq!(lines[2].author_time, 1700000000);
        assert_eq!(lines[2].line_no, 3);
        assert_eq!(lines[2].content, "line three (same commit as 1)");
    }

    #[test]
    fn empty_output_yields_no_lines() {
        assert!(parse_blame_porcelain("").is_empty());
    }
}
