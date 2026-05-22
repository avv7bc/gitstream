export type FileState =
  | "modified"
  | "added"
  | "deleted"
  | "renamed"
  | "conflicted"
  | "untracked";

export type StagedState = "staged" | "unstaged" | "partial";

export interface FileStatus {
  path: string;
  state: FileState;
  staged: StagedState;
}

export interface CommitInfo {
  oid: string;
  short_oid: string;
  message: string;
  author: string;
  author_email: string;
  date: string;
  parents: string[];
  refs: RefLabel[];
  column: number;
  lines: GraphLine[];
}

export interface RefLabel {
  name: string;
  kind: "local-branch" | "remote-branch" | "tag" | "head" | "stash" | "current-branch";
}

export interface GraphLine {
  from_column: number;
  to_column: number;
  color: number;
  style: "straight" | "merge-left" | "merge-right" | "fork";
}

export interface BranchInfo {
  name: string;
  is_remote: boolean;
  upstream: string | null;
  ahead: number;
  behind: number;
  is_current: boolean;
}

export interface TagInfo {
  name: string;
  oid: string;
  message: string | null;
}

export interface StashEntry {
  index: number;
  message: string;
  date: string;
}

export interface DiffLine {
  kind: "context" | "added" | "removed";
  old_lineno: number | null;
  new_lineno: number | null;
  content: string;
}

export interface DiffHunk {
  header: string;
  lines: DiffLine[];
  raw: string;
}

export interface FileDiff {
  path: string;
  hunks: DiffHunk[];
  insertions: number;
  deletions: number;
  header: string;
  binary: boolean;
  old_image: string | null;
  new_image: string | null;
  byte_size: number | null;
}

export type LineOp = "stage" | "unstage" | "discard";

export interface LineHunkSelection {
  raw: string;
  selected: number[];
}

export type DiffMode = "unified" | "side-by-side";

export interface RepoInfo {
  path: string;
  current_branch: string;
  head_oid: string;
}

export interface RepoPathCheck {
  exists: boolean;
  is_git_repo: boolean;
  display_name: string;
}

export interface AuthorStat {
  name: string;
  email: string;
  commits: number;
  insertions: number;
  deletions: number;
  active_days: number;
  first_date: string;
  last_date: string;
}

export interface MonthEntry {
  month: string;
  commits: number;
}

export interface DayEntry {
  date: string;
  count: number;
}

export interface RepoStats {
  total_commits: number;
  total_insertions: number;
  total_deletions: number;
  first_commit_date: string;
  last_commit_date: string;
  active_days: number;
  total_authors: number;
  authors: AuthorStat[];
  by_weekday: number[];
  by_hour: number[];
  by_month: MonthEntry[];
  by_day: DayEntry[];
}

export interface UpdateInfo {
  version: string;
  release_url: string;
}
