use serde::{Deserialize, Serialize};

use super::enums::ComponentPropertyType;

// ─── ComponentProperty ───────────────────────────────────────────────────────

/// A resolved component property value on an instance.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentProperty {
    #[serde(rename = "type", default = "default_component_property_type")]
    pub property_type: ComponentPropertyType,
    #[serde(default)]
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_values: Option<Vec<ComponentPropertyPreferredValue>>,
}

fn default_component_property_type() -> ComponentPropertyType {
    ComponentPropertyType::Text
}

// ─── ComponentPropertyDefinition ─────────────────────────────────────────────

/// A component property definition declared on a component or component set.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentPropertyDefinition {
    #[serde(rename = "type", default = "default_component_property_type")]
    pub property_type: ComponentPropertyType,
    #[serde(default)]
    pub default_value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_options: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_values: Option<Vec<ComponentPropertyPreferredValue>>,
}

// ─── ComponentPropertyPreferredValue ─────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentPropertyPreferredValue {
    #[serde(rename = "type", default = "default_preferred_value_type")]
    pub value_type: PreferredValueType,
    #[serde(default)]
    pub key: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PreferredValueType {
    Component,
    ComponentSet,
    #[serde(other)]
    Unknown,
}

fn default_preferred_value_type() -> PreferredValueType {
    PreferredValueType::Component
}

// ─── Override ────────────────────────────────────────────────────────────────

/// An override on a component instance (maps a node inside the component to changed fields).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Override {
    /// The node ID (within the component) being overridden.
    #[serde(default)]
    pub id: String,
    /// The fields that are overridden.
    #[serde(default)]
    pub overridden_fields: Vec<String>,
}

// ─── ComponentMetadata ───────────────────────────────────────────────────────

/// Metadata about a component as returned by the Figma file API.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentMetadata {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_links: Option<Vec<DocumentationLink>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component_set_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DocumentationLink {
    #[serde(default)]
    pub uri: String,
}

// ─── ComponentSetMetadata ────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSetMetadata {
    #[serde(default)]
    pub key: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation_links: Option<Vec<DocumentationLink>>,
}
