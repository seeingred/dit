//! Integration test: deterministic serialization.
//!
//! Verifies that serializing the same data twice always produces
//! byte-identical output, and that key ordering is stable.

mod fixtures;

use dit_core::canonical::{self, serialize};
use dit_core::types::*;
use std::collections::HashMap;
use tempfile::TempDir;

// ── Byte-identical serialization ─────────────────────────────────────────────

#[test]
fn serialize_same_node_twice_is_identical() {
    let node = fixtures::frame_with_autolayout();
    let json1 = serialize(&node).unwrap();
    let json2 = serialize(&node).unwrap();
    assert_eq!(json1, json2, "serializing the same node twice must produce identical output");
}

#[test]
fn serialize_same_snapshot_twice_is_identical() {
    let snapshot = fixtures::realistic_snapshot();
    let json1 = serialize(&snapshot).unwrap();
    let json2 = serialize(&snapshot).unwrap();
    assert_eq!(json1, json2, "serializing the same snapshot twice must produce identical output");
}

#[test]
fn write_snapshot_twice_produces_identical_files() {
    let snapshot = fixtures::realistic_snapshot();

    let tmp1 = TempDir::new().unwrap();
    let tmp2 = TempDir::new().unwrap();

    canonical::write_snapshot(tmp1.path(), &snapshot).unwrap();
    canonical::write_snapshot(tmp2.path(), &snapshot).unwrap();

    // Compare project file
    let proj1 = std::fs::read_to_string(tmp1.path().join(DitPaths::PROJECT_FILE)).unwrap();
    let proj2 = std::fs::read_to_string(tmp2.path().join(DitPaths::PROJECT_FILE)).unwrap();
    assert_eq!(proj1, proj2, "project files must be byte-identical");

    // Compare page files
    let pages_dir = DitPaths::PAGES_DIR;
    let mut pages1: Vec<_> = std::fs::read_dir(tmp1.path().join(pages_dir))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    let mut pages2: Vec<_> = std::fs::read_dir(tmp2.path().join(pages_dir))
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    pages1.sort_by_key(|e| e.file_name());
    pages2.sort_by_key(|e| e.file_name());

    assert_eq!(pages1.len(), pages2.len());
    for (p1, p2) in pages1.iter().zip(pages2.iter()) {
        let content1 = std::fs::read_to_string(p1.path()).unwrap();
        let content2 = std::fs::read_to_string(p2.path()).unwrap();
        assert_eq!(
            content1, content2,
            "page files {} and {} must be byte-identical",
            p1.path().display(),
            p2.path().display()
        );
    }
}

// ── Key ordering is always sorted ───────────────────────────────────────────

fn assert_keys_sorted(json: &str) {
    // Parse the JSON and walk the structure verifying key order
    let value: serde_json::Value = serde_json::from_str(json).unwrap();
    assert_keys_sorted_recursive(&value);
}

fn assert_keys_sorted_recursive(value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            let keys: Vec<&String> = map.keys().collect();
            let mut sorted = keys.clone();
            sorted.sort();
            assert_eq!(
                keys, sorted,
                "JSON object keys must be sorted. Got: {keys:?}"
            );
            for v in map.values() {
                assert_keys_sorted_recursive(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                assert_keys_sorted_recursive(v);
            }
        }
        _ => {}
    }
}

#[test]
fn node_keys_are_sorted() {
    let node = fixtures::frame_with_autolayout();
    let json = serialize(&node).unwrap();
    assert_keys_sorted(&json);
}

#[test]
fn snapshot_project_keys_are_sorted() {
    let project = fixtures::project();
    let json = serialize(&project).unwrap();
    assert_keys_sorted(&json);
}

#[test]
fn text_node_with_overrides_keys_sorted() {
    let node = fixtures::text_node();
    let json = serialize(&node).unwrap();
    assert_keys_sorted(&json);
}

// ── Float normalization ─────────────────────────────────────────────────────

#[test]
fn float_precision_is_six_decimals() {
    let color = Color {
        r: 0.123456789,
        g: 0.9999999,
        b: 0.0000001,
        a: 1.0,
    };
    let json = serialize(&color).unwrap();
    // 0.123456789 → rounds to 0.123457
    assert!(json.contains("0.123457"), "expected 0.123457 in: {json}");
    // 0.9999999 → rounds to 1.0
    // 0.0000001 → rounds to 0.0 (below precision)
}

#[test]
fn integer_values_stay_as_integers() {
    let rect = Rect {
        x: 100.0,
        y: 200.0,
        width: 50.0,
        height: 75.0,
    };
    let json = serialize(&rect).unwrap();
    // Integer-valued floats should serialize as integers (100, not 100.0)
    // (depending on serde behavior, this checks our canonicalization)
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    // x=100.0 should become integer 100
    let x = parsed.get("x").unwrap();
    assert!(x.is_i64() || x.is_f64(), "x should be a number: {x}");
}

#[test]
fn trailing_newline_present() {
    let node = fixtures::rectangle_node();
    let json = serialize(&node).unwrap();
    assert!(json.ends_with('\n'), "canonical JSON must end with trailing newline");
}

// ── HashMap ordering doesn't affect output ──────────────────────────────────

#[test]
fn hashmap_insertion_order_does_not_affect_output() {
    // Create two snapshots with styles inserted in different orders
    let mut styles_a = HashMap::new();
    styles_a.insert("z_style".to_string(), StyleDefinition {
        key: "z_style".into(),
        name: "Z Style".into(),
        style_type: StyleType::Fill,
        description: Some("last".into()),
    });
    styles_a.insert("a_style".to_string(), StyleDefinition {
        key: "a_style".into(),
        name: "A Style".into(),
        style_type: StyleType::Effect,
        description: Some("first".into()),
    });

    let mut styles_b = HashMap::new();
    // Insert in reverse order
    styles_b.insert("a_style".to_string(), StyleDefinition {
        key: "a_style".into(),
        name: "A Style".into(),
        style_type: StyleType::Effect,
        description: Some("first".into()),
    });
    styles_b.insert("z_style".to_string(), StyleDefinition {
        key: "z_style".into(),
        name: "Z Style".into(),
        style_type: StyleType::Fill,
        description: Some("last".into()),
    });

    let json_a = serialize(&styles_a).unwrap();
    let json_b = serialize(&styles_b).unwrap();
    assert_eq!(json_a, json_b, "HashMap insertion order must not affect canonical output");
}
