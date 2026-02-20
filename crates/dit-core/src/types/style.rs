use serde::{Deserialize, Serialize};

use super::enums::StyleType;

/// A named reusable style (fill style, text style, effect style, or grid style).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StyleDefinition {
    /// The style's unique key.
    #[serde(default)]
    pub key: String,
    /// The human-readable name.
    #[serde(default)]
    pub name: String,
    /// The style type.
    #[serde(default = "default_style_type")]
    pub style_type: StyleType,
    /// Description text.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_style_type() -> StyleType {
    StyleType::Fill
}
