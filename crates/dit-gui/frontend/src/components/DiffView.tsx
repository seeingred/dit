import { useState, useEffect } from "react";
import { getDiffPreviews, getDiffTrees } from "../commands";
import { DiffTreeViewer } from "./DiffTreeViewer";
import type { CommitInfo, DiffResult, DiffTreeResult } from "../types";

interface DiffViewProps {
  commitA: CommitInfo | null;
  commitB: CommitInfo | null;
  onClose: () => void;
}

export function DiffView({ commitA, commitB, onClose }: DiffViewProps) {
  const [diff, setDiff] = useState<DiffResult | null>(null);
  const [diffTrees, setDiffTrees] = useState<DiffTreeResult | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (!commitA || !commitB) {
      setDiff(null);
      setDiffTrees(null);
      return;
    }

    let cancelled = false;
    setLoading(true);

    Promise.all([
      getDiffPreviews(commitA.hash, commitB.hash),
      getDiffTrees(commitA.hash, commitB.hash),
    ])
      .then(([previewResult, treeResult]) => {
        if (!cancelled) {
          setDiff(previewResult);
          setDiffTrees(treeResult);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setDiff(null);
          setDiffTrees(null);
        }
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [commitA?.hash, commitB?.hash]);

  if (!commitA || !commitB) return null;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-4 py-3 border-b border-dit-border flex items-center justify-between">
        <div>
          <h2 className="text-sm font-semibold text-dit-text">Visual Diff</h2>
          <p className="text-xs text-dit-text-muted mt-0.5">
            <span className="font-mono">{commitA.hash.slice(0, 7)}</span>
            {" vs "}
            <span className="font-mono">{commitB.hash.slice(0, 7)}</span>
          </p>
        </div>
        <button
          onClick={onClose}
          className="p-1.5 rounded hover:bg-dit-surface text-dit-text-muted hover:text-dit-text transition-colors"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {loading ? (
        <div className="flex-1 flex items-center justify-center text-dit-text-muted animate-pulse">
          Loading diff...
        </div>
      ) : (
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Preview images */}
          <div className="flex border-b border-dit-border" style={{ minHeight: "200px", maxHeight: "45%" }}>
            {/* Before preview */}
            <div className="flex-1 border-r border-dit-border flex flex-col">
              <div className="px-3 py-2 border-b border-dit-border bg-dit-surface/50">
                <p className="text-xs font-medium text-dit-text truncate">{commitA.message}</p>
                <p className="text-[10px] text-dit-text-muted">{commitA.author} · {commitA.hash.slice(0, 7)}</p>
              </div>
              <div className="flex-1 overflow-auto p-4 flex items-center justify-center">
                {diff?.before_image ? (
                  <img
                    src={`data:image/png;base64,${diff.before_image}`}
                    alt="Before"
                    className="max-w-full max-h-full object-contain rounded"
                  />
                ) : (
                  <p className="text-sm text-dit-text-muted">No preview available</p>
                )}
              </div>
            </div>

            {/* After preview */}
            <div className="flex-1 flex flex-col">
              <div className="px-3 py-2 border-b border-dit-border bg-dit-surface/50">
                <p className="text-xs font-medium text-dit-text truncate">{commitB.message}</p>
                <p className="text-[10px] text-dit-text-muted">{commitB.author} · {commitB.hash.slice(0, 7)}</p>
              </div>
              <div className="flex-1 overflow-auto p-4 flex items-center justify-center">
                {diff?.after_image ? (
                  <img
                    src={`data:image/png;base64,${diff.after_image}`}
                    alt="After"
                    className="max-w-full max-h-full object-contain rounded"
                  />
                ) : (
                  <p className="text-sm text-dit-text-muted">No preview available</p>
                )}
              </div>
            </div>
          </div>

          {/* Tree diff section */}
          <div className="flex-1 flex overflow-hidden">
            {/* Before tree */}
            <div className="flex-1 border-r border-dit-border flex flex-col overflow-hidden">
              <div className="px-3 py-1.5 border-b border-dit-border bg-dit-surface/50 flex items-center gap-2">
                <span className="text-[10px] font-semibold text-dit-text-muted uppercase tracking-wider">Tree</span>
                {diffTrees && (
                  <DiffLegend />
                )}
              </div>
              <div className="flex-1 overflow-y-auto">
                {diffTrees ? (
                  <DiffTreeViewer tree={diffTrees.before} />
                ) : (
                  <div className="flex items-center justify-center py-6 text-dit-text-muted text-xs">
                    No tree data
                  </div>
                )}
              </div>
            </div>

            {/* After tree */}
            <div className="flex-1 flex flex-col overflow-hidden">
              <div className="px-3 py-1.5 border-b border-dit-border bg-dit-surface/50">
                <span className="text-[10px] font-semibold text-dit-text-muted uppercase tracking-wider">Tree</span>
              </div>
              <div className="flex-1 overflow-y-auto">
                {diffTrees ? (
                  <DiffTreeViewer tree={diffTrees.after} />
                ) : (
                  <div className="flex items-center justify-center py-6 text-dit-text-muted text-xs">
                    No tree data
                  </div>
                )}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

function DiffLegend() {
  return (
    <div className="flex items-center gap-2 ml-auto">
      <span className="flex items-center gap-1 text-[10px]">
        <span className="w-2 h-2 rounded-sm bg-green-500/30" />
        <span className="text-dit-text-muted">Added</span>
      </span>
      <span className="flex items-center gap-1 text-[10px]">
        <span className="w-2 h-2 rounded-sm bg-red-500/30" />
        <span className="text-dit-text-muted">Removed</span>
      </span>
      <span className="flex items-center gap-1 text-[10px]">
        <span className="w-2 h-2 rounded-sm bg-amber-500/30" />
        <span className="text-dit-text-muted">Modified</span>
      </span>
    </div>
  );
}
