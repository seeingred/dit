//! Convert .fig files to DIT snapshots using the fig2json crate.
//!
//! Uses `fig2json::convert_raw()` for lossless JSON output, then maps
//! the Figma document tree into DIT canonical types via serde roundtrip.

use std::path::Path;

use anyhow::{bail, Context, Result};
use tracing::debug;

use crate::types::{
    Color, DesignPlatform, DitNode, DitPage, DitProject, DitSnapshot,
};

/// Convert a .fig file to a DIT snapshot.
///
/// Reads the file, parses it with fig2json, and maps the raw Figma
/// document structure to DIT canonical format.
pub fn fig_to_snapshot(fig_path: &Path, file_key: &str) -> Result<DitSnapshot> {
    let bytes = std::fs::read(fig_path)
        .with_context(|| format!("failed to read .fig file: {}", fig_path.display()))?;

    let raw_json = fig2json::convert_raw(&bytes)
        .map_err(|e| anyhow::anyhow!("fig2json conversion failed: {e}"))?;

    debug!("Parsed .fig file, extracting document structure");

    // The raw JSON from fig2json has a document tree structure.
    // Extract the document node which contains pages as children.
    let document = raw_json
        .get("document")
        .or_else(|| raw_json.get("documentNode"))
        .unwrap_or(&raw_json);

    let pages = extract_pages(document)?;

    // Extract metadata from the raw JSON
    let name = raw_json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Untitled")
        .to_string();

    let version = raw_json
        .get("version")
        .map(|v| match v {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            other => other.to_string(),
        })
        .unwrap_or_else(|| "0".to_string());

    let schema_version = raw_json
        .get("schemaVersion")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;

    let project = DitProject {
        file_key: file_key.to_string(),
        name,
        last_modified: chrono_free_now(),
        version,
        platform: DesignPlatform::Figma,
        schema_version,
        thumbnail_url: None,
        editor_type: raw_json
            .get("editorType")
            .and_then(|v| v.as_str())
            .map(String::from),
        role: None,
    };

    Ok(DitSnapshot {
        project,
        pages,
        components: None,
        component_sets: None,
        styles: None,
    })
}

/// Extract a string value from a JSON field that may be either a plain string
/// or a fig2json enum object like `{"__enum__": "NodeType", "value": "CANVAS"}`.
fn enum_str<'a>(val: &'a serde_json::Value) -> Option<&'a str> {
    val.as_str().or_else(|| {
        val.get("value").and_then(|v| v.as_str())
    })
}

/// Convert a fig2json `guid` object (`{"localID": N, "sessionID": M}`) into
/// a Figma-style node ID string like `"M:N"`.
fn guid_to_id(guid: &serde_json::Value) -> Option<String> {
    let local = guid.get("localID").and_then(|v| v.as_u64())?;
    let session = guid.get("sessionID").and_then(|v| v.as_u64())?;
    Some(format!("{session}:{local}"))
}

/// Recursively normalize fig2json output for DitNode deserialization:
/// - Flatten enum objects (`{"__enum__": ..., "value": "X"}` → `"X"`)
/// - Convert `guid` to `id` string
fn normalize_fig_json(val: &serde_json::Value) -> serde_json::Value {
    match val {
        serde_json::Value::Object(map) => {
            // If this object is an enum wrapper, collapse it to its value.
            if map.contains_key("__enum__") {
                if let Some(inner) = map.get("value") {
                    return inner.clone();
                }
            }
            // Recurse into all fields, with key remapping.
            let mut new_map = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                if k == "guid" {
                    // Convert guid → id
                    if let Some(id_str) = guid_to_id(v) {
                        new_map.insert("id".to_string(), serde_json::Value::String(id_str));
                    }
                    // Keep guid too in case anything references it
                    new_map.insert(k.clone(), normalize_fig_json(v));
                } else {
                    new_map.insert(k.clone(), normalize_fig_json(v));
                }
            }
            serde_json::Value::Object(new_map)
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(normalize_fig_json).collect())
        }
        other => other.clone(),
    }
}

/// Extract pages (CANVAS nodes) from the document JSON.
fn extract_pages(document: &serde_json::Value) -> Result<Vec<DitPage>> {
    let children = document
        .get("children")
        .and_then(|v| v.as_array())
        .context("document has no children array")?;

    let mut pages = Vec::with_capacity(children.len());

    for canvas in children {
        let node_type = canvas
            .get("type")
            .and_then(enum_str)
            .unwrap_or("");

        if node_type != "CANVAS" {
            debug!("Skipping non-CANVAS child (type: {node_type})");
            continue;
        }

        let id = canvas
            .get("id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| canvas.get("guid").and_then(guid_to_id))
            .unwrap_or_else(|| "0:0".to_string());

        let name = canvas
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Page")
            .to_string();

        // Extract background color
        let background_color = canvas
            .get("backgroundColor")
            .and_then(|v| {
                Some(Color {
                    r: v.get("r")?.as_f64()?,
                    g: v.get("g")?.as_f64()?,
                    b: v.get("b")?.as_f64()?,
                    a: v.get("a")?.as_f64()?,
                })
            });

        // Convert child nodes via JSON roundtrip (same approach as the old converter)
        let page_children = canvas
            .get("children")
            .and_then(|v| v.as_array())
            .map(|kids| {
                kids.iter()
                    .map(node_roundtrip)
                    .collect::<Result<Vec<DitNode>>>()
            })
            .transpose()?
            .unwrap_or_default();

        pages.push(DitPage {
            id,
            name,
            background_color,
            children: page_children,
        });
    }

    if pages.is_empty() {
        bail!("no CANVAS pages found in .fig document");
    }

    Ok(pages)
}

/// Convert a JSON node to a DitNode via serde roundtrip.
///
/// The raw fig2json output may use enum wrapper objects like
/// `{"__enum__": "NodeType", "value": "FRAME"}` instead of plain strings,
/// and `guid` instead of `id`. We normalize these before deserializing.
/// Unknown fields land in DitNode's `extra` HashMap for lossless preservation.
fn node_roundtrip(json_node: &serde_json::Value) -> Result<DitNode> {
    let normalized = normalize_fig_json(json_node);
    serde_json::from_value::<DitNode>(normalized)
        .context("failed to deserialize node from .fig JSON")
}

/// Produce a rough ISO-8601 "now" timestamp without chrono.
fn chrono_free_now() -> String {
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    let s = secs;
    let days = s / 86400;
    let rem = s % 86400;
    let h = rem / 3600;
    let m = (rem % 3600) / 60;
    let sec = rem % 60;

    let (y, mo, d) = days_to_ymd(days);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{sec:02}Z")
}

fn days_to_ymd(mut days: u64) -> (u64, u64, u64) {
    let mut year = 1970u64;
    loop {
        let ydays = if is_leap(year) { 366 } else { 365 };
        if days < ydays {
            break;
        }
        days -= ydays;
        year += 1;
    }
    let leap = is_leap(year);
    let mdays = [
        31,
        if leap { 29 } else { 28 },
        31, 30, 31, 30, 31, 31, 30, 31, 30, 31,
    ];
    let mut month = 1u64;
    for &md in &mdays {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    (year, month, days + 1)
}

fn is_leap(y: u64) -> bool {
    y.is_multiple_of(4) && (!y.is_multiple_of(100) || y.is_multiple_of(400))
}
