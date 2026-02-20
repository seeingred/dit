use serde::{Deserialize, Serialize};

use super::enums::{BlendMode, PaintType, ScaleMode};
use super::primitives::{Color, Transform, Vector};

// ─── ColorStop ───────────────────────────────────────────────────────────────

/// A position + color pair for gradient definitions.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ColorStop {
    #[serde(default)]
    pub position: f64,
    #[serde(default)]
    pub color: Color,
}

// ─── ImageFilters ────────────────────────────────────────────────────────────

/// Image adjustment filters applied to image paints.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exposure: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contrast: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub saturation: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tint: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub highlights: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadows: Option<f64>,
}

// ─── Paint ───────────────────────────────────────────────────────────────────

/// A fill or stroke paint. Covers solid colors, gradients, and image fills.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Paint {
    /// The paint type (SOLID, GRADIENT_LINEAR, IMAGE, etc.).
    #[serde(rename = "type", default = "default_paint_type")]
    pub paint_type: PaintType,

    /// Whether this paint is visible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,

    /// Opacity multiplier [0, 1].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,

    /// Solid color (for SOLID paints).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    /// Blend mode of this paint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_mode: Option<BlendMode>,

    /// Three handle positions defining a gradient (for gradient paints).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gradient_handle_positions: Option<Vec<Vector>>,

    /// Color stops for gradient paints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gradient_stops: Option<Vec<ColorStop>>,

    /// Scale mode for image paints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale_mode: Option<ScaleMode>,

    /// Affine transform applied to image paints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_transform: Option<Transform>,

    /// Image content hash (asset reference, e.g. "sha256:<hash>" in DIT canonical form).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_ref: Option<String>,

    /// GIF asset reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gif_ref: Option<String>,

    /// Image adjustment filters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<ImageFilters>,

    /// Rotation in degrees for image fills.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f64>,
}

fn default_paint_type() -> PaintType {
    PaintType::Solid
}

// ─── PaintOverride ───────────────────────────────────────────────────────────

/// Override entry in `fillsOverrideTable`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaintOverride {
    #[serde(default)]
    pub fills: Vec<Paint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit_fill_style_id: Option<String>,
}
