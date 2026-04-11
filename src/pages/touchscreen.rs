use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{
    self, Edge, EdgeZone, PinchDir, RotateDir, SwipeDir, TouchBindEntry, TouchscreenSettings,
    Trigger, MAX_FINGERS, MIN_FINGERS,
};

// ---------------------------------------------------------------------------
// Curated action list: (display name, KDL action name)
// ---------------------------------------------------------------------------

const ACTION_OPTIONS: &[(&str, &str)] = &[
    // Navigation (continuous)
    ("Focus Workspace Up", "focus-workspace-up"),
    ("Focus Workspace Down", "focus-workspace-down"),
    ("Focus Column Left", "focus-column-left"),
    ("Focus Column Right", "focus-column-right"),
    // Overview
    ("Toggle Overview", "toggle-overview"),
    ("Open Overview", "open-overview"),
    ("Close Overview", "close-overview"),
    // Window management
    ("Close Window", "close-window"),
    ("Fullscreen Window", "fullscreen-window"),
    ("Maximize Column", "maximize-column"),
    ("Center Column", "center-column"),
    ("Toggle Floating", "toggle-window-floating"),
    ("Focus Window Up", "focus-window-up"),
    ("Focus Window Down", "focus-window-down"),
    // Screenshots
    ("Screenshot", "screenshot"),
    ("Screenshot Screen", "screenshot-screen"),
    ("Screenshot Window", "screenshot-window"),
    // Workspace
    ("Move Window to Workspace Down", "move-window-to-workspace-down"),
    ("Move Window to Workspace Up", "move-window-to-workspace-up"),
    ("Move Column to Workspace Down", "move-column-to-workspace-down"),
    ("Move Column to Workspace Up", "move-column-to-workspace-up"),
    // Monitor
    ("Focus Monitor Left", "focus-monitor-left"),
    ("Focus Monitor Right", "focus-monitor-right"),
    ("Focus Monitor Down", "focus-monitor-down"),
    ("Focus Monitor Up", "focus-monitor-up"),
    // System
    ("Quit", "quit"),
    ("Suspend", "suspend"),
    ("Power Off Monitors", "power-off-monitors"),
    ("Power On Monitors", "power-on-monitors"),
    ("Show Hotkey Overlay", "show-hotkey-overlay"),
    // Launch
    ("Spawn Command...", "spawn"),
    // IPC-only
    ("Noop (IPC only)", "noop"),
];

// ---------------------------------------------------------------------------
// Gesture family (UI-only)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Family { Swipe, Pinch, Rotate, Edge }

impl Family {
    fn all_labels() -> [&'static str; 4] {
        ["Swipe", "Pinch", "Rotate", "Edge"]
    }
    fn from_index(i: u32) -> Self {
        match i {
            0 => Self::Swipe,
            1 => Self::Pinch,
            2 => Self::Rotate,
            _ => Self::Edge,
        }
    }
}

