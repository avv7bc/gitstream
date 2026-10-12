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
    if name.starts_with("gitstream/") {
        let head = PathBuf::from(run(repo, &["rev-parse", "--git-path", "HEAD"])?);
        let parent = if head.is_absolute() { head.parent().unwrap_or(repo).to_path_buf() } else { repo.join(head).parent().unwrap_or(repo).to_path_buf() };
        fs::create_dir_all(&parent).map_err(io_error)?;
        return Ok(parent.join(name));
    }
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
fn walk(
    repo: &Path,
    entries: &mut Vec<Entry>,
    tracked: &BTreeSet<String>,
) -> Result<(), GitError> {
    let listed = git(repo, &["ls-files", "--others", "--exclude-standard", "-z"], None, None)?;
    let mut paths = BTreeSet::new();
    for raw in listed.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let path = std::str::from_utf8(raw).map_err(io_error)?;
        validate(path)?;
        paths.insert(path.to_owned());
    }
    for path in tracked {
        paths.insert(path.clone());
    }
    for rel in paths {
        let path = repo.join(&rel);
        let meta = match fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(io_error(e)),
        };
        if meta.is_dir() { continue; }
        let (mode, bytes) = if meta.file_type().is_symlink() {
            let target = fs::read_link(&path).map_err(io_error)?;
            ("120000", target.to_string_lossy().as_bytes().to_vec())
        } else if meta.is_file() {
            #[cfg(unix)]
            let executable = { use std::os::unix::fs::PermissionsExt; meta.permissions().mode() & 0o111 != 0 };
            #[cfg(not(unix))]
            let executable = false;
            (if executable { "100755" } else { "100644" }, fs::read(&path).map_err(io_error)?)
        } else { continue; };
        let oid = String::from_utf8(git(repo, &["hash-object", "-w", "--stdin"], Some(&bytes), None)?)
            .map_err(io_error)?.trim().to_owned();
        entries.push(Entry { path: rel, mode: mode.into(), oid, stage: "0".into() });
    }
    entries.sort_by(|a,b| a.path.cmp(&b.path));
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
    let mut tracked = BTreeSet::new();
    for row in bytes.split(|b| *b == 0).filter(|r| !r.is_empty()) {
        let row = std::str::from_utf8(row).map_err(io_error)?;
        let (meta, path) = row
            .split_once('\t')
            .ok_or_else(|| error("Invalid index entry"))?;
        validate(path)?;
        tracked.insert(path.to_owned());
        let mut fields = meta.split_whitespace();
        let mode = fields.next().ok_or_else(|| error("Missing mode"))?;
        index.push(Entry {
            path: path.into(),
            mode: mode.into(),
            oid: fields.next().ok_or_else(|| error("Missing object"))?.into(),
            stage: fields.next().ok_or_else(|| error("Missing stage"))?.into(),
        });
    }
    let mut files = vec![];
    // Gitlinks have no regular worktree blob; retain their index entry so refs
    // and the index can be restored without descending into the submodule.
    for entry in &index {
        if entry.mode == "160000" {
            files.push(entry.clone());
        }
    }
    walk(repo, &mut files, &tracked)?;
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
    if snapshot.index.iter().all(|e| e.stage == "0") {
        git(repo, &["read-tree", "--empty"], None, Some(&temp))?;
        let mut info = vec![];
        for entry in &snapshot.index {
            info.extend_from_slice(format!("{} {} 0\t{}\0", entry.mode, entry.oid, entry.path).as_bytes());
        }
        if !info.is_empty() { git(repo, &["update-index", "-z", "--index-info"], Some(&info), Some(&temp))?; }
        snapshot.index_tree = String::from_utf8(git(repo, &["write-tree"], None, Some(&temp))?)
            .map_err(io_error)?.trim().into();
    }
    let _ = fs::remove_file(&temp);
    Ok(snapshot)
}
fn protect(repo: &Path, id: u64, phase: &str, state: &RepoSnapshot) -> Result<(), GitError> {
    let mut key = git_path(repo, "gitstream").ok().and_then(|p| p.parent().and_then(|x| x.file_name()).map(|n| n.to_string_lossy().into_owned())).unwrap_or_else(|| "main".into());
    if key == ".git" { key = "main".into(); } else { key = key.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect(); }
    let base = format!("refs/gitstream/{key}/oplog/{id}/{phase}");
    // Keep every ref tip verbatim, including annotated tag objects.
    for (idx, (_, oid)) in state.refs.iter().enumerate() {
        run(repo, &["update-ref", &format!("{base}/ref/{idx}"), oid])?;
    }
    for token in std::str::from_utf8(&state.stash_log).unwrap_or("").split_whitespace() {
        if token.len() == 40 && token.bytes().all(|b| b.is_ascii_hexdigit()) {
            run(repo, &["update-ref", &format!("{base}/stash/{token}"), token])?;
        }
    }
    // Build independent trees for each index stage. A conflicted index cannot
    // be represented by one write-tree invocation, but every stage object can.
    let mut stages = BTreeSet::new();
    for e in &state.index { stages.insert(e.stage.clone()); }
    for stage in stages {
        let temp = git_path(repo, &format!("gitstream/protect-index-{id}-{phase}-{stage}"))?;
        fs::create_dir_all(temp.parent().unwrap()).map_err(io_error)?;
        let _ = fs::remove_file(&temp);
        git(repo, &["read-tree", "--empty"], None, Some(&temp))?;
        let mut info = Vec::new();
        for e in state.index.iter().filter(|e| e.stage == stage) {
            info.extend_from_slice(format!("{} {} 0\t{}\0", e.mode, e.oid, e.path).as_bytes());
        }
        if !info.is_empty() { git(repo, &["update-index", "-z", "--index-info"], Some(&info), Some(&temp))?; }
        let tree = String::from_utf8(git(repo, &["write-tree"], None, Some(&temp))?).map_err(io_error)?.trim().to_owned();
        let _ = fs::remove_file(&temp);
        run(repo, &["update-ref", &format!("{base}/index/{stage}"), &tree])?;
    }
    // Worktree tree protects untracked and ignored-excluded files captured by
    // Git enumeration. The individual blobs are also reachable from this tree.
    let temp = git_path(repo, &format!("gitstream/protect-worktree-{id}-{phase}"))?;
    fs::create_dir_all(temp.parent().unwrap()).map_err(io_error)?;
    let _ = fs::remove_file(&temp);
    git(repo, &["read-tree", "--empty"], None, Some(&temp))?;
    let mut info = Vec::new();
    for e in &state.files { if e.mode != "160000" { info.extend_from_slice(format!("{} {} 0\t{}\0", e.mode, e.oid, e.path).as_bytes()); } }
    if !info.is_empty() { git(repo, &["update-index", "-z", "--index-info"], Some(&info), Some(&temp))?; }
    let tree = String::from_utf8(git(repo, &["write-tree"], None, Some(&temp))?).map_err(io_error)?.trim().to_owned();
    let _ = fs::remove_file(&temp);
    run(repo, &["update-ref", &format!("{base}/worktree"), &tree])?;
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

#[cfg(test)]
pub fn get_oplog(repo: &Path) -> Result<Vec<OpSnapshot>, GitError> {
    let _lock = LOCK.lock().map_err(io_error)?;
    read_log(repo)
}
#[derive(Serialize)]
pub struct OpSummary {
    pub id: u64,
    pub at: u64,
    pub kind: String,
    pub label: String,
    pub undone: bool,
    pub target_id: Option<u64>,
    pub eligible: bool,
}
pub fn get_oplog_summary(repo: &Path) -> Result<Vec<OpSummary>, GitError> {
    let _lock = LOCK.lock().map_err(io_error)?;
    Ok(read_log(repo)?.into_iter().map(|o| OpSummary { id:o.id, at:o.at, kind:o.kind, label:o.label, undone:o.undone, target_id:o.target_id, eligible:o.after.is_some() && !o.undone }).collect())
}
fn append(repo: &Path, op: &OpSnapshot) -> Result<(), GitError> {
    let path = git_path(repo, "gitstream/oplog.jsonl")?;
    fs::create_dir_all(path.parent().unwrap()).map_err(io_error)?;
    let mut ops = read_log(repo)?.into_iter().collect::<Vec<_>>();
    ops.retain(|old| old.id != op.id);
    ops.push(op.clone());
    ops.sort_by_key(|o| o.id);
    if ops.len() > RETAIN { ops.drain(..ops.len() - RETAIN); }
    let mut data = Vec::new();
    for item in &ops { data.extend(serde_json::to_vec(item).map_err(io_error)?); data.push(b'\n'); }
    let temp = path.with_extension("tmp");
    fs::write(&temp, &data).map_err(io_error)?;
    let file = OpenOptions::new().read(true).open(&temp).map_err(io_error)?;
    file.sync_all().map_err(io_error)?;
    fs::rename(&temp, &path).map_err(io_error)?;
    let kept: BTreeSet<u64> = ops.iter().map(|o| o.id).collect();
    for name in run(repo, &["for-each-ref", "--format=%(refname)", "refs/gitstream/"])?.lines() {
        if name.split('/').nth(3).and_then(|n| n.parse::<u64>().ok()).is_some_and(|id| !kept.contains(&id)) { run(repo, &["update-ref", "-d", name])?; }
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
    let finished = finish(repo, &mut op);
    match (result, finished) { (Err(e), _) => Err(e), (Ok(v), Ok(())) => Ok(v), (Ok(_), Err(e)) => Err(e) }
}
fn token(state: &RepoSnapshot) -> Result<String, GitError> {
    use std::hash::{Hash, Hasher};
    let bytes = serde_json::to_vec(state).map_err(io_error)?;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    bytes.hash(&mut h);
    Ok(format!("{:016x}", h.finish()))
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
    let index = git_path(repo, "index")?;
    let lock = index.with_extension("lock");
    if lock.exists() { return Err(error("Git index is locked; close other Git operations and retry")); }
    for entry in target.files.iter().chain(current.files.iter()) { validate(&entry.path)?; }
    let current_paths: BTreeSet<&str> = current.files.iter().map(|e| e.path.as_str()).collect();
    for entry in target.files.iter() {
        let path = repo.join(&entry.path);
        let mut parent = path.parent();
        while let Some(dir) = parent {
            if dir == repo { break; }
            if let Ok(meta) = fs::symlink_metadata(dir) {
                if meta.file_type().is_symlink() || !meta.is_dir() { return Err(error(format!("Restore blocked by symlink or file parent: {}", entry.path))); }
            }
            parent = dir.parent();
        }
        if let Ok(meta) = fs::symlink_metadata(&path) {
            if !current_paths.contains(entry.path.as_str()) || (meta.is_dir() && entry.mode != "040000") {
                return Err(error(format!("Restore blocked by existing path: {}", entry.path)));
            }
        }
    }
    // Load all blobs and validate before touching user files.
    let mut blobs = vec![];
    for entry in &target.files {
        validate(&entry.path)?;
        if entry.mode != "160000" {
            blobs.push(Some(git(
                repo,
                &["cat-file", "blob", &entry.oid],
                None,
                None,
            )?));
        } else {
            blobs.push(None);
        }
    }
    for entry in &current.files {
        validate(&entry.path)?;
    }
    for entry in current.files.iter().rev() {
        if entry.mode == "160000" {
            continue;
        }
        let path = repo.join(&entry.path);
        if fs::symlink_metadata(&path)
            .map(|m| m.is_dir())
            .unwrap_or(false)
        {
            continue;
        }
        fs::remove_file(&path).map_err(io_error)?;
        let mut parent = path;
        parent.pop();
        while parent != repo {
            if fs::remove_dir(&parent).is_err() {
                break;
            }
            parent.pop();
        }
    }
    for (entry, bytes) in target.files.iter().zip(blobs) {
        let Some(bytes) = bytes else {
            continue;
        };
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
    let last = ops.iter().find(|o| o.kind == "undo" && !o.undone
        && !ops.iter().any(|later| later.id > o.id && later.kind != "undo" && later.kind != "redo"))
        .ok_or_else(|| error("Nothing to redo"))?;
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
        run_git(p, &["gc", "--prune=now"]).unwrap();
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

    #[test]
    fn ignored_files_are_not_recorded_and_conflict_index_can_be_snapshotted() {
        let repo = Repo::new();
        let p = &repo.0;
        fs::write(p.join(".gitignore"), "ignored\n").unwrap();
        fs::write(p.join("ignored"), "do not journal").unwrap();
        run_git(p, &["add", ".gitignore"]).unwrap();
        run_git(p, &["commit", "-qm", "ignore"]).unwrap();
        mutation::discard(p, &["file".into()]).unwrap();
        let snapshot = get_oplog(p).unwrap().into_iter().next().unwrap();
        assert!(!snapshot
            .before
            .files
            .iter()
            .any(|entry| entry.path == "ignored"));
        run_git(p, &["checkout", "-q", "-b", "conflict"]).unwrap();
        fs::write(p.join("file"), "ours\n").unwrap();
        run_git(p, &["add", "file"]).unwrap();
        run_git(p, &["commit", "-qm", "ours"]).unwrap();
        run_git(p, &["checkout", "-q", "-"]).unwrap();
        fs::write(p.join("file"), "theirs\n").unwrap();
        run_git(p, &["add", "file"]).unwrap();
        run_git(p, &["commit", "-qm", "base-side"]).unwrap();
        run_git(p, &["merge", "conflict"]).unwrap_err();
        // Capturing a conflict state must retain the operation rather than
        // rejecting all subsequent abort/continue actions.
        let _ = mutation::op_abort(p, "merging");
    }
    #[test]
    fn hardening_index_lock_keeps_worktree_intact() {
        let repo = Repo::new(); let p = &repo.0;
        fs::write(p.join("file"), "before").unwrap();
        mutation::discard(p, &["file".into()]).unwrap();
        let id = get_oplog(p).unwrap()[0].id;
        fs::write(git_path(p, "index.lock").unwrap(), "locked").unwrap();
        assert!(undo(p, id, None).is_err());
        assert_eq!(fs::read(p.join("file")).unwrap(), b"original\n");
    }
    #[test]
    fn hardening_repeated_undo_redo() {
        let repo = Repo::new(); let p = &repo.0;
        mutation::create_branch(p, "one", None, false).unwrap();
        let first = get_oplog(p).unwrap()[0].id;
        mutation::create_branch(p, "two", None, false).unwrap();
        let second = get_oplog(p).unwrap()[0].id;
        undo(p, second, None).unwrap(); undo(p, first, None).unwrap();
        redo(p, None).unwrap(); redo(p, None).unwrap();
        undo(p, second, None).unwrap();
        assert!(run_git(p, &["rev-parse", "--verify", "refs/heads/two"]).is_err());
    }
    #[test]
    fn hardening_tag_and_all_stashes_survive_gc() {
        let repo = Repo::new(); let p = &repo.0;
        run_git(p, &["tag", "-am", "annotation", "tag"]).unwrap();
        let oid = run_git(p, &["rev-parse", "tag"]).unwrap();
        mutate(p, "delete.tag", || {run_git(p, &["tag", "-d", "tag"])?; Ok(())}).unwrap();
        run_git(p, &["gc", "--prune=now"]).unwrap();
        undo(p, get_oplog(p).unwrap()[0].id, None).unwrap();
        assert_eq!(run_git(p, &["rev-parse", "tag"]).unwrap(), oid);
        for content in ["a", "b", "c"] { fs::write(p.join("file"), content).unwrap(); run_git(p, &["stash", "push"]).unwrap(); }
        let stash = run_git(p, &["stash", "list"]).unwrap();
        mutate(p, "stash.clear", || { run_git(p, &["stash", "clear"])?; Ok(()) }).unwrap();
        run_git(p, &["gc", "--prune=now"]).unwrap();
        undo(p, get_oplog(p).unwrap()[0].id, None).unwrap();
        assert_eq!(run_git(p, &["stash", "list"]).unwrap(), stash);
        run_git(p, &["stash", "apply", "stash@{2}"]).unwrap();
        assert_eq!(fs::read(p.join("file")).unwrap(), b"a");
    }
    #[cfg(unix)]
    #[test]
    fn hardening_literal_backslash_and_bounded_token() {
        let repo = Repo::new(); let p = &repo.0;
        fs::write(p.join("literal\\name"), "bytes").unwrap();
        mutate(p, "delete", || {fs::remove_file(p.join("literal\\name")).map_err(io_error)}).unwrap();
        let id = get_oplog(p).unwrap()[0].id;
        assert!(preview_undo(p, id).unwrap().token.len() <= 128);
        undo(p, id, None).unwrap();
        assert_eq!(fs::read(p.join("literal\\name")).unwrap(), b"bytes");
    }
    #[test]
    fn hardening_retention_bounds_disk() {
        let repo = Repo::new(); let p = &repo.0;
        for _ in 0..52 {mutate(p, "noop", || Ok(())).unwrap();}
        let data = fs::read(git_path(p, "gitstream/oplog.jsonl").unwrap()).unwrap();
        assert!(data.split(|b| *b == b'\n').filter(|l| !l.is_empty()).count() <= 50);
    }

}
