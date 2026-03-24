import { invoke } from "@tauri-apps/api/core";
import type { CommitInfo, RepoStatus, DiffResult, BranchInfo, RestoreInfo, DirCheck, CloneInfo, SshKeyInfo, TreeNode, DiffTreeResult } from "./types";

export async function checkDirectory(path: string): Promise<DirCheck> {
  return invoke("check_directory", { path });
}

export async function listSshKeys(): Promise<SshKeyInfo[]> {
  return invoke("list_ssh_keys");
}

export async function cloneRepo(url: string, path: string, sshKeyPath?: string | null): Promise<CloneInfo> {
  return invoke("clone_repo", { url, path, sshKeyPath: sshKeyPath ?? null });
}

export async function initRepo(
  path: string,
  authCookie: string | null,
  authEmail: string | null,
  authPassword: string | null,
  fileKey: string,
  fileName: string,
  force: boolean = false,
  sshKeyPath: string | null = null,
): Promise<string> {
  return invoke("init_repo", { path, authCookie, authEmail, authPassword, fileKey, fileName, force, sshKeyPath });
}

export async function openRepo(path: string): Promise<string> {
  return invoke("open_repo", { path });
}

export async function getStatus(): Promise<RepoStatus> {
  return invoke("get_status");
}

export async function getLog(): Promise<CommitInfo[]> {
  return invoke("get_log");
}

export async function getBranches(): Promise<BranchInfo[]> {
  return invoke("get_branches");
}

export async function commitChanges(message: string): Promise<CommitInfo> {
  return invoke("commit", { message });
}

export async function submit2faCode(code: string): Promise<void> {
  return invoke("submit_2fa_code", { code });
}

export async function restoreCommit(hash: string): Promise<RestoreInfo> {
  return invoke("restore", { hash });
}

export async function openFigFile(path: string): Promise<void> {
  return invoke("open_fig_file", { path });
}

export async function checkoutRef(reference: string): Promise<string> {
  return invoke("checkout", { reference });
}

export async function createBranch(name: string): Promise<string> {
  return invoke("create_branch", { name });
}

export async function mergeBranch(branch: string): Promise<string> {
  return invoke("merge", { branch });
}

export async function push(): Promise<string> {
  return invoke("push");
}

export async function pull(): Promise<string> {
  return invoke("pull");
}

export async function getPreviewImage(
  commitHash: string,
): Promise<string | null> {
  return invoke("get_preview_image", { commitHash });
}

export async function getDiffPreviews(
  hash1: string,
  hash2: string,
): Promise<DiffResult> {
  return invoke("get_diff_previews", { hash1, hash2 });
}

export async function getCommitTree(): Promise<TreeNode[]> {
  return invoke("get_commit_tree");
}

export async function getDiffTrees(
  hash1: string,
  hash2: string,
): Promise<DiffTreeResult> {
  return invoke("get_diff_trees", { hash1, hash2 });
}
