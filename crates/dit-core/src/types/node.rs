use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::component::{ComponentProperty, ComponentPropertyDefinition, Override};
use super::effect::Effect;
use super::enums::{
    AxisSizingMode, BlendMode, BooleanOperationType, CounterAxisAlignContent,
    CounterAxisAlignItems, EasingType, LayoutAlign, LayoutMode, LayoutPositioning, LayoutSizing,
    LayoutWrap, MaskType, NodeType, OverflowDirection, PrimaryAxisAlignItems, StrokeAlign,
    StrokeCap, StrokeJoin,
};
use super::export::ExportSetting;
use super::layout::{Guide, LayoutConstraint, LayoutGrid};
use super::paint::{Paint, PaintOverride};
use super::primitives::{ArcData, Color, Rect, StrokeWeights, Transform, Vector};
use super::typography::TypeStyle;
use super::vector::{VectorNetwork, VectorPath};

fn default_node_type() -> NodeType {
    NodeType::Frame
}

/// The central node type representing any element in a Figma design tree.
///
/// Uses a flat struct with `Option` fields so that every Figma node type can be
/// represented losslessly with a single Rust type. Fields that don't apply to a
/// given `node_type` are simply `None`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitNode {
    // ── Identity ─────────────────────────────────────────────────────────
    /// Figma node ID (e.g. "1:2").
    #[serde(default)]
    pub id: String,
    /// Human-readable name.
    #[serde(default)]
    pub name: String,
    /// The node type discriminator.
    #[serde(rename = "type", default = "default_node_type")]
    pub node_type: NodeType,

    // ── Visibility / locking ─────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked: Option<bool>,

    // ── Blend / opacity ──────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_mode: Option<BlendMode>,

    // ── Geometry ─────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_bounding_box: Option<Rect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub absolute_render_bounds: Option<Rect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative_transform: Option<Transform>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Vector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<f64>,

    // ── Fills / strokes / effects ────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fills: Option<Vec<Paint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strokes: Option<Vec<Paint>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_weight: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub individual_stroke_weights: Option<StrokeWeights>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_align: Option<StrokeAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_cap: Option<StrokeCap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_join: Option<StrokeJoin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_dashes: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_miter_angle: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<Effect>>,

    // ── Corner radius ────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corner_smoothing: Option<f64>,
    /// Per-corner radii: [top-left, top-right, bottom-right, bottom-left].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rectangle_corner_radii: Option<[f64; 4]>,

    // ── Layout constraints ───────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<LayoutConstraint>,

    // ── Auto-layout (frame) ──────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_mode: Option<LayoutMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_axis_sizing_mode: Option<AxisSizingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_axis_sizing_mode: Option<AxisSizingMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_axis_align_items: Option<PrimaryAxisAlignItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_axis_align_items: Option<CounterAxisAlignItems>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_axis_align_content: Option<CounterAxisAlignContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_spacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counter_axis_spacing: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_wrap: Option<LayoutWrap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub item_reverse_z_index: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strokes_included_in_layout: Option<bool>,

    // ── Child layout positioning ─────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_align: Option<LayoutAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_grow: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_positioning: Option<LayoutPositioning>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_sizing_horizontal: Option<LayoutSizing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_sizing_vertical: Option<LayoutSizing>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<f64>,

    // ── Frame-specific ───────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clips_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout_grids: Option<Vec<LayoutGrid>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow_direction: Option<OverflowDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guides: Option<Vec<Guide>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number_of_fixed_children: Option<i32>,

    // ── Children ─────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<DitNode>>,

    // ── Mask ─────────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_mask: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mask_type: Option<MaskType>,

    // ── Text ─────────────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub characters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<TypeStyle>,
    /// Parallel array: for each character, an index into `style_override_table`.
    /// 0 means "use the base `style`".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub character_style_overrides: Option<Vec<i32>>,
    /// Map from style-override index (as string key) → TypeStyle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_override_table: Option<HashMap<String, TypeStyle>>,

    // ── Component / instance ─────────────────────────────────────────────
    /// For Instance nodes: the ID of the component this is an instance of.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,
    /// Resolved component property values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_properties: Option<HashMap<String, ComponentProperty>>,
    /// Component property definitions (on Component / ComponentSet nodes).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_property_definitions: Option<HashMap<String, ComponentPropertyDefinition>>,
    /// Overrides applied to this instance.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<Vec<Override>>,

    // ── Boolean operation ────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boolean_operation: Option<BooleanOperationType>,

    // ── Star / polygon ───────────────────────────────────────────────────
    /// Number of points (star, regular polygon).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<i32>,
    /// Inner radius ratio (star only, [0,1]).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inner_radius: Option<f64>,

    // ── Arc / ellipse ────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arc_data: Option<ArcData>,

    // ── Vector geometry ──────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_geometry: Option<Vec<VectorPath>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_geometry: Option<Vec<VectorPath>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vector_network: Option<VectorNetwork>,

    // ── Export settings ──────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub export_settings: Option<Vec<ExportSetting>>,

    // ── Style references (IDs of shared styles) ──────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill_style_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_style_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_style_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect_style_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub grid_style_id: Option<String>,

    // ── Background (CANVAS / page nodes) ─────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Vec<Paint>>,

    // ── Prototype / interaction ──────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_duration: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_easing: Option<EasingType>,

    // ── Section-specific ─────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fills_override_table: Option<HashMap<String, PaintOverride>>,

    // ── Plugin data ──────────────────────────────────────────────────────
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plugin_data: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_plugin_data: Option<serde_json::Value>,
    /// References to component properties that drive this node's values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_property_references: Option<HashMap<String, String>>,

    // ── Catch-all for forward-compatible lossless round-trip ─────────────
    /// Any Figma properties we don't explicitly model are preserved here.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