fn display_action_name(action_name: &str, action_args: &[String]) -> String {
    if action_name == "spawn" && !action_args.is_empty() {
        return format!("Spawn: {}", action_args.join(" "));
    }
    if let Some((display, _)) = ACTION_OPTIONS.iter().find(|(_, k)| *k == action_name) {
        return display.to_string();
    }
    // Fallback: kebab-case to Title Case
    action_name
        .split('-')
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

// ---------------------------------------------------------------------------
// Page builder
// ---------------------------------------------------------------------------

pub fn build() -> gtk::Box {
    let settings = Rc::new(RefCell::new(config::read_touchscreen_settings()));

    let stack = gtk::Stack::new();
    stack.add_titled(&build_general(&settings), Some("general"), "Detection");
    stack.add_titled(&build_binds_page(&settings), Some("binds"), "Gesture Binds");

    let switcher = gtk::StackSwitcher::new();
    switcher.set_stack(Some(&stack));
    switcher.set_halign(gtk::Align::Center);
    switcher.set_margin_top(8);
    switcher.set_margin_bottom(8);

    let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
    container.append(&switcher);
    container.append(&stack);
    container
}

// ---------------------------------------------------------------------------
// General tab
// ---------------------------------------------------------------------------

fn build_general(settings: &Rc<RefCell<TouchscreenSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .description("input { touchscreen { gestures {} } }")
        .build();

    // Device group
    let group = adw::PreferencesGroup::builder()
        .title("Touchscreen Input")
        .description("Writes to touchscreen-gestures.kdl → replaces input { touchscreen {} }")
        .build();

    let enable_row = adw::SwitchRow::builder()
        .title("Touchscreen")
        .subtitle("Disables all touch input, not just gestures")
        .active(!settings.borrow().off)
        .build();
    {
        let settings = settings.clone();
        enable_row.connect_active_notify(move |row| {
            settings.borrow_mut().off = !row.is_active();
            save_and_reload(&settings.borrow());
        });
    }
    group.add(&enable_row);

    let output_row = adw::EntryRow::builder()
        .title("Map to Output")
        .text(settings.borrow().map_to_output.as_deref().unwrap_or(""))
        .build();
    {
        let settings = settings.clone();
        output_row.connect_changed(move |row| {
            let text = row.text().to_string();
            settings.borrow_mut().map_to_output = if text.is_empty() { None } else { Some(text) };
            save_and_reload(&settings.borrow());
        });
    }
    group.add(&output_row);
    page.add(&group);

    // Gesture recognition thresholds
    let thresh_group = adw::PreferencesGroup::builder()
        .title("Gesture Recognition")
        .description("Fine-tune how gestures are detected")
        .build();

    add_threshold_row(&thresh_group, settings, "Swipe Trigger Distance",
        "Pixels of centroid movement before a swipe commits",
        settings.borrow().swipe_trigger_distance, 4.0, 500.0, 0,
        |s, v| s.swipe_trigger_distance = v);

    add_threshold_row(&thresh_group, settings, "Edge Start Distance",
        "Width of the screen-edge start zone (px)",
        settings.borrow().edge_start_distance, 5.0, 200.0, 0,
        |s, v| s.edge_start_distance = v);

    add_threshold_row(&thresh_group, settings, "Swipe Multi-Finger Scale",
        "Extra swipe threshold per finger above 3 (higher = harder with more fingers)",
        settings.borrow().swipe_multi_finger_scale, 1.0, 5.0, 1,
        |s, v| s.swipe_multi_finger_scale = v);

    page.add(&thresh_group);

    // Pinch detection thresholds
    let pinch_group = adw::PreferencesGroup::builder()
        .title("Pinch Detection")
        .description("Tuning for pinch gesture recognition")
        .build();

    add_threshold_row(&pinch_group, settings, "Pinch Trigger Distance",
        "Minimum spread change (px) before a pinch commits",
        settings.borrow().pinch_trigger_distance, 5.0, 500.0, 0,
        |s, v| s.pinch_trigger_distance = v);

    add_threshold_row(&pinch_group, settings, "Pinch Dominance Ratio",
        "Spread must exceed swipe distance by this factor (higher = stricter pinch)",
        settings.borrow().pinch_dominance_ratio, 0.1, 5.0, 1,
        |s, v| s.pinch_dominance_ratio = v);

    add_threshold_row(&pinch_group, settings, "Pinch Sensitivity",
        "Multiplier for pinch gesture deltas",
        settings.borrow().pinch_sensitivity, 0.01, 5.0, 2,
        |s, v| s.pinch_sensitivity = v);

    page.add(&pinch_group);

    // Rotation detection
    let rot_group = adw::PreferencesGroup::builder()
        .title("Rotation Detection")
        .description("Tuning for rotation gesture recognition")
        .build();

    add_threshold_row(&rot_group, settings, "Rotation Trigger Angle",
        "Minimum rotation (degrees) before a rotate commits",
        settings.borrow().rotation_trigger_angle, 1.0, 180.0, 0,
        |s, v| s.rotation_trigger_angle = v);

    add_threshold_row(&rot_group, settings, "Rotation Dominance Ratio",
        "Rotation arc must exceed swipe+spread by this factor (higher = stricter rotate)",
        settings.borrow().rotation_dominance_ratio, 0.1, 5.0, 1,
        |s, v| s.rotation_dominance_ratio = v);

    add_threshold_row(&rot_group, settings, "Rotation Progress Angle",
        "Degrees of rotation for IPC progress to reach 1.0",
        settings.borrow().rotation_progress_angle, 10.0, 360.0, 0,
        |s, v| s.rotation_progress_angle = v);

    page.add(&rot_group);

    // IPC progress scaling
    let ipc_group = adw::PreferencesGroup::builder()
        .title("IPC Progress")
        .description("Controls progress scaling for external tools (screen pixels).\n\
             Noop gestures: progress directly drives the external app (1:1 sync).\n\
             Compositor actions (workspace switch, etc): progress is informational only — \
             niri uses its own internal thresholds to decide when to commit.")
        .build();

    add_threshold_row(&ipc_group, settings, "Swipe Progress Distance",
        "Screen pixels of swipe movement for IPC progress to reach 1.0",
        settings.borrow().swipe_progress_distance, 50.0, 1000.0, 0,
        |s, v| s.swipe_progress_distance = v);

    page.add(&ipc_group);

    page
}

fn add_threshold_row(
    group: &adw::PreferencesGroup,
    settings: &Rc<RefCell<TouchscreenSettings>>,
    title: &str,
    subtitle: &str,
    value: f64,
    min: f64,
    max: f64,
    digits: u32,
    setter: fn(&mut TouchscreenSettings, f64),
) {
    let step = if digits == 0 { 1.0 } else { 0.1_f64.powi(digits as i32) };
    let row = adw::SpinRow::builder()
        .title(title)
        .subtitle(subtitle)
        .adjustment(&gtk::Adjustment::new(value, min, max, step, step * 10.0, 0.0))
        .digits(digits)
        .build();

    let settings = settings.clone();
    row.connect_value_notify(move |row| {
        setter(&mut settings.borrow_mut(), row.value());
        save_and_reload(&settings.borrow());
    });

    group.add(&row);
}

// ---------------------------------------------------------------------------
// Touch Binds tab
// ---------------------------------------------------------------------------

fn build_binds_page(settings: &Rc<RefCell<TouchscreenSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    // Info
    let info = adw::PreferencesGroup::builder()
        .description(
            "Writes to touchscreen-gestures.kdl → merges into binds {}\n\
             Continuous actions (workspace switch, column scroll, overview) \
             track your finger. All others fire once."
        )
        .build();
    page.add(&info);

    // Active binds group
    let binds_group = Rc::new(adw::PreferencesGroup::builder()
        .title("Active Binds")
        .build());

    // Add new bind form (at top for easy access)
    let add_group = build_add_form(settings, &binds_group);
    page.add(&add_group);

    populate_binds_group(&binds_group, settings);
    page.add(&*binds_group);

    page
}

fn populate_binds_group(
    group: &Rc<adw::PreferencesGroup>,
    settings: &Rc<RefCell<TouchscreenSettings>>,
) {
    let binds = settings.borrow().binds.clone();
    for bind in &binds {
        let row = build_bind_row(bind, group, settings);
        group.add(&row);
    }
}

// ---------------------------------------------------------------------------
// Bind row (ExpanderRow per bind)
// ---------------------------------------------------------------------------

fn build_bind_row(
    bind: &TouchBindEntry,
    group: &Rc<adw::PreferencesGroup>,
    settings: &Rc<RefCell<TouchscreenSettings>>,
) -> adw::ExpanderRow {
    let gesture_display = bind.trigger.display_name();
    let action_display = display_action_name(&bind.action_name, &bind.action_args);
    let key = bind.trigger.key();

    let row = adw::ExpanderRow::builder()
        .title(&gesture_display)
        .subtitle(&action_display)
        .build();

    // Enable/disable toggle in suffix (iOS/Android style)
    let enable_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(bind.enabled)
        .build();

    {
        let key = key.clone();
        let settings = settings.clone();
        enable_switch.connect_active_notify(move |switch| {
            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.enabled = switch.is_active();
            }
            save_and_reload(&settings.borrow());
        });
    }
    row.add_suffix(&enable_switch);

    // Delete button with confirmation
    let delete_btn = gtk::Button::builder()
        .icon_name("user-trash-symbolic")
        .valign(gtk::Align::Center)
        .css_classes(vec!["flat".to_string(), "circular".to_string()])
        .build();

    {
        let key = key.clone();
        let gesture_display = gesture_display.clone();
        let row_clone = row.clone();
        let group = group.clone();
        let settings = settings.clone();
        delete_btn.connect_clicked(move |btn| {
            let window = btn.root().and_downcast::<gtk::Window>();
            let dialog = adw::MessageDialog::new(
                window.as_ref(),
                Some("Delete Bind?"),
                Some(&format!("Remove {} gesture bind?", gesture_display)),
            );
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("delete", "Delete");
            dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
            dialog.set_default_response(Some("cancel"));

            let key = key.clone();
            let row_clone = row_clone.clone();
            let group = group.clone();
            let settings = settings.clone();
            dialog.connect_response(None, move |_, response| {
                if response == "delete" {
                    settings.borrow_mut().binds.retain(|b| b.trigger.key() != key);
                    save_and_reload(&settings.borrow());
                    group.remove(&row_clone);
                }
            });

            dialog.present();
        });
    }
    row.add_suffix(&delete_btn);

    // Action dropdown — lets user change what this gesture does
    let action_labels: Vec<&str> = ACTION_OPTIONS.iter().map(|(d, _)| *d).collect();
    let action_model = gtk::StringList::new(&action_labels);
    let action_combo = adw::ComboRow::builder()
        .title("Action")
        .model(&action_model)
        .build();

    // Set current selection
    let current_idx = ACTION_OPTIONS.iter()
        .position(|(_, k)| *k == bind.action_name)
        .unwrap_or(0) as u32;
    action_combo.set_selected(current_idx);

    {
        let key = key.clone();
        let settings = settings.clone();
        let row_ref = row.clone();
        action_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected() as usize;
            if idx >= ACTION_OPTIONS.len() { return; }
            let new_action = ACTION_OPTIONS[idx].1.to_string();
            let new_display = ACTION_OPTIONS[idx].0;

            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.action_name = new_action;
                b.action_args.clear();
            }
            row_ref.set_subtitle(new_display);
            save_and_reload(&settings.borrow());
        });
    }
    row.add_row(&action_combo);

    // Sensitivity
    let sens_row = adw::SpinRow::builder()
        .title("Sensitivity")
        .subtitle("Speed multiplier (continuous actions)")
        .adjustment(&gtk::Adjustment::new(
            bind.sensitivity.unwrap_or(1.0),
            0.1, 5.0, 0.1, 0.5, 0.0,
        ))
        .digits(1)
        .build();

    {
        let key = key.clone();
        let settings = settings.clone();
        sens_row.connect_value_notify(move |spin| {
            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.sensitivity = Some(spin.value());
            }
            save_and_reload(&settings.borrow());
        });
    }
    row.add_row(&sens_row);

    // Natural scroll
    let natural_row = adw::SwitchRow::builder()
        .title("Natural Scroll")
        .subtitle("Invert gesture direction")
        .active(bind.natural_scroll)
        .build();

    {
        let key = key.clone();
        let settings = settings.clone();
        natural_row.connect_active_notify(move |switch| {
            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.natural_scroll = switch.is_active();
            }
            save_and_reload(&settings.borrow());
        });
    }
    row.add_row(&natural_row);

    // Tag — IPC event identifier for external tools
    let tag_row = adw::EntryRow::builder()
        .title("Tag")
        .text(bind.tag.as_deref().unwrap_or(""))
        .build();

    {
        let key = key.clone();
        let settings = settings.clone();
        tag_row.connect_changed(move |entry| {
            let text = entry.text().to_string();
            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.tag = if text.is_empty() { None } else { Some(text) };
            }
            save_and_reload(&settings.borrow());
        });
    }
    row.add_row(&tag_row);

    row
}

