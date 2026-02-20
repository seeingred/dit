//! Test fixtures: realistic DitSnapshot with multiple pages, node types,
//! components, styles, and assets for integration tests.

use std::collections::HashMap;

use dit_core::types::*;

/// Create a minimal valid DitProject.
pub fn project() -> DitProject {
    DitProject {
        file_key: "test_file_key_abc123".into(),
        name: "Test Design Project".into(),
        last_modified: "2025-06-15T10:30:00Z".into(),
        version: "42".into(),
        platform: DesignPlatform::Figma,
        schema_version: 1,
        thumbnail_url: None,
        editor_type: Some("figma".into()),
        role: Some("editor".into()),
    }
}

/// Build a DitNode with just the required fields set.
fn base_node(id: &str, name: &str, node_type: NodeType) -> DitNode {
    DitNode {
        id: id.into(),
        name: name.into(),
        node_type,
        visible: Some(true),
        locked: Some(false),
        opacity: Some(1.0),
        blend_mode: Some(BlendMode::Normal),
        absolute_bounding_box: Some(Rect {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        }),
        absolute_render_bounds: None,
        relative_transform: Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        size: Some(Vector { x: 100.0, y: 100.0 }),
        rotation: Some(0.0),
        fills: None,
        strokes: None,
        stroke_weight: None,
        individual_stroke_weights: None,
        stroke_align: None,
        stroke_cap: None,
        stroke_join: None,
        stroke_dashes: None,
        stroke_miter_angle: None,
        effects: None,
        corner_radius: None,
        corner_smoothing: None,
        rectangle_corner_radii: None,
        constraints: None,
        layout_mode: None,
        primary_axis_sizing_mode: None,
        counter_axis_sizing_mode: None,
        primary_axis_align_items: None,
        counter_axis_align_items: None,
        counter_axis_align_content: None,
        padding_left: None,
        padding_right: None,
        padding_top: None,
        padding_bottom: None,
        item_spacing: None,
        counter_axis_spacing: None,
        layout_wrap: None,
        item_reverse_z_index: None,
        strokes_included_in_layout: None,
        layout_align: None,
        layout_grow: None,
        layout_positioning: None,
        layout_sizing_horizontal: None,
        layout_sizing_vertical: None,
        min_width: None,
        max_width: None,
        min_height: None,
        max_height: None,
        clips_content: None,
        layout_grids: None,
        overflow_direction: None,
        guides: None,
        number_of_fixed_children: None,
        children: None,
        is_mask: None,
        mask_type: None,
        characters: None,
        style: None,
        character_style_overrides: None,
        style_override_table: None,
        component_id: None,
        component_properties: None,
        component_property_definitions: None,
        overrides: None,
        boolean_operation: None,
        count: None,
        inner_radius: None,
        arc_data: None,
        fill_geometry: None,
        stroke_geometry: None,
        vector_network: None,
        export_settings: None,
        fill_style_id: None,
        stroke_style_id: None,
        text_style_id: None,
        effect_style_id: None,
        grid_style_id: None,
        background_color: None,
        background: None,
        transition_node_id: None,
        transition_duration: None,
        transition_easing: None,
        fills_override_table: None,
        plugin_data: None,
        shared_plugin_data: None,
        component_property_references: None,
        extra: HashMap::new(),
    }
}

/// A frame with auto-layout and children.
pub fn frame_with_autolayout() -> DitNode {
    let mut node = base_node("1:10", "Auto Layout Frame", NodeType::Frame);
    node.absolute_bounding_box = Some(Rect { x: 50.0, y: 50.0, width: 400.0, height: 300.0 });
    node.size = Some(Vector { x: 400.0, y: 300.0 });
    node.clips_content = Some(true);
    node.layout_mode = Some(LayoutMode::Vertical);
    node.primary_axis_sizing_mode = Some(AxisSizingMode::Auto);
    node.counter_axis_sizing_mode = Some(AxisSizingMode::Fixed);
    node.primary_axis_align_items = Some(PrimaryAxisAlignItems::Min);
    node.counter_axis_align_items = Some(CounterAxisAlignItems::Center);
    node.padding_left = Some(16.0);
    node.padding_right = Some(16.0);
    node.padding_top = Some(24.0);
    node.padding_bottom = Some(24.0);
    node.item_spacing = Some(12.0);
    node.corner_radius = Some(8.0);
    node.fills = Some(vec![Paint {
        paint_type: PaintType::Solid,
        visible: Some(true),
        opacity: Some(1.0),
        color: Some(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
        blend_mode: None,
        gradient_handle_positions: None,
        gradient_stops: None,
        scale_mode: None,
        image_transform: None,
        image_ref: None,
        gif_ref: None,
        filters: None,
        rotation: None,
    }]);
    node.effects = Some(vec![Effect {
        effect_type: EffectType::DropShadow,
        visible: Some(true),
        radius: Some(4.0),
        color: Some(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.1 }),
        blend_mode: Some(BlendMode::Normal),
        offset: Some(Vector { x: 0.0, y: 2.0 }),
        spread: Some(0.0),
        show_shadow_behind_node: None,
    }]);
    node.children = Some(vec![
        rectangle_node(),
        text_node(),
        ellipse_node(),
    ]);
    node
}

