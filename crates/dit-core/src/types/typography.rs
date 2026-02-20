use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::enums::{
    HyperlinkType, LineHeightUnit, TextAlignHorizontal, TextAlignVertical, TextAutoResize,
    TextCase, TextDecoration, TextTruncation,
};
use super::paint::Paint;

// ─── Hyperlink ───────────────────────────────────────────────────────────────

/// A hyperlink on a text range.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Hyperlink {
    #[serde(rename = "type", default = "default_hyperlink_type")]
    pub hyperlink_type: HyperlinkType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
}

fn default_hyperlink_type() -> HyperlinkType {
    HyperlinkType::Url
}

// ─── TypeStyle ───────────────────────────────────────────────────────────────

/// Complete text style describing font, alignment, spacing, and decorations.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TypeStyle {
    // ── Font ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_post_script_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub italic: Option<bool>,

    // ── Alignment ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align_horizontal: Option<TextAlignHorizontal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align_vertical: Option<TextAlignVertical>,

    // ── Spacing ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height_px: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height_percent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height_percent_font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height_unit: Option<LineHeightUnit>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_spacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paragraph_indent: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub list_spacing: Option<f64>,

    // ── Resize ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_auto_resize: Option<TextAutoResize>,

    // ── Decoration / transform ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<TextDecoration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_case: Option<TextCase>,

    // ── Truncation ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_truncation: Option<TextTruncation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_lines: Option<i32>,

    // ── Fills ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fills: Option<Vec<Paint>>,

    // ── Hyperlink ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hyperlink: Option<Hyperlink>,

    // ── OpenType features ──
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opentype_flags: Option<HashMap<String, i32>>,
}
