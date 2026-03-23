use std::fs;
use std::path::PathBuf;

use kdl::{KdlDocument, KdlNode, KdlValue};

const TOUCH_GESTURES_FILE: &str = "touch-gestures.kdl";
const INCLUDE_LINE: &str = "include \"touch-gestures.kdl\" optional=true\n";

/// Resolved touch gesture settings with defaults applied.
#[derive(Debug, Clone)]
pub struct TouchSettings {
    pub off: bool,
    pub natural_scroll: bool,
    pub map_to_output: Option<String>,
    pub workspace_switch: GestureAction,
    pub view_scroll: GestureAction,
    pub overview_toggle: GestureAction,
    pub recognition_threshold: f64,
}

#[derive(Debug, Clone)]
pub struct GestureAction {
    pub enabled: bool,
    pub finger_count: u8,
    pub sensitivity: f64,
    pub natural_scroll: bool,
}

impl Default for TouchSettings {
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

/// Find the niri config directory.
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

/// Path to the separate touch gestures config file.
pub fn touch_config_path() -> PathBuf {
    config_dir().join(TOUCH_GESTURES_FILE)
}

/// Path to the main niri config file.
fn main_config_path() -> PathBuf {
    if let Ok(path) = std::env::var("NIRI_CONFIG") {
        return PathBuf::from(path);
    }

    config_dir().join("config.kdl")
}

/// Ensure the main config.kdl has an include line for touch-gestures.kdl.
pub fn ensure_include() {
    let main_path = main_config_path();
    let content = fs::read_to_string(&main_path).unwrap_or_default();

    // Check if the include already exists (any form of it).
    if content.contains("touch-gestures.kdl") {
        return;
    }

    // Append the include line to the end of the main config.
    let new_content = if content.ends_with('\n') {
        format!("{content}\n{INCLUDE_LINE}")
    } else {
        format!("{content}\n\n{INCLUDE_LINE}")
    };

    fs::write(&main_path, new_content).expect("failed to update main config");
}

/// Read touch gesture settings from the separate file.
pub fn read_settings() -> TouchSettings {
    let path = touch_config_path();
    let content = fs::read_to_string(&path).unwrap_or_default();
    parse_touch_settings(&content)
}

/// Parse touch settings from KDL content string.
/// The file contains just an `input { touch { ... } }` block.
fn parse_touch_settings(content: &str) -> TouchSettings {
    let doc: KdlDocument = content.parse().unwrap_or_default();
    let mut settings = TouchSettings::default();

    let Some(input_node) = doc.get("input") else {
        return settings;
    };
    let Some(input_children) = input_node.children() else {
        return settings;
    };
    let Some(touch_node) = input_children.get("touch") else {
        return settings;
    };
    let Some(touch_children) = touch_node.children() else {
        return settings;
    };

    settings.off = touch_children.get("off").is_some();
    settings.natural_scroll = touch_children.get("natural-scroll").is_some();

    if let Some(node) = touch_children.get("map-to-output") {
        if let Some(entry) = node.entries().first() {
            if let Some(s) = entry.value().as_string() {
                settings.map_to_output = Some(s.to_string());
            }
        }
    }

    let Some(gestures_node) = touch_children.get("gestures") else {
        return settings;
    };
    let Some(gestures_children) = gestures_node.children() else {
        return settings;
    };

    read_gesture_action(gestures_children, "workspace-switch", &mut settings.workspace_switch);
    read_gesture_action(gestures_children, "view-scroll", &mut settings.view_scroll);
    read_gesture_action(gestures_children, "overview-toggle", &mut settings.overview_toggle);

    if let Some(node) = gestures_children.get("recognition-threshold") {
        if let Some(entry) = node.entries().first() {
            if let Some(v) = entry.value().as_float() {
                settings.recognition_threshold = v;
            } else if let Some(v) = entry.value().as_integer() {
                settings.recognition_threshold = v as f64;
            }
        }
    }

    settings
}

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

/// Write touch settings to the separate touch-gestures.kdl file.
/// This NEVER touches the main config.kdl (except adding the include line once).
pub fn write_settings(settings: &TouchSettings) {
    let mut doc = KdlDocument::new();

    // Build: input { touch { ... } }
    let mut input_node = KdlNode::new("input");
    let input_children = input_node.ensure_children();

    let mut touch_node = KdlNode::new("touch");
    let touch_children = touch_node.ensure_children();

    if settings.off {
        touch_children.nodes_mut().push(KdlNode::new("off"));
    }
    if settings.natural_scroll {
        touch_children.nodes_mut().push(KdlNode::new("natural-scroll"));
    }
    if let Some(ref output) = settings.map_to_output {
        let mut node = KdlNode::new("map-to-output");
        node.push(kdl::KdlEntry::new(KdlValue::String(output.clone())));
        touch_children.nodes_mut().push(node);
    }

    // Gestures block.
    let mut gestures_node = KdlNode::new("gestures");
    let gestures_children = gestures_node.ensure_children();

    write_gesture_action(gestures_children, "workspace-switch", &settings.workspace_switch);
    write_gesture_action(gestures_children, "view-scroll", &settings.view_scroll);
    write_gesture_action(gestures_children, "overview-toggle", &settings.overview_toggle);

    let mut threshold_node = KdlNode::new("recognition-threshold");
    threshold_node.push(kdl::KdlEntry::new(KdlValue::Float(settings.recognition_threshold)));
    gestures_children.nodes_mut().push(threshold_node);

    touch_children.nodes_mut().push(gestures_node);
    input_children.nodes_mut().push(touch_node);
    doc.nodes_mut().push(input_node);

    // Write to the separate file.
    let path = touch_config_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    fs::write(&path, doc.to_string()).expect("failed to write touch gestures config");
}

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

/// Trigger niri config reload via IPC.
pub fn reload_config() {
    let _ = std::process::Command::new("niri")
        .args(["msg", "action", "load-config-file"])
        .spawn();
}
