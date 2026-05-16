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
}

export interface RefLabel {
  name: string;
  kind: "local-branch" | "remote-branch" | "tag" | "head" | "stash" | "current-branch";
}

export interface GraphRow {
  commit: CommitInfo;
  column: number;
  lines: GraphLine[];
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
}

export interface FileDiff {
  path: string;
  hunks: DiffHunk[];
  insertions: number;
  deletions: number;
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