/// A rectangle with rounded corners and gradient fill.
pub fn rectangle_node() -> DitNode {
    let mut node = base_node("1:20", "Rounded Rect", NodeType::Rectangle);
    node.absolute_bounding_box = Some(Rect { x: 66.0, y: 74.0, width: 368.0, height: 80.0 });
    node.size = Some(Vector { x: 368.0, y: 80.0 });
    node.rectangle_corner_radii = Some([12.0, 12.0, 4.0, 4.0]);
    node.fills = Some(vec![Paint {
        paint_type: PaintType::GradientLinear,
        visible: Some(true),
        opacity: Some(1.0),
        color: None,
        blend_mode: None,
        gradient_handle_positions: Some(vec![
            Vector { x: 0.0, y: 0.5 },
            Vector { x: 1.0, y: 0.5 },
        ]),
        gradient_stops: Some(vec![
            ColorStop { position: 0.0, color: Color { r: 0.2, g: 0.4, b: 0.8, a: 1.0 } },
            ColorStop { position: 1.0, color: Color { r: 0.8, g: 0.2, b: 0.6, a: 1.0 } },
        ]),
        scale_mode: None,
        image_transform: None,
        image_ref: None,
        gif_ref: None,
        filters: None,
        rotation: None,
    }]);
    node.strokes = Some(vec![Paint {
        paint_type: PaintType::Solid,
        visible: Some(true),
        opacity: Some(0.5),
        color: Some(Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }),
        blend_mode: None,
        gradient_handle_positions: None,
        gradient_stops: None,
        scale_mode: None,
        image_transform: None,
        image_ref: None,
        gif_ref: None,
        filters: None,
        rotation: None,
    }]);
    node.stroke_weight = Some(1.0);
    node.stroke_align = Some(StrokeAlign::Inside);
    node
}

/// A text node with mixed formatting.
pub fn text_node() -> DitNode {
    let mut node = base_node("1:30", "Styled Text", NodeType::Text);
    node.absolute_bounding_box = Some(Rect { x: 66.0, y: 166.0, width: 368.0, height: 40.0 });
    node.size = Some(Vector { x: 368.0, y: 40.0 });
    node.characters = Some("Hello, World!".into());
    node.style = Some(TypeStyle {
        font_family: Some("Inter".into()),
        font_post_script_name: Some("Inter-Regular".into()),
        font_weight: Some(400.0),
        font_size: Some(16.0),
        italic: Some(false),
        text_align_horizontal: Some(TextAlignHorizontal::Left),
        text_align_vertical: Some(TextAlignVertical::Top),
        letter_spacing: Some(0.0),
        line_height_px: Some(24.0),
        line_height_percent: Some(150.0),
        line_height_percent_font_size: None,
        line_height_unit: Some(LineHeightUnit::Pixels),
        paragraph_spacing: Some(0.0),
        paragraph_indent: None,
        list_spacing: None,
        text_auto_resize: Some(TextAutoResize::Height),
        text_decoration: Some(TextDecoration::None),
        text_case: Some(TextCase::Original),
        text_truncation: None,
        max_lines: None,
        fills: Some(vec![Paint {
            paint_type: PaintType::Solid,
            visible: Some(true),
            opacity: Some(1.0),
            color: Some(Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 }),
            blend_mode: None,
            gradient_handle_positions: None,
            gradient_stops: None,
            scale_mode: None,
            image_transform: None,
            image_ref: None,
            gif_ref: None,
            filters: None,
            rotation: None,
        }]),
        hyperlink: None,
        opentype_flags: None,
    });
    // Mixed formatting: "Hello" in bold, ", World!" in regular
    node.character_style_overrides = Some(vec![1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0]);
    let mut overrides = HashMap::new();
    overrides.insert("1".to_string(), TypeStyle {
        font_family: Some("Inter".into()),
        font_post_script_name: Some("Inter-Bold".into()),
        font_weight: Some(700.0),
        font_size: None,
        italic: None,
        text_align_horizontal: None,
        text_align_vertical: None,
        letter_spacing: None,
        line_height_px: None,
        line_height_percent: None,
        line_height_percent_font_size: None,
        line_height_unit: None,
        paragraph_spacing: None,
        paragraph_indent: None,
        list_spacing: None,
        text_auto_resize: None,
        text_decoration: None,
        text_case: None,
        text_truncation: None,
        max_lines: None,
        fills: None,
        hyperlink: None,
        opentype_flags: None,
    });
    node.style_override_table = Some(overrides);
    node
}

