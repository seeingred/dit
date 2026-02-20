use serde::{Deserialize, Serialize};

use super::enums::{BlendMode, EffectType};
use super::primitives::{Color, Vector};

/// A visual effect applied to a node (shadow or blur).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Effect {
    /// The effect type (DROP_SHADOW, INNER_SHADOW, LAYER_BLUR, BACKGROUND_BLUR).
    #[serde(rename = "type", default = "default_effect_type")]
    pub effect_type: EffectType,

    /// Whether this effect is visible.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visible: Option<bool>,

    /// Blur radius (pixels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,

    /// Shadow color.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,

    /// Blend mode for the effect.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_mode: Option<BlendMode>,

    /// Shadow offset (for shadow effects).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<Vector>,

    /// Shadow spread (pixels, for shadow effects).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spread: Option<f64>,

    /// Whether the shadow is rendered behind the node (for drop shadows).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_shadow_behind_node: Option<bool>,
}

fn default_effect_type() -> EffectType {
    EffectType::DropShadow
}
