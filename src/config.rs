use std::fs;
use std::path::PathBuf;

use kdl::{KdlDocument, KdlEntry, KdlIdentifier, KdlNode, KdlValue};

const TOUCHSCREEN_GESTURES_FILE: &str = "touchscreen-gestures.kdl";
const TOUCHPAD_GESTURES_FILE: &str = "touchpad-gestures.kdl";

pub const MIN_FINGERS: u8 = 3;
pub const MAX_FINGERS: u8 = 10;

// ---------------------------------------------------------------------------
// KDL v1 <-> v2 compatibility
// ---------------------------------------------------------------------------
// The `kdl` crate v6 implements KDL v2, where booleans are `#true`/`#false`.
// Niri's compositor uses `knuffel` which implements KDL v1 (`true`/`false`).
// These helpers bridge the two specs for property values.

fn kdl_v1_to_v2(content: &str) -> String {
    content
        .replace("=true", "=#true")
        .replace("=false", "=#false")
}

fn kdl_v2_to_v1(content: &str) -> String {
    content
        .replace("=#true", "=true")
        .replace("=#false", "=false")
}

const INCLUDE_BLOCK: &str = "\
// Includes override existing sections.\n\
// Settings in these files replace the corresponding blocks in this config.\n\
// Gesture binds in these files merge into the main binds {} block.\n\
include \"touchscreen-gestures.kdl\" optional=true\n\
include \"touchpad-gestures.kdl\" optional=true\n";

// ---------------------------------------------------------------------------
// Structured gesture trigger
// ---------------------------------------------------------------------------
//
// Mirrors niri's property-form trigger model (niri commit 558adc5f).
// Each variant maps 1:1 to a KDL bind node of the shape:
//
//   TouchSwipe    fingers=3 direction="up"                { ... }
//   TouchPinch    fingers=4 direction="in"                { ... }
//   TouchRotate   fingers=5 direction="cw"                { ... }
//   TouchEdge     edge="left"                             { ... }
//   TouchEdge     edge="top" zone="left"                  { ... }
//   TouchpadSwipe fingers=3 direction="up"                { ... }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwipeDir {
    Up,
    Down,
    Left,
    Right,
}

impl SwipeDir {
    pub fn as_kdl(self) -> &'static str {
        match self {
            Self::Up => "up",
            Self::Down => "down",
            Self::Left => "left",
            Self::Right => "right",
        }
    }
    pub fn display(self) -> &'static str {
        match self {
            Self::Up => "Up",
            Self::Down => "Down",
            Self::Left => "Left",
            Self::Right => "Right",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            _ => None,
        }
    }
    pub const ALL: &'static [Self] = &[Self::Up, Self::Down, Self::Left, Self::Right];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PinchDir {
    In,
    Out,
}

impl PinchDir {
    pub fn as_kdl(self) -> &'static str {
        match self {
            Self::In => "in",
            Self::Out => "out",
        }
    }
    pub fn display(self) -> &'static str {
        match self {
            Self::In => "In",
            Self::Out => "Out",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "in" => Some(Self::In),
            "out" => Some(Self::Out),
            _ => None,
        }
    }
    pub const ALL: &'static [Self] = &[Self::In, Self::Out];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RotateDir {
    Cw,
    Ccw,
}

impl RotateDir {
    pub fn as_kdl(self) -> &'static str {
        match self {
            Self::Cw => "cw",
            Self::Ccw => "ccw",
        }
    }
    pub fn display(self) -> &'static str {
        match self {
            Self::Cw => "Clockwise",
            Self::Ccw => "Counter-Clockwise",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "cw" => Some(Self::Cw),
            "ccw" => Some(Self::Ccw),
            _ => None,
        }
    }
    pub const ALL: &'static [Self] = &[Self::Cw, Self::Ccw];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Edge {
    Left,
    Right,
    Top,
    Bottom,
}

impl Edge {
    pub fn as_kdl(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
    pub fn display(self) -> &'static str {
        match self {
            Self::Left => "Left",
            Self::Right => "Right",
            Self::Top => "Top",
            Self::Bottom => "Bottom",
        }
    }
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "top" => Some(Self::Top),
            "bottom" => Some(Self::Bottom),
            _ => None,
        }
    }
    pub const ALL: &'static [Self] = &[Self::Left, Self::Right, Self::Top, Self::Bottom];
}

/// Which third of an edge a touch landed in. The perpendicular axis of the
/// edge is split into thirds; the KDL vocabulary rotates per edge axis
/// (Top/Bottom take left|center|right; Left/Right take top|center|bottom).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeZone {
    Start,
    Center,
    End,
}

