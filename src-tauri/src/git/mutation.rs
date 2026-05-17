use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use super::error::{classify_git_error, GitError};

#[derive(serde::Deserialize)]
pub struct LineHunkSelection {
    pub raw: String,
    pub selected: Vec<usize>,
}

#[derive(serde::Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LineOp {
    Stage,
    Unstage,
    Discard,
}

fn run_git_mut(repo_path: &Path, args: &[&str]) -> Result<String, GitError> {
    super::query::run_git(repo_path, args)
}

/// Передаёт `patch` в stdin `git apply`. `reverse` — откат, `cached` — в индекс.
fn apply_patch(repo_path: &Path, patch: &str, reverse: bool, cached: bool) -> Result<(), GitError> {
    let mut args: Vec<&str> = vec!["apply", "--recount", "--whitespace=nowarn"];
    if cached {
        args.push("--cached");
    }
    if reverse {
        args.push("--reverse");
    }
    args.push("-");

    let mut child = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .args(&args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| GitError::CommandFailed {
            message: format!("Failed to run git apply: {}", e),
            hint: Some("Is git installed and in PATH?".into()),
        })?;

    {
        let stdin = child.stdin.as_mut().ok_or_else(|| GitError::CommandFailed {
            message: "Failed to open git apply stdin".into(),
            hint: None,
        })?;
        let mut data = patch.to_string();
        if !data.ends_with('\n') {
            data.push('\n');
        }
        stdin
            .write_all(data.as_bytes())
            .map_err(|e| GitError::CommandFailed {
                message: format!("Failed to write patch: {}", e),
                hint: None,
            })?;
    }

    let output = child.wait_with_output().map_err(|e| GitError::CommandFailed {
        message: format!("git apply failed: {}", e),
        hint: None,
    })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(classify_git_error(&String::from_utf8_lossy(&output.stderr)))
    }
}

/// stage выбранного хунка: патч из unstaged-диффа → индекс.
pub fn stage_hunk(repo_path: &Path, patch: &str) -> Result<(), GitError> {
    apply_patch(repo_path, patch, false, true)
}

/// unstage выбранного хунка: патч из staged-диффа → откат в индексе.
pub fn unstage_hunk(repo_path: &Path, patch: &str) -> Result<(), GitError> {
    apply_patch(repo_path, patch, true, true)
}

/// discard выбранного хунка: патч из unstaged-диффа → откат в рабочем дереве.
pub fn discard_hunk(repo_path: &Path, patch: &str) -> Result<(), GitError> {
    apply_patch(repo_path, patch, true, false)
}