/// An ellipse (partial arc).
pub fn ellipse_node() -> DitNode {
    let mut node = base_node("1:40", "Progress Ring", NodeType::Ellipse);
    node.absolute_bounding_box = Some(Rect { x: 66.0, y: 218.0, width: 48.0, height: 48.0 });
    node.size = Some(Vector { x: 48.0, y: 48.0 });
    node.arc_data = Some(ArcData {
        starting_angle: 0.0,
        ending_angle: 4.71238898038,
        inner_radius: 0.8,
    });
    node.fills = Some(vec![Paint {
        paint_type: PaintType::Solid,
        visible: Some(true),
        opacity: Some(1.0),
        color: Some(Color { r: 0.2, g: 0.6, b: 1.0, a: 1.0 }),
        blend_mode: None,
        gradient_handle_positions: None,
        gradient_stops: None,
        scale_mode: None,
        image_transform: None,
        image_ref: None,
        gif_ref: None,
        filters: None,
        rotation: None,
    }]);
    node
}

/// A component node.
pub fn component_node() -> DitNode {
    let mut node = base_node("1:50", "Button Component", NodeType::Component);
    node.absolute_bounding_box = Some(Rect { x: 0.0, y: 0.0, width: 120.0, height: 40.0 });
    node.size = Some(Vector { x: 120.0, y: 40.0 });
    node.clips_content = Some(true);
    node.corner_radius = Some(6.0);
    node.layout_mode = Some(LayoutMode::Horizontal);
    node.primary_axis_align_items = Some(PrimaryAxisAlignItems::Center);
    node.counter_axis_align_items = Some(CounterAxisAlignItems::Center);
    node.padding_left = Some(16.0);
    node.padding_right = Some(16.0);
    node.padding_top = Some(8.0);
    node.padding_bottom = Some(8.0);
    node.fills = Some(vec![Paint {
        paint_type: PaintType::Solid,
        visible: Some(true),
        opacity: Some(1.0),
        color: Some(Color { r: 0.2, g: 0.4, b: 0.9, a: 1.0 }),
        blend_mode: None,
        gradient_handle_positions: None,
        gradient_stops: None,
        scale_mode: None,
        image_transform: None,
        image_ref: None,
        gif_ref: None,
        filters: None,
        rotation: None,
    }]);

    // Button label child
    let mut label = base_node("1:51", "Label", NodeType::Text);
    label.absolute_bounding_box = Some(Rect { x: 16.0, y: 8.0, width: 88.0, height: 24.0 });
    label.size = Some(Vector { x: 88.0, y: 24.0 });
    label.characters = Some("Click me".into());
    label.style = Some(TypeStyle {
        font_family: Some("Inter".into()),
        font_post_script_name: Some("Inter-Medium".into()),
        font_weight: Some(500.0),
        font_size: Some(14.0),
        italic: None,
        text_align_horizontal: Some(TextAlignHorizontal::Center),
        text_align_vertical: Some(TextAlignVertical::Center),
        letter_spacing: Some(0.0),
        line_height_px: Some(20.0),
        line_height_percent: None,
        line_height_percent_font_size: None,
        line_height_unit: Some(LineHeightUnit::Pixels),
        paragraph_spacing: None,
        paragraph_indent: None,
        list_spacing: None,
        text_auto_resize: None,
        text_decoration: Some(TextDecoration::None),
        text_case: None,
        text_truncation: None,
        max_lines: None,
        fills: Some(vec![Paint {
            paint_type: PaintType::Solid,
            visible: Some(true),
            opacity: Some(1.0),
            color: Some(Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 }),
            blend_mode: None,
            gradient_handle_positions: None,
            gradient_stops: None,
            scale_mode: None,
            image_transform: None,
            image_ref: None,
            gif_ref: None,
            filters: None,
            rotation: None,
        }]),
        hyperlink: None,
        opentype_flags: None,
    });

    node.children = Some(vec![label]);
    node
}

