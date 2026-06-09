import { ref } from "vue";
import { invoke } from "@/composables/useProgress";
import type { BranchInfo, TagInfo, StashEntry, RemoteInfo } from "@/types";
import { useRepo } from "@/composables/useRepo";
import { useSettings } from "@/composables/useSettings";

const branches = ref<BranchInfo[]>([]);
const tags = ref<TagInfo[]>([]);
const stashes = ref<StashEntry[]>([]);
const remotes = ref<string[]>([]);
const remoteUrls = ref<RemoteInfo[]>([]);

// Защита от гонки перекрывающихся refresh: применяем только самый свежий ответ.
let refreshSeq = 0;

export function useBranches() {
  const { repoPath } = useRepo();
  const { networkTimeoutSecs } = useSettings();

  async function refresh() {
    if (!repoPath.value) {
      // Репозиторий закрыт/удалён — сбрасываем состояние, иначе панель
      // BRANCHES сохраняет ветки/теги/stashes предыдущего репо (артефакты).
      ++refreshSeq;
      branches.value = [];
      tags.value = [];
      stashes.value = [];
      remotes.value = [];
      remoteUrls.value = [];
      return;
    }
    const seq = ++refreshSeq;
    const [b, t, s, r, ru] = await Promise.all([
      invoke<BranchInfo[]>("get_branches", { repoPath: repoPath.value }),
      invoke<TagInfo[]>("get_tags", { repoPath: repoPath.value }),
      invoke<StashEntry[]>("get_stashes", { repoPath: repoPath.value }),
      invoke<string[]>("get_remotes", { repoPath: repoPath.value }),
      invoke<RemoteInfo[]>("get_remote_urls", { repoPath: repoPath.value }),
    ]);
    // Более новый refresh уже стартовал — отбрасываем устаревший ответ.
    if (seq !== refreshSeq) return;
    branches.value = b;
    tags.value = t;
    stashes.value = s;
    remotes.value = r;
    remoteUrls.value = ru;
  }

  async function addRemote(name: string, url: string) {
    if (!repoPath.value) return;
    await invoke("add_remote", { repoPath: repoPath.value, name, url });
  }

  async function removeRemote(name: string) {
    if (!repoPath.value) return;
    await invoke("remove_remote", { repoPath: repoPath.value, name });
  }

  async function renameRemote(oldName: string, newName: string) {
    if (!repoPath.value) return;
    await invoke("rename_remote", { repoPath: repoPath.value, oldName, newName });
  }

  async function setRemoteUrl(name: string, url: string) {
    if (!repoPath.value) return;
    await invoke("set_remote_url", { repoPath: repoPath.value, name, url });
  }

  async function setBranchUpstream(branch: string, upstream: string | null) {
    if (!repoPath.value) return;
    await invoke("set_branch_upstream", { repoPath: repoPath.value, branch, upstream });
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

  async function createBranch(
    name: string,
    startPoint: string | null,
    checkout: boolean,
  ) {
    if (!repoPath.value) return;
    await invoke("do_create_branch", {
      repoPath: repoPath.value,
      name,
      startPoint,
      checkout,
    });
  }

  async function rebaseOnto(onto: string) {
    if (!repoPath.value) return;
    await invoke("do_rebase", { repoPath: repoPath.value, onto });
  }

  async function renameBranch(oldName: string, newName: string) {
    if (!repoPath.value) return;
    await invoke("do_rename_branch", { repoPath: repoPath.value, oldName, newName });
  }

  async function deleteBranch(branch: string, force: boolean) {
    if (!repoPath.value) return;
    await invoke("do_delete_branch", { repoPath: repoPath.value, branch, force });
  }

  async function deleteRemoteBranch(remote: string, branch: string) {
    if (!repoPath.value) return;
    await invoke("do_delete_remote_branch", {
      repoPath: repoPath.value,
      remote,
      branch,
      timeoutSecs: networkTimeoutSecs.value,
    });
  }

  async function pushBranch(branch: string, remote: string, force: boolean) {
    if (!repoPath.value) return;
    await invoke("do_push_branch", {
      repoPath: repoPath.value,
      remote,
      branch,
      force,
      timeoutSecs: networkTimeoutSecs.value,
    });
  }

  async function createTag(
    name: string,
    message: string | null,
    target: string | null,
    force: boolean,
  ) {
    if (!repoPath.value) return;
    await invoke("do_create_tag", {
      repoPath: repoPath.value,
      name,
      message,
      target,
      force,
    });
  }

  async function stashSave(message: string | null, includeUntracked: boolean) {
    if (!repoPath.value) return;
    await invoke("do_stash_save", {
      repoPath: repoPath.value,
      message,
      includeUntracked,
    });
  }

  async function stashApply(index: number) {
    if (!repoPath.value) return;
    await invoke("do_stash_apply", { repoPath: repoPath.value, index });
  }

  async function stashPop(index: number) {
    if (!repoPath.value) return;
    await invoke("do_stash_pop", { repoPath: repoPath.value, index });
  }

  async function stashDrop(index: number) {
    if (!repoPath.value) return;
    await invoke("do_stash_drop", { repoPath: repoPath.value, index });
  }

  async function deleteTag(name: string) {
    if (!repoPath.value) return;
    await invoke("do_delete_tag", { repoPath: repoPath.value, name });
  }

  async function pushTag(remote: string, name: string, del: boolean) {
    if (!repoPath.value) return;
    await invoke("do_push_tag", {
      repoPath: repoPath.value,
      remote,
      name,
      delete: del,
      timeoutSecs: networkTimeoutSecs.value,
    });
  }

  return {
    branches, tags, stashes, remotes, remoteUrls,
    refresh, checkout, checkoutRemote,
    createBranch, mergeBranch, rebaseOnto, renameBranch, deleteBranch, deleteRemoteBranch, pushBranch,
    createTag, deleteTag, pushTag,
    stashSave, stashApply, stashPop, stashDrop,
    addRemote, removeRemote, renameRemote, setRemoteUrl, setBranchUpstream,
  };
}
