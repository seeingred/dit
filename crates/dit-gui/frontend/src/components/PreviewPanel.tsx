import { useState, useEffect } from "react";
import { getPreviewImage, getCommitTree } from "../commands";
import type { CommitInfo, TreeNode } from "../types";
import { TreeViewer } from "./TreeViewer";

interface PreviewPanelProps {
  commit: CommitInfo | null;
}

export function PreviewPanel({ commit }: PreviewPanelProps) {
  const [imageData, setImageData] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [tree, setTree] = useState<TreeNode[]>([]);
  const [treeLoading, setTreeLoading] = useState(false);
  const [showTree, setShowTree] = useState(true);

  useEffect(() => {
    if (!commit) {
      setImageData(null);
      setTree([]);
      return;
    }

    let cancelled = false;
    setLoading(true);
    setTreeLoading(true);

    getPreviewImage(commit.hash)
      .then((data) => {
        if (!cancelled) setImageData(data);
      })
      .catch(() => {
        if (!cancelled) setImageData(null);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });

    getCommitTree()
      .then((data) => {
        if (!cancelled) setTree(data);
      })
      .catch(() => {
        if (!cancelled) setTree([]);
      })
      .finally(() => {
        if (!cancelled) setTreeLoading(false);
      });

    return () => {
      cancelled = true;
    };
  }, [commit?.hash]);

  if (!commit) {
    return (
      <div className="flex items-center justify-center h-full text-dit-text-muted">
        <div className="text-center">
          <svg className="w-12 h-12 mx-auto mb-3 opacity-30" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
              d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5A2.25 2.25 0 0022.5 18.75V5.25A2.25 2.25 0 0020.25 3H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z" />
          </svg>
          <p className="text-sm">Select a commit to preview</p>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="px-4 py-3 border-b border-dit-border">
        <div className="flex items-center justify-between">
          <h2 className="text-sm font-semibold text-dit-text">Preview</h2>
          <span className="text-xs font-mono text-dit-text-muted">
            {commit.hash.slice(0, 7)}
          </span>
        </div>
        <p className="text-xs text-dit-text-muted mt-0.5 truncate">{commit.message}</p>
      </div>

      {/* Image area */}
      <div className="flex-1 overflow-auto p-4 flex items-center justify-center min-h-0">
        {loading ? (
          <div className="text-dit-text-muted text-sm animate-pulse">Loading preview...</div>
        ) : imageData ? (
          <img
            src={`data:image/png;base64,${imageData}`}
            alt={commit.message}
            className="max-w-full max-h-full object-contain rounded shadow-lg"
          />
        ) : (
          <div className="text-center text-dit-text-muted">
            <svg className="w-16 h-16 mx-auto mb-3 opacity-20" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5A2.25 2.25 0 0022.5 18.75V5.25A2.25 2.25 0 0020.25 3H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z" />
            </svg>
            <p className="text-sm">No preview available</p>
            <p className="text-xs mt-1">Preview images will be generated from design files</p>
          </div>
        )}
      </div>

      {/* Tree viewer section */}
      <div className="border-t border-dit-border flex flex-col min-h-0">
        <button
          onClick={() => setShowTree(!showTree)}
          className="flex items-center justify-between px-4 py-2 hover:bg-dit-surface/50 transition-colors"
        >
          <h3 className="text-xs font-semibold text-dit-text">Design Tree</h3>
          <svg
            className={`w-3 h-3 text-dit-text-muted transition-transform ${showTree ? "rotate-180" : ""}`}
            fill="none" viewBox="0 0 24 24" stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
          </svg>
        </button>
        {showTree && (
          <div className="flex-1 overflow-y-auto max-h-64">
            {treeLoading ? (
              <div className="flex items-center justify-center py-4 text-dit-text-muted text-xs animate-pulse">
                Loading tree...
              </div>
            ) : (
              <TreeViewer tree={tree} />
            )}
          </div>
        )}
      </div>

      {/* Commit details */}
      <div className="px-4 py-3 border-t border-dit-border text-xs text-dit-text-muted space-y-1">
        <div className="flex justify-between">
          <span>Author</span>
          <span className="text-dit-text">{commit.author}</span>
        </div>
        <div className="flex justify-between">
          <span>Date</span>
          <span className="text-dit-text">{new Date(commit.date).toLocaleString()}</span>
        </div>
        {commit.branch && (
          <div className="flex justify-between">
            <span>Branch</span>
            <span className="text-dit-accent">{commit.branch}</span>
          </div>
        )}
      </div>
    </div>
  );
}
