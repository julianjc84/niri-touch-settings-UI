use std::fs;
use std::path::PathBuf;

use kdl::{KdlDocument, KdlNode, KdlValue};

const TOUCHSCREEN_GESTURES_FILE: &str = "touchscreen-gestures.kdl";
const TOUCHSCREEN_INCLUDE: &str = "include \"touchscreen-gestures.kdl\" optional=true\n";

const TOUCHPAD_GESTURES_FILE: &str = "touchpad-gestures.kdl";
const TOUCHPAD_INCLUDE: &str = "include \"touchpad-gestures.kdl\" optional=true\n";

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
        }
    }
}

// ---------------------------------------------------------------------------
// Touchpad settings
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TouchpadSettings {
    pub workspace_switch: GestureAction,
    pub view_scroll: GestureAction,
    pub overview_toggle: GestureAction,
    pub recognition_threshold: f64,
}

impl Default for TouchpadSettings {
    fn default() -> Self {
        Self {
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

    let mut additions = String::new();

    if !content.contains("touchscreen-gestures.kdl") && !content.contains("touch-gestures.kdl") {
        additions.push_str(TOUCHSCREEN_INCLUDE);
    }
    if !content.contains("touchpad-gestures.kdl") {
        additions.push_str(TOUCHPAD_INCLUDE);
    }

    if !additions.is_empty() {
        let new_content = if content.ends_with('\n') {
            format!("{content}\n{additions}")
        } else {
            format!("{content}\n\n{additions}")
        };
        fs::write(&main_path, new_content).expect("failed to update main config");
    }
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

    let Some(gestures_node) = tp_children.get("gestures") else { return settings };
    let Some(gestures_children) = gestures_node.children() else { return settings };

    read_gesture_action(gestures_children, "workspace-switch", &mut settings.workspace_switch);
    read_gesture_action(gestures_children, "view-scroll", &mut settings.view_scroll);
    read_gesture_action(gestures_children, "overview-toggle", &mut settings.overview_toggle);
    read_threshold(gestures_children, &mut settings.recognition_threshold);

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

    ts_children.nodes_mut().push(gestures_node);
    input_children.nodes_mut().push(ts_node);
    doc.nodes_mut().push(input_node);

    write_config_file(&touchscreen_config_path(), &doc);
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

    let mut gestures_node = KdlNode::new("gestures");
    let gestures_children = gestures_node.ensure_children();

    write_gesture_action(gestures_children, "workspace-switch", &settings.workspace_switch);
    write_gesture_action(gestures_children, "view-scroll", &settings.view_scroll);
    write_gesture_action(gestures_children, "overview-toggle", &settings.overview_toggle);
    write_threshold(gestures_children, settings.recognition_threshold);

    tp_children.nodes_mut().push(gestures_node);
    input_children.nodes_mut().push(tp_node);
    doc.nodes_mut().push(input_node);

    write_config_file(&touchpad_config_path(), &doc);
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
    sens_node.push(kdl::KdlEntry::new(KdlValue::Float(action.sensitivity)));
    children.nodes_mut().push(sens_node);

    doc.nodes_mut().push(node);
}

fn write_threshold(doc: &mut KdlDocument, threshold: f64) {
    let mut node = KdlNode::new("recognition-threshold");
    node.push(kdl::KdlEntry::new(KdlValue::Float(threshold)));
    doc.nodes_mut().push(node);
}

fn write_config_file(path: &PathBuf, doc: &KdlDocument) {
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
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
