import { useState, useRef, useCallback } from "react";
import { commitChanges, push, pull } from "../commands";
import type { RepoStatus } from "../types";

interface ActionToolbarProps {
  status: RepoStatus;
  isCommitting?: boolean;
  onCommitStart?: () => void;
  onCommitEnd?: (success: boolean) => void;
  onAction: (message: string) => void;
  onMerge: () => void;
  onRestore: () => void;
}

export function ActionToolbar({
  status,
  isCommitting,
  onCommitStart,
  onCommitEnd,
  onAction,
  onMerge,
  onRestore,
}: ActionToolbarProps) {
  const [commitMsg, setCommitMsg] = useState("");
  const [showCommitInput, setShowCommitInput] = useState(false);
  const [isPushing, setIsPushing] = useState(false);
  const [isPulling, setIsPulling] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const autoResize = useCallback((el: HTMLTextAreaElement) => {
    el.style.height = "auto";
    const maxHeight = 4 * 24; // ~4 lines
    el.style.height = `${Math.min(el.scrollHeight, maxHeight)}px`;
  }, []);

  const handleCommit = async () => {
    if (!commitMsg.trim()) return;
    onCommitStart?.();
    try {
      await commitChanges(commitMsg);
      onCommitEnd?.(true);
      onAction(`Committed: ${commitMsg}`);
      setCommitMsg("");
      setShowCommitInput(false);
    } catch (e) {
      onCommitEnd?.(false);
      onAction(`Commit failed: ${e}`);
    }
  };

  const handlePush = async () => {
    setIsPushing(true);
    try {
      const result = await push();
      onAction(result);
    } catch (e) {
      onAction(`Push failed: ${e}`);
    } finally {
      setIsPushing(false);
    }
  };

  const handlePull = async () => {
    setIsPulling(true);
    try {
      const result = await pull();
      onAction(result);
    } catch (e) {
      onAction(`Pull failed: ${e}`);
    } finally {
      setIsPulling(false);
    }
  };

  return (
    <div className="flex items-center gap-2 px-4 py-2 border-b border-dit-border bg-dit-surface/30">
      {/* Commit */}
      {showCommitInput ? (
        <div className="flex items-start gap-2 flex-1">
          <textarea
            ref={textareaRef}
            value={commitMsg}
            onChange={(e) => {
              setCommitMsg(e.target.value);
              autoResize(e.target);
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                e.preventDefault();
                if (commitMsg.trim() && !isCommitting) handleCommit();
              }
            }}
            placeholder="Commit message..."
            autoFocus
            rows={1}
            disabled={isCommitting}
            className="flex-1 bg-dit-bg border border-dit-border rounded px-3 py-1.5 text-sm
                       text-dit-text placeholder:text-dit-text-muted
                       focus:outline-none focus:border-dit-accent transition-colors
                       disabled:opacity-40 resize-none overflow-y-auto leading-6"
            style={{ maxHeight: `${4 * 24}px` }}
          />
          <button
            onClick={handleCommit}
            disabled={!commitMsg.trim() || isCommitting}
            className="px-3 py-1.5 bg-dit-accent text-white text-sm rounded font-medium
                       hover:bg-dit-accent-hover transition-colors
                       disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Commit
          </button>
          <button
            onClick={() => setShowCommitInput(false)}
            disabled={isCommitting}
            className="px-3 py-1.5 text-dit-text-muted text-sm rounded
                       hover:text-dit-text transition-colors
                       disabled:opacity-40 disabled:cursor-not-allowed"
          >
            Cancel
          </button>
        </div>
      ) : (
        <>
          <ToolbarButton
            label="Commit"
            icon={
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                d="M12 9v6m3-3H9m12 0a9 9 0 11-18 0 9 9 0 0118 0z" />
            }
            onClick={() => setShowCommitInput(true)}
            accent={status.has_changes}
            disabled={isCommitting}
          />
          <ToolbarButton
            label="Restore"
            icon={
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                d="M9 15L3 9m0 0l6-6M3 9h12a6 6 0 010 12h-3" />
            }
            onClick={onRestore}
            disabled={isCommitting}
          />
          <div className="w-px h-5 bg-dit-border" />
          <ToolbarButton
            label={isPulling ? "Pulling..." : "Pull"}
            icon={
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5M16.5 12L12 16.5m0 0L7.5 12m4.5 4.5V3" />
            }
            onClick={handlePull}
            disabled={isCommitting || isPulling || isPushing}
          />
          <ToolbarButton
            label={isPushing ? "Pushing..." : "Push"}
            icon={
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                d="M3 16.5v2.25A2.25 2.25 0 005.25 21h13.5A2.25 2.25 0 0021 18.75V16.5m-13.5-9L12 3m0 0l4.5 4.5M12 3v13.5" />
            }
            onClick={handlePush}
            disabled={isCommitting || isPulling || isPushing}
          />
          <ToolbarButton
            label="Merge"
            icon={
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
                d="M7.5 21L3 16.5m0 0L7.5 12M3 16.5h13.5m0-13.5L21 7.5m0 0L16.5 12M21 7.5H7.5" />
            }
            onClick={onMerge}
            disabled
            title="Not implemented yet"
          />
        </>
      )}
    </div>
  );
}

function ToolbarButton({
  label,
  icon,
  onClick,
  accent,
  disabled,
  title,
}: {
  label: string;
  icon: React.ReactNode;
  onClick: () => void;
  accent?: boolean;
  disabled?: boolean;
  title?: string;
}) {
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      title={title}
      className={`flex items-center gap-1.5 px-3 py-1.5 rounded text-sm transition-colors
        disabled:opacity-40 disabled:cursor-not-allowed
        ${accent
          ? "bg-dit-accent/10 text-dit-accent hover:bg-dit-accent/20"
          : "text-dit-text-muted hover:text-dit-text hover:bg-dit-surface"
        }`}
    >
      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
        {icon}
      </svg>
      {label}
    </button>
  );
}
