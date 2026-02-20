use serde::{Deserialize, Serialize};

// ─── NodeType ────────────────────────────────────────────────────────────────

/// All Figma node types.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NodeType {
    Document,
    Canvas,
    Frame,
    Group,
    Vector,
    BooleanOperation,
    Star,
    Line,
    Ellipse,
    RegularPolygon,
    Rectangle,
    Text,
    Slice,
    Component,
    ComponentSet,
    Instance,
    Section,
    ShapeWithText,
    Sticky,
    Connector,
    WashiTape,
    Table,
    TableCell,
    Widget,
    Embed,
    LinkUnfurl,
    #[serde(other)]
    Unknown,
}

// ─── BlendMode ───────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BlendMode {
    PassThrough,
    Normal,
    Darken,
    Multiply,
    LinearBurn,
    ColorBurn,
    Lighten,
    Screen,
    LinearDodge,
    ColorDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    #[serde(other)]
    Unknown,
}

// ─── Stroke / Cap / Join ─────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrokeAlign {
    Inside,
    Outside,
    Center,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrokeCap {
    None,
    Round,
    Square,
    LineArrow,
    TriangleArrow,
    DiamondFilled,
    CircleFilled,
    TriangleFilled,
    #[serde(rename = "WASHI_TAPE")]
    WashiTape,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StrokeJoin {
    Miter,
    Bevel,
    Round,
    #[serde(other)]
    Unknown,
}

// ─── Paint ───────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PaintType {
    Solid,
    GradientLinear,
    GradientRadial,
    GradientAngular,
    GradientDiamond,
    Image,
    Emoji,
    Video,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ScaleMode {
    Fill,
    Fit,
    Tile,
    Stretch,
    Crop,
    #[serde(other)]
    Unknown,
}

// ─── Effect ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EffectType {
    InnerShadow,
    DropShadow,
    LayerBlur,
    BackgroundBlur,
    #[serde(other)]
    Unknown,
}

// ─── Text / Typography enums ─────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextAlignHorizontal {
    Left,
    Center,
    Right,
    Justified,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextAlignVertical {
    Top,
    Center,
    Bottom,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextAutoResize {
    None,
    Height,
    WidthAndHeight,
    Truncate,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextDecoration {
    None,
    Strikethrough,
    Underline,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextCase {
    Original,
    Upper,
    Lower,
    Title,
    SmallCaps,
    SmallCapsForced,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LineHeightUnit {
    Pixels,
    #[serde(rename = "FONT_SIZE_%")]
    FontSizePercentage,
    Intrinsic,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TextTruncation {
    Disabled,
    Ending,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HyperlinkType {
    Url,
    Node,
    None,
    #[serde(other)]
    Unknown,
}

// ─── Layout enums ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ConstraintType {
    Min,
    Center,
    Max,
    Stretch,
    Scale,
    // Figma also sends these as aliases for Min/Max
    Left,
    Right,
    Top,
    Bottom,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutMode {
    None,
    Horizontal,
    Vertical,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutAlign {
    Min,
    Center,
    Max,
    Stretch,
    Inherit,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutSizing {
    Fixed,
    Hug,
    Fill,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutPositioning {
    Auto,
    Absolute,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutWrap {
    NoWrap,
    Wrap,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PrimaryAxisAlignItems {
    Min,
    Center,
    Max,
    SpaceBetween,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CounterAxisAlignItems {
    Min,
    Center,
    Max,
    Baseline,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CounterAxisAlignContent {
    Auto,
    SpaceBetween,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AxisSizingMode {
    Fixed,
    Auto,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverflowDirection {
    None,
    HorizontalScrolling,
    VerticalScrolling,
    HorizontalAndVerticalScrolling,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutGridPattern {
    Columns,
    Rows,
    Grid,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LayoutGridAlignment {
    Min,
    Center,
    Max,
    Stretch,
    #[serde(other)]
    Unknown,
}

// ─── Boolean operation ───────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BooleanOperationType {
    Union,
    Intersect,
    Subtract,
    Exclude,
    #[serde(other)]
    Unknown,
}

// ─── Vector path enums ───────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WindingRule {
    Nonzero,
    Evenodd,
    None,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HandleMirroring {
    None,
    Angle,
    AngleAndLength,
    #[serde(other)]
    Unknown,
}

// ─── Mask ────────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MaskType {
    Alpha,
    Vector,
    Luminance,
    #[serde(other)]
    Unknown,
}

// ─── Export ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportFormat {
    Jpg,
    Png,
    Svg,
    Pdf,
    #[serde(other)]
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExportConstraintType {
    Scale,
    Width,
    Height,
    #[serde(other)]
    Unknown,
}

// ─── Easing ──────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EasingType {
    EaseIn,
    EaseOut,
    EaseInAndOut,
    Linear,
    GentleSpring,
    #[serde(rename = "EASE_IN_BACK")]
    EaseInBack,
    #[serde(rename = "EASE_OUT_BACK")]
    EaseOutBack,
    #[serde(rename = "EASE_IN_AND_OUT_BACK")]
    EaseInAndOutBack,
    CustomBezier,
    CustomSpring,
    #[serde(other)]
    Unknown,
}

// ─── Component property ──────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ComponentPropertyType {
    Boolean,
    InstanceSwap,
    Text,
    Variant,
    #[serde(other)]
    Unknown,
}

// ─── Style type ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StyleType {
    Fill,
    Text,
    Effect,
    Grid,
    #[serde(other)]
    Unknown,
}

// ─── Design platform ────────────────────────────────────────────────────────

/// Design tool that produced the data (for future multi-tool support).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DesignPlatform {
    Figma,
}

// ─── Repository change type ──────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
}
