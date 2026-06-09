import type { CommitInfo } from "@/types";

// Клиентский фильтр коммитов в CommitGraph: подстрока (без регистра) по
// сообщению / автору / e-mail / полному и короткому SHA / дате / именам refs.
// Чистая логика — вынесена для юнит-тестов.

export function matchesCommitFilter(commit: CommitInfo, query: string): boolean {
  const q = query.trim().toLowerCase();
  if (!q) return true;
  return (
    commit.message.toLowerCase().includes(q) ||
    commit.author.toLowerCase().includes(q) ||
    commit.author_email.toLowerCase().includes(q) ||
    commit.oid.toLowerCase().includes(q) ||
    commit.short_oid.toLowerCase().includes(q) ||
    commit.date.toLowerCase().includes(q) ||
    commit.refs.some((r) => r.name.toLowerCase().includes(q))
  );
}

export function filterCommits(commits: CommitInfo[], query: string): CommitInfo[] {
  if (!query.trim()) return commits;
  return commits.filter((c) => matchesCommitFilter(c, query));
}
