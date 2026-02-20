//! Core type definitions for DIT's canonical design format.
//!
//! This module contains every type needed to represent a Figma design file
//! losslessly: primitives, enums, paints, effects, typography, layout,
//! vector paths, nodes, components, styles, project containers, repository
//! metadata, and the plugin interface.

pub mod component;
pub mod effect;
pub mod enums;
pub mod export;
pub mod layout;
pub mod node;
pub mod paint;
pub mod paths;
pub mod primitives;
pub mod project;
pub mod repository;
pub mod style;
pub mod typography;
pub mod vector;

// ─── Re-exports ──────────────────────────────────────────────────────────────
// Flat re-export so consumers can write `use dit_core::types::DitNode;`

// Primitives
pub use primitives::{ArcData, Color, Rect, Size, StrokeWeights, Vector};
pub type Transform = primitives::Transform;

// Enums
pub use enums::{
    AxisSizingMode, BlendMode, BooleanOperationType, ChangeType, ComponentPropertyType,
    ConstraintType, CounterAxisAlignContent, CounterAxisAlignItems, DesignPlatform, EasingType,
    EffectType, ExportConstraintType, ExportFormat, HandleMirroring, HyperlinkType,
    LayoutGridAlignment, LayoutGridPattern, LayoutAlign, LayoutMode, LayoutPositioning,
    LayoutSizing, LayoutWrap, LineHeightUnit, MaskType, NodeType, OverflowDirection, PaintType,
    PrimaryAxisAlignItems, ScaleMode, StrokeAlign, StrokeCap, StrokeJoin, StyleType,
    TextAlignHorizontal, TextAlignVertical, TextAutoResize, TextCase, TextDecoration,
    TextTruncation, WindingRule,
};

// Paint & effects
pub use paint::{ColorStop, ImageFilters, Paint, PaintOverride};
pub use effect::Effect;

// Typography
pub use typography::{Hyperlink, TypeStyle};

// Layout
pub use layout::{Guide, GuideAxis, LayoutConstraint, LayoutGrid};

// Vector
pub use vector::{VectorNetwork, VectorPath, VectorRegion, VectorSegment, VectorVertex};

// Export
pub use export::{ExportConstraint, ExportSetting};

// Component
pub use component::{
    ComponentMetadata, ComponentPropertyDefinition, ComponentProperty,
    ComponentSetMetadata, DocumentationLink, Override, PreferredValueType,
    ComponentPropertyPreferredValue,
};

// Style
pub use style::StyleDefinition;

// Node
pub use node::DitNode;

// Project / snapshot
pub use project::{DitPage, DitProject, DitSnapshot};

// Repository
pub use repository::{DitBranch, DitCommitMeta, DitConfig, DitLock, DitStatus, DitStatusChange};

// Paths
pub use paths::{
    DitPaths, asset_path, asset_ref, filename_to_node_id, node_id_to_filename, page_path,
    parse_asset_ref,
};