impl EdgeZone {
    /// The lowercased KDL identifier that niri expects for this
    /// (edge, zone) pair. The vocabulary rotates per axis.
    pub fn as_kdl(self, edge: Edge) -> &'static str {
        match (edge, self) {
            (Edge::Top | Edge::Bottom, Self::Start) => "left",
            (Edge::Top | Edge::Bottom, Self::Center) => "center",
            (Edge::Top | Edge::Bottom, Self::End) => "right",
            (Edge::Left | Edge::Right, Self::Start) => "top",
            (Edge::Left | Edge::Right, Self::Center) => "center",
            (Edge::Left | Edge::Right, Self::End) => "bottom",
        }
    }
    pub fn display(self, edge: Edge) -> &'static str {
        match (edge, self) {
            (Edge::Top | Edge::Bottom, Self::Start) => "Left",
            (Edge::Top | Edge::Bottom, Self::Center) => "Center",
            (Edge::Top | Edge::Bottom, Self::End) => "Right",
            (Edge::Left | Edge::Right, Self::Start) => "Top",
            (Edge::Left | Edge::Right, Self::Center) => "Center",
            (Edge::Left | Edge::Right, Self::End) => "Bottom",
        }
    }
    /// Inverse of `as_kdl`: given the edge and the KDL zone keyword,
    /// return the abstract zone. Returns None on invalid combinations.
    pub fn parse(edge: Edge, s: &str) -> Option<Self> {
        match (edge, s) {
            (Edge::Top | Edge::Bottom, "left") => Some(Self::Start),
            (Edge::Top | Edge::Bottom, "center") => Some(Self::Center),
            (Edge::Top | Edge::Bottom, "right") => Some(Self::End),
            (Edge::Left | Edge::Right, "top") => Some(Self::Start),
            (Edge::Left | Edge::Right, "center") => Some(Self::Center),
            (Edge::Left | Edge::Right, "bottom") => Some(Self::End),
            _ => None,
        }
    }
    pub const ALL: &'static [Self] = &[Self::Start, Self::Center, Self::End];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trigger {
    TouchSwipe {
        fingers: u8,
        direction: SwipeDir,
    },
    TouchPinch {
        fingers: u8,
        direction: PinchDir,
    },
    TouchRotate {
        fingers: u8,
        direction: RotateDir,
    },
    TouchEdge {
        edge: Edge,
        zone: Option<EdgeZone>,
    },
    TouchTap {
        fingers: u8,
    },
    TouchTapHoldDrag {
        fingers: u8,
        direction: Option<SwipeDir>,
    },
    TouchpadSwipe {
        fingers: u8,
        direction: SwipeDir,
    },
    TouchpadTapHold {
        fingers: u8,
    },
    TouchpadTapHoldDrag {
        fingers: u8,
    },
}

