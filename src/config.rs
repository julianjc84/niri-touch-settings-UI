use std::fs;
use std::path::PathBuf;

use kdl::{KdlDocument, KdlNode, KdlValue};

const TOUCHSCREEN_GESTURES_FILE: &str = "touchscreen-gestures.kdl";
const TOUCHPAD_GESTURES_FILE: &str = "touchpad-gestures.kdl";

const INCLUDE_BLOCK: &str = "\
// Includes override existing sections.\n\
// Settings in these files replace the corresponding blocks in this config.\n\
include \"touchscreen-gestures.kdl\" optional=true\n\
include \"touchpad-gestures.kdl\" optional=true\n";

// ---------------------------------------------------------------------------
// Shared gesture action type
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GestureAction {
    pub enabled: bool,
    pub finger_count: u8,
    pub sensitivity: f64,
    pub natural_scroll: bool,
}

// ---------------------------------------------------------------------------
// Edge swipe settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct EdgeSwipeSettings {
    pub enabled: bool,
    pub action: String,       // "view-scroll", "workspace-switch", "overview-toggle"
    pub sensitivity: f64,
}

impl Default for EdgeSwipeSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            action: String::new(),
            sensitivity: 0.4,
        }
    }
}

// ---------------------------------------------------------------------------
// Touchscreen settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TouchscreenSettings {
    pub off: bool,
    pub natural_scroll: bool,
    pub map_to_output: Option<String>,
    pub workspace_switch: GestureAction,
    pub view_scroll: GestureAction,
    pub overview_toggle: GestureAction,
    pub recognition_threshold: f64,
    pub edge_threshold: f64,
    pub edge_swipe_left: EdgeSwipeSettings,
    pub edge_swipe_right: EdgeSwipeSettings,
    pub edge_swipe_top: EdgeSwipeSettings,
    pub edge_swipe_bottom: EdgeSwipeSettings,
}

