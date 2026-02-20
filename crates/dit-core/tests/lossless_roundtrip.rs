//! Integration test: lossless round-trip through canonical serialization.
//!
//! Creates snapshots with every node type, writes them to disk via
//! `write_snapshot`, reads them back via `read_snapshot`, and asserts the
//! deserialized data matches the original.

mod fixtures;

use dit_core::canonical::{self, serialize};
use dit_core::types::*;
use std::collections::HashMap;
use tempfile::TempDir;

// ── Full snapshot round-trip ─────────────────────────────────────────────────

#[test]
fn realistic_snapshot_round_trips_through_disk() {
    let snapshot = fixtures::realistic_snapshot();
    let tmp = TempDir::new().unwrap();

    canonical::write_snapshot(tmp.path(), &snapshot).unwrap();
    let loaded = canonical::read_snapshot(tmp.path()).unwrap();

    // Project metadata
    assert_eq!(loaded.project.file_key, snapshot.project.file_key);
    assert_eq!(loaded.project.name, snapshot.project.name);
    assert_eq!(loaded.project.last_modified, snapshot.project.last_modified);
    assert_eq!(loaded.project.version, snapshot.project.version);
    assert_eq!(loaded.project.schema_version, snapshot.project.schema_version);
    assert_eq!(loaded.project.editor_type, snapshot.project.editor_type);
    assert_eq!(loaded.project.role, snapshot.project.role);

    // Pages
    assert_eq!(loaded.pages.len(), snapshot.pages.len());
    for (orig_page, loaded_page) in snapshot.pages.iter().zip(loaded.pages.iter()) {
        assert_eq!(orig_page.id, loaded_page.id);
        assert_eq!(orig_page.name, loaded_page.name);
        assert_eq!(orig_page.children.len(), loaded_page.children.len());
    }

    // Components
    assert!(loaded.components.is_some());
    let loaded_comps = loaded.components.as_ref().unwrap();
    let orig_comps = snapshot.components.as_ref().unwrap();
    assert_eq!(loaded_comps.len(), orig_comps.len());
    for (key, orig_meta) in orig_comps {
        let loaded_meta = loaded_comps.get(key).expect("component key missing");
        assert_eq!(orig_meta.name, loaded_meta.name);
        assert_eq!(orig_meta.key, loaded_meta.key);
        assert_eq!(orig_meta.description, loaded_meta.description);
    }

    // Styles
    assert!(loaded.styles.is_some());
    let loaded_styles = loaded.styles.as_ref().unwrap();
    let orig_styles = snapshot.styles.as_ref().unwrap();
    assert_eq!(loaded_styles.len(), orig_styles.len());
}

// ── Node-level round-trips ──────────────────────────────────────────────────

/// Serialize a DitNode to canonical JSON and back, assert equality.
fn node_round_trip(node: &DitNode) -> DitNode {
    let json = serialize(node).unwrap();
    canonical::deserialize::<DitNode>(&json).unwrap()
}

#[test]
fn frame_with_autolayout_round_trips() {
    let orig = fixtures::frame_with_autolayout();
    let loaded = node_round_trip(&orig);

    assert_eq!(orig.id, loaded.id);
    assert_eq!(orig.name, loaded.name);
    assert_eq!(orig.node_type, loaded.node_type);
    assert_eq!(orig.layout_mode, loaded.layout_mode);
    assert_eq!(orig.primary_axis_sizing_mode, loaded.primary_axis_sizing_mode);
    assert_eq!(orig.counter_axis_sizing_mode, loaded.counter_axis_sizing_mode);
    assert_eq!(orig.primary_axis_align_items, loaded.primary_axis_align_items);
    assert_eq!(orig.counter_axis_align_items, loaded.counter_axis_align_items);
    assert_eq!(orig.padding_left, loaded.padding_left);
    assert_eq!(orig.padding_right, loaded.padding_right);
    assert_eq!(orig.padding_top, loaded.padding_top);
    assert_eq!(orig.padding_bottom, loaded.padding_bottom);
    assert_eq!(orig.item_spacing, loaded.item_spacing);
    assert_eq!(orig.corner_radius, loaded.corner_radius);
    assert_eq!(orig.clips_content, loaded.clips_content);

    // Effects
    let orig_effects = orig.effects.as_ref().unwrap();
    let loaded_effects = loaded.effects.as_ref().unwrap();
    assert_eq!(orig_effects.len(), loaded_effects.len());
    assert_eq!(orig_effects[0].effect_type, loaded_effects[0].effect_type);
    assert_eq!(orig_effects[0].radius, loaded_effects[0].radius);

    // Children
    let orig_children = orig.children.as_ref().unwrap();
    let loaded_children = loaded.children.as_ref().unwrap();
    assert_eq!(orig_children.len(), loaded_children.len());
}