impl Trigger {
    /// The bare KDL node name (family) without any properties.
    pub fn kdl_family(self) -> &'static str {
        match self {
            Self::TouchSwipe { .. } => "TouchSwipe",
            Self::TouchPinch { .. } => "TouchPinch",
            Self::TouchRotate { .. } => "TouchRotate",
            Self::TouchEdge { .. } => "TouchEdge",
            Self::TouchTap { .. } => "TouchTap",
            Self::TouchTapHoldDrag { .. } => "TouchTapHoldDrag",
            Self::TouchpadSwipe { .. } => "TouchpadSwipe",
            Self::TouchpadTapHold { .. } => "TouchpadTapHold",
            Self::TouchpadTapHoldDrag { .. } => "TouchpadTapHoldDrag",
        }
    }

    /// Stable dedup/equality key (used to find or delete binds without
    /// relying on struct hashing through interior fields).
    pub fn key(self) -> String {
        match self {
            Self::TouchSwipe { fingers, direction } => {
                format!("TouchSwipe:{fingers}:{}", direction.as_kdl())
            }
            Self::TouchPinch { fingers, direction } => {
                format!("TouchPinch:{fingers}:{}", direction.as_kdl())
            }
            Self::TouchRotate { fingers, direction } => {
                format!("TouchRotate:{fingers}:{}", direction.as_kdl())
            }
            Self::TouchEdge { edge, zone } => {
                let z = zone.map(|z| z.as_kdl(edge)).unwrap_or("full");
                format!("TouchEdge:{}:{z}", edge.as_kdl())
            }
            Self::TouchTap { fingers } => {
                format!("TouchTap:{fingers}")
            }
            Self::TouchTapHoldDrag { fingers, direction } => {
                let d = direction.map(|d| d.as_kdl()).unwrap_or("any");
                format!("TouchTapHoldDrag:{fingers}:{d}")
            }
            Self::TouchpadSwipe { fingers, direction } => {
                format!("TouchpadSwipe:{fingers}:{}", direction.as_kdl())
            }
            Self::TouchpadTapHold { fingers } => {
                format!("TouchpadTapHold:{fingers}")
            }
            Self::TouchpadTapHoldDrag { fingers } => {
                format!("TouchpadTapHoldDrag:{fingers}")
            }
        }
    }

    /// Human-readable label used in UI list/rows.
    pub fn display_name(self) -> String {
        match self {
            Self::TouchSwipe { fingers, direction } => {
                format!("{fingers}-Finger Swipe {}", direction.display())
            }
            Self::TouchPinch { fingers, direction } => {
                format!("{fingers}-Finger Pinch {}", direction.display())
            }
            Self::TouchRotate { fingers, direction } => {
                format!("{fingers}-Finger Rotate {}", direction.display())
            }
            Self::TouchEdge { edge, zone: None } => {
                format!("Edge {} (full)", edge.display())
            }
            Self::TouchEdge {
                edge,
                zone: Some(z),
            } => {
                format!("Edge {} — {}", edge.display(), z.display(edge))
            }
            Self::TouchTap { fingers } => {
                format!("{fingers}-Finger Tap")
            }
            Self::TouchTapHoldDrag {
                fingers,
                direction: None,
            } => {
                format!("{fingers}-Finger Tap-Hold-Drag")
            }
            Self::TouchTapHoldDrag {
                fingers,
                direction: Some(d),
            } => {
                format!("{fingers}-Finger Tap-Hold-Drag {}", d.display())
            }
            Self::TouchpadSwipe { fingers, direction } => {
                format!("{fingers}-Finger Swipe {}", direction.display())
            }
            Self::TouchpadTapHold { fingers } => {
                format!("{fingers}-Finger Tap-Hold")
            }
            Self::TouchpadTapHoldDrag { fingers } => {
                format!("{fingers}-Finger Tap-Hold-Drag")
            }
        }
    }

    /// Parse a KDL node (by its name + property entries) into a Trigger.
    /// Returns None for unrecognized families or invalid property
    /// combinations. Modifier prefixes (`Mod+TouchSwipe`) are stripped.
    pub fn parse_node(node: &KdlNode) -> Option<Self> {
        let name = node.name().to_string();
        let family = name.rsplit('+').next().unwrap_or(&name);

        let get_int = |k: &str| -> Option<u8> {
            node.get(k)
                .and_then(|v| v.as_integer())
                .and_then(|i| u8::try_from(i).ok())
        };
        let get_str = |k: &str| -> Option<String> {
            node.get(k)
                .and_then(|v| v.as_string())
                .map(|s| s.to_string())
        };

        match family {
            "TouchSwipe" => {
                let fingers = get_int("fingers")?;
                if !(MIN_FINGERS..=MAX_FINGERS).contains(&fingers) {
                    return None;
                }
                let direction = SwipeDir::parse(&get_str("direction")?)?;
                Some(Self::TouchSwipe { fingers, direction })
            }
            "TouchPinch" => {
                let fingers = get_int("fingers")?;
                if !(MIN_FINGERS..=MAX_FINGERS).contains(&fingers) {
                    return None;
                }
                let direction = PinchDir::parse(&get_str("direction")?)?;
                Some(Self::TouchPinch { fingers, direction })
            }
            "TouchRotate" => {
                let fingers = get_int("fingers")?;
                if !(MIN_FINGERS..=MAX_FINGERS).contains(&fingers) {
                    return None;
                }
                let direction = RotateDir::parse(&get_str("direction")?)?;
                Some(Self::TouchRotate { fingers, direction })
            }
            "TouchEdge" => {
                let edge = Edge::parse(&get_str("edge")?)?;
                let zone = match get_str("zone") {
                    Some(z) => Some(EdgeZone::parse(edge, &z)?),
                    None => None,
                };
                Some(Self::TouchEdge { edge, zone })
            }
            "TouchTap" => {
                let fingers = get_int("fingers")?;
                if !(MIN_FINGERS..=MAX_FINGERS).contains(&fingers) {
                    return None;
                }
                Some(Self::TouchTap { fingers })
            }
            "TouchTapHoldDrag" => {
                let fingers = get_int("fingers")?;
                if !(MIN_FINGERS..=MAX_FINGERS).contains(&fingers) {
                    return None;
                }
                let direction = match get_str("direction") {
                    Some(s) => Some(SwipeDir::parse(&s)?),
                    None => None,
                };
                Some(Self::TouchTapHoldDrag { fingers, direction })
            }
            "TouchpadSwipe" => {
                let fingers = get_int("fingers")?;
                if !(MIN_FINGERS..=MAX_FINGERS).contains(&fingers) {
                    return None;
                }
                let direction = SwipeDir::parse(&get_str("direction")?)?;
                Some(Self::TouchpadSwipe { fingers, direction })
            }
            "TouchpadTapHold" => {
                let fingers = get_int("fingers")?;
                if !(MIN_FINGERS..=MAX_FINGERS).contains(&fingers) {
                    return None;
                }
                Some(Self::TouchpadTapHold { fingers })
            }
            "TouchpadTapHoldDrag" => {
                let fingers = get_int("fingers")?;
                if !(MIN_FINGERS..=MAX_FINGERS).contains(&fingers) {
                    return None;
                }
                Some(Self::TouchpadTapHoldDrag { fingers })
            }
            _ => None,
        }
    }

    /// Populate a freshly-created `KdlNode` (already named with `kdl_family()`)
    /// with the property entries for this trigger. Strings are written with
    /// forced double-quoted repr because niri's knuffel parser requires it.
    pub fn write_props(self, node: &mut KdlNode) {
        match self {
            Self::TouchSwipe { fingers, direction }
            | Self::TouchpadSwipe { fingers, direction } => {
                push_int_prop(node, "fingers", fingers as i128);
                push_string_prop(node, "direction", direction.as_kdl());
            }
            Self::TouchPinch { fingers, direction } => {
                push_int_prop(node, "fingers", fingers as i128);
                push_string_prop(node, "direction", direction.as_kdl());
            }
            Self::TouchRotate { fingers, direction } => {
                push_int_prop(node, "fingers", fingers as i128);
                push_string_prop(node, "direction", direction.as_kdl());
            }
            Self::TouchEdge { edge, zone } => {
                push_string_prop(node, "edge", edge.as_kdl());
                if let Some(z) = zone {
                    push_string_prop(node, "zone", z.as_kdl(edge));
                }
            }
            Self::TouchTap { fingers }
            | Self::TouchpadTapHold { fingers }
            | Self::TouchpadTapHoldDrag { fingers } => {
                push_int_prop(node, "fingers", fingers as i128);
            }
            Self::TouchTapHoldDrag { fingers, direction } => {
                push_int_prop(node, "fingers", fingers as i128);
                if let Some(d) = direction {
                    push_string_prop(node, "direction", d.as_kdl());
                }
            }
        }
    }
}

