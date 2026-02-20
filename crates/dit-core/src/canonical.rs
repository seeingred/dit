//! Canonical JSON serialization for DIT.
//!
//! Produces deterministic, diff-friendly JSON output:
//! - Keys sorted lexicographically at every nesting level
//! - Floats rounded to 6 decimal places
//! - 2-space indent, trailing newline
//! - Serializing the same data twice always yields byte-identical output

use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::types::{
    ComponentMetadata, ComponentSetMetadata, DitPage, DitProject, DitSnapshot, DitPaths,
    StyleDefinition, node_id_to_filename,
};

// ─── Core serialization ──────────────────────────────────────────────────────

/// Serialize any value to canonical JSON (sorted keys, stable floats, 2-space indent).
pub fn serialize<T: Serialize>(value: &T) -> Result<String> {
    // Step 1: serialize to serde_json::Value (preserves all data)
    let json_value = serde_json::to_value(value)
        .context("failed to convert to JSON value")?;

    // Step 2: sort keys and normalize floats
    let canonical = canonicalize_value(json_value);

    // Step 3: pretty-print with 2-space indent
    let mut output = serde_json::to_string_pretty(&canonical)
        .context("failed to pretty-print JSON")?;

    // Step 4: trailing newline
    output.push('\n');

    Ok(output)
}

/// Deserialize canonical JSON back into a typed value.
pub fn deserialize<T: DeserializeOwned>(json: &str) -> Result<T> {
    serde_json::from_str(json).context("failed to deserialize canonical JSON")
}

// ─── Value canonicalization ──────────────────────────────────────────────────

/// Recursively sort all object keys and normalize floats.
fn canonicalize_value(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            // Collect into a Vec, sort by key, rebuild as a serde_json::Map
            // (serde_json::Map preserves insertion order)
            let mut entries: Vec<(String, serde_json::Value)> = map.into_iter().collect();
            entries.sort_by(|(a, _), (b, _)| a.cmp(b));

            let mut sorted = serde_json::Map::with_capacity(entries.len());
            for (k, v) in entries {
                sorted.insert(k, canonicalize_value(v));
            }
            serde_json::Value::Object(sorted)
        }
        serde_json::Value::Array(arr) => {
            // Canonicalize each element but preserve array order (document order)
            serde_json::Value::Array(
                arr.into_iter().map(canonicalize_value).collect(),
            )
        }
        serde_json::Value::Number(n) => {
            // Normalize floats to 6 decimal places; integers pass through
            if let Some(f) = n.as_f64() {
                // Check if this is actually an integer value
                if f.fract() == 0.0 && f.abs() < (i64::MAX as f64) {
                    // Keep integers as integers
                    if let Some(i) = n.as_i64() {
                        return serde_json::Value::Number(serde_json::Number::from(i));
                    }
                }
                // Round float to 6 decimal places
                let rounded = (f * 1_000_000.0).round() / 1_000_000.0;
                serde_json::Number::from_f64(rounded)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null) // NaN/Inf → null
            } else {
                serde_json::Value::Number(n)
            }
        }
        // Strings, bools, null pass through unchanged
        other => other,
    }
}

// ─── Snapshot I/O ────────────────────────────────────────────────────────────

/// Write a snapshot as chunked files under `repo_root`:
///
/// - `dit.json`           – project metadata
/// - `dit.pages/<id>.json` – one file per page
/// - `dit.styles.json`     – shared styles
/// - `dit.components.json` – component + component set metadata
pub fn write_snapshot(repo_root: &Path, snapshot: &DitSnapshot) -> Result<()> {
    // Ensure directories exist
    let pages_dir = repo_root.join(DitPaths::PAGES_DIR);
    std::fs::create_dir_all(&pages_dir)
        .context("failed to create pages directory")?;

    // 1. Write project metadata
    let project_json = serialize(&snapshot.project)?;
    std::fs::write(repo_root.join(DitPaths::PROJECT_FILE), &project_json)
        .context("failed to write project file")?;

    // 2. Remove stale page files that are no longer in the snapshot
    if pages_dir.exists() {
        let current_page_filenames: std::collections::HashSet<String> = snapshot
            .pages
            .iter()
            .map(|p| format!("{}.json", node_id_to_filename(&p.id)))
            .collect();

        if let Ok(entries) = std::fs::read_dir(&pages_dir) {
            for entry in entries.flatten() {
                let fname = entry.file_name().to_string_lossy().to_string();
                if fname.ends_with(".json") && !current_page_filenames.contains(&fname) {
                    std::fs::remove_file(entry.path()).ok();
                }
            }
        }
    }

    // 3. Write each page
    for page in &snapshot.pages {
        let filename = format!("{}.json", node_id_to_filename(&page.id));
        let page_json = serialize(page)?;
        std::fs::write(pages_dir.join(&filename), &page_json)
            .with_context(|| format!("failed to write page {}", page.id))?;
    }

    // 4. Write styles (if any; remove stale file if no styles)
    let styles_path = repo_root.join(DitPaths::STYLES_FILE);
    if snapshot.styles.is_none() && styles_path.exists() {
        std::fs::remove_file(&styles_path).ok();
    }
    if let Some(styles) = &snapshot.styles {
        let styles_json = serialize(styles)?;
        std::fs::write(repo_root.join(DitPaths::STYLES_FILE), &styles_json)
            .context("failed to write styles file")?;
    }

    // 5. Write components (merge components + component_sets into one file)
    let components_path = repo_root.join(DitPaths::COMPONENTS_FILE);
    let has_components = snapshot.components.is_some() || snapshot.component_sets.is_some();
    if !has_components && components_path.exists() {
        std::fs::remove_file(&components_path).ok();
    }
    if has_components {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct ComponentsFile<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            components: Option<&'a HashMap<String, ComponentMetadata>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            component_sets: Option<&'a HashMap<String, ComponentSetMetadata>>,
        }

        let file = ComponentsFile {
            components: snapshot.components.as_ref(),
            component_sets: snapshot.component_sets.as_ref(),
        };
        let components_json = serialize(&file)?;
        std::fs::write(repo_root.join(DitPaths::COMPONENTS_FILE), &components_json)
            .context("failed to write components file")?;
    }

    Ok(())
}

