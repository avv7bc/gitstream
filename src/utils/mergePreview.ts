export interface MergePreview {
  clean: boolean;
  conflicted_files: string[];
  fast_forward: boolean;
}

/** Parse the human-readable conflict section emitted by `git merge-tree`. */
export function parseMergeTreePreview(output: string, fastForward = false): MergePreview {
  const conflicted_files: string[] = [];
  for (const line of output.split(/\r?\n/)) {
    const match = line.match(/Merge conflict in (.+?)\s*$/);
    if (match && !conflicted_files.includes(match[1])) conflicted_files.push(match[1]);
  }
  return { clean: conflicted_files.length === 0, conflicted_files, fast_forward: fastForward };
}

export function isMergePreviewEnabled(versionOutput: string): boolean {
  const match = versionOutput.match(/(?:^|\s)(\d+)\.(\d+)(?:\.(\d+))?/);
  if (!match) return false;
  const major = Number(match[1]);
  const minor = Number(match[2]);
  return major > 2 || (major === 2 && minor >= 38);
}
