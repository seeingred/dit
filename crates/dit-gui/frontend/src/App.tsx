import { useState, useCallback } from "react";
import type { AppView, CommitInfo, RepoStatus } from "./types";
import { getLog, getBranches, getStatus } from "./commands";
import { StartupFlow } from "./components/StartupFlow";
import { MainLayout } from "./components/MainLayout";

export default function App() {
  const [view, setView] = useState<AppView>("startup");
  const [repoPath, setRepoPath] = useState<string>("");
  const [commits, setCommits] = useState<CommitInfo[]>([]);
  const [branches, setBranches] = useState<string[]>(["main"]);
  const [currentBranch, setCurrentBranch] = useState("main");
  const [status, setStatus] = useState<RepoStatus>({
    branch: "main",
    has_changes: false,
    changed_files: [],
  });

  const loadRepoData = useCallback(async () => {
    try {
      const [logData, branchData, statusData] = await Promise.all([
        getLog(),
        getBranches(),
        getStatus(),
      ]);
      setCommits(logData);
      setBranches(branchData.map((b) => b.name));
      const current = branchData.find((b) => b.is_current);
      if (current) setCurrentBranch(current.name);
      setStatus(statusData);
    } catch (e) {
      console.error("Failed to load repo data:", e);
    }
  }, []);

  const handleRepoOpened = useCallback(
    async (path: string) => {
      setRepoPath(path);
      setView("main");
      await loadRepoData();
    },
    [loadRepoData],
  );

  const handleBranchChange = useCallback(
    async (branch: string) => {
      setCurrentBranch(branch);
      await loadRepoData();
    },
    [loadRepoData],
  );

  if (view === "startup") {
    return <StartupFlow onRepoOpened={handleRepoOpened} />;
  }

  return (
    <MainLayout
      repoPath={repoPath}
      commits={commits}
      branches={branches}
      currentBranch={currentBranch}
      status={status}
      onBranchChange={handleBranchChange}
      onRefresh={loadRepoData}
      onCloseRepo={() => setView("startup")}
    />
  );
}
