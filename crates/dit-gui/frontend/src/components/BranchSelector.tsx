import { useState, useRef, useEffect } from "react";
import { checkoutRef, createBranch } from "../commands";

interface BranchSelectorProps {
  branches: string[];
  currentBranch: string;
  onBranchChange: (branch: string) => void;
}

export function BranchSelector({
  branches,
  currentBranch,
  onBranchChange,
}: BranchSelectorProps) {
  const [open, setOpen] = useState(false);
  const [creating, setCreating] = useState(false);
  const [newName, setNewName] = useState("");
  const dropdownRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setOpen(false);
        setCreating(false);
      }
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  const handleSwitch = async (branch: string) => {
    try {
      await checkoutRef(branch);
      onBranchChange(branch);
    } catch {
      // TODO: show error
    }
    setOpen(false);
  };

  const handleCreate = async () => {
    if (!newName.trim()) return;
    try {
      await createBranch(newName);
      await checkoutRef(newName);
      onBranchChange(newName);
      setNewName("");
      setCreating(false);
    } catch {
      // TODO: show error
    }
    setOpen(false);
  };

  return (
    <div className="relative" ref={dropdownRef}>
      <button
        onClick={() => setOpen(!open)}
        className="flex items-center gap-2 px-3 py-1.5 rounded text-sm
                   bg-dit-surface border border-dit-border
                   hover:border-dit-accent transition-colors"
      >
        <svg className="w-4 h-4 text-dit-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5}
            d="M3.75 6A2.25 2.25 0 016 3.75h2.25A2.25 2.25 0 0110.5 6v2.25a2.25 2.25 0 01-2.25 2.25H6a2.25 2.25 0 01-2.25-2.25V6zM3.75 15.75A2.25 2.25 0 016 13.5h2.25a2.25 2.25 0 012.25 2.25V18a2.25 2.25 0 01-2.25 2.25H6A2.25 2.25 0 013.75 18v-2.25zM13.5 6a2.25 2.25 0 012.25-2.25H18A2.25 2.25 0 0120.25 6v2.25A2.25 2.25 0 0118 10.5h-2.25a2.25 2.25 0 01-2.25-2.25V6z" />
        </svg>
        <span className="text-dit-text font-medium">{currentBranch}</span>
        <svg className="w-3 h-3 text-dit-text-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19.5 8.25l-7.5 7.5-7.5-7.5" />
        </svg>
      </button>

      {open && (
        <div className="absolute top-full left-0 mt-1 w-56 bg-dit-surface border border-dit-border
                        rounded-lg shadow-xl z-50 overflow-hidden">
          <div className="py-1">
            {branches.map((b) => (
              <button
                key={b}
                onClick={() => handleSwitch(b)}
                className={`w-full text-left px-3 py-2 text-sm transition-colors
                  ${b === currentBranch
                    ? "bg-dit-accent/10 text-dit-accent"
                    : "text-dit-text hover:bg-dit-bg"
                  }`}
              >
                {b}
                {b === currentBranch && (
                  <span className="ml-2 text-xs text-dit-accent">current</span>
                )}
              </button>
            ))}
          </div>
          <div className="border-t border-dit-border p-2">
            {creating ? (
              <div className="flex gap-1">
                <input
                  type="text"
                  value={newName}
                  onChange={(e) => setNewName(e.target.value)}
                  onKeyDown={(e) => e.key === "Enter" && handleCreate()}
                  placeholder="branch-name"
                  autoFocus
                  className="flex-1 bg-dit-bg border border-dit-border rounded px-2 py-1 text-xs
                             text-dit-text placeholder:text-dit-text-muted
                             focus:outline-none focus:border-dit-accent"
                />
                <button
                  onClick={handleCreate}
                  className="px-2 py-1 bg-dit-accent text-white text-xs rounded hover:bg-dit-accent-hover"
                >
                  Create
                </button>
              </div>
            ) : (
              <button
                onClick={() => setCreating(true)}
                className="w-full text-left px-2 py-1.5 text-xs text-dit-text-muted
                           hover:text-dit-text transition-colors rounded hover:bg-dit-bg"
              >
                + New branch
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
