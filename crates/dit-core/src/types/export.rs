use serde::{Deserialize, Serialize};

use super::enums::{ExportConstraintType, ExportFormat};

/// Constraint applied to an export (scale, target width, or target height).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportConstraint {
    #[serde(rename = "type", default = "default_export_constraint_type")]
    pub constraint_type: ExportConstraintType,
    #[serde(default)]
    pub value: f64,
}

fn default_export_constraint_type() -> ExportConstraintType {
    ExportConstraintType::Scale
}

/// An export setting on a node (e.g. "export at 2x PNG").
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExportSetting {
    #[serde(default)]
    pub suffix: String,
    #[serde(default = "default_export_format")]
    pub format: ExportFormat,
    #[serde(default = "default_export_constraint")]
    pub constraint: ExportConstraint,
}

fn default_export_format() -> ExportFormat {
    ExportFormat::Png
}

fn default_export_constraint() -> ExportConstraint {
    ExportConstraint {
        constraint_type: ExportConstraintType::Scale,
        value: 1.0,
    }
}
