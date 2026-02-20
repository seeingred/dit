use serde::{Deserialize, Serialize};

use super::enums::{HandleMirroring, StrokeCap, StrokeJoin, WindingRule};
use super::paint::Paint;
use super::primitives::Vector as Vec2;

// ─── VectorPath ──────────────────────────────────────────────────────────────

/// An SVG-style path string with winding rule.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VectorPath {
    /// SVG path data string (e.g. "M 0 0 L 100 100 Z").
    #[serde(default)]
    pub path: String,
    /// Fill rule for this path.
    #[serde(default = "default_winding_rule")]
    pub winding_rule: WindingRule,
    /// Optional ID of the node whose geometry this overrides.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overriding_id: Option<String>,
}

fn default_winding_rule() -> WindingRule {
    WindingRule::Nonzero
}

// ─── VectorVertex ────────────────────────────────────────────────────────────

/// A vertex in a vector network.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VectorVertex {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_cap: Option<StrokeCap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_join: Option<StrokeJoin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle_mirroring: Option<HandleMirroring>,
}

// ─── VectorSegment ───────────────────────────────────────────────────────────

/// An edge between two vertices in a vector network.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VectorSegment {
    /// Index of the start vertex.
    #[serde(default)]
    pub start: usize,
    /// Index of the end vertex.
    #[serde(default)]
    pub end: usize,
    /// Bézier tangent at the start vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tangent_start: Option<Vec2>,
    /// Bézier tangent at the end vertex.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tangent_end: Option<Vec2>,
}

// ─── VectorRegion ────────────────────────────────────────────────────────────

/// A filled region in a vector network, defined by loops of segment indices.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VectorRegion {
    #[serde(default = "default_winding_rule")]
    pub winding_rule: WindingRule,
    /// Each inner Vec is a loop of segment indices forming a closed path.
    #[serde(default)]
    pub loops: Vec<Vec<usize>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fills: Option<Vec<Paint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_style_id: Option<String>,
}

// ─── VectorNetwork ───────────────────────────────────────────────────────────

/// A complete vector network (vertices + edges + regions).
/// This is Figma's native vector representation, richer than SVG paths.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VectorNetwork {
    #[serde(default)]
    pub vertices: Vec<VectorVertex>,
    #[serde(default)]
    pub segments: Vec<VectorSegment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regions: Option<Vec<VectorRegion>>,
}
