use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::component::{ComponentMetadata, ComponentSetMetadata};
use super::enums::DesignPlatform;
use super::node::DitNode;
use super::primitives::Color;
use super::style::StyleDefinition;

// ─── DitProject ──────────────────────────────────────────────────────────────

/// Root metadata for a DIT-managed design project.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitProject {
    /// Unique file key (from Figma).
    pub file_key: String,
    /// Human-readable project name.
    pub name: String,
    /// Last-modified timestamp (ISO 8601).
    pub last_modified: String,
    /// Figma file version string.
    pub version: String,
    /// Design tool that produced this project.
    pub platform: DesignPlatform,
    /// Schema version of the DIT canonical format.
    pub schema_version: u32,
    /// Thumbnail URL of the file (optional, not stored in git).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    /// Editor type (e.g. "figma").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub editor_type: Option<String>,
    /// Role of the current user on this file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

// ─── DitPage ─────────────────────────────────────────────────────────────────

/// A single page extracted from the design document.
/// Stored as an individual JSON file under `dit.pages/`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitPage {
    /// Page node ID (e.g. "0:1").
    pub id: String,
    /// Page name.
    pub name: String,
    /// Page background color (from CANVAS node).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<Color>,
    /// All direct children of the page (frames, groups, etc.).
    pub children: Vec<DitNode>,
}

// ─── DitSnapshot ─────────────────────────────────────────────────────────────

/// Complete state container: everything needed to reconstruct the full file.
/// This is the top-level structure written during `dit save`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DitSnapshot {
    /// Project metadata.
    pub project: DitProject,
    /// All pages in the document.
    pub pages: Vec<DitPage>,
    /// Component definitions (component key → metadata).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<HashMap<String, ComponentMetadata>>,
    /// Component set definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_sets: Option<HashMap<String, ComponentSetMetadata>>,
    /// Shared style definitions (style key → metadata).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<HashMap<String, StyleDefinition>>,
}
