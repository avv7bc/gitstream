use serde::Serialize;

#[derive(Serialize, Clone, Debug)]
pub struct FileStatus {
    pub path: String,
    pub state: String,
    pub staged: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RefLabel {
    pub name: String,
    pub kind: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct CommitInfo {
    pub oid: String,
    pub short_oid: String,
    pub message: String,
    pub author: String,
    pub author_email: String,
    pub date: String,
    pub parents: Vec<String>,
    pub refs: Vec<RefLabel>,
    pub column: u32,
    pub lines: Vec<GraphLine>,
}

#[derive(Serialize, Clone, Debug)]
pub struct GraphLine {
    pub from_column: u32,
    pub to_column: u32,
    pub color: u32,
    pub style: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct BranchInfo {
    pub name: String,
    pub is_remote: bool,
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
    pub is_current: bool,
}

#[derive(Serialize, Clone, Debug)]
pub struct TagInfo {
    pub name: String,
    pub oid: String,
    pub message: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct StashEntry {
    pub index: usize,
    pub message: String,
    pub date: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DiffLine {
    pub kind: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
    pub content: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct DiffHunk {
    pub header: String,
    pub lines: Vec<DiffLine>,
    /// Дословный текст хунка (строка `@@ ...` + тело), для `git apply`.
    pub raw: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct FileDiff {
    pub path: String,
    pub hunks: Vec<DiffHunk>,
    pub insertions: u32,
    pub deletions: u32,
    /// Дословные строки заголовка патча до первого хунка
    /// (`diff --git`, `index`, `---`, `+++`), для `git apply`.
    pub header: String,
    /// Файл бинарный — текстового диффа нет.
    pub binary: bool,
    /// base64 изображения до изменения (только для бинарных картинок).
    pub old_image: Option<String>,
    /// base64 изображения после изменения.
    pub new_image: Option<String>,
    /// Размер файла в байтах (для бинарных файлов).
    pub byte_size: Option<u64>,
}

#[derive(Serialize, Clone, Debug)]
pub struct RepoInfo {
    pub path: String,
    pub current_branch: String,
    pub head_oid: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct RepoPathCheck {
    pub exists: bool,
    pub is_git_repo: bool,
    pub display_name: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct AuthorStat {
    pub name: String,
    pub email: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub active_days: u32,
    pub first_date: String,
    pub last_date: String,
}

#[derive(Serialize, Clone, Debug)]
pub struct MonthEntry {
    pub month: String,
    pub commits: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct DayEntry {
    pub date: String,
    pub count: u32,
}

#[derive(Serialize, Clone, Debug)]
pub struct RepoStats {
    pub total_commits: u32,
    pub total_insertions: u32,
    pub total_deletions: u32,
    pub first_commit_date: String,
    pub last_commit_date: String,
    pub active_days: u32,
    pub total_authors: u32,
    pub authors: Vec<AuthorStat>,
    pub by_weekday: Vec<u32>,
    pub by_hour: Vec<u32>,
    pub by_month: Vec<MonthEntry>,
    pub by_day: Vec<DayEntry>,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct UpdateInfo {
    pub version: String,
    pub release_url: String,
}
