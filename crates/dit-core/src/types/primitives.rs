use serde::{Deserialize, Serialize};

// ─── Color ───────────────────────────────────────────────────────────────────

/// RGBA color with components in the range [0, 1].
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Color {
    #[serde(default)]
    pub r: f64,
    #[serde(default)]
    pub g: f64,
    #[serde(default)]
    pub b: f64,
    #[serde(default = "default_one")]
    pub a: f64,
}

fn default_one() -> f64 {
    1.0
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        }
    }
}

// ─── Vector ──────────────────────────────────────────────────────────────────

/// 2D vector / point.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Vector {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
}

// ─── Size ────────────────────────────────────────────────────────────────────

/// Width × height dimensions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Size {
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub height: f64,
}

// ─── Rectangle ───────────────────────────────────────────────────────────────

/// Axis-aligned bounding box (used for absoluteBoundingBox, absoluteRenderBounds).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct Rect {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
    #[serde(default)]
    pub width: f64,
    #[serde(default)]
    pub height: f64,
}

// ─── Transform ───────────────────────────────────────────────────────────────

/// 2D affine transform matrix stored as [[a, b, tx], [c, d, ty]].
/// Matches Figma's `relativeTransform` representation.
pub type Transform = [[f64; 3]; 2];

// ─── ArcData ─────────────────────────────────────────────────────────────────

/// Ellipse arc parameters (for partial ellipses).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ArcData {
    #[serde(default)]
    pub starting_angle: f64,
    #[serde(default)]
    pub ending_angle: f64,
    #[serde(default)]
    pub inner_radius: f64,
}

// ─── StrokeWeights ───────────────────────────────────────────────────────────

/// Individual stroke weights per side (when not uniform).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StrokeWeights {
    #[serde(default)]
    pub top: f64,
    #[serde(default)]
    pub right: f64,
    #[serde(default)]
    pub bottom: f64,
    #[serde(default)]
    pub left: f64,
}