// ---------------------------------------------------------------------------
// Add bind form — nested picker for the property-form trigger schema
// ---------------------------------------------------------------------------

fn build_add_form(
    settings: &Rc<RefCell<TouchscreenSettings>>,
    binds_group: &Rc<adw::PreferencesGroup>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Add New Bind")
        .build();

    // Family selector (Swipe / Pinch / Rotate / Edge)
    let family_labels: [&str; 4] = Family::all_labels();
    let family_model = gtk::StringList::new(&family_labels);
    let family_combo = adw::ComboRow::builder()
        .title("Gesture Family")
        .model(&family_model)
        .selected(0)
        .build();
    group.add(&family_combo);

    // Fingers (Swipe/Pinch/Rotate only)
    let fingers_row = adw::SpinRow::builder()
        .title("Fingers")
        .subtitle("Number of fingers required")
        .adjustment(&gtk::Adjustment::new(
            3.0,
            MIN_FINGERS as f64,
            MAX_FINGERS as f64,
            1.0, 1.0, 0.0,
        ))
        .build();
    group.add(&fingers_row);

    // Direction (Swipe/Pinch/Rotate — vocab changes per family)
    // Model is replaced wholesale when family changes; start on Swipe.
    let dir_labels_swipe = ["Up", "Down", "Left", "Right"];
    let dir_model = gtk::StringList::new(&dir_labels_swipe);
    let dir_combo = adw::ComboRow::builder()
        .title("Direction")
        .model(&dir_model)
        .selected(0)
        .build();
    group.add(&dir_combo);

    // Edge selector (Edge family only)
    let edge_labels = ["Left", "Right", "Top", "Bottom"];
    let edge_model = gtk::StringList::new(&edge_labels);
    let edge_combo = adw::ComboRow::builder()
        .title("Edge")
        .model(&edge_model)
        .selected(0)
        .visible(false)
        .build();
    group.add(&edge_combo);

    // Zone selector (Edge family only) — vocab rotates per axis.
    let zone_model = gtk::StringList::new(&["Full", "Top", "Center", "Bottom"]);
    let zone_combo = adw::ComboRow::builder()
        .title("Zone")
        .model(&zone_model)
        .selected(0)
        .visible(false)
        .build();
    group.add(&zone_combo);

    // Action dropdown
    let action_labels: Vec<&str> = ACTION_OPTIONS.iter().map(|(d, _)| *d).collect();
    let action_model = gtk::StringList::new(&action_labels);
    let action_combo = adw::ComboRow::builder()
        .title("Action")
        .model(&action_model)
        .selected(0)
        .build();
    group.add(&action_combo);

    // Spawn command entry (hidden unless spawn selected)
    let spawn_entry = adw::EntryRow::builder()
        .title("Command")
        .visible(false)
        .build();
    group.add(&spawn_entry);

    // React to family changes: swap direction vocab / show/hide edge rows.
    {
        let fingers_row = fingers_row.clone();
        let dir_combo = dir_combo.clone();
        let edge_combo = edge_combo.clone();
        let zone_combo = zone_combo.clone();
        family_combo.connect_selected_notify(move |combo| {
            let family = Family::from_index(combo.selected());
            match family {
                Family::Swipe => {
                    fingers_row.set_visible(true);
                    dir_combo.set_visible(true);
                    dir_combo.set_model(Some(&gtk::StringList::new(&["Up", "Down", "Left", "Right"])));
                    dir_combo.set_selected(0);
                    edge_combo.set_visible(false);
                    zone_combo.set_visible(false);
                }
                Family::Pinch => {
                    fingers_row.set_visible(true);
                    dir_combo.set_visible(true);
                    dir_combo.set_model(Some(&gtk::StringList::new(&["In", "Out"])));
                    dir_combo.set_selected(0);
                    edge_combo.set_visible(false);
                    zone_combo.set_visible(false);
                }
                Family::Rotate => {
                    fingers_row.set_visible(true);
                    dir_combo.set_visible(true);
                    dir_combo.set_model(Some(&gtk::StringList::new(&["Clockwise", "Counter-Clockwise"])));
                    dir_combo.set_selected(0);
                    edge_combo.set_visible(false);
                    zone_combo.set_visible(false);
                }
                Family::Edge => {
                    fingers_row.set_visible(false);
                    dir_combo.set_visible(false);
                    edge_combo.set_visible(true);
                    zone_combo.set_visible(true);
                }
            }
        });
    }

    // React to edge changes: re-label zone options per axis.
    {
        let zone_combo = zone_combo.clone();
        edge_combo.connect_selected_notify(move |combo| {
            let edge_idx = combo.selected();
            let new_labels: [&str; 4] = match edge_idx {
                0 | 1 => ["Full", "Top", "Center", "Bottom"], // Left / Right → y-axis
                _ => ["Full", "Left", "Center", "Right"],     // Top / Bottom → x-axis
            };
            zone_combo.set_model(Some(&gtk::StringList::new(&new_labels)));
            zone_combo.set_selected(0);
        });
    }

    // Show/hide spawn entry based on action selection
    {
        let spawn_entry = spawn_entry.clone();
        action_combo.connect_selected_notify(move |row| {
            let idx = row.selected() as usize;
            if idx < ACTION_OPTIONS.len() {
                spawn_entry.set_visible(ACTION_OPTIONS[idx].1 == "spawn");
            }
        });
    }

    // "Add Bind" action row
    let add_row = adw::ActionRow::builder()
        .title("Add Bind")
        .activatable(true)
        .build();
    add_row.add_prefix(&gtk::Image::from_icon_name("list-add-symbolic"));

    {
        let settings = settings.clone();
        let binds_group = binds_group.clone();
        let family_combo = family_combo.clone();
        let fingers_row = fingers_row.clone();
        let dir_combo = dir_combo.clone();
        let edge_combo = edge_combo.clone();
        let zone_combo = zone_combo.clone();
        let action_combo = action_combo.clone();
        let spawn_entry = spawn_entry.clone();

        add_row.connect_activated(move |_| {
            let family = Family::from_index(family_combo.selected());
            let fingers = fingers_row.value() as u8;
            let dir_idx = dir_combo.selected();
            let edge_idx = edge_combo.selected();
            let zone_idx = zone_combo.selected();

            let trigger = match family {
                Family::Swipe => Trigger::TouchSwipe {
                    fingers,
                    direction: SwipeDir::ALL[dir_idx as usize % SwipeDir::ALL.len()],
                },
                Family::Pinch => Trigger::TouchPinch {
                    fingers,
                    direction: PinchDir::ALL[dir_idx as usize % PinchDir::ALL.len()],
                },
                Family::Rotate => Trigger::TouchRotate {
                    fingers,
                    direction: RotateDir::ALL[dir_idx as usize % RotateDir::ALL.len()],
                },
                Family::Edge => {
                    let edge = Edge::ALL[edge_idx as usize % Edge::ALL.len()];
                    let zone = if zone_idx == 0 {
                        None
                    } else {
                        Some(EdgeZone::ALL[(zone_idx - 1) as usize % EdgeZone::ALL.len()])
                    };
                    Trigger::TouchEdge { edge, zone }
                }
            };

            let action_idx = action_combo.selected() as usize;
            if action_idx >= ACTION_OPTIONS.len() { return }
            let action_name = ACTION_OPTIONS[action_idx].1.to_string();

            // Check for duplicates
            let key = trigger.key();
            if settings.borrow().binds.iter().any(|b| b.trigger.key() == key) {
                return;
            }

            let action_args = if action_name == "spawn" {
                let cmd = spawn_entry.text().to_string();
                if cmd.is_empty() { return; }
                vec![cmd]
            } else {
                vec![]
            };

            let bind = TouchBindEntry {
                trigger,
                action_name,
                action_args,
                sensitivity: None,
                natural_scroll: false,
                tag: None,
                enabled: true,
            };

            // Add to settings
            settings.borrow_mut().binds.push(bind.clone());
            save_and_reload(&settings.borrow());

            // Add row to UI
            let row = build_bind_row(&bind, &binds_group, &settings);
            binds_group.add(&row);
        });
    }
    group.add(&add_row);

    group
}

// ---------------------------------------------------------------------------
// Save helper
// ---------------------------------------------------------------------------

fn save_and_reload(settings: &TouchscreenSettings) {
    config::write_touchscreen_settings(settings);
    config::reload_config();
}
