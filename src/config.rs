use std::fs;
use std::path::PathBuf;

use kdl::{KdlDocument, KdlEntry, KdlIdentifier, KdlNode, KdlValue};

const TOUCHSCREEN_GESTURES_FILE: &str = "touchscreen-gestures.kdl";
const TOUCHPAD_GESTURES_FILE: &str = "touchpad-gestures.kdl";

// ---------------------------------------------------------------------------
// KDL v1 <-> v2 compatibility
// ---------------------------------------------------------------------------
// The `kdl` crate v6 implements KDL v2, where booleans are `#true`/`#false`.
// Niri's compositor uses `knuffel` which implements KDL v1 (`true`/`false`).
// These helpers bridge the two specs for property values.

fn kdl_v1_to_v2(content: &str) -> String {
    content.replace("=true", "=#true").replace("=false", "=#false")
}

fn kdl_v2_to_v1(content: &str) -> String {
    content.replace("=#true", "=true").replace("=#false", "=false")
}

const INCLUDE_BLOCK: &str = "\
// Includes override existing sections.\n\
// Settings in these files replace the corresponding blocks in this config.\n\
// Gesture binds in these files merge into the main binds {} block.\n\
include \"touchscreen-gestures.kdl\" optional=true\n\
include \"touchpad-gestures.kdl\" optional=true\n";

// ---------------------------------------------------------------------------
// Touch bind entry (touchscreen dynamic binds)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TouchBindEntry {
    /// KDL node name, e.g. "Touch3SwipeUp", "TouchEdgeLeft"
    pub gesture_name: String,
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
    /// Whether this bind is enabled (disabled binds are commented out with /-)
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
    // Detection thresholds
    pub recognition_threshold: f64,
    pub edge_threshold: f64,
    pub pinch_threshold: f64,
    pub pinch_ratio: f64,
    pub pinch_sensitivity: f64,
    pub finger_threshold_scale: f64,
    // Dynamic touch binds
    pub binds: Vec<TouchBindEntry>,
}

