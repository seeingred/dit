import { useState } from "react";
import type { DiffTreeNode } from "../types";

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

const CHANGE_STYLES: Record<string, string> = {
  added: "bg-green-500/15 text-green-400",
  removed: "bg-red-500/15 text-red-400",
  modified: "bg-amber-500/15 text-amber-400",
};

const CHANGE_BADGE: Record<string, { label: string; cls: string }> = {
  added: { label: "+", cls: "text-green-400" },
  removed: { label: "-", cls: "text-red-400" },
  modified: { label: "~", cls: "text-amber-400" },
};

/** Check if a node or any descendant has changes. */
function hasChangesInSubtree(node: DiffTreeNode): boolean {
  if (node.change_type) return true;
  return node.children.some(hasChangesInSubtree);
}

function DiffTreeNodeItem({
  node,
  depth,
}: {
  node: DiffTreeNode;
  depth: number;
}) {
  const subtreeHasChanges = hasChangesInSubtree(node);
  const [expanded, setExpanded] = useState(
    depth < 1 || subtreeHasChanges,
  );
  const hasChildren = node.children.length > 0;
  const icon = TYPE_ICONS[node.node_type] ?? "?";
  const changeCls = node.change_type
    ? CHANGE_STYLES[node.change_type] ?? ""
    : "";
  const badge = node.change_type ? CHANGE_BADGE[node.change_type] : null;

  return (
    <div>
      <div
        className={`flex items-center gap-1.5 py-0.5 pr-2 cursor-default rounded transition-colors ${
          hasChildren ? "cursor-pointer" : ""
        } ${changeCls || "hover:bg-dit-surface/50"}`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={() => hasChildren && setExpanded(!expanded)}
      >
        {hasChildren ? (
          <svg
            className={`w-3 h-3 text-dit-text-muted shrink-0 transition-transform ${
              expanded ? "rotate-90" : ""
            }`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 5l7 7-7 7"
            />
          </svg>
        ) : (
          <span className="w-3 shrink-0" />
        )}
        <span className="text-[9px] font-bold px-1 py-px rounded bg-dit-accent/15 text-dit-accent shrink-0 leading-tight">
          {icon}
        </span>
        <span className="text-xs text-dit-text truncate">{node.name}</span>
        {badge && (
          <span className={`text-xs font-bold shrink-0 ${badge.cls}`}>
            {badge.label}
          </span>
        )}
        {hasChildren && (
          <span className="text-[10px] text-dit-text-muted ml-auto shrink-0">
            {node.children.length}
          </span>
        )}
      </div>
      {expanded && hasChildren && (
        <div>
          {node.children.map((child, i) => (
            <DiffTreeNodeItem
              key={`${child.id}-${i}`}
              node={child}
              depth={depth + 1}
            />
          ))}
        </div>
      )}
    </div>
  );
}

interface DiffTreeViewerProps {
  tree: DiffTreeNode[];
}

export function DiffTreeViewer({ tree }: DiffTreeViewerProps) {
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
        <DiffTreeNodeItem key={`${node.id}-${i}`} node={node} depth={0} />
      ))}
    </div>
  );
}