impl Default for TouchscreenSettings {
    fn default() -> Self {
        Self {
            off: false,
            natural_scroll: false,
            map_to_output: None,
            workspace_switch: GestureAction {
                enabled: true,
                finger_count: 3,
                sensitivity: 0.4,
                natural_scroll: false,
            },
            view_scroll: GestureAction {
                enabled: true,
                finger_count: 3,
                sensitivity: 0.4,
                natural_scroll: false,
            },
            overview_toggle: GestureAction {
                enabled: true,
                finger_count: 4,
                sensitivity: 0.4,
                natural_scroll: false,
            },
            recognition_threshold: 16.0,
            edge_threshold: 20.0,
            edge_swipe_left: EdgeSwipeSettings::default(),
            edge_swipe_right: EdgeSwipeSettings::default(),
            edge_swipe_top: EdgeSwipeSettings::default(),
            edge_swipe_bottom: EdgeSwipeSettings::default(),
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
    pub workspace_switch: GestureAction,
    pub view_scroll: GestureAction,
    pub overview_toggle: GestureAction,
    pub recognition_threshold: f64,
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
            workspace_switch: GestureAction {
                enabled: true,
                finger_count: 3,
                sensitivity: 1.0,
                natural_scroll: false,
            },
            view_scroll: GestureAction {
                enabled: true,
                finger_count: 3,
                sensitivity: 1.0,
                natural_scroll: false,
            },
            overview_toggle: GestureAction {
                enabled: true,
                finger_count: 4,
                sensitivity: 1.0,
                natural_scroll: false,
            },
            recognition_threshold: 16.0,
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
    let doc: KdlDocument = content.parse().unwrap_or_default();
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

    read_gesture_action(gestures_children, "workspace-switch", &mut settings.workspace_switch);
    read_gesture_action(gestures_children, "view-scroll", &mut settings.view_scroll);
    read_gesture_action(gestures_children, "overview-toggle", &mut settings.overview_toggle);
    read_threshold(gestures_children, &mut settings.recognition_threshold);

    if let Some(v) = read_float_arg(gestures_children, "edge-threshold") {
        settings.edge_threshold = v;
    }
    read_edge_swipe(gestures_children, "edge-swipe-left", &mut settings.edge_swipe_left);
    read_edge_swipe(gestures_children, "edge-swipe-right", &mut settings.edge_swipe_right);
    read_edge_swipe(gestures_children, "edge-swipe-top", &mut settings.edge_swipe_top);
    read_edge_swipe(gestures_children, "edge-swipe-bottom", &mut settings.edge_swipe_bottom);

    settings
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

    // Gesture settings
    if let Some(gestures_node) = tp_children.get("gestures") {
        if let Some(gestures_children) = gestures_node.children() {
            read_gesture_action(gestures_children, "workspace-switch", &mut settings.workspace_switch);
            read_gesture_action(gestures_children, "view-scroll", &mut settings.view_scroll);
            read_gesture_action(gestures_children, "overview-toggle", &mut settings.overview_toggle);
            read_threshold(gestures_children, &mut settings.recognition_threshold);
        }
    }

    settings
}

// ---------------------------------------------------------------------------
// Shared KDL readers
// ---------------------------------------------------------------------------

fn read_gesture_action(doc: &KdlDocument, name: &str, action: &mut GestureAction) {
    let Some(node) = doc.get(name) else { return };
    let Some(children) = node.children() else { return };

    if children.get("off").is_some() {
        action.enabled = false;
    }

    if let Some(node) = children.get("finger-count") {
        if let Some(entry) = node.entries().first() {
            if let Some(v) = entry.value().as_integer() {
                action.finger_count = v as u8;
            }
        }
    }

    if let Some(node) = children.get("sensitivity") {
        if let Some(entry) = node.entries().first() {
            if let Some(v) = entry.value().as_float() {
                action.sensitivity = v;
            } else if let Some(v) = entry.value().as_integer() {
                action.sensitivity = v as f64;
            }
        }
    }

    if children.get("natural-scroll").is_some() {
        action.natural_scroll = true;
    }
}

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

fn read_edge_swipe(doc: &KdlDocument, name: &str, edge: &mut EdgeSwipeSettings) {
    let Some(node) = doc.get(name) else { return };

    // The action is the first string argument: edge-swipe-left "view-scroll"
    if let Some(entry) = node.entries().first() {
        if let Some(s) = entry.value().as_string() {
            edge.action = s.to_string();
            edge.enabled = true;
        }
    }

    // Optional child properties: sensitivity, off
    if let Some(children) = node.children() {
        if children.get("off").is_some() {
            edge.enabled = false;
        }
        if let Some(sens_node) = children.get("sensitivity") {
            if let Some(entry) = sens_node.entries().first() {
                if let Some(v) = entry.value().as_float() {
                    edge.sensitivity = v;
                } else if let Some(v) = entry.value().as_integer() {
                    edge.sensitivity = v as f64;
                }
            }
        }
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

    write_gesture_action(gestures_children, "workspace-switch", &settings.workspace_switch);
    write_gesture_action(gestures_children, "view-scroll", &settings.view_scroll);
    write_gesture_action(gestures_children, "overview-toggle", &settings.overview_toggle);
    write_threshold(gestures_children, settings.recognition_threshold);

    write_float_node(gestures_children, "edge-threshold", settings.edge_threshold);
    write_edge_swipe(gestures_children, "edge-swipe-left", &settings.edge_swipe_left);
    write_edge_swipe(gestures_children, "edge-swipe-right", &settings.edge_swipe_right);
    write_edge_swipe(gestures_children, "edge-swipe-top", &settings.edge_swipe_top);
    write_edge_swipe(gestures_children, "edge-swipe-bottom", &settings.edge_swipe_bottom);

    ts_children.nodes_mut().push(gestures_node);
    input_children.nodes_mut().push(ts_node);
    doc.nodes_mut().push(input_node);

    write_config_file(&touchscreen_config_path(), &mut doc);
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

    // Device settings — bool flags
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

    // Gesture settings
    let mut gestures_node = KdlNode::new("gestures");
    let gestures_children = gestures_node.ensure_children();

    write_gesture_action(gestures_children, "workspace-switch", &settings.workspace_switch);
    write_gesture_action(gestures_children, "view-scroll", &settings.view_scroll);
    write_gesture_action(gestures_children, "overview-toggle", &settings.overview_toggle);
    write_threshold(gestures_children, settings.recognition_threshold);

    tp_children.nodes_mut().push(gestures_node);
    input_children.nodes_mut().push(tp_node);
    doc.nodes_mut().push(input_node);

    write_config_file(&touchpad_config_path(), &mut doc);
}

// ---------------------------------------------------------------------------
// Shared KDL writers
// ---------------------------------------------------------------------------

fn write_gesture_action(doc: &mut KdlDocument, name: &str, action: &GestureAction) {
    let mut node = KdlNode::new(name);
    let children = node.ensure_children();

    if !action.enabled {
        children.nodes_mut().push(KdlNode::new("off"));
    }
    if action.natural_scroll {
        children.nodes_mut().push(KdlNode::new("natural-scroll"));
    }

    let mut fc_node = KdlNode::new("finger-count");
    fc_node.push(kdl::KdlEntry::new(KdlValue::Integer(action.finger_count as i128)));
    children.nodes_mut().push(fc_node);

    let mut sens_node = KdlNode::new("sensitivity");
    let rounded_sens = (action.sensitivity * 100.0).round() / 100.0;
    sens_node.push(kdl::KdlEntry::new(KdlValue::Float(rounded_sens)));
    children.nodes_mut().push(sens_node);

    doc.nodes_mut().push(node);
}

fn write_threshold(doc: &mut KdlDocument, threshold: f64) {
    let mut node = KdlNode::new("recognition-threshold");
    let rounded_threshold = (threshold * 100.0).round() / 100.0;
    node.push(kdl::KdlEntry::new(KdlValue::Float(rounded_threshold)));
    doc.nodes_mut().push(node);
}

fn write_edge_swipe(doc: &mut KdlDocument, name: &str, edge: &EdgeSwipeSettings) {
    // Only write if the edge has an action configured.
    if edge.action.is_empty() {
        return;
    }

    let mut node = KdlNode::new(name);

    // Write the action as a quoted string argument.
    let mut entry = kdl::KdlEntry::new(KdlValue::String(edge.action.clone()));
    let mut fmt = kdl::KdlEntryFormat::default();
    fmt.leading = " ".into();
    fmt.value_repr = format!("\"{}\"", edge.action);
    fmt.autoformat_keep = true;
    entry.set_format(fmt);
    node.push(entry);

    // Write child properties if non-default.
    let has_children = !edge.enabled || edge.sensitivity != 0.4;
    if has_children {
        let children = node.ensure_children();
        if !edge.enabled {
            children.nodes_mut().push(KdlNode::new("off"));
        }
        if (edge.sensitivity - 0.4).abs() > 0.001 {
            let mut sens_node = KdlNode::new("sensitivity");
            let rounded = (edge.sensitivity * 100.0).round() / 100.0;
            sens_node.push(kdl::KdlEntry::new(KdlValue::Float(rounded)));
            children.nodes_mut().push(sens_node);
        }
    }

    doc.nodes_mut().push(node);
}

fn write_string_node(doc: &mut KdlDocument, name: &str, value: &str) {
    let mut node = KdlNode::new(name);
    let mut entry = kdl::KdlEntry::new(KdlValue::String(value.to_string()));
    // Force quoted output — KDL v6 outputs bare identifiers for simple strings,
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
    fs::write(path, doc.to_string()).expect("failed to write config");
}

// ---------------------------------------------------------------------------
// Reload
// ---------------------------------------------------------------------------

pub fn reload_config() {
    let _ = std::process::Command::new("niri")
        .args(["msg", "action", "load-config-file"])
        .spawn();
}