impl Default for TouchscreenSettings {
    fn default() -> Self {
        Self {
            off: false,
            natural_scroll: false,
            map_to_output: None,
            recognition_threshold: 16.0,
            edge_threshold: 20.0,
            pinch_threshold: 20.0,
            pinch_ratio: 2.0,
            pinch_sensitivity: 1.0,
            finger_threshold_scale: 2.6,
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
    pub recognition_threshold: f64,
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
            recognition_threshold: 16.0,
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

    let has_touchscreen = content.contains("touchscreen-gestures.kdl") || content.contains("touch-gestures.kdl");
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

    let Some(input_node) = doc.get("input") else { return settings };
    let Some(input_children) = input_node.children() else { return settings };
    let Some(ts_node) = input_children.get("touchscreen") else { return settings };
    let Some(ts_children) = ts_node.children() else { return settings };

    settings.off = ts_children.get("off").is_some();
    settings.natural_scroll = ts_children.get("natural-scroll").is_some();

    if let Some(node) = ts_children.get("map-to-output") {
        if let Some(entry) = node.entries().first() {
            if let Some(s) = entry.value().as_string() {
                settings.map_to_output = Some(s.to_string());
            }
        }
    }

    let Some(gestures_node) = ts_children.get("gestures") else { return settings };
    let Some(gestures_children) = gestures_node.children() else { return settings };

    // Detection thresholds
    if let Some(v) = read_float_arg(gestures_children, "recognition-threshold") {
        settings.recognition_threshold = v;
    }
    if let Some(v) = read_float_arg(gestures_children, "edge-threshold") {
        settings.edge_threshold = v;
    }
    if let Some(v) = read_float_arg(gestures_children, "pinch-threshold") {
        settings.pinch_threshold = v;
    }
    if let Some(v) = read_float_arg(gestures_children, "pinch-ratio") {
        settings.pinch_ratio = v;
    }
    if let Some(v) = read_float_arg(gestures_children, "pinch-sensitivity") {
        settings.pinch_sensitivity = v;
    }
    if let Some(v) = read_float_arg(gestures_children, "finger-threshold-scale") {
        settings.finger_threshold_scale = v;
    }

    // Gesture binds — now in a top-level `binds {}` block (same file or main config)
    if let Some(binds_node) = doc.get("binds") {
        if let Some(binds_children) = binds_node.children() {
            settings.binds = read_gesture_binds(binds_children);
        }
    }

    settings
}

/// Check if a node name is a gesture trigger (Touch* or Touchpad* or Mod+Touch*).
fn is_gesture_bind(name: &str) -> bool {
    // Handle modifier prefixes like "Mod+Touch3SwipeUp"
    let trigger = if let Some(pos) = name.rfind('+') {
        &name[pos + 1..]
    } else {
        name
    };
    trigger.starts_with("Touch") || trigger.starts_with("Touchpad")
}

/// Read gesture binds from a `binds {}` KdlDocument.
/// Filters to only Touch*/Touchpad* nodes.
fn read_gesture_binds(binds_doc: &KdlDocument) -> Vec<TouchBindEntry> {
    let mut binds = Vec::new();

    for node in binds_doc.nodes() {
        let gesture_name = node.name().to_string();

        // Only read gesture binds, skip keyboard/mouse
        if !is_gesture_bind(&gesture_name) {
            continue;
        }

        // Read properties: sensitivity, natural-scroll, tag
        let sensitivity = node.get("sensitivity").and_then(|v| {
            v.as_float().or_else(|| v.as_integer().map(|i| i as f64))
        });
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
                gesture_name,
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

    // Detection thresholds
    write_float_node(gestures_children, "recognition-threshold", settings.recognition_threshold);
    write_float_node(gestures_children, "edge-threshold", settings.edge_threshold);
    write_float_node(gestures_children, "pinch-threshold", settings.pinch_threshold);
    write_float_node(gestures_children, "pinch-ratio", settings.pinch_ratio);
    write_float_node(gestures_children, "pinch-sensitivity", settings.pinch_sensitivity);
    write_float_node(gestures_children, "finger-threshold-scale", settings.finger_threshold_scale);

    ts_children.nodes_mut().push(gestures_node);
    input_children.nodes_mut().push(ts_node);
    doc.nodes_mut().push(input_node);

    // Gesture binds as top-level `binds {}` block (niri merges with main config)
    write_gesture_binds(&mut doc, &settings.binds);

    write_config_file(&touchscreen_config_path(), &mut doc);
}

/// Write gesture binds as a top-level `binds {}` block.
/// Niri's include merge replaces matching bind keys, so this safely
/// coexists with the user's keyboard/mouse binds in config.kdl.
fn write_gesture_binds(doc: &mut KdlDocument, binds: &[TouchBindEntry]) {
    if binds.is_empty() {
        return;
    }

    let mut binds_node = KdlNode::new("binds");
    let binds_children = binds_node.ensure_children();

    for bind in binds {
        if !bind.enabled {
            continue; // Skip disabled binds
        }

        let mut node = KdlNode::new(bind.gesture_name.as_str());

        // Properties: tag, sensitivity, natural-scroll
        if let Some(ref tag) = bind.tag {
            let mut entry = KdlEntry::new(KdlValue::String(tag.clone()));
            entry.set_name(Some(KdlIdentifier::from("tag")));
            let mut fmt = kdl::KdlEntryFormat::default();
            fmt.value_repr = format!("\"{}\"", tag);
            fmt.autoformat_keep = true;
            entry.set_format(fmt);
            node.push(entry);
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

    let Some(input_node) = doc.get("input") else { return settings };
    let Some(input_children) = input_node.children() else { return settings };
    let Some(tp_node) = input_children.get("touchpad") else { return settings };
    let Some(tp_children) = tp_node.children() else { return settings };

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

    // Gesture recognition threshold
    if let Some(gestures_node) = tp_children.get("gestures") {
        if let Some(gestures_children) = gestures_node.children() {
            read_threshold(gestures_children, &mut settings.recognition_threshold);
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
    if settings.off { tp_children.nodes_mut().push(KdlNode::new("off")); }
    if settings.tap { tp_children.nodes_mut().push(KdlNode::new("tap")); }
    if settings.dwt { tp_children.nodes_mut().push(KdlNode::new("dwt")); }
    if settings.dwtp { tp_children.nodes_mut().push(KdlNode::new("dwtp")); }
    if let Some(drag) = settings.drag {
        let mut node = KdlNode::new("drag");
        node.push(kdl::KdlEntry::new(KdlValue::Bool(drag)));
        tp_children.nodes_mut().push(node);
    }
    if settings.drag_lock { tp_children.nodes_mut().push(KdlNode::new("drag-lock")); }
    if settings.natural_scroll { tp_children.nodes_mut().push(KdlNode::new("natural-scroll")); }
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
    if settings.scroll_button_lock { tp_children.nodes_mut().push(KdlNode::new("scroll-button-lock")); }
    if let Some(ref map) = settings.tap_button_map {
        write_string_node(tp_children, "tap-button-map", map);
    }
    if settings.left_handed { tp_children.nodes_mut().push(KdlNode::new("left-handed")); }
    if settings.disabled_on_external_mouse { tp_children.nodes_mut().push(KdlNode::new("disabled-on-external-mouse")); }
    if settings.middle_emulation { tp_children.nodes_mut().push(KdlNode::new("middle-emulation")); }
    if let Some(factor) = settings.scroll_factor {
        write_float_node(tp_children, "scroll-factor", factor);
    }

    // Gesture recognition threshold
    let mut gestures_node = KdlNode::new("gestures");
    let gestures_children = gestures_node.ensure_children();
    write_threshold(gestures_children, settings.recognition_threshold);

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

fn read_threshold(doc: &KdlDocument, threshold: &mut f64) {
    if let Some(node) = doc.get("recognition-threshold") {
        if let Some(entry) = node.entries().first() {
            if let Some(v) = entry.value().as_float() {
                *threshold = v;
            } else if let Some(v) = entry.value().as_integer() {
                *threshold = v as f64;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Shared KDL writers
// ---------------------------------------------------------------------------

fn write_threshold(doc: &mut KdlDocument, threshold: f64) {
    let mut node = KdlNode::new("recognition-threshold");
    let rounded_threshold = (threshold * 100.0).round() / 100.0;
    node.push(kdl::KdlEntry::new(KdlValue::Float(rounded_threshold)));
    doc.nodes_mut().push(node);
}

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
