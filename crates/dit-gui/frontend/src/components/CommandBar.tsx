import { useState } from "react";

interface CommandBarProps {
  onCommand: (command: string) => void;
}

export function CommandBar({ onCommand }: CommandBarProps) {
  const [input, setInput] = useState("");
  const [history, setHistory] = useState<string[]>([]);
  const [historyIdx, setHistoryIdx] = useState(-1);

  const handleSubmit = () => {
    const cmd = input.trim();
    if (!cmd) return;

    setHistory((prev) => [cmd, ...prev]);
    setHistoryIdx(-1);
    onCommand(cmd);
    setInput("");
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleSubmit();
    } else if (e.key === "ArrowUp") {
      e.preventDefault();
      if (historyIdx < history.length - 1) {
        const idx = historyIdx + 1;
        setHistoryIdx(idx);
        setInput(history[idx]);
      }
    } else if (e.key === "ArrowDown") {
      e.preventDefault();
      if (historyIdx > 0) {
        const idx = historyIdx - 1;
        setHistoryIdx(idx);
        setInput(history[idx]);
      } else {
        setHistoryIdx(-1);
        setInput("");
      }
    }
  };

  return (
    <div className="flex items-center gap-2 px-4 py-2 border-t border-dit-border bg-dit-surface/30">
      <span className="text-xs font-mono text-dit-accent select-none">dit&gt;</span>
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Type a command... (commit, push, pull, status, log)"
        className="flex-1 bg-transparent text-sm text-dit-text placeholder:text-dit-text-muted
                   focus:outline-none font-mono"
      />
    </div>
  );
}