/// Reconstruct a snapshot from chunked files under `repo_root`.
pub fn read_snapshot(repo_root: &Path) -> Result<DitSnapshot> {
    // 1. Read project metadata
    let project_json = std::fs::read_to_string(repo_root.join(DitPaths::PROJECT_FILE))
        .context("failed to read project file")?;
    let project: DitProject = deserialize(&project_json)?;

    // 2. Read all pages
    let pages_dir = repo_root.join(DitPaths::PAGES_DIR);
    let mut pages = Vec::new();

    if pages_dir.exists() {
        let mut entries: Vec<_> = std::fs::read_dir(&pages_dir)
            .context("failed to read pages directory")?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();

        // Sort by filename for deterministic ordering
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let page_json = std::fs::read_to_string(entry.path())
                .with_context(|| format!("failed to read page file {:?}", entry.path()))?;
            let page: DitPage = deserialize(&page_json)?;
            pages.push(page);
        }
    }

    // 3. Read styles (optional)
    let styles_path = repo_root.join(DitPaths::STYLES_FILE);
    let styles = if styles_path.exists() {
        let json = std::fs::read_to_string(&styles_path)
            .context("failed to read styles file")?;
        Some(deserialize::<HashMap<String, StyleDefinition>>(&json)?)
    } else {
        None
    };

    // 4. Read components (optional)
    let components_path = repo_root.join(DitPaths::COMPONENTS_FILE);
    let (components, component_sets) = if components_path.exists() {
        let json = std::fs::read_to_string(&components_path)
            .context("failed to read components file")?;

        #[derive(serde::Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct ComponentsFile {
            components: Option<HashMap<String, ComponentMetadata>>,
            component_sets: Option<HashMap<String, ComponentSetMetadata>>,
        }

        let file: ComponentsFile = deserialize(&json)?;
        (file.components, file.component_sets)
    } else {
        (None, None)
    };

    Ok(DitSnapshot {
        project,
        pages,
        components,
        component_sets,
        styles,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn canonical_keys_are_sorted() {
        let node = DitNode {
            id: "1:2".into(),
            name: "Test".into(),
            node_type: NodeType::Frame,
            visible: Some(true),
            locked: None,
            opacity: Some(0.5),
            blend_mode: None,
            absolute_bounding_box: None,
            absolute_render_bounds: None,
            relative_transform: None,
            size: None,
            rotation: None,
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
            extra: Default::default(),
        };

        let json = serialize(&node).unwrap();

        // The keys that appear should be in sorted order
        let lines: Vec<&str> = json.lines().collect();
        let key_lines: Vec<&str> = lines
            .iter()
            .filter(|l| l.contains('"') && l.contains(':'))
            .copied()
            .collect();

        // Extract key names
        let keys: Vec<String> = key_lines
            .iter()
            .filter_map(|l| {
                let trimmed = l.trim();
                if trimmed.starts_with('"') {
                    let end = trimmed[1..].find('"')?;
                    Some(trimmed[1..=end].to_string())
                } else {
                    None
                }
            })
            .collect();

        let mut sorted_keys = keys.clone();
        sorted_keys.sort();
        assert_eq!(keys, sorted_keys, "JSON keys must be sorted");
    }

    #[test]
    fn float_rounding() {
        let color = Color {
            r: 0.1234567890,
            g: 1.0,
            b: 0.0,
            a: 0.999999999,
        };
        let json = serialize(&color).unwrap();
        assert!(json.contains("0.123457"), "expected rounded float: {json}");
        assert!(json.contains("1.0"), "expected 1.0: {json}");
    }

    #[test]
    fn deterministic_output() {
        let color = Color {
            r: 0.5,
            g: 0.25,
            b: 0.75,
            a: 1.0,
        };
        let json1 = serialize(&color).unwrap();
        let json2 = serialize(&color).unwrap();
        assert_eq!(json1, json2, "serialize must be deterministic");
    }

    #[test]
    fn trailing_newline() {
        let json = serialize(&42_i32).unwrap();
        assert!(json.ends_with('\n'));
    }

    #[test]
    fn roundtrip_snapshot() {
        let snapshot = DitSnapshot {
            project: DitProject {
                file_key: "abc123".into(),
                name: "Test Project".into(),
                last_modified: "2025-01-01T00:00:00Z".into(),
                version: "1".into(),
                platform: DesignPlatform::Figma,
                schema_version: 1,
                thumbnail_url: None,
                editor_type: None,
                role: None,
            },
            pages: vec![DitPage {
                id: "0:1".into(),
                name: "Page 1".into(),
                background_color: None,
                children: vec![],
            }],
            components: None,
            component_sets: None,
            styles: None,
        };

        let dir = tempfile::tempdir().unwrap();
        write_snapshot(dir.path(), &snapshot).unwrap();
        let loaded = read_snapshot(dir.path()).unwrap();

        assert_eq!(snapshot.project.file_key, loaded.project.file_key);
        assert_eq!(snapshot.pages.len(), loaded.pages.len());
        assert_eq!(snapshot.pages[0].id, loaded.pages[0].id);
    }
}