#[test]
fn rectangle_with_gradient_round_trips() {
    let orig = fixtures::rectangle_node();
    let loaded = node_round_trip(&orig);

    assert_eq!(orig.rectangle_corner_radii, loaded.rectangle_corner_radii);
    assert_eq!(orig.stroke_weight, loaded.stroke_weight);
    assert_eq!(orig.stroke_align, loaded.stroke_align);

    // Gradient fill
    let orig_fill = &orig.fills.as_ref().unwrap()[0];
    let loaded_fill = &loaded.fills.as_ref().unwrap()[0];
    assert_eq!(orig_fill.paint_type, loaded_fill.paint_type);
    assert_eq!(
        orig_fill.gradient_handle_positions.as_ref().unwrap().len(),
        loaded_fill.gradient_handle_positions.as_ref().unwrap().len()
    );
    assert_eq!(
        orig_fill.gradient_stops.as_ref().unwrap().len(),
        loaded_fill.gradient_stops.as_ref().unwrap().len()
    );
}

#[test]
fn text_with_mixed_formatting_round_trips() {
    let orig = fixtures::text_node();
    let loaded = node_round_trip(&orig);

    assert_eq!(orig.characters, loaded.characters);

    // Base style
    let orig_style = orig.style.as_ref().unwrap();
    let loaded_style = loaded.style.as_ref().unwrap();
    assert_eq!(orig_style.font_family, loaded_style.font_family);
    assert_eq!(orig_style.font_weight, loaded_style.font_weight);
    assert_eq!(orig_style.font_size, loaded_style.font_size);
    assert_eq!(orig_style.text_align_horizontal, loaded_style.text_align_horizontal);
    assert_eq!(orig_style.line_height_unit, loaded_style.line_height_unit);
    assert_eq!(orig_style.text_auto_resize, loaded_style.text_auto_resize);

    // Style overrides
    assert_eq!(orig.character_style_overrides, loaded.character_style_overrides);
    let orig_table = orig.style_override_table.as_ref().unwrap();
    let loaded_table = loaded.style_override_table.as_ref().unwrap();
    assert_eq!(orig_table.len(), loaded_table.len());
    let orig_override = orig_table.get("1").unwrap();
    let loaded_override = loaded_table.get("1").unwrap();
    assert_eq!(orig_override.font_weight, loaded_override.font_weight);
    assert_eq!(orig_override.font_post_script_name, loaded_override.font_post_script_name);
}

#[test]
fn ellipse_arc_data_round_trips() {
    let orig = fixtures::ellipse_node();
    let loaded = node_round_trip(&orig);

    let orig_arc = orig.arc_data.as_ref().unwrap();
    let loaded_arc = loaded.arc_data.as_ref().unwrap();
    assert!((orig_arc.starting_angle - loaded_arc.starting_angle).abs() < 1e-6);
    assert!((orig_arc.ending_angle - loaded_arc.ending_angle).abs() < 1e-6);
    assert!((orig_arc.inner_radius - loaded_arc.inner_radius).abs() < 1e-6);
}

#[test]
fn component_and_instance_round_trip() {
    let orig_comp = fixtures::component_node();
    let loaded_comp = node_round_trip(&orig_comp);
    assert_eq!(orig_comp.node_type, loaded_comp.node_type);
    assert_eq!(orig_comp.children.as_ref().unwrap().len(), loaded_comp.children.as_ref().unwrap().len());

    let orig_inst = fixtures::instance_node();
    let loaded_inst = node_round_trip(&orig_inst);
    assert_eq!(orig_inst.component_id, loaded_inst.component_id);
    assert_eq!(
        orig_inst.overrides.as_ref().unwrap().len(),
        loaded_inst.overrides.as_ref().unwrap().len()
    );
    let orig_ovr = &orig_inst.overrides.as_ref().unwrap()[0];
    let loaded_ovr = &loaded_inst.overrides.as_ref().unwrap()[0];
    assert_eq!(orig_ovr.id, loaded_ovr.id);
    assert_eq!(orig_ovr.overridden_fields, loaded_ovr.overridden_fields);
}