/// Push a property entry like `direction="up"` with forced quoted repr.
fn push_string_prop(node: &mut KdlNode, key: &str, value: &str) {
    let mut entry = KdlEntry::new(KdlValue::String(value.to_string()));
    entry.set_name(Some(KdlIdentifier::from(key)));
    let mut fmt = kdl::KdlEntryFormat::default();
    fmt.value_repr = format!("\"{}\"", value);
    fmt.autoformat_keep = true;
    entry.set_format(fmt);
    node.push(entry);
}

/// Push an integer property entry like `fingers=4`.
fn push_int_prop(node: &mut KdlNode, key: &str, value: i128) {
    let mut entry = KdlEntry::new(KdlValue::Integer(value));
    entry.set_name(Some(KdlIdentifier::from(key)));
    node.push(entry);
}

// ---------------------------------------------------------------------------
// Touch bind entry (touchscreen + touchpad dynamic binds)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TouchBindEntry {
    /// Structured gesture trigger (family + properties).
    pub trigger: Trigger,
    /// Action KDL name, e.g. "focus-workspace-up", "close-window", "spawn", "noop"
    pub action_name: String,
    /// Arguments for actions that take them (e.g. spawn "alacritty")
    pub action_args: Vec<String>,
    /// Sensitivity multiplier (only meaningful for continuous actions)
    pub sensitivity: Option<f64>,
    /// Natural scroll toggle
    pub natural_scroll: bool,
    /// IPC tag for gesture events (external tool integration)
    pub tag: Option<String>,
    /// Whether this bind is enabled (disabled binds are dropped on write)
    pub enabled: bool,
}

// ---------------------------------------------------------------------------
// Touchscreen settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TouchscreenSettings {
    pub off: bool,
    pub natural_scroll: bool,
    pub map_to_output: Option<String>,
    // Classifier commit gates
    pub swipe_trigger_distance: f64,
    pub edge_start_distance: f64,
    pub pinch_trigger_distance: f64,
    pub pinch_dominance_ratio: f64,
    pub pinch_sensitivity: f64,
    pub swipe_multi_finger_scale: f64,
    // Rotation detection
    pub rotation_trigger_angle: f64,
    pub rotation_dominance_ratio: f64,
    pub rotation_progress_angle: f64,
    // Tap detection
    pub tap_wobble_threshold: f64,
    pub tap_timeout_ms: f64,
    pub tap_hold_trigger_delay_ms: f64,
    // IPC progress scaling
    pub swipe_progress_distance: f64,
    // Dynamic touch binds
    pub binds: Vec<TouchBindEntry>,
}