pub fn stage(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    let mut args = vec!["add", "--"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn unstage(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    let mut args = vec!["restore", "--staged", "--"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn discard(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    let mut args = vec!["restore", "--"];
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    args.extend(file_refs);
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn commit(repo_path: &Path, message: &str, amend: bool) -> Result<String, GitError> {
    let mut args = vec!["commit", "-m", message];
    if amend { args.push("--amend"); }
    run_git_mut(repo_path, &args)
}

pub fn checkout(repo_path: &Path, branch: &str) -> Result<(), GitError> {
    let result = run_git_mut(repo_path, &["switch", branch]);
    if result.is_ok() { return Ok(()); }
    if let Some(local) = branch.split('/').last() {
        run_git_mut(repo_path, &["switch", "-c", local, branch])?;
        return Ok(());
    }
    result.map(|_| ())
}

pub fn checkout_remote(repo_path: &Path, remote_branch: &str, local_name: Option<&str>) -> Result<(), GitError> {
    match local_name {
        Some(local) => {
            run_git_mut(repo_path, &["switch", "-c", local, "--track", remote_branch])?;
        }
        None => {
            run_git_mut(repo_path, &["checkout", "--detach", remote_branch])?;
        }
    }
    Ok(())
}

pub fn merge(repo_path: &Path, branch: &str) -> Result<String, GitError> {
    run_git_mut(repo_path, &["merge", branch])
}

/// Перебазировать текущую ветку на `onto`. Конфликты разрешаются через ConflictBar.
pub fn rebase(repo_path: &Path, onto: &str) -> Result<String, GitError> {
    run_git_mut(repo_path, &["-c", "core.editor=true", "rebase", onto])
}

/// Разрешить конфликт в файлах в пользу нашей (`--ours`) версии и пометить resolved.
pub fn accept_ours(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    resolve_side(repo_path, files, "--ours")
}

/// Разрешить конфликт в файлах в пользу их (`--theirs`) версии и пометить resolved.
pub fn accept_theirs(repo_path: &Path, files: &[String]) -> Result<(), GitError> {
    resolve_side(repo_path, files, "--theirs")
}

fn resolve_side(repo_path: &Path, files: &[String], side: &str) -> Result<(), GitError> {
    let file_refs: Vec<&str> = files.iter().map(|s| s.as_str()).collect();
    let mut co: Vec<&str> = vec!["checkout", side, "--"];
    co.extend(&file_refs);
    run_git_mut(repo_path, &co)?;
    let mut add: Vec<&str> = vec!["add", "--"];
    add.extend(&file_refs);
    run_git_mut(repo_path, &add)?;
    Ok(())
}

/// Прервать незавершённую операцию по состоянию репозитория.
pub fn op_abort(repo_path: &Path, state: &str) -> Result<(), GitError> {
    let args: &[&str] = match state {
        "rebasing" => &["rebase", "--abort"],
        "cherry-picking" => &["cherry-pick", "--abort"],
        "reverting" => &["revert", "--abort"],
        _ => &["merge", "--abort"],
    };
    run_git_mut(repo_path, args)?;
    Ok(())
}

/// Продолжить незавершённую операцию (после разрешения конфликтов).
pub fn op_continue(repo_path: &Path, state: &str) -> Result<(), GitError> {
    // GIT_EDITOR=true — не открываем редактор для сообщения.
    let args: Vec<&str> = match state {
        "rebasing" => vec!["-c", "core.editor=true", "rebase", "--continue"],
        "cherry-picking" => vec!["-c", "core.editor=true", "cherry-pick", "--continue"],
        "reverting" => vec!["-c", "core.editor=true", "revert", "--continue"],
        _ => vec!["commit", "--no-edit"],
    };
    run_git_mut(repo_path, &args)?;
    Ok(())
}

/// `mode`: "soft" | "mixed" | "hard".
pub fn reset(repo_path: &Path, oid: &str, mode: &str) -> Result<(), GitError> {
    let flag = match mode {
        "soft" => "--soft",
        "hard" => "--hard",
        _ => "--mixed",
    };
    run_git_mut(repo_path, &["reset", flag, oid])?;
    Ok(())
}

pub fn revert(repo_path: &Path, oid: &str, no_commit: bool) -> Result<(), GitError> {
    let mut args: Vec<&str> = vec!["revert", "--no-edit"];
    if no_commit {
        args.push("--no-commit");
    }
    args.push(oid);
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn cherry_pick(repo_path: &Path, oid: &str) -> Result<(), GitError> {
    run_git_mut(repo_path, &["cherry-pick", oid])?;
    Ok(())
}

fn stash_ref(index: usize) -> String {
    format!("stash@{{{}}}", index)
}

pub fn stash_save(
    repo_path: &Path,
    message: Option<&str>,
    include_untracked: bool,
) -> Result<(), GitError> {
    let mut args: Vec<&str> = vec!["stash", "push"];
    if include_untracked {
        args.push("--include-untracked");
    }
    if let Some(msg) = message {
        if !msg.is_empty() {
            args.push("-m");
            args.push(msg);
        }
    }
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn stash_apply(repo_path: &Path, index: usize) -> Result<(), GitError> {
    run_git_mut(repo_path, &["stash", "apply", &stash_ref(index)])?;
    Ok(())
}

pub fn stash_pop(repo_path: &Path, index: usize) -> Result<(), GitError> {
    run_git_mut(repo_path, &["stash", "pop", &stash_ref(index)])?;
    Ok(())
}

pub fn stash_drop(repo_path: &Path, index: usize) -> Result<(), GitError> {
    run_git_mut(repo_path, &["stash", "drop", &stash_ref(index)])?;
    Ok(())
}

pub fn create_branch(
    repo_path: &Path,
    name: &str,
    start_point: Option<&str>,
    checkout: bool,
) -> Result<(), GitError> {
    let mut args: Vec<&str> = if checkout {
        vec!["switch", "-c", name]
    } else {
        vec!["branch", name]
    };
    if let Some(sp) = start_point {
        if !sp.is_empty() {
            args.push(sp);
        }
    }
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn rename_branch(repo_path: &Path, old_name: &str, new_name: &str) -> Result<(), GitError> {
    run_git_mut(repo_path, &["branch", "-m", old_name, new_name])?;
    Ok(())
}

pub fn delete_branch(repo_path: &Path, branch: &str, force: bool) -> Result<(), GitError> {
    let flag = if force { "-D" } else { "-d" };
    run_git_mut(repo_path, &["branch", flag, branch])?;
    Ok(())
}

pub fn create_tag(
    repo_path: &Path,
    name: &str,
    message: Option<&str>,
    target: Option<&str>,
    force: bool,
) -> Result<(), GitError> {
    let mut args: Vec<&str> = vec!["tag"];
    if force {
        args.push("-f");
    }
    if let Some(msg) = message {
        args.push("-a");
        args.push("-m");
        args.push(msg);
    }
    args.push(name);
    if let Some(t) = target {
        args.push(t);
    }
    run_git_mut(repo_path, &args)?;
    Ok(())
}

pub fn delete_tag(repo_path: &Path, name: &str) -> Result<(), GitError> {
    run_git_mut(repo_path, &["tag", "-d", name])?;
    Ok(())
}

pub fn fetch_args(remote: &str) -> Vec<String> {
    vec!["fetch".into(), remote.into()]
}

pub fn pull_args(remote: &str, rebase: bool) -> Vec<String> {
    if rebase {
        vec!["pull".into(), "--rebase".into(), remote.into()]
    } else {
        vec!["pull".into(), remote.into()]
    }
}

pub fn push_args(remote: &str, force: bool) -> Vec<String> {
    if force {
        vec!["push".into(), "--force".into(), remote.into()]
    } else {
        vec!["push".into(), remote.into()]
    }
}

pub fn push_branch_args(remote: &str, branch: &str, force: bool) -> Vec<String> {
    if force {
        vec!["push".into(), "--force".into(), remote.into(), branch.into()]
    } else {
        vec!["push".into(), remote.into(), branch.into()]
    }
}

pub fn push_tag_args(remote: &str, name: &str, delete: bool) -> Vec<String> {
    let refspec = if delete {
        format!(":refs/tags/{}", name)
    } else {
        format!("refs/tags/{}", name)
    };
    vec!["push".into(), remote.into(), refspec]
}

pub fn clone_args(url: &str, dest: &str) -> Vec<String> {
    vec!["clone".into(), url.into(), dest.into()]
}

/// Разбирает заголовок хунка "@@ -a,b +c,d @@ tail".
/// Возвращает (old_start, new_start, tail) — где tail включает ведущий пробел.
fn parse_hunk_header(h: &str) -> Option<(usize, usize, String)> {
    let rest = h.strip_prefix("@@ ")?;
    let close = rest.find(" @@")?;
    let ranges = &rest[..close];
    let tail = &rest[close + 3..];
    let mut it = ranges.split(' ');
    let old = it.next()?;
    let new = it.next()?;
    let old_start: usize = old
        .trim_start_matches('-')
        .split(',')
        .next()?
        .parse()
        .ok()?;
    let new_start: usize = new
        .trim_start_matches('+')
        .split(',')
        .next()?
        .parse()
        .ok()?;
    Some((old_start, new_start, tail.to_string()))
}

/// Пересобирает один хунк, оставляя только выбранные изменённые (+/-) строки.
/// `selected` — 0-based порядковые номера +/- строк тела хунка (в порядке raw).
/// Невыбранные '-' превращаются в контекст, невыбранные '+' выбрасываются.
/// Возвращает None, если в результате не осталось ни одной +/- строки.
fn rebuild_hunk(raw: &str, selected: &[usize]) -> Option<String> {
    let mut lines = raw.split('\n');
    let header = lines.next()?;
    let (old_start, new_start, tail) = parse_hunk_header(header)?;

    let sel: std::collections::HashSet<usize> = selected.iter().copied().collect();
    let mut body = String::new();
    let mut old_count = 0usize;
    let mut new_count = 0usize;
    let mut changed_ord = 0usize;
    let mut kept_any = false;
    let mut last_kept = false;

    for line in lines {
        if line.is_empty() {
            continue; // хвостовой пустой элемент после последнего '\n'
        }
        let tag = line.as_bytes()[0] as char;
        match tag {
            ' ' => {
                body.push_str(line);
                body.push('\n');
                old_count += 1;
                new_count += 1;
                last_kept = true;
            }
            '-' => {
                let is_sel = sel.contains(&changed_ord);
                changed_ord += 1;
                old_count += 1;
                if is_sel {
                    body.push_str(line);
                    body.push('\n');
                    kept_any = true;
                    last_kept = true;
                } else {
                    body.push(' ');
                    body.push_str(&line[1..]);
                    body.push('\n');
                    new_count += 1;
                    last_kept = true;
                }
            }
            '+' => {
                let is_sel = sel.contains(&changed_ord);
                changed_ord += 1;
                if is_sel {
                    body.push_str(line);
                    body.push('\n');
                    new_count += 1;
                    kept_any = true;
                    last_kept = true;
                } else {
                    last_kept = false;
                }
            }
            '\\' => {
                if last_kept {
                    body.push_str(line);
                    body.push('\n');
                }
            }
            _ => {
                // Недостижимо для корректного unified diff (строки тела
                // начинаются с ' '/'+'/'-'/'\\'); счётчики намеренно не трогаем.
                body.push_str(line);
                body.push('\n');
                last_kept = true;
            }
        }
    }

    if !kept_any {
        return None;
    }

    let new_header = format!(
        "@@ -{},{} +{},{} @@{}",
        old_start, old_count, new_start, new_count, tail
    );
    Some(format!("{}\n{}", new_header, body))
}

/// Собирает частичный патч: file_header + непустые пересобранные хунки.
/// None, если ни в одном хунке ничего не выбрано.
fn build_partial_patch(file_header: &str, hunks: &[LineHunkSelection]) -> Option<String> {
    let mut out = String::new();
    let mut any = false;
    for h in hunks {
        if let Some(rebuilt) = rebuild_hunk(&h.raw, &h.selected) {
            out.push_str(&rebuilt);
            any = true;
        }
    }
    if !any {
        return None;
    }
    Some(format!("{}{}", file_header, out))
}

/// Применяет выбранные строки. Трансформация хунка одинакова для всех op —
/// различаются только флаги git apply.
pub fn apply_lines(
    repo_path: &Path,
    file_header: &str,
    hunks: &[LineHunkSelection],
    op: LineOp,
) -> Result<(), GitError> {
    let patch = match build_partial_patch(file_header, hunks) {
        Some(p) => p,
        None => return Ok(()),
    };
    let (reverse, cached) = match op {
        LineOp::Stage => (false, true),
        LineOp::Unstage => (true, true),
        LineOp::Discard => (true, false),
    };
    apply_patch(repo_path, &patch, reverse, cached)
}

#[cfg(test)]
mod tag_tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    fn temp_repo() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gitstream_tag_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            Command::new("git").current_dir(&dir).args(args).output().unwrap();
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t.t"]);
        run(&["config", "user.name", "t"]);
        fs::write(dir.join("a.txt"), "hello").unwrap();
        run(&["add", "."]);
        run(&["commit", "-qm", "init"]);
        dir
    }

    fn list_tags(dir: &std::path::Path) -> String {
        let out = Command::new("git")
            .current_dir(dir)
            .args(["tag", "-l"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    #[test]
    fn creates_lightweight_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        assert!(list_tags(&dir).contains("v1.0"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn creates_annotated_tag_with_message() {
        let dir = temp_repo();
        create_tag(&dir, "v2.0", Some("release two"), None, false).unwrap();
        let out = Command::new("git")
            .current_dir(&dir)
            .args(["cat-file", "-t", "v2.0"])
            .output()
            .unwrap();
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "tag");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn force_overwrites_existing_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        assert!(create_tag(&dir, "v1.0", None, None, false).is_err());
        create_tag(&dir, "v1.0", None, None, true).unwrap();
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn deletes_tag() {
        let dir = temp_repo();
        create_tag(&dir, "v1.0", None, None, false).unwrap();
        delete_tag(&dir, "v1.0").unwrap();
        assert!(!list_tags(&dir).contains("v1.0"));
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn fetch_args_basic() {
        assert_eq!(fetch_args("origin"), vec!["fetch", "origin"]);
    }

    #[test]
    fn pull_args_rebase_toggle() {
        assert_eq!(pull_args("origin", false), vec!["pull", "origin"]);
        assert_eq!(pull_args("origin", true), vec!["pull", "--rebase", "origin"]);
    }

    #[test]
    fn push_args_force_toggle() {
        assert_eq!(push_args("origin", false), vec!["push", "origin"]);
        assert_eq!(push_args("origin", true), vec!["push", "--force", "origin"]);
    }

    #[test]
    fn push_branch_args_force_toggle() {
        assert_eq!(push_branch_args("origin", "main", false), vec!["push", "origin", "main"]);
        assert_eq!(
            push_branch_args("origin", "main", true),
            vec!["push", "--force", "origin", "main"]
        );
    }

    #[test]
    fn push_tag_args_delete_toggle() {
        assert_eq!(
            push_tag_args("origin", "v1.0", false),
            vec!["push", "origin", "refs/tags/v1.0"]
        );
        assert_eq!(
            push_tag_args("origin", "v1.0", true),
            vec!["push", "origin", ":refs/tags/v1.0"]
        );
    }

    #[test]
    fn clone_args_basic() {
        assert_eq!(
            clone_args("https://x/y.git", "/tmp/y"),
            vec!["clone", "https://x/y.git", "/tmp/y"]
        );
    }
}

#[cfg(test)]
mod partial_line_tests {
    use super::*;
    use std::fs;
    use std::process::Command as ProcCommand;

    #[test]
    fn parses_header_with_section_tail() {
        let (os, ns, tail) = parse_hunk_header("@@ -12,7 +12,8 @@ fn foo()").unwrap();
        assert_eq!(os, 12);
        assert_eq!(ns, 12);
        assert_eq!(tail, " fn foo()");
    }

    #[test]
    fn parses_header_without_counts_and_tail() {
        let (os, ns, tail) = parse_hunk_header("@@ -1 +1 @@").unwrap();
        assert_eq!(os, 1);
        assert_eq!(ns, 1);
        assert_eq!(tail, "");
    }

    #[test]
    fn rebuild_keeps_only_selected_added_line() {
        let raw = "@@ -1,1 +1,3 @@\n ctx\n+A\n+B\n";
        let out = rebuild_hunk(raw, &[0]).unwrap();
        assert_eq!(out, "@@ -1,1 +1,2 @@\n ctx\n+A\n");
    }

    #[test]
    fn rebuild_unselected_removal_becomes_context() {
        let raw = "@@ -1,3 +1,1 @@\n ctx\n-X\n-Y\n";
        let out = rebuild_hunk(raw, &[1]).unwrap();
        assert_eq!(out, "@@ -1,3 +1,2 @@\n ctx\n X\n-Y\n");
    }

    #[test]
    fn rebuild_returns_none_when_nothing_selected() {
        let raw = "@@ -1,1 +1,2 @@\n ctx\n+A\n";
        assert!(rebuild_hunk(raw, &[]).is_none());
    }

    #[test]
    fn rebuild_no_newline_marker_follows_kept_line() {
        let raw = "@@ -0,0 +1,1 @@\n+A\n\\ No newline at end of file\n";
        let out = rebuild_hunk(raw, &[0]).unwrap();
        assert_eq!(out, "@@ -0,0 +1,1 @@\n+A\n\\ No newline at end of file\n");
    }

    #[test]
    fn rebuild_no_newline_marker_dropped_with_unselected_line() {
        let raw = "@@ -1,1 +1,2 @@\n ctx\n+A\n\\ No newline at end of file\n";
        assert!(rebuild_hunk(raw, &[]).is_none());
    }

    #[test]
    fn rebuild_no_newline_marker_kept_after_unselected_removal() {
        // -X (ord 0) не выбрана → контекст; +A (ord 1) выбрана → результат Some.
        // Маркер "\ No newline" относится к -X-as-context и должен сохраниться.
        let raw = "@@ -1,2 +1,2 @@\n ctx\n-X\n\\ No newline at end of file\n+A\n";
        let out = rebuild_hunk(raw, &[1]).unwrap();
        // Невыбранное -X стало контекстом → новая сторона выросла до 3 строк.
        assert_eq!(
            out,
            "@@ -1,2 +1,3 @@\n ctx\n X\n\\ No newline at end of file\n+A\n"
        );
    }

    #[test]
    fn rebuild_preserves_header_tail() {
        let raw = "@@ -10,2 +10,3 @@ fn foo()\n ctx\n+A\n ctx2\n";
        let out = rebuild_hunk(raw, &[0]).unwrap();
        assert_eq!(out, "@@ -10,2 +10,3 @@ fn foo()\n ctx\n+A\n ctx2\n");
    }

    fn temp_repo_pl() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "gitstream_pl_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let run = |args: &[&str]| {
            ProcCommand::new("git").current_dir(&dir).args(args).output().unwrap();
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@t.t"]);
        run(&["config", "user.name", "t"]);
        fs::write(dir.join("f.txt"), "l1\nl2\nl3\n").unwrap();
        run(&["add", "."]);
        run(&["commit", "-qm", "init"]);
        dir
    }

    fn staged_content(dir: &std::path::Path) -> String {
        let out = ProcCommand::new("git")
            .current_dir(dir)
            .args(["show", ":f.txt"])
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    #[test]
    fn build_partial_patch_none_when_empty() {
        let hunks = vec![LineHunkSelection {
            raw: "@@ -1,1 +1,2 @@\n l1\n+x\n".to_string(),
            selected: vec![],
        }];
        assert!(build_partial_patch("HEADER\n", &hunks).is_none());
    }

    #[test]
    fn build_partial_patch_prepends_header_and_skips_empty_hunks() {
        let hunks = vec![
            LineHunkSelection {
                raw: "@@ -1,1 +1,2 @@\n l1\n+x\n".to_string(),
                selected: vec![0],
            },
            LineHunkSelection {
                raw: "@@ -5,1 +6,2 @@\n l5\n+y\n".to_string(),
                selected: vec![],
            },
        ];
        let p = build_partial_patch("HDR\n", &hunks).unwrap();
        assert_eq!(p, "HDR\n@@ -1,1 +1,2 @@\n l1\n+x\n");
    }

    #[test]
    fn apply_lines_stages_only_selected_line() {
        let dir = temp_repo_pl();
        fs::write(dir.join("f.txt"), "l1\nl2\nNEW\nl3X\n").unwrap();
        let file_header =
            "diff --git a/f.txt b/f.txt\nindex 0000000..1111111 100644\n--- a/f.txt\n+++ b/f.txt\n";
        let raw = "@@ -1,3 +1,4 @@\n l1\n l2\n+NEW\n-l3\n+l3X\n";
        let hunks = vec![LineHunkSelection {
            raw: raw.to_string(),
            selected: vec![0],
        }];
        apply_lines(&dir, file_header, &hunks, LineOp::Stage).unwrap();
        assert_eq!(staged_content(&dir), "l1\nl2\nNEW\nl3\n");
        fs::remove_dir_all(&dir).ok();
    }
}