#[test]
fn vector_path_data_round_trips() {
    let orig = fixtures::vector_node();
    let loaded = node_round_trip(&orig);

    let orig_geo = orig.fill_geometry.as_ref().unwrap();
    let loaded_geo = loaded.fill_geometry.as_ref().unwrap();
    assert_eq!(orig_geo.len(), loaded_geo.len());
    assert_eq!(orig_geo[0].path, loaded_geo[0].path);
    assert_eq!(orig_geo[0].winding_rule, loaded_geo[0].winding_rule);
}

#[test]
fn boolean_operation_round_trips() {
    let orig = fixtures::boolean_op_node();
    let loaded = node_round_trip(&orig);

    assert_eq!(orig.boolean_operation, loaded.boolean_operation);
    assert_eq!(
        orig.children.as_ref().unwrap().len(),
        loaded.children.as_ref().unwrap().len()
    );
}

#[test]
fn image_fill_reference_round_trips() {
    let orig = fixtures::image_node("sha256:abc123def456");
    let loaded = node_round_trip(&orig);

    let orig_fill = &orig.fills.as_ref().unwrap()[0];
    let loaded_fill = &loaded.fills.as_ref().unwrap()[0];
    assert_eq!(orig_fill.paint_type, loaded_fill.paint_type);
    assert_eq!(orig_fill.image_ref, loaded_fill.image_ref);
    assert_eq!(orig_fill.scale_mode, loaded_fill.scale_mode);
    assert_eq!(orig_fill.image_transform, loaded_fill.image_transform);
}

// ── Extra fields (catch-all) round-trip ─────────────────────────────────────

#[test]
fn unknown_fields_preserved_in_extra() {
    // Simulate a future Figma property by injecting it into the JSON
    let mut node = fixtures::rectangle_node();
    node.extra.insert(
        "futureProperty".to_string(),
        serde_json::json!({ "nested": true, "value": 42 }),
    );

    let loaded = node_round_trip(&node);
    assert_eq!(
        loaded.extra.get("futureProperty"),
        Some(&serde_json::json!({ "nested": true, "value": 42 }))
    );
}

// ── Project metadata round-trip ─────────────────────────────────────────────

#[test]
fn project_metadata_round_trips() {
    let orig = fixtures::project();
    let json = serialize(&orig).unwrap();
    let loaded: DitProject = canonical::deserialize(&json).unwrap();

    assert_eq!(orig.file_key, loaded.file_key);
    assert_eq!(orig.name, loaded.name);
    assert_eq!(orig.last_modified, loaded.last_modified);
    assert_eq!(orig.version, loaded.version);
    assert_eq!(orig.schema_version, loaded.schema_version);
    assert_eq!(orig.thumbnail_url, loaded.thumbnail_url);
    assert_eq!(orig.editor_type, loaded.editor_type);
    assert_eq!(orig.role, loaded.role);
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn empty_snapshot_round_trips() {
    let snapshot = DitSnapshot {
        project: DitProject {
            file_key: "empty".into(),
            name: "Empty".into(),
            last_modified: "2025-01-01T00:00:00Z".into(),
            version: "0".into(),
            platform: DesignPlatform::Figma,
            schema_version: 1,
            thumbnail_url: None,
            editor_type: None,
            role: None,
        },
        pages: vec![],
        components: None,
        component_sets: None,
        styles: None,
    };

    let tmp = TempDir::new().unwrap();
    canonical::write_snapshot(tmp.path(), &snapshot).unwrap();
    let loaded = canonical::read_snapshot(tmp.path()).unwrap();

    assert_eq!(loaded.project.file_key, "empty");
    assert!(loaded.pages.is_empty());
    assert!(loaded.components.is_none());
    assert!(loaded.styles.is_none());
}

#[test]
fn snapshot_with_all_optional_collections() {
    let mut snapshot = fixtures::realistic_snapshot();
    // Add component sets too
    snapshot.component_sets = Some({
        let mut map = HashMap::new();
        map.insert("cs:1".into(), ComponentSetMetadata {
            key: "cs_key_1".into(),
            name: "Button Set".into(),
            description: "Primary button variants".into(),
            documentation_links: None,
        });
        map
    });

    let tmp = TempDir::new().unwrap();
    canonical::write_snapshot(tmp.path(), &snapshot).unwrap();
    let loaded = canonical::read_snapshot(tmp.path()).unwrap();

    assert!(loaded.component_sets.is_some());
    let loaded_cs = loaded.component_sets.as_ref().unwrap();
    assert_eq!(loaded_cs.len(), 1);
    assert_eq!(loaded_cs.get("cs:1").unwrap().name, "Button Set");
}
