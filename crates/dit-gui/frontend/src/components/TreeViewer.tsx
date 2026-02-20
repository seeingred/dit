import { useState } from "react";
import type { TreeNode } from "../types";

interface TreeViewerProps {
  tree: TreeNode[];
}

const TYPE_ICONS: Record<string, string> = {
  Page: "P",
  Frame: "F",
  Group: "G",
  Component: "C",
  ComponentSet: "CS",
  Instance: "I",
  Text: "T",
  Vector: "V",
  Rectangle: "R",
  Ellipse: "E",
  Line: "L",
  BooleanOperation: "B",
  Section: "S",
};

function TreeNodeItem({ node, depth }: { node: TreeNode; depth: number }) {
  const [expanded, setExpanded] = useState(depth < 2);
  const hasChildren = node.children.length > 0;
  const icon = TYPE_ICONS[node.node_type] ?? "?";

  return (
    <div>
      <div
        className={`flex items-center gap-1.5 py-0.5 pr-2 cursor-default hover:bg-dit-surface/50 rounded transition-colors ${
          hasChildren ? "cursor-pointer" : ""
        }`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={() => hasChildren && setExpanded(!expanded)}
      >
        {hasChildren ? (
          <svg
            className={`w-3 h-3 text-dit-text-muted shrink-0 transition-transform ${expanded ? "rotate-90" : ""}`}
            fill="none" viewBox="0 0 24 24" stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
          </svg>
        ) : (
          <span className="w-3 shrink-0" />
        )}
        <span className="text-[9px] font-bold px-1 py-px rounded bg-dit-accent/15 text-dit-accent shrink-0 leading-tight">
          {icon}
        </span>
        <span className="text-xs text-dit-text truncate">{node.name}</span>
        {hasChildren && (
          <span className="text-[10px] text-dit-text-muted ml-auto shrink-0">{node.children.length}</span>
        )}
      </div>
      {expanded && hasChildren && (
        <div>
          {node.children.map((child, i) => (
            <TreeNodeItem key={`${child.id}-${i}`} node={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}

export function TreeViewer({ tree }: TreeViewerProps) {
  if (tree.length === 0) {
    return (
      <div className="flex items-center justify-center py-6 text-dit-text-muted text-xs">
        No tree data available
      </div>
    );
  }

  return (
    <div className="overflow-y-auto py-1">
      {tree.map((node, i) => (
        <TreeNodeItem key={`${node.id}-${i}`} node={node} depth={0} />
      ))}
    </div>
  );
}