/// An instance of the button component.
pub fn instance_node() -> DitNode {
    let mut node = base_node("1:60", "Button Instance", NodeType::Instance);
    node.absolute_bounding_box = Some(Rect { x: 200.0, y: 200.0, width: 120.0, height: 40.0 });
    node.size = Some(Vector { x: 120.0, y: 40.0 });
    node.component_id = Some("1:50".into());
    node.overrides = Some(vec![Override {
        id: "1:51".into(),
        overridden_fields: vec!["characters".into()],
    }]);
    node
}

/// A vector node with SVG path data.
pub fn vector_node() -> DitNode {
    let mut node = base_node("1:70", "Star Icon", NodeType::Vector);
    node.absolute_bounding_box = Some(Rect { x: 300.0, y: 0.0, width: 24.0, height: 24.0 });
    node.size = Some(Vector { x: 24.0, y: 24.0 });
    node.fill_geometry = Some(vec![VectorPath {
        path: "M 12 2 L 15.09 8.26 L 22 9.27 L 17 14.14 L 18.18 21.02 L 12 17.77 L 5.82 21.02 L 7 14.14 L 2 9.27 L 8.91 8.26 Z".into(),
        winding_rule: WindingRule::Nonzero,
        overriding_id: None,
    }]);
    node.fills = Some(vec![Paint {
        paint_type: PaintType::Solid,
        visible: Some(true),
        opacity: Some(1.0),
        color: Some(Color { r: 1.0, g: 0.8, b: 0.0, a: 1.0 }),
        blend_mode: None,
        gradient_handle_positions: None,
        gradient_stops: None,
        scale_mode: None,
        image_transform: None,
        image_ref: None,
        gif_ref: None,
        filters: None,
        rotation: None,
    }]);
    node
}

/// A boolean operation node.
pub fn boolean_op_node() -> DitNode {
    let mut node = base_node("1:80", "Union Shape", NodeType::BooleanOperation);
    node.boolean_operation = Some(BooleanOperationType::Union);

    let mut child1 = base_node("1:81", "Circle", NodeType::Ellipse);
    child1.absolute_bounding_box = Some(Rect { x: 0.0, y: 0.0, width: 40.0, height: 40.0 });
    child1.size = Some(Vector { x: 40.0, y: 40.0 });

    let mut child2 = base_node("1:82", "Rect", NodeType::Rectangle);
    child2.absolute_bounding_box = Some(Rect { x: 20.0, y: 0.0, width: 40.0, height: 40.0 });
    child2.size = Some(Vector { x: 40.0, y: 40.0 });

    node.children = Some(vec![child1, child2]);
    node
}

/// A node with an image fill (references an asset).
pub fn image_node(asset_ref: &str) -> DitNode {
    let mut node = base_node("1:90", "Photo", NodeType::Rectangle);
    node.absolute_bounding_box = Some(Rect { x: 0.0, y: 400.0, width: 200.0, height: 150.0 });
    node.size = Some(Vector { x: 200.0, y: 150.0 });
    node.fills = Some(vec![Paint {
        paint_type: PaintType::Image,
        visible: Some(true),
        opacity: Some(1.0),
        color: None,
        blend_mode: None,
        gradient_handle_positions: None,
        gradient_stops: None,
        scale_mode: Some(ScaleMode::Fill),
        image_transform: Some([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]),
        image_ref: Some(asset_ref.into()),
        gif_ref: None,
        filters: None,
        rotation: None,
    }]);
    node
}

/// Build a realistic multi-page snapshot with various node types.
pub fn realistic_snapshot() -> DitSnapshot {
    DitSnapshot {
        project: project(),
        pages: vec![
            DitPage {
                id: "0:1".into(),
                name: "Home".into(),
                background_color: None,
                children: vec![
                    frame_with_autolayout(),
                    vector_node(),
                    boolean_op_node(),
                ],
            },
            DitPage {
                id: "0:2".into(),
                name: "Components".into(),
                background_color: None,
                children: vec![
                    component_node(),
                    instance_node(),
                ],
            },
        ],
        components: Some({
            let mut map = HashMap::new();
            map.insert("1:50".into(), ComponentMetadata {
                key: "button_component_key".into(),
                name: "Button Component".into(),
                description: "A primary action button".into(),
                documentation_links: None,
                component_set_id: None,
            });
            map
        }),
        component_sets: None,
        styles: Some({
            let mut map = HashMap::new();
            map.insert("fill_primary".into(), StyleDefinition {
                key: "fill_primary".into(),
                name: "Primary / Blue".into(),
                style_type: StyleType::Fill,
                description: Some("Primary brand color".into()),
            });
            map
        }),
    }
}