impl Default for TouchscreenSettings {
    fn default() -> Self {
        // Values match niri's current Touchscreen defaults
        // (niri-config/src/input.rs).
        Self {
            off: false,
            natural_scroll: false,
            map_to_output: None,
            swipe_trigger_distance: 100.0,
            edge_start_distance: 12.0,
            pinch_trigger_distance: 100.0,
            pinch_dominance_ratio: 1.0,
            pinch_sensitivity: 1.0,
            swipe_multi_finger_scale: 1.2,
            rotation_trigger_angle: 20.0,
            rotation_dominance_ratio: 0.5,
            rotation_progress_angle: 90.0,
            tap_wobble_threshold: 15.0,
            tap_timeout_ms: 500.0,
            tap_hold_trigger_delay_ms: 200.0,
            swipe_progress_distance: 200.0,
            binds: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Touchpad settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TouchpadSettings {
    // Device settings
    pub off: bool,
    pub tap: bool,
    pub dwt: bool,
    pub dwtp: bool,
    pub drag: Option<bool>,
    pub drag_lock: bool,
    pub natural_scroll: bool,
    pub click_method: Option<String>,
    pub accel_speed: f64,
    pub accel_profile: Option<String>,
    pub scroll_method: Option<String>,
    pub scroll_button: Option<u32>,
    pub scroll_button_lock: bool,
    pub tap_button_map: Option<String>,
    pub left_handed: bool,
    pub disabled_on_external_mouse: bool,
    pub middle_emulation: bool,
    pub scroll_factor: Option<f64>,
    // Gesture settings
    pub swipe_trigger_distance: f64,
    // IPC progress scaling
    pub swipe_progress_distance: f64,
    // Dynamic gesture binds (in binds {} block)
    pub binds: Vec<TouchBindEntry>,
}

impl Default for TouchpadSettings {
    fn default() -> Self {
        Self {
            off: false,
            tap: false,
            dwt: false,
            dwtp: false,
            drag: None,
            drag_lock: false,
            natural_scroll: false,
            click_method: None,
            accel_speed: 0.0,
            accel_profile: None,
            scroll_method: None,
            scroll_button: None,
            scroll_button_lock: false,
            tap_button_map: None,
            left_handed: false,
            disabled_on_external_mouse: false,
            middle_emulation: false,
            scroll_factor: None,
            swipe_trigger_distance: 16.0,
            swipe_progress_distance: 40.0,
            binds: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Config paths
// ---------------------------------------------------------------------------

fn config_dir() -> PathBuf {
    if let Ok(path) = std::env::var("NIRI_CONFIG") {
        let p = PathBuf::from(path);
        if let Some(parent) = p.parent() {
            return parent.to_path_buf();
        }
    }

    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        let p = PathBuf::from(xdg).join("niri");
        if p.exists() {
            return p;
        }
    }

    let home = std::env::var("HOME").expect("HOME not set");
    PathBuf::from(home).join(".config/niri")
}

fn main_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("NIRI_CONFIG") {
        return PathBuf::from(path);
    }
    config_dir().join("config.kdl")
}

fn touchscreen_config_path() -> PathBuf {
    config_dir().join(TOUCHSCREEN_GESTURES_FILE)
}

fn touchpad_config_path() -> PathBuf {
    config_dir().join(TOUCHPAD_GESTURES_FILE)
}

// ---------------------------------------------------------------------------
// Ensure include lines
// ---------------------------------------------------------------------------

pub fn ensure_includes() {
    let main_path = main_config_path();
    let content = fs::read_to_string(&main_path).unwrap_or_default();

    let has_touchscreen =
        content.contains("touchscreen-gestures.kdl") || content.contains("touch-gestures.kdl");
    let has_touchpad = content.contains("touchpad-gestures.kdl");

    if has_touchscreen && has_touchpad {
        return;
    }

    // Add the full include block if either is missing.
    let new_content = if content.ends_with('\n') {
        format!("{content}\n{INCLUDE_BLOCK}")
    } else {
        format!("{content}\n\n{INCLUDE_BLOCK}")
    };
    fs::write(&main_path, new_content).expect("failed to update main config");
}

// ---------------------------------------------------------------------------
// Read touchscreen settings
// ---------------------------------------------------------------------------

pub fn read_touchscreen_settings() -> TouchscreenSettings {
    let path = touchscreen_config_path();
    let content = fs::read_to_string(&path).unwrap_or_default();
    parse_touchscreen_settings(&content)
}

fn parse_touchscreen_settings(content: &str) -> TouchscreenSettings {
    let v2_content = kdl_v1_to_v2(content);
    let doc: KdlDocument = v2_content.parse().unwrap_or_default();
    let mut settings = TouchscreenSettings::default();

    let Some(input_node) = doc.get("input") else {
        return settings;
    };
    let Some(input_children) = input_node.children() else {
        return settings;
    };
    let Some(ts_node) = input_children.get("touchscreen") else {
        return settings;
    };
    let Some(ts_children) = ts_node.children() else {
        return settings;
    };

    settings.off = ts_children.get("off").is_some();
    settings.natural_scroll = ts_children.get("natural-scroll").is_some();

    if let Some(node) = ts_children.get("map-to-output") {
        if let Some(entry) = node.entries().first() {
            if let Some(s) = entry.value().as_string() {
                settings.map_to_output = Some(s.to_string());
            }
        }
    }

    if let Some(gestures_node) = ts_children.get("gestures") {
        if let Some(gestures_children) = gestures_node.children() {
            if let Some(v) = read_float_arg(gestures_children, "swipe-trigger-distance") {
                settings.swipe_trigger_distance = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "edge-start-distance") {
                settings.edge_start_distance = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "pinch-trigger-distance") {
                settings.pinch_trigger_distance = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "pinch-dominance-ratio") {
                settings.pinch_dominance_ratio = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "pinch-sensitivity") {
                settings.pinch_sensitivity = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "swipe-multi-finger-scale") {
                settings.swipe_multi_finger_scale = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "rotation-trigger-angle") {
                settings.rotation_trigger_angle = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "rotation-dominance-ratio") {
                settings.rotation_dominance_ratio = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "rotation-progress-angle") {
                settings.rotation_progress_angle = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "tap-wobble-threshold") {
                settings.tap_wobble_threshold = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "tap-timeout-ms") {
                settings.tap_timeout_ms = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "tap-hold-trigger-delay-ms") {
                settings.tap_hold_trigger_delay_ms = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "swipe-progress-distance") {
                settings.swipe_progress_distance = v;
            }
        }
    }

    // Gesture binds — now in a top-level `binds {}` block (same file or main config)
    if let Some(binds_node) = doc.get("binds") {
        if let Some(binds_children) = binds_node.children() {
            settings.binds = read_gesture_binds(binds_children);
        }
    }

    settings
}

/// Read gesture binds from a `binds {}` KdlDocument. Filters to only
/// nodes whose name (stripped of any modifier prefix) parses as a
/// known Trigger family with valid properties.
fn read_gesture_binds(binds_doc: &KdlDocument) -> Vec<TouchBindEntry> {
    let mut binds = Vec::new();

    for node in binds_doc.nodes() {
        let Some(trigger) = Trigger::parse_node(node) else {
            continue;
        };

        // Read properties: sensitivity, natural-scroll, tag
        let sensitivity = node
            .get("sensitivity")
            .and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64)));
        let natural_scroll = node
            .get("natural-scroll")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let tag = node
            .get("tag")
            .and_then(|v| v.as_string())
            .map(|s| s.to_string());

        // Read action from the single child node
        let mut action_name = String::new();
        let mut action_args = Vec::new();

        if let Some(children) = node.children() {
            if let Some(action_node) = children.nodes().first() {
                action_name = action_node.name().to_string();
                for entry in action_node.entries() {
                    if entry.name().is_none() {
                        if let Some(s) = entry.value().as_string() {
                            action_args.push(s.to_string());
                        }
                    }
                }
            }
        }

        if !action_name.is_empty() {
            binds.push(TouchBindEntry {
                trigger,
                action_name,
                action_args,
                sensitivity,
                natural_scroll,
                tag,
                enabled: true,
            });
        }
    }

    binds
}

// ---------------------------------------------------------------------------
// Write touchscreen settings
// ---------------------------------------------------------------------------

pub fn write_touchscreen_settings(settings: &TouchscreenSettings) {
    let mut doc = KdlDocument::new();
    let mut input_node = KdlNode::new("input");
    let input_children = input_node.ensure_children();

    let mut ts_node = KdlNode::new("touchscreen");
    let ts_children = ts_node.ensure_children();

    if settings.off {
        ts_children.nodes_mut().push(KdlNode::new("off"));
    }
    if settings.natural_scroll {
        ts_children.nodes_mut().push(KdlNode::new("natural-scroll"));
    }
    if let Some(ref output) = settings.map_to_output {
        let mut node = KdlNode::new("map-to-output");
        node.push(kdl::KdlEntry::new(KdlValue::String(output.clone())));
        ts_children.nodes_mut().push(node);
    }

    let mut gestures_node = KdlNode::new("gestures");
    let gestures_children = gestures_node.ensure_children();

    // Classifier commit gates + rotation detection + IPC scaling
    write_float_node(
        gestures_children,
        "swipe-trigger-distance",
        settings.swipe_trigger_distance,
    );
    write_float_node(
        gestures_children,
        "edge-start-distance",
        settings.edge_start_distance,
    );
    write_float_node(
        gestures_children,
        "pinch-trigger-distance",
        settings.pinch_trigger_distance,
    );
    write_float_node(
        gestures_children,
        "pinch-dominance-ratio",
        settings.pinch_dominance_ratio,
    );
    write_float_node(
        gestures_children,
        "pinch-sensitivity",
        settings.pinch_sensitivity,
    );
    write_float_node(
        gestures_children,
        "swipe-multi-finger-scale",
        settings.swipe_multi_finger_scale,
    );
    write_float_node(
        gestures_children,
        "rotation-trigger-angle",
        settings.rotation_trigger_angle,
    );
    write_float_node(
        gestures_children,
        "rotation-dominance-ratio",
        settings.rotation_dominance_ratio,
    );
    write_float_node(
        gestures_children,
        "rotation-progress-angle",
        settings.rotation_progress_angle,
    );
    write_float_node(
        gestures_children,
        "tap-wobble-threshold",
        settings.tap_wobble_threshold,
    );
    write_float_node(gestures_children, "tap-timeout-ms", settings.tap_timeout_ms);
    write_float_node(
        gestures_children,
        "tap-hold-trigger-delay-ms",
        settings.tap_hold_trigger_delay_ms,
    );
    write_float_node(
        gestures_children,
        "swipe-progress-distance",
        settings.swipe_progress_distance,
    );

    ts_children.nodes_mut().push(gestures_node);
    input_children.nodes_mut().push(ts_node);
    doc.nodes_mut().push(input_node);

    // Gesture binds as top-level `binds {}` block (niri merges with main config)
    write_gesture_binds(&mut doc, &settings.binds);

    write_config_file(&touchscreen_config_path(), &mut doc);
}

/// Write gesture binds as a top-level `binds {}` block using the
/// property-form trigger syntax niri requires.
fn write_gesture_binds(doc: &mut KdlDocument, binds: &[TouchBindEntry]) {
    if binds.is_empty() {
        return;
    }

    let mut binds_node = KdlNode::new("binds");
    let binds_children = binds_node.ensure_children();

    for bind in binds {
        if !bind.enabled {
            continue;
        }

        let mut node = KdlNode::new(bind.trigger.kdl_family());

        // Trigger properties (fingers, direction, edge, zone) go first
        // so the node name reads naturally when autoformatted.
        bind.trigger.write_props(&mut node);

        // User-facing bind properties: tag, sensitivity, natural-scroll
        if let Some(ref tag) = bind.tag {
            push_string_prop(&mut node, "tag", tag);
        }

        if let Some(sens) = bind.sensitivity {
            let rounded = (sens * 100.0).round() / 100.0;
            let mut entry = KdlEntry::new(KdlValue::Float(rounded));
            entry.set_name(Some(KdlIdentifier::from("sensitivity")));
            node.push(entry);
        }

        if bind.natural_scroll {
            let mut entry = KdlEntry::new(KdlValue::Bool(true));
            entry.set_name(Some(KdlIdentifier::from("natural-scroll")));
            node.push(entry);
        }

        // Action as child node
        let action_children = node.ensure_children();
        let mut action_node = KdlNode::new(bind.action_name.as_str());
        for arg in &bind.action_args {
            let mut entry = KdlEntry::new(KdlValue::String(arg.clone()));
            let mut fmt = kdl::KdlEntryFormat::default();
            fmt.leading = " ".into();
            fmt.value_repr = format!("\"{}\"", arg);
            fmt.autoformat_keep = true;
            entry.set_format(fmt);
            action_node.push(entry);
        }
        action_children.nodes_mut().push(action_node);

        binds_children.nodes_mut().push(node);
    }

    doc.nodes_mut().push(binds_node);
}

// ---------------------------------------------------------------------------
// Read touchpad settings
// ---------------------------------------------------------------------------

pub fn read_touchpad_settings() -> TouchpadSettings {
    let path = touchpad_config_path();
    let content = fs::read_to_string(&path).unwrap_or_default();
    parse_touchpad_settings(&content)
}

fn parse_touchpad_settings(content: &str) -> TouchpadSettings {
    let doc: KdlDocument = content.parse().unwrap_or_default();
    let mut settings = TouchpadSettings::default();

    let Some(input_node) = doc.get("input") else {
        return settings;
    };
    let Some(input_children) = input_node.children() else {
        return settings;
    };
    let Some(tp_node) = input_children.get("touchpad") else {
        return settings;
    };
    let Some(tp_children) = tp_node.children() else {
        return settings;
    };

    // Device settings
    settings.off = tp_children.get("off").is_some();
    settings.tap = tp_children.get("tap").is_some();
    settings.dwt = tp_children.get("dwt").is_some();
    settings.dwtp = tp_children.get("dwtp").is_some();
    settings.drag = read_optional_bool(tp_children, "drag");
    settings.drag_lock = tp_children.get("drag-lock").is_some();
    settings.natural_scroll = tp_children.get("natural-scroll").is_some();
    settings.click_method = read_string_arg(tp_children, "click-method");
    settings.accel_speed = read_float_arg(tp_children, "accel-speed").unwrap_or(0.0);
    settings.accel_profile = read_string_arg(tp_children, "accel-profile");
    settings.scroll_method = read_string_arg(tp_children, "scroll-method");
    settings.scroll_button = read_int_arg(tp_children, "scroll-button").map(|v| v as u32);
    settings.scroll_button_lock = tp_children.get("scroll-button-lock").is_some();
    settings.tap_button_map = read_string_arg(tp_children, "tap-button-map");
    settings.left_handed = tp_children.get("left-handed").is_some();
    settings.disabled_on_external_mouse = tp_children.get("disabled-on-external-mouse").is_some();
    settings.middle_emulation = tp_children.get("middle-emulation").is_some();
    settings.scroll_factor = read_float_arg(tp_children, "scroll-factor");

    // Gesture tuning
    if let Some(gestures_node) = tp_children.get("gestures") {
        if let Some(gestures_children) = gestures_node.children() {
            if let Some(v) = read_float_arg(gestures_children, "swipe-trigger-distance") {
                settings.swipe_trigger_distance = v;
            }
            if let Some(v) = read_float_arg(gestures_children, "swipe-progress-distance") {
                settings.swipe_progress_distance = v;
            }
        }
    }

    // Gesture binds from top-level binds {} block
    let v2_content = kdl_v1_to_v2(content);
    let v2_doc: KdlDocument = v2_content.parse().unwrap_or_default();
    if let Some(binds_node) = v2_doc.get("binds") {
        if let Some(binds_children) = binds_node.children() {
            settings.binds = read_gesture_binds(binds_children);
        }
    }

    settings
}

// ---------------------------------------------------------------------------
// Write touchpad settings
// ---------------------------------------------------------------------------

pub fn write_touchpad_settings(settings: &TouchpadSettings) {
    let mut doc = KdlDocument::new();
    let mut input_node = KdlNode::new("input");
    let input_children = input_node.ensure_children();

    let mut tp_node = KdlNode::new("touchpad");
    let tp_children = tp_node.ensure_children();

    // Device settings -- bool flags
    if settings.off {
        tp_children.nodes_mut().push(KdlNode::new("off"));
    }
    if settings.tap {
        tp_children.nodes_mut().push(KdlNode::new("tap"));
    }
    if settings.dwt {
        tp_children.nodes_mut().push(KdlNode::new("dwt"));
    }
    if settings.dwtp {
        tp_children.nodes_mut().push(KdlNode::new("dwtp"));
    }
    if let Some(drag) = settings.drag {
        let mut node = KdlNode::new("drag");
        node.push(kdl::KdlEntry::new(KdlValue::Bool(drag)));
        tp_children.nodes_mut().push(node);
    }
    if settings.drag_lock {
        tp_children.nodes_mut().push(KdlNode::new("drag-lock"));
    }
    if settings.natural_scroll {
        tp_children.nodes_mut().push(KdlNode::new("natural-scroll"));
    }
    if let Some(ref method) = settings.click_method {
        write_string_node(tp_children, "click-method", method);
    }
    if settings.accel_speed != 0.0 {
        write_float_node(tp_children, "accel-speed", settings.accel_speed);
    }
    if let Some(ref profile) = settings.accel_profile {
        write_string_node(tp_children, "accel-profile", profile);
    }
    if let Some(ref method) = settings.scroll_method {
        write_string_node(tp_children, "scroll-method", method);
    }
    if let Some(button) = settings.scroll_button {
        let mut node = KdlNode::new("scroll-button");
        node.push(kdl::KdlEntry::new(KdlValue::Integer(button as i128)));
        tp_children.nodes_mut().push(node);
    }
    if settings.scroll_button_lock {
        tp_children
            .nodes_mut()
            .push(KdlNode::new("scroll-button-lock"));
    }
    if let Some(ref map) = settings.tap_button_map {
        write_string_node(tp_children, "tap-button-map", map);
    }
    if settings.left_handed {
        tp_children.nodes_mut().push(KdlNode::new("left-handed"));
    }
    if settings.disabled_on_external_mouse {
        tp_children
            .nodes_mut()
            .push(KdlNode::new("disabled-on-external-mouse"));
    }
    if settings.middle_emulation {
        tp_children
            .nodes_mut()
            .push(KdlNode::new("middle-emulation"));
    }
    if let Some(factor) = settings.scroll_factor {
        write_float_node(tp_children, "scroll-factor", factor);
    }

    // Gesture settings
    let mut gestures_node = KdlNode::new("gestures");
    let gestures_children = gestures_node.ensure_children();
    write_float_node(
        gestures_children,
        "swipe-trigger-distance",
        settings.swipe_trigger_distance,
    );
    write_float_node(
        gestures_children,
        "swipe-progress-distance",
        settings.swipe_progress_distance,
    );

    tp_children.nodes_mut().push(gestures_node);
    input_children.nodes_mut().push(tp_node);
    doc.nodes_mut().push(input_node);

    // Gesture binds as top-level binds {} block
    write_gesture_binds(&mut doc, &settings.binds);

    write_config_file(&touchpad_config_path(), &mut doc);
}

// ---------------------------------------------------------------------------
// Shared KDL readers
// ---------------------------------------------------------------------------

fn read_string_arg(doc: &KdlDocument, name: &str) -> Option<String> {
    let node = doc.get(name)?;
    let entry = node.entries().first()?;
    entry.value().as_string().map(|s| s.to_string())
}

fn read_float_arg(doc: &KdlDocument, name: &str) -> Option<f64> {
    let node = doc.get(name)?;
    let entry = node.entries().first()?;
    if let Some(v) = entry.value().as_float() {
        Some(v)
    } else {
        entry.value().as_integer().map(|v| v as f64)
    }
}

fn read_int_arg(doc: &KdlDocument, name: &str) -> Option<i128> {
    let node = doc.get(name)?;
    let entry = node.entries().first()?;
    entry.value().as_integer()
}

fn read_optional_bool(doc: &KdlDocument, name: &str) -> Option<bool> {
    let node = doc.get(name)?;
    if let Some(entry) = node.entries().first() {
        entry.value().as_bool()
    } else {
        Some(true)
    }
}

// ---------------------------------------------------------------------------
// Shared KDL writers
// ---------------------------------------------------------------------------

fn write_string_node(doc: &mut KdlDocument, name: &str, value: &str) {
    let mut node = KdlNode::new(name);
    let mut entry = kdl::KdlEntry::new(KdlValue::String(value.to_string()));
    // Force quoted output -- KDL v6 outputs bare identifiers for simple strings,
    // but niri's knuffel parser requires quoted strings.
    let mut fmt = kdl::KdlEntryFormat::default();
    fmt.leading = " ".into();
    fmt.value_repr = format!("\"{}\"", value);
    fmt.autoformat_keep = true;
    entry.set_format(fmt);
    node.push(entry);
    doc.nodes_mut().push(node);
}

fn write_float_node(doc: &mut KdlDocument, name: &str, value: f64) {
    let rounded = (value * 100.0).round() / 100.0;
    let mut node = KdlNode::new(name);
    node.push(kdl::KdlEntry::new(KdlValue::Float(rounded)));
    doc.nodes_mut().push(node);
}

fn write_config_file(path: &PathBuf, doc: &mut KdlDocument) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    doc.autoformat();
    // Convert KDL v2 booleans (#true/#false) back to v1 (true/false)
    // so niri's knuffel parser can read them.
    let output = kdl_v2_to_v1(&doc.to_string());
    fs::write(path, output).expect("failed to write config");
}

// ---------------------------------------------------------------------------
// Reload
// ---------------------------------------------------------------------------

pub fn reload_config() {
    let _ = std::process::Command::new("niri")
        .args(["msg", "action", "load-config-file"])
        .spawn();
}
