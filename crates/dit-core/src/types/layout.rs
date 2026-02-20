use serde::{Deserialize, Serialize};

use super::enums::{ConstraintType, LayoutGridAlignment, LayoutGridPattern};
use super::primitives::Color;

// ─── LayoutConstraint ────────────────────────────────────────────────────────

/// Positioning constraints (how the node is pinned relative to its parent).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutConstraint {
    #[serde(default = "default_constraint")]
    pub vertical: ConstraintType,
    #[serde(default = "default_constraint")]
    pub horizontal: ConstraintType,
}

fn default_constraint() -> ConstraintType {
    ConstraintType::Min
}

// ─── LayoutGrid ──────────────────────────────────────────────────────────────

/// A layout grid applied to a frame (columns, rows, or pixel grid).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutGrid {
    #[serde(default = "default_grid_pattern")]
    pub pattern: LayoutGridPattern,
    #[serde(default)]
    pub section_size: f64,
    #[serde(default)]
    pub visible: bool,
    #[serde(default)]
    pub color: Color,
    #[serde(default = "default_grid_alignment")]
    pub alignment: LayoutGridAlignment,
    #[serde(default)]
    pub gutter_size: f64,
    #[serde(default)]
    pub offset: f64,
    #[serde(default)]
    pub count: i32,
}

fn default_grid_pattern() -> LayoutGridPattern {
    LayoutGridPattern::Grid
}

fn default_grid_alignment() -> LayoutGridAlignment {
    LayoutGridAlignment::Min
}

// ─── Guide ───────────────────────────────────────────────────────────────────

/// A ruler guide on a frame.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Guide {
    #[serde(default = "default_guide_axis")]
    pub axis: GuideAxis,
    #[serde(default)]
    pub offset: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GuideAxis {
    X,
    Y,
    #[serde(other)]
    Unknown,
}

fn default_guide_axis() -> GuideAxis {
    GuideAxis::X
}
