import { useState, useEffect, useRef } from "react";
import { getPreviewImage } from "../commands";
import type { CommitInfo } from "../types";

interface CommitListProps {
  commits: CommitInfo[];
  selectedHash: string | null;
  diffSelection: [string | null, string | null];
  onSelect: (hash: string) => void;
  onDiffToggle: (hash: string) => void;
}

function formatDate(iso: string): string {
  const d = new Date(iso);
  const now = new Date();
  const diffMs = now.getTime() - d.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffMins < 1) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return d.toLocaleDateString();
}

function Thumbnail({ hash }: { hash: string }) {
  const [src, setSrc] = useState<string | null>(null);
  const [loaded, setLoaded] = useState(false);
  const attempted = useRef(false);

  useEffect(() => {
    if (attempted.current) return;
    attempted.current = true;
    getPreviewImage(hash)
      .then((data) => {
        if (data) setSrc(data);
      })
      .catch(() => {});
  }, [hash]);

  if (!src) {
    return (
      <div className="w-9 h-9 rounded bg-dit-border/50 flex items-center justify-center shrink-0">
        <svg className="w-4 h-4 text-dit-text-muted/40" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
            d="M2.25 15.75l5.159-5.159a2.25 2.25 0 013.182 0l5.159 5.159m-1.5-1.5l1.409-1.409a2.25 2.25 0 013.182 0l2.909 2.909M3.75 21h16.5A2.25 2.25 0 0022.5 18.75V5.25A2.25 2.25 0 0020.25 3H3.75A2.25 2.25 0 001.5 5.25v13.5A2.25 2.25 0 003.75 21z" />
        </svg>
      </div>
    );
  }

  return (
    <div className="w-9 h-9 rounded bg-dit-border/30 shrink-0 overflow-hidden">
      <img
        src={`data:image/png;base64,${src}`}
        alt=""
        className={`w-full h-full object-cover transition-opacity ${loaded ? "opacity-100" : "opacity-0"}`}
        onLoad={() => setLoaded(true)}
      />
    </div>
  );
}

export function CommitList({
  commits,
  selectedHash,
  diffSelection,
  onSelect,
  onDiffToggle,
}: CommitListProps) {
  return (
    <div className="flex flex-col h-full">
      <div className="px-4 py-3 border-b border-dit-border">
        <h2 className="text-sm font-semibold text-dit-text">History</h2>
        {diffSelection[0] && (
          <p className="text-xs text-dit-text-muted mt-1">
            Comparing: {diffSelection[0]?.slice(0, 7)}
            {diffSelection[1] ? ` vs ${diffSelection[1].slice(0, 7)}` : " — select second commit"}
          </p>
        )}
      </div>
      <div className="flex-1 overflow-y-auto">
        {commits.length === 0 ? (
          <div className="flex items-center justify-center h-full text-dit-text-muted text-sm">
            No commits yet
          </div>
        ) : (
          <ul className="py-1">
            {commits.map((c) => {
              const isSelected = selectedHash === c.hash;
              const isDiffA = diffSelection[0] === c.hash;
              const isDiffB = diffSelection[1] === c.hash;
              return (
                <li
                  key={c.hash}
                  onClick={() => onSelect(c.hash)}
                  onDoubleClick={() => onDiffToggle(c.hash)}
                  className={`px-4 py-3 cursor-pointer border-l-2 transition-colors
                    ${isSelected ? "border-dit-accent bg-dit-surface" : "border-transparent hover:bg-dit-surface/50"}
                    ${isDiffA || isDiffB ? "ring-1 ring-inset ring-dit-accent/40" : ""}`}
                >
                  <div className="flex items-start gap-3">
                    <Thumbnail hash={c.hash} />
                    <div className="min-w-0 flex-1">
                      <div className="flex items-start justify-between gap-2">
                        <p className="text-sm text-dit-text truncate">{c.message}</p>
                        <div className="flex items-center gap-1.5 shrink-0">
                          {c.branch && (
                            <span className="text-[10px] px-1.5 py-0.5 rounded bg-dit-accent/20 text-dit-accent font-medium">
                              {c.branch}
                            </span>
                          )}
                          <span className="text-[10px] font-mono text-dit-text-muted">
                            {c.hash.slice(0, 7)}
                          </span>
                        </div>
                      </div>
                      <div className="flex items-center gap-2 mt-1">
                        <span className="text-xs text-dit-text-muted">{c.author}</span>
                        <span className="text-xs text-dit-text-muted">·</span>
                        <span className="text-xs text-dit-text-muted">{formatDate(c.date)}</span>
                      </div>
                    </div>
                  </div>
                </li>
              );
            })}
          </ul>
        )}
      </div>
      <div className="px-4 py-2 border-t border-dit-border">
        <p className="text-[10px] text-dit-text-muted">
          Double-click commits to compare
        </p>
      </div>
    </div>
  );
}
