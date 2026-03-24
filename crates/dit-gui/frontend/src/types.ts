export interface CommitInfo {
  hash: string;
  message: string;
  author: string;
  date: string;
  branch: string | null;
}

export interface RepoStatus {
  branch: string;
  has_changes: boolean;
  changed_files: string[];
}

export interface BranchInfo {
  name: string;
  is_current: boolean;
}

export interface RestoreInfo {
  message: string;
  fig_file_path: string | null;
}

export interface DiffResult {
  before_image: string | null;
  after_image: string | null;
}

export interface DirCheck {
  exists: boolean;
  has_dit: boolean;
  has_git: boolean;
}

export interface TreeNode {
  id: string;
  name: string;
  node_type: string;
  children: TreeNode[];
}

export interface DiffTreeNode {
  id: string;
  name: string;
  node_type: string;
  change_type: "added" | "removed" | "modified" | null;
  children: DiffTreeNode[];
}

export interface DiffTreeResult {
  before: DiffTreeNode[];
  after: DiffTreeNode[];
}

export interface CloneInfo {
  is_dit_repo: boolean;
  path: string;
  name: string | null;
}

export interface SshKeyInfo {
  name: string;
  path: string;
}

export type AppView = "startup" | "main";
