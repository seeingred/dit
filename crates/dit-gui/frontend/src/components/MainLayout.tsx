import { useState, useCallback, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { CommitInfo, RepoStatus, RestoreInfo } from "../types";
import { restoreCommit, openFigFile } from "../commands";
import { CommitList } from "./CommitList";
import { CommitOverlay } from "./CommitOverlay";
import { PreviewPanel } from "./PreviewPanel";
import { DiffView } from "./DiffView";
import { ActionToolbar } from "./ActionToolbar";
import { BranchSelector } from "./BranchSelector";
import { CommandBar } from "./CommandBar";

interface MainLayoutProps {
  repoPath: string;
  commits: CommitInfo[];
  branches: string[];
  currentBranch: string;
  status: RepoStatus;
  onBranchChange: (branch: string) => void;
  onRefresh: () => Promise<void>;
}

export function MainLayout({
  repoPath,
  commits,
  branches,
  currentBranch,
  status,
  onBranchChange,
  onRefresh,
}: MainLayoutProps) {
  const [selectedHash, setSelectedHash] = useState<string | null>(null);
  const [diffSelection, setDiffSelection] = useState<[string | null, string | null]>([null, null]);
  const [showDiff, setShowDiff] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [restoreResult, setRestoreResult] = useState<RestoreInfo | null>(null);
  const [isCommitting, setIsCommitting] = useState(false);
  const [commitSteps, setCommitSteps] = useState<string[]>([]);
  const [commitComplete, setCommitComplete] = useState(false);

  const selectedCommit = commits.find((c) => c.hash === selectedHash) ?? null;
  const diffCommitA = commits.find((c) => c.hash === diffSelection[0]) ?? null;
  const diffCommitB = commits.find((c) => c.hash === diffSelection[1]) ?? null;

  // Listen for commit-progress events from backend
  useEffect(() => {
    const unlisten = listen<string>("commit-progress", (event) => {
      setCommitSteps((prev) => {
        if (prev.includes(event.payload)) return prev;
        return [...prev, event.payload];
      });
      if (event.payload === "Commit complete!") {
        setCommitComplete(true);
      }
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  const handleCommitStart = useCallback(() => {
    setIsCommitting(true);
    setCommitSteps([]);
    setCommitComplete(false);
  }, []);

  const handleCommitEnd = useCallback((success: boolean) => {
    if (success) {
      setTimeout(() => {
        setIsCommitting(false);
        setCommitSteps([]);
        setCommitComplete(false);
      }, 1500);
    } else {
      setIsCommitting(false);
      setCommitSteps([]);
      setCommitComplete(false);
    }
  }, []);

  const handleSelect = useCallback((hash: string) => {
    setSelectedHash(hash);
  }, []);

  const handleDiffToggle = useCallback((hash: string) => {
    setDiffSelection((prev) => {
      if (prev[0] === null) return [hash, null];
      if (prev[0] === hash) return [null, null];
      if (prev[1] === hash) return [prev[0], null];
      const next: [string | null, string | null] = [prev[0], hash];
      setShowDiff(true);
      return next;
    });
  }, []);

  const showToast = useCallback((message: string) => {
    setToast(message);
    setTimeout(() => setToast(null), 3000);
  }, []);

  const handleCommand = useCallback((cmd: string) => {
    // TODO: wire to actual command execution
    showToast(`Executed: dit ${cmd}`);
  }, [showToast]);

  return (
    <div className="flex flex-col h-screen bg-dit-bg">
      {/* Top bar: branch selector + repo path */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-dit-border bg-dit-surface/30">
        <div className="flex items-center gap-3">
          <span className="text-sm font-bold text-dit-accent tracking-tight">DIT</span>
          <BranchSelector
            branches={branches}
            currentBranch={currentBranch}
            onBranchChange={onBranchChange}
          />
        </div>
        <div className="flex items-center gap-3">
          {status.has_changes && (
            <span className="text-xs px-2 py-0.5 rounded-full bg-dit-warning/10 text-dit-warning">
              Unsaved changes
            </span>
          )}
          <span className="text-xs text-dit-text-muted font-mono truncate max-w-[300px]">
            {repoPath}
          </span>
        </div>
      </div>

      {/* Action toolbar */}
      <ActionToolbar
        status={status}
        isCommitting={isCommitting}
        onCommitStart={handleCommitStart}
        onCommitEnd={handleCommitEnd}
        onAction={(msg) => { showToast(msg); onRefresh(); }}
        onMerge={() => showToast("Merge dialog coming soon")}
        onRestore={async () => {
          if (!selectedHash) {
            showToast("Select a commit to restore");
            return;
          }
          showToast(`Restoring to ${selectedHash.slice(0, 7)}...`);
          try {
            const result = await restoreCommit(selectedHash);
            setRestoreResult(result);
          } catch (e) {
            showToast(`Restore failed: ${e}`);
          }
        }}
      />

      {/* Main content: sidebar + preview/diff */}
      <div className="flex flex-1 overflow-hidden">
        {/* Left panel: commit list */}
        <div className="w-80 min-w-[280px] border-r border-dit-border flex flex-col">
          <CommitList
            commits={commits}
            selectedHash={selectedHash}
            diffSelection={diffSelection}
            onSelect={handleSelect}
            onDiffToggle={handleDiffToggle}
          />
        </div>

        {/* Right panel: preview or diff */}
        <div className="flex-1 flex flex-col">
          {showDiff && diffCommitA && diffCommitB ? (
            <DiffView
              commitA={diffCommitA}
              commitB={diffCommitB}
              onClose={() => {
                setShowDiff(false);
                setDiffSelection([null, null]);
              }}
            />
          ) : (
            <PreviewPanel commit={selectedCommit} />
          )}
        </div>
      </div>

      {/* Bottom: command bar */}
      <CommandBar onCommand={handleCommand} />

      {/* Commit overlay */}
      {isCommitting && (
        <CommitOverlay steps={commitSteps} isComplete={commitComplete} />
      )}

      {/* Restore result dialog */}
      {restoreResult && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-dit-surface border border-dit-border rounded-xl p-6 max-w-md mx-4 shadow-2xl">
            <h3 className="text-lg font-semibold text-dit-text mb-3">Design Restored</h3>
            <p className="text-sm text-dit-text-muted mb-4">{restoreResult.message}</p>

            {restoreResult.fig_file_path ? (
              <div className="bg-dit-bg rounded-lg p-4 mb-4 border border-dit-border">
                <p className="text-sm font-medium text-dit-text mb-2">Open this file in Figma:</p>
                <p className="text-xs font-mono text-dit-text-muted break-all mb-3">
                  {restoreResult.fig_file_path}
                </p>
                <button
                  onClick={async () => {
                    if (restoreResult.fig_file_path) {
                      try {
                        await openFigFile(restoreResult.fig_file_path);
                        showToast("Opening .fig file in Figma...");
                      } catch (e) {
                        showToast(`Failed to open: ${e}`);
                      }
                    }
                  }}
                  className="w-full px-4 py-2 bg-dit-accent text-white text-sm rounded-lg font-medium
                             hover:bg-dit-accent-hover transition-colors"
                >
                  Open in Figma
                </button>
              </div>
            ) : (
              <div className="bg-dit-bg rounded-lg p-4 mb-4 border border-dit-border">
                <p className="text-sm text-dit-text-muted">
                  No .fig file available for this commit. Design JSON files are in the working directory.
                </p>
              </div>
            )}

            <button
              onClick={() => setRestoreResult(null)}
              className="w-full px-4 py-2 bg-dit-surface border border-dit-border text-dit-text text-sm
                         rounded-lg font-medium hover:border-dit-accent transition-colors"
            >
              Close
            </button>
          </div>
        </div>
      )}

      {/* Toast notification */}
      {toast && (
        <div className="fixed bottom-14 left-1/2 -translate-x-1/2 px-4 py-2 bg-dit-surface
                        border border-dit-border rounded-lg shadow-xl text-sm text-dit-text
                        animate-fade-in z-50">
          {toast}
        </div>
      )}
    </div>
  );
}
