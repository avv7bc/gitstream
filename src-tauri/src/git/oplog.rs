//! Durable before/after snapshots. Worktree blobs include ignored and untracked
//! files; no stash command is used. Private refs keep every referenced object alive.
use super::error::{classify_git_error, GitError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static LOCK: Mutex<()> = Mutex::new(());
const RETAIN: usize = 50;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct Entry {
    path: String,
    mode: String,
    oid: String,
    stage: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepoSnapshot {
    pub head: String,
    pub head_sha: String,
    pub refs: Vec<(String, String)>,
    index: Vec<Entry>,
    files: Vec<Entry>,
    stash_log: Vec<u8>,
    config: Vec<u8>,
    pub index_tree: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OpSnapshot {
    pub id: u64,
    pub at: u64,
    pub kind: String,
    pub label: String,
    pub before: RepoSnapshot,
    pub after: Option<RepoSnapshot>,
    pub undone: bool,
    pub target_id: Option<u64>,
}
#[derive(Serialize)]
pub struct UndoPreview {
    pub token: String,
    pub changed_files: Vec<String>,
    pub needs_confirmation: bool,
}
fn error(message: impl Into<String>) -> GitError {
    GitError::CommandFailed {
        message: message.into(),
        hint: None,
    }
}
fn io_error(e: impl std::fmt::Display) -> GitError {
    error(format!("Operation snapshot: {e}"))
}
fn git(
    repo: &Path,
    args: &[&str],
    input: Option<&[u8]>,
    index: Option<&Path>,
) -> Result<Vec<u8>, GitError> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(repo)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd.env("GIT_AUTHOR_NAME", "GitStream")
        .env("GIT_AUTHOR_EMAIL", "snapshot@gitstream.local")
        .env("GIT_COMMITTER_NAME", "GitStream")
        .env("GIT_COMMITTER_EMAIL", "snapshot@gitstream.local");
    if let Some(path) = index {
        cmd.env("GIT_INDEX_FILE", path);
    }
    if input.is_some() {
        cmd.stdin(Stdio::piped());
    }
    let mut child = cmd.spawn().map_err(io_error)?;
    if let Some(data) = input {
        child
            .stdin
            .take()
            .ok_or_else(|| error("Missing Git stdin"))?
            .write_all(data)
            .map_err(io_error)?;
    }
    let out = child.wait_with_output().map_err(io_error)?;
    if out.status.success() {
        Ok(out.stdout)
    } else {
        Err(classify_git_error(&String::from_utf8_lossy(&out.stderr)))
    }
}
fn run(repo: &Path, args: &[&str]) -> Result<String, GitError> {
    String::from_utf8(git(repo, args, None, None)?)
        .map(|s| s.trim().into())
        .map_err(io_error)
}
fn git_path(repo: &Path, name: &str) -> Result<PathBuf, GitError> {
    let path = PathBuf::from(run(repo, &["rev-parse", "--git-path", name])?);
    Ok(if path.is_absolute() {
        path
    } else {
        repo.join(path)
    })
}
fn validate(path: &str) -> Result<(), GitError> {
    if path.is_empty()
        || Path::new(path)
            .components()
            .any(|c| !matches!(c, Component::Normal(_)))
        || Path::new(path)
            .components()
            .any(|c| c.as_os_str().to_string_lossy().eq_ignore_ascii_case(".git"))
    {
        return Err(error(format!("Unsafe snapshot path: {path}")));
    }
    Ok(())
}
fn walk(repo: &Path, dir: &Path, entries: &mut Vec<Entry>) -> Result<(), GitError> {
    for item in fs::read_dir(dir).map_err(io_error)? {
        let item = item.map_err(io_error)?;
        let path = item.path();
        if dir == repo && item.file_name() == ".git" {
            continue;
        }
        let rel = path
            .strip_prefix(repo)
            .map_err(io_error)?
            .to_str()
            .ok_or_else(|| error("Snapshot requires UTF-8 paths"))?
            .replace('\\', "/");
        validate(&rel)?;
        let meta = fs::symlink_metadata(&path).map_err(io_error)?;
        if meta.is_dir() {
            if path.join(".git").exists() {
                return Err(error(format!(
                    "Nested repositories/submodules need a separate snapshot: {rel}"
                )));
            }
            walk(repo, &path, entries)?;
        } else {
            let (mode, bytes) = if meta.file_type().is_symlink() {
                let target = fs::read_link(&path).map_err(io_error)?;
                (
                    "120000",
                    target
                        .to_str()
                        .ok_or_else(|| error("Snapshot requires UTF-8 symlink targets"))?
                        .as_bytes()
                        .to_vec(),
                )
            } else if meta.is_file() {
                #[cfg(unix)]
                let executable = {
                    use std::os::unix::fs::PermissionsExt;
                    meta.permissions().mode() & 0o111 != 0
                };
                #[cfg(not(unix))]
                let executable = false;
                (
                    if executable { "100755" } else { "100644" },
                    fs::read(&path).map_err(io_error)?,
                )
            } else {
                return Err(error(format!("Unsupported snapshot file: {rel}")));
            };
            let oid = String::from_utf8(git(
                repo,
                &["hash-object", "-w", "--stdin"],
                Some(&bytes),
                None,
            )?)
            .map_err(io_error)?
            .trim()
            .to_string();
            entries.push(Entry {
                path: rel,
                mode: mode.into(),
                oid,
                stage: "0".into(),
            });
        }
    }
    entries.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(())
}
fn read_optional(path: &Path) -> Result<Vec<u8>, GitError> {
    match fs::read(path) {
        Ok(data) => Ok(data),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
        Err(e) => Err(io_error(e)),
    }
}
fn capture(repo: &Path) -> Result<RepoSnapshot, GitError> {
    // Canonical entries avoid false conflicts from Git's index stat-cache refresh.
    let bytes = git(repo, &["ls-files", "--stage", "-z"], None, None)?;
    let mut index = vec![];
    for row in bytes.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let row = std::str::from_utf8(row).map_err(io_error)?;
        let (meta, path) = row
            .split_once('\t')
            .ok_or_else(|| error("Invalid index entry"))?;
        validate(path)?;
        let mut fields = meta.split_whitespace();
        let mode = fields.next().ok_or_else(|| error("Missing mode"))?;
        if mode == "160000" {
            return Err(error(
                "Submodule undo is not supported; no mutation performed",
            ));
        }
        index.push(Entry {
            path: path.into(),
            mode: mode.into(),
            oid: fields.next().ok_or_else(|| error("Missing object"))?.into(),
            stage: fields.next().ok_or_else(|| error("Missing stage"))?.into(),
        });
    }
    let mut files = vec![];
    walk(repo, repo, &mut files)?;
    let head_sha = run(repo, &["rev-parse", "--verify", "HEAD"]).unwrap_or_default();
    let head = run(repo, &["symbolic-ref", "-q", "HEAD"])
        .unwrap_or_else(|_| format!("detached:{head_sha}"));
    let refs = run(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname) %(objectname)",
            "refs/heads/",
            "refs/tags/",
            "refs/stash",
            "refs/gitstream/backup/",
        ],
    )?
    .lines()
    .filter_map(|l| l.split_once(' ').map(|(a, b)| (a.into(), b.into())))
    .collect();
    let mut snapshot = RepoSnapshot {
        head,
        head_sha,
        refs,
        index,
        files,
        stash_log: read_optional(&git_path(repo, "logs/refs/stash")?)?,
        config: read_optional(&git_path(repo, "config")?)?,
        index_tree: String::new(),
    };
    let temp = git_path(repo, "gitstream/snapshot-index")?;
    fs::create_dir_all(temp.parent().unwrap()).map_err(io_error)?;
    let _ = fs::remove_file(&temp);
    let result = (|| {
        git(repo, &["read-tree", "--empty"], None, Some(&temp))?;
        let mut info = vec![];
        for entry in &snapshot.index {
            info.extend_from_slice(
                format!(
                    "{} {} {}\t{}\0",
                    entry.mode, entry.oid, entry.stage, entry.path
                )
                .as_bytes(),
            );
        }
        git(
            repo,
            &["update-index", "-z", "--index-info"],
            Some(&info),
            Some(&temp),
        )?;
        String::from_utf8(git(repo, &["write-tree"], None, Some(&temp))?)
            .map(|s| s.trim().into())
            .map_err(io_error)
    })();
    let _ = fs::remove_file(&temp);
    snapshot.index_tree = result?;
    Ok(snapshot)
}
fn protect(repo: &Path, id: u64, phase: &str, state: &RepoSnapshot) -> Result<(), GitError> {
    let mut owned = vec!["commit-tree".into(), state.index_tree.clone()];
    let mut parents = BTreeSet::new();
    for oid in state
        .refs
        .iter()
        .map(|(_, o)| o)
        .chain(std::iter::once(&state.head_sha))
        .filter(|o| !o.is_empty())
    {
        if let Ok(commit) = run(repo, &["rev-parse", &format!("{oid}^{{commit}}")]) {
            parents.insert(commit);
        }
    }
    for oid in parents {
        owned.extend(["-p".into(), oid]);
    }
    let args: Vec<&str> = owned.iter().map(String::as_str).collect();
    let commit = String::from_utf8(git(
        repo,
        &args,
        Some(b"GitStream operation snapshot\n"),
        None,
    )?)
    .map_err(io_error)?;
    run(
        repo,
        &[
            "update-ref",
            &format!("refs/gitstream/oplog/{id}/{phase}"),
            commit.trim(),
        ],
    )?;
    Ok(())
}
fn read_log(repo: &Path) -> Result<Vec<OpSnapshot>, GitError> {
    let data = read_optional(&git_path(repo, "gitstream/oplog.jsonl")?)?;
    let mut ops = BTreeMap::new();
    for line in data.split(|b| *b == b'\n').filter(|l| !l.is_empty()) {
        let op: OpSnapshot = serde_json::from_slice(line).map_err(io_error)?;
        ops.insert(op.id, op);
    }
    Ok(ops.into_values().rev().take(RETAIN).collect())
}
pub fn get_oplog(repo: &Path) -> Result<Vec<OpSnapshot>, GitError> {
    let _lock = LOCK.lock().map_err(io_error)?;
    read_log(repo)
}
fn append(repo: &Path, op: &OpSnapshot) -> Result<(), GitError> {
    let path = git_path(repo, "gitstream/oplog.jsonl")?;
    fs::create_dir_all(path.parent().unwrap()).map_err(io_error)?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(io_error)?;
    let mut line = serde_json::to_vec(op).map_err(io_error)?;
    line.push(b'\n');
    file.write_all(&line).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    let kept: BTreeSet<u64> = read_log(repo)?.iter().map(|o| o.id).collect();
    for name in run(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname)",
            "refs/gitstream/oplog/",
        ],
    )?
    .lines()
    {
        if name
            .split('/')
            .nth(3)
            .and_then(|n| n.parse::<u64>().ok())
            .is_some_and(|id| !kept.contains(&id))
        {
            run(repo, &["update-ref", "-d", name])?;
        }
    }
    Ok(())
}
fn begin(repo: &Path, kind: &str, target_id: Option<u64>) -> Result<OpSnapshot, GitError> {
    let id = read_log(repo)?.first().map_or(1, |o| o.id + 1);
    let before = capture(repo)?;
    protect(repo, id, "before", &before)?;
    let op = OpSnapshot {
        id,
        at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(io_error)?
            .as_secs(),
        kind: kind.into(),
        label: kind.into(),
        before,
        after: None,
        undone: false,
        target_id,
    };
    append(repo, &op)?;
    Ok(op)
}
fn finish(repo: &Path, op: &mut OpSnapshot) -> Result<(), GitError> {
    let after = capture(repo)?;
    protect(repo, op.id, "after", &after)?;
    op.after = Some(after);
    append(repo, op)
}
pub fn mutate<T>(
    repo: &Path,
    kind: &str,
    f: impl FnOnce() -> Result<T, GitError>,
) -> Result<T, GitError> {
    let _lock = LOCK.lock().map_err(io_error)?;
    let mut op = begin(repo, kind, None)?;
    let result = f();
    // Partial failures (e.g. merge conflicts) also retain both recovery points.
    finish(repo, &mut op)?;
    result
}
fn token(state: &RepoSnapshot) -> Result<String, GitError> {
    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD
        .encode(serde_json::to_vec(state).map_err(io_error)?))
}
fn check(repo: &Path, op: &OpSnapshot, state: &RepoSnapshot) -> Result<UndoPreview, GitError> {
    if super::query::repo_state(repo)? != "clean" {
        return Err(error("Abort the merge/rebase/cherry-pick before Undo"));
    }
    let after = op
        .after
        .as_ref()
        .ok_or_else(|| error("Incomplete operation; recovery snapshot retained"))?;
    if state.head != after.head
        || state.head_sha != after.head_sha
        || state.refs != after.refs
        || state.config != after.config
        || state.stash_log != after.stash_log
    {
        return Err(error(
            "Operation is stale: HEAD or refs changed. Review the reflog before restoring",
        ));
    }
    let mut changed = BTreeSet::new();
    for (current, expected) in [(&state.files, &after.files), (&state.index, &after.index)] {
        for entry in current.iter().chain(expected.iter()) {
            if current.contains(entry) != expected.contains(entry) {
                changed.insert(entry.path.clone());
            }
        }
    }
    Ok(UndoPreview {
        token: token(state)?,
        needs_confirmation: !changed.is_empty(),
        changed_files: changed.into_iter().collect(),
    })
}
pub fn preview_undo(repo: &Path, id: u64) -> Result<UndoPreview, GitError> {
    let _lock = LOCK.lock().map_err(io_error)?;
    let op = read_log(repo)?
        .into_iter()
        .find(|o| o.id == id)
        .ok_or_else(|| error("Operation no longer retained"))?;
    check(repo, &op, &capture(repo)?)
}
fn restore(repo: &Path, target: &RepoSnapshot, current: &RepoSnapshot) -> Result<(), GitError> {
    // Load all blobs and validate before touching user files.
    let mut blobs = vec![];
    for entry in &target.files {
        validate(&entry.path)?;
        blobs.push(git(repo, &["cat-file", "blob", &entry.oid], None, None)?);
    }
    for entry in &current.files {
        validate(&entry.path)?;
    }
    for entry in current.files.iter().rev() {
        fs::remove_file(repo.join(&entry.path)).map_err(io_error)?;
        let mut parent = repo.join(&entry.path);
        parent.pop();
        while parent != repo {
            if fs::remove_dir(&parent).is_err() {
                break;
            }
            parent.pop();
        }
    }
    for (entry, bytes) in target.files.iter().zip(blobs) {
        let path = repo.join(&entry.path);
        fs::create_dir_all(path.parent().unwrap()).map_err(io_error)?;
        if entry.mode == "120000" {
            #[cfg(unix)]
            std::os::unix::fs::symlink(std::str::from_utf8(&bytes).map_err(io_error)?, &path)
                .map_err(io_error)?;
            #[cfg(not(unix))]
            return Err(error("Symlink restoration is unsupported on this platform"));
        } else {
            fs::write(&path, bytes).map_err(io_error)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(
                    &path,
                    fs::Permissions::from_mode(if entry.mode == "100755" { 0o755 } else { 0o644 }),
                )
                .map_err(io_error)?;
            }
        }
    }
    // One update-ref transaction restores all affected refs or none of them.
    let mut updates = String::from("start\n");
    for (name, oid) in &current.refs {
        if !target.refs.iter().any(|(n, _)| n == name) {
            updates.push_str(&format!("delete {name} {oid}\n"));
        }
    }
    for (name, oid) in &target.refs {
        let old = current
            .refs
            .iter()
            .find(|(n, _)| n == name)
            .map_or("0000000000000000000000000000000000000000", |(_, o)| {
                o.as_str()
            });
        updates.push_str(&format!("update {name} {oid} {old}\n"));
    }
    updates.push_str("prepare\ncommit\n");
    git(
        repo,
        &["update-ref", "--stdin"],
        Some(updates.as_bytes()),
        None,
    )?;
    if let Some(oid) = target.head.strip_prefix("detached:") {
        run(repo, &["update-ref", "--no-deref", "HEAD", oid])?;
    } else {
        run(repo, &["symbolic-ref", "HEAD", &target.head])?;
    }
    git(repo, &["read-tree", "--empty"], None, None)?;
    let mut info = vec![];
    for entry in &target.index {
        info.extend_from_slice(
            format!(
                "{} {} {}\t{}\0",
                entry.mode, entry.oid, entry.stage, entry.path
            )
            .as_bytes(),
        );
    }
    git(
        repo,
        &["update-index", "-z", "--index-info"],
        Some(&info),
        None,
    )?;
    let stash = git_path(repo, "logs/refs/stash")?;
    if target.stash_log.is_empty() {
        if stash.exists() {
            fs::remove_file(stash).map_err(io_error)?;
        }
    } else {
        fs::create_dir_all(stash.parent().unwrap()).map_err(io_error)?;
        fs::write(stash, &target.stash_log).map_err(io_error)?;
    }
    fs::write(git_path(repo, "config")?, &target.config).map_err(io_error)?;
    Ok(())
}
fn undo_locked(
    repo: &Path,
    id: u64,
    confirmation: Option<&str>,
    kind: &str,
) -> Result<(), GitError> {
    let mut ops = read_log(repo)?;
    let target = ops
        .iter_mut()
        .find(|o| o.id == id)
        .ok_or_else(|| error("Operation no longer retained"))?;
    if target.undone {
        return Err(error("Operation was already undone"));
    }
    let state = capture(repo)?;
    let preview = check(repo, target, &state)?;
    if let Some(provided) = confirmation {
        if provided != preview.token {
            return Err(error(
                "Confirmation expired: repository changed; review again",
            ));
        }
    } else if preview.needs_confirmation {
        return Err(error(
            "Working tree/index changed; explicit confirmation required",
        ));
    }
    let mut recovery = begin(repo, kind, Some(id))?;
    restore(repo, &target.before, &state)?;
    finish(repo, &mut recovery)?;
    target.undone = true;
    append(repo, target)?;
    Ok(())
}
pub fn undo(repo: &Path, id: u64, confirmation: Option<&str>) -> Result<(), GitError> {
    let _lock = LOCK.lock().map_err(io_error)?;
    undo_locked(repo, id, confirmation, "undo")
}
pub fn redo(repo: &Path, confirmation: Option<&str>) -> Result<(), GitError> {
    let _lock = LOCK.lock().map_err(io_error)?;
    let ops = read_log(repo)?;
    let last = ops.first().ok_or_else(|| error("Nothing to redo"))?;
    if last.kind != "undo" || last.undone {
        return Err(error("Nothing to redo"));
    }
    let target_id = last.target_id;
    undo_locked(repo, last.id, confirmation, "redo")?;
    if let Some(id) = target_id {
        if let Some(mut op) = read_log(repo)?.into_iter().find(|o| o.id == id) {
            op.undone = false;
            append(repo, &op)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::{mutation, query::run_git};
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Repo(PathBuf);
    impl Repo {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "gitstream-oplog-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::SeqCst)
            ));
            fs::create_dir_all(&path).unwrap();
            run_git(&path, &["init", "-q"]).unwrap();
            run_git(&path, &["config", "user.name", "Test"]).unwrap();
            run_git(&path, &["config", "user.email", "test@example.com"]).unwrap();
            fs::write(path.join("file"), b"original\n").unwrap();
            run_git(&path, &["add", "."]).unwrap();
            run_git(&path, &["commit", "-qm", "initial"]).unwrap();
            Self(path)
        }
    }
    impl Drop for Repo {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn oplog_reset_restores_complete_state_and_redoes() {
        let repo = Repo::new();
        let p = &repo.0;
        fs::write(p.join("file"), b"staged\n").unwrap();
        run_git(p, &["add", "file"]).unwrap();
        fs::write(p.join("file"), b"unstaged\n").unwrap();
        fs::write(p.join("binary"), [0, 255, 42]).unwrap();
        fs::write(p.join("untracked"), b"precious").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink("file", p.join("link")).unwrap();
        mutation::reset(p, "HEAD", "hard").unwrap();
        let id = get_oplog(p).unwrap()[0].id;
        undo(p, id, None).unwrap();
        assert_eq!(fs::read(p.join("file")).unwrap(), b"unstaged\n");
        assert_eq!(run_git(p, &["show", ":file"]).unwrap(), "staged\n");
        assert_eq!(fs::read(p.join("binary")).unwrap(), [0, 255, 42]);
        assert_eq!(fs::read(p.join("untracked")).unwrap(), b"precious");
        #[cfg(unix)]
        assert_eq!(
            fs::read_link(p.join("link")).unwrap(),
            PathBuf::from("file")
        );
        redo(p, None).unwrap();
        assert_eq!(fs::read(p.join("file")).unwrap(), b"original\n");
    }
    #[test]
    fn oplog_guard_preserves_edits_and_rejects_stale_confirmation() {
        let repo = Repo::new();
        let p = &repo.0;
        fs::write(p.join("file"), "discard me").unwrap();
        mutation::discard(p, &["file".into()]).unwrap();
        let id = get_oplog(p).unwrap()[0].id;
        fs::write(p.join("file"), "new work").unwrap();
        assert!(undo(p, id, None).is_err());
        assert_eq!(fs::read_to_string(p.join("file")).unwrap(), "new work");
        let check = preview_undo(p, id).unwrap();
        fs::write(p.join("another"), "newer work").unwrap();
        assert!(undo(p, id, Some(&check.token)).is_err());
        let check = preview_undo(p, id).unwrap();
        undo(p, id, Some(&check.token)).unwrap();
        redo(p, None).unwrap();
        assert_eq!(fs::read_to_string(p.join("another")).unwrap(), "newer work");
    }
}
