import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import type { BranchInfo, TagInfo, StashEntry } from "@/types";
import { useRepo } from "./useRepo";

const branches = ref<BranchInfo[]>([]);
const tags = ref<TagInfo[]>([]);
const stashes = ref<StashEntry[]>([]);
const remotes = ref<string[]>([]);

export function useBranches() {
  const { repoPath } = useRepo();

  async function refresh() {
    if (!repoPath.value) return;
    const [b, t, s, r] = await Promise.all([
      invoke<BranchInfo[]>("get_branches", { repoPath: repoPath.value }),
      invoke<TagInfo[]>("get_tags", { repoPath: repoPath.value }),
      invoke<StashEntry[]>("get_stashes", { repoPath: repoPath.value }),
      invoke<string[]>("get_remotes", { repoPath: repoPath.value }),
    ]);
    branches.value = b;
    tags.value = t;
    stashes.value = s;
    remotes.value = r;
  }

  async function checkout(branch: string) {
    if (!repoPath.value) return;
    await invoke("do_checkout", { repoPath: repoPath.value, branch });
  }

  async function checkoutRemote(remoteBranch: string, localName: string | null) {
    if (!repoPath.value) return;
    await invoke("do_checkout_remote", {
      repoPath: repoPath.value,
      remoteBranch,
      localName,
    });
  }

  async function mergeBranch(branch: string) {
    if (!repoPath.value) return;
    await invoke("do_merge", { repoPath: repoPath.value, branch });
  }

  async function renameBranch(oldName: string, newName: string) {
    if (!repoPath.value) return;
    await invoke("do_rename_branch", { repoPath: repoPath.value, oldName, newName });
  }

  async function deleteBranch(branch: string, force: boolean) {
    if (!repoPath.value) return;
    await invoke("do_delete_branch", { repoPath: repoPath.value, branch, force });
  }

  async function pushBranch(branch: string, remote: string, force: boolean) {
    if (!repoPath.value) return;
    await invoke("do_push_branch", { repoPath: repoPath.value, remote, branch, force });
  }

  return {
    branches, tags, stashes, remotes,
    refresh, checkout, checkoutRemote,
    mergeBranch, renameBranch, deleteBranch, pushBranch,
  };
}
