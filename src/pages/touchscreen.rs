use adw::prelude::*;
use std::cell::{Cell, RefCell};
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
    (
        "Move Window to Workspace Down",
        "move-window-to-workspace-down",
    ),
    ("Move Window to Workspace Up", "move-window-to-workspace-up"),
    (
        "Move Column to Workspace Down",
        "move-column-to-workspace-down",
    ),
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
enum Family {
    Swipe,
    Pinch,
    Rotate,
    Edge,
    Tap,
    TapHoldDrag,
}

impl Family {
    fn all_labels() -> [&'static str; 6] {
        ["Swipe", "Pinch", "Rotate", "Edge", "Tap", "Tap-Hold-Drag"]
    }
    fn from_index(i: u32) -> Self {
        match i {
            0 => Self::Swipe,
            1 => Self::Pinch,
            2 => Self::Rotate,
            3 => Self::Edge,
            4 => Self::Tap,
            _ => Self::TapHoldDrag,
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

    add_threshold_row(
        &thresh_group,
        settings,
        ThresholdRow {
            title: "Swipe Trigger Distance",
            subtitle: "Pixels of centroid movement before a swipe commits",
            value: settings.borrow().swipe_trigger_distance,
            min: 4.0,
            max: 500.0,
            digits: 0,
            setter: |s, v| s.swipe_trigger_distance = v,
        },
    );

    add_threshold_row(
        &thresh_group,
        settings,
        ThresholdRow {
            title: "Edge Start Distance",
            subtitle: "Width of the screen-edge start zone (px)",
            value: settings.borrow().edge_start_distance,
            min: 5.0,
            max: 200.0,
            digits: 0,
            setter: |s, v| s.edge_start_distance = v,
        },
    );

    add_threshold_row(
        &thresh_group,
        settings,
        ThresholdRow {
            title: "Swipe Multi-Finger Scale",
            subtitle:
                "Extra swipe threshold per finger above 3 (higher = harder with more fingers)",
            value: settings.borrow().swipe_multi_finger_scale,
            min: 1.0,
            max: 5.0,
            digits: 1,
            setter: |s, v| s.swipe_multi_finger_scale = v,
        },
    );

    page.add(&thresh_group);

    // Pinch detection thresholds
    let pinch_group = adw::PreferencesGroup::builder()
        .title("Pinch Detection")
        .description("Tuning for pinch gesture recognition")
        .build();

    add_threshold_row(
        &pinch_group,
        settings,
        ThresholdRow {
            title: "Pinch Trigger Distance",
            subtitle: "Minimum spread change (px) before a pinch commits",
            value: settings.borrow().pinch_trigger_distance,
            min: 5.0,
            max: 500.0,
            digits: 0,
            setter: |s, v| s.pinch_trigger_distance = v,
        },
    );

    add_threshold_row(
        &pinch_group,
        settings,
        ThresholdRow {
            title: "Pinch Dominance Ratio",
            subtitle: "Spread must exceed swipe distance by this factor (higher = stricter pinch)",
            value: settings.borrow().pinch_dominance_ratio,
            min: 0.1,
            max: 5.0,
            digits: 1,
            setter: |s, v| s.pinch_dominance_ratio = v,
        },
    );

    add_threshold_row(
        &pinch_group,
        settings,
        ThresholdRow {
            title: "Pinch Sensitivity",
            subtitle: "Multiplier for pinch gesture deltas",
            value: settings.borrow().pinch_sensitivity,
            min: 0.01,
            max: 5.0,
            digits: 2,
            setter: |s, v| s.pinch_sensitivity = v,
        },
    );

    page.add(&pinch_group);

    // Rotation detection
    let rot_group = adw::PreferencesGroup::builder()
        .title("Rotation Detection")
        .description("Tuning for rotation gesture recognition")
        .build();

    add_threshold_row(
        &rot_group,
        settings,
        ThresholdRow {
            title: "Rotation Trigger Angle",
            subtitle: "Minimum rotation (degrees) before a rotate commits",
            value: settings.borrow().rotation_trigger_angle,
            min: 1.0,
            max: 180.0,
            digits: 0,
            setter: |s, v| s.rotation_trigger_angle = v,
        },
    );

    add_threshold_row(
        &rot_group,
        settings,
        ThresholdRow {
            title: "Rotation Dominance Ratio",
            subtitle:
                "Rotation arc must exceed swipe+spread by this factor (higher = stricter rotate)",
            value: settings.borrow().rotation_dominance_ratio,
            min: 0.1,
            max: 5.0,
            digits: 1,
            setter: |s, v| s.rotation_dominance_ratio = v,
        },
    );

    add_threshold_row(
        &rot_group,
        settings,
        ThresholdRow {
            title: "Rotation Progress Angle",
            subtitle: "Degrees of rotation for IPC progress to reach 1.0",
            value: settings.borrow().rotation_progress_angle,
            min: 10.0,
            max: 360.0,
            digits: 0,
            setter: |s, v| s.rotation_progress_angle = v,
        },
    );

    page.add(&rot_group);

    // Tap detection
    let tap_group = adw::PreferencesGroup::builder()
        .title("Tap Detection")
        .description("Tuning for tap and tap-hold-drag recognition")
        .build();

    add_threshold_row(
        &tap_group,
        settings,
        ThresholdRow {
            title: "Tap Wobble Threshold",
            subtitle: "Pixels per finger before a tap candidate is cancelled",
            value: settings.borrow().tap_wobble_threshold,
            min: 1.0,
            max: 100.0,
            digits: 0,
            setter: |s, v| s.tap_wobble_threshold = v,
        },
    );

    add_threshold_row(
        &tap_group,
        settings,
        ThresholdRow {
            title: "Tap Timeout (ms)",
            subtitle: "Maximum ms between touch-down and all fingers lifting for a tap to fire",
            value: settings.borrow().tap_timeout_ms,
            min: 50.0,
            max: 2000.0,
            digits: 0,
            setter: |s, v| s.tap_timeout_ms = v,
        },
    );

    add_threshold_row(
        &tap_group,
        settings,
        ThresholdRow {
            title: "Tap-Hold Trigger Delay (ms)",
            subtitle: "Minimum ms of stationary hold before a wobble can trigger tap-hold-drag \
             (shorter delays make fast swipes trigger hold-drag; longer makes it require a deliberate hold)",
            value: settings.borrow().tap_hold_trigger_delay_ms,
            min: 0.0,
            max: 1000.0,
            digits: 0,
            setter: |s, v| s.tap_hold_trigger_delay_ms = v,
        },
    );

    page.add(&tap_group);

    // IPC progress scaling
    let ipc_group = adw::PreferencesGroup::builder()
        .title("IPC Progress")
        .description(
            "Controls progress scaling for external tools (screen pixels).\n\
             Noop gestures: progress directly drives the external app (1:1 sync).\n\
             Compositor actions (workspace switch, etc): progress is informational only — \
             niri uses its own internal thresholds to decide when to commit.",
        )
        .build();

    add_threshold_row(
        &ipc_group,
        settings,
        ThresholdRow {
            title: "Swipe Progress Distance",
            subtitle: "Screen pixels of swipe movement for IPC progress to reach 1.0",
            value: settings.borrow().swipe_progress_distance,
            min: 50.0,
            max: 1000.0,
            digits: 0,
            setter: |s, v| s.swipe_progress_distance = v,
        },
    );

    page.add(&ipc_group);

    page
}

struct ThresholdRow<'a> {
    title: &'a str,
    subtitle: &'a str,
    value: f64,
    min: f64,
    max: f64,
    digits: u32,
    setter: fn(&mut TouchscreenSettings, f64),
}

fn add_threshold_row(
    group: &adw::PreferencesGroup,
    settings: &Rc<RefCell<TouchscreenSettings>>,
    row: ThresholdRow<'_>,
) {
    let step = if row.digits == 0 {
        1.0
    } else {
        0.1_f64.powi(row.digits as i32)
    };
    let spin_row = adw::SpinRow::builder()
        .title(row.title)
        .subtitle(row.subtitle)
        .adjustment(&gtk::Adjustment::new(
            row.value,
            row.min,
            row.max,
            step,
            step * 10.0,
            0.0,
        ))
        .digits(row.digits)
        .build();

    let setter = row.setter;
    let settings = settings.clone();
    spin_row.connect_value_notify(move |spin_row| {
        setter(&mut settings.borrow_mut(), spin_row.value());
        save_and_reload(&settings.borrow());
    });

    group.add(&spin_row);
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
             track your finger. All others fire once.",
        )
        .build();
    page.add(&info);

    // Tracked rows — each added row is recorded here so the search entry
    // can filter by visibility without rebuilding the list.
    let tracked_rows: Rc<RefCell<Vec<adw::ExpanderRow>>> = Rc::new(RefCell::new(Vec::new()));

    // Active binds group
    let binds_group = Rc::new(
        adw::PreferencesGroup::builder()
            .title("Active Binds")
            .build(),
    );

    // Add new bind form (at top for easy access)
    let add_group = build_add_form(settings, &binds_group, &tracked_rows);
    page.add(&add_group);

    // Search / filter entry — sits just above the Active Binds list
    let search_group = adw::PreferencesGroup::builder().build();
    let search_row = adw::EntryRow::builder().title("Filter Binds").build();
    search_row.add_prefix(&gtk::Image::from_icon_name("system-search-symbolic"));
    {
        let tracked_rows = tracked_rows.clone();
        search_row.connect_changed(move |entry| {
            let query = entry.text().to_string().to_lowercase();
            for r in tracked_rows.borrow().iter() {
                let title = r.title().to_string().to_lowercase();
                let subtitle = r.subtitle().to_string().to_lowercase();
                let visible =
                    query.is_empty() || title.contains(&query) || subtitle.contains(&query);
                r.set_visible(visible);
            }
        });
    }
    search_group.add(&search_row);
    page.add(&search_group);

    populate_binds_group(&binds_group, settings, &tracked_rows);
    page.add(&*binds_group);

    page
}

fn populate_binds_group(
    group: &Rc<adw::PreferencesGroup>,
    settings: &Rc<RefCell<TouchscreenSettings>>,
    tracked_rows: &Rc<RefCell<Vec<adw::ExpanderRow>>>,
) {
    let binds = settings.borrow().binds.clone();
    for bind in &binds {
        let row = build_bind_row(bind, group, settings, tracked_rows);
        group.add(&row);
        tracked_rows.borrow_mut().push(row);
    }
}

// ---------------------------------------------------------------------------
// Bind row (ExpanderRow per bind)
// ---------------------------------------------------------------------------

/// Apply an in-place edit to the trigger of the bind currently identified
/// by `current_key`. Returns `true` on success, `false` on collision (another
/// bind already has the new shape) or missing bind.
///
/// On success, updates `current_key` to the new shape and retitles `row`.
fn try_update_trigger<F>(
    settings: &Rc<RefCell<TouchscreenSettings>>,
    current_key: &Rc<RefCell<String>>,
    row: &adw::ExpanderRow,
    mutate: F,
) -> bool
where
    F: FnOnce(Trigger) -> Trigger,
{
    let new_trigger;
    let new_key;
    {
        let mut s = settings.borrow_mut();
        let key = current_key.borrow().clone();
        let Some(idx) = s.binds.iter().position(|b| b.trigger.key() == key) else {
            return false;
        };
        new_trigger = mutate(s.binds[idx].trigger);
        new_key = new_trigger.key();
        if new_key == key {
            return true; // No-op (same shape)
        }
        if s.binds
            .iter()
            .enumerate()
            .any(|(i, b)| i != idx && b.trigger.key() == new_key)
        {
            return false; // Collision
        }
        s.binds[idx].trigger = new_trigger;
    }
    *current_key.borrow_mut() = new_key;
    row.set_title(&new_trigger.display_name());
    save_and_reload(&settings.borrow());
    true
}

fn build_bind_row(
    bind: &TouchBindEntry,
    group: &Rc<adw::PreferencesGroup>,
    settings: &Rc<RefCell<TouchscreenSettings>>,
    tracked_rows: &Rc<RefCell<Vec<adw::ExpanderRow>>>,
) -> adw::ExpanderRow {
    let gesture_display = bind.trigger.display_name();
    let action_display = display_action_name(&bind.action_name, &bind.action_args);

    // Live key — updated whenever the trigger is edited in-place.
    let current_key = Rc::new(RefCell::new(bind.trigger.key()));
    // Re-entry guard for revert-on-collision.
    let suppress = Rc::new(Cell::new(false));

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
        let settings = settings.clone();
        let current_key = current_key.clone();
        enable_switch.connect_active_notify(move |switch| {
            let key = current_key.borrow().clone();
            if let Some(b) = settings
                .borrow_mut()
                .binds
                .iter_mut()
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
        let current_key = current_key.clone();
        let row_clone = row.clone();
        let group = group.clone();
        let settings = settings.clone();
        let tracked_rows = tracked_rows.clone();
        delete_btn.connect_clicked(move |btn| {
            let window = btn.root().and_downcast::<gtk::Window>();
            // Compute the display name fresh — trigger may have been edited.
            let key = current_key.borrow().clone();
            let display = settings
                .borrow()
                .binds
                .iter()
                .find(|b| b.trigger.key() == key)
                .map(|b| b.trigger.display_name())
                .unwrap_or_else(|| "this".to_string());
            let dialog = adw::MessageDialog::new(
                window.as_ref(),
                Some("Delete Bind?"),
                Some(&format!("Remove {} gesture bind?", display)),
            );
            dialog.add_response("cancel", "Cancel");
            dialog.add_response("delete", "Delete");
            dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
            dialog.set_default_response(Some("cancel"));

            let current_key = current_key.clone();
            let row_clone = row_clone.clone();
            let group = group.clone();
            let settings = settings.clone();
            let tracked_rows = tracked_rows.clone();
            dialog.connect_response(None, move |_, response| {
                if response == "delete" {
                    let key = current_key.borrow().clone();
                    settings
                        .borrow_mut()
                        .binds
                        .retain(|b| b.trigger.key() != key);
                    save_and_reload(&settings.borrow());
                    group.remove(&row_clone);
                    tracked_rows.borrow_mut().retain(|r| r != &row_clone);
                }
            });

            dialog.present();
        });
    }
    row.add_suffix(&delete_btn);

    // -----------------------------------------------------------------
    // Editable trigger rows — shape depends on family.
    // -----------------------------------------------------------------
    match bind.trigger {
        Trigger::TouchSwipe {
            fingers,
            direction: _,
        }
        | Trigger::TouchPinch {
            fingers,
            direction: _,
        }
        | Trigger::TouchRotate {
            fingers,
            direction: _,
        } => {
            add_fingers_row(&row, settings, &current_key, &suppress, fingers);
            add_direction_row(&row, settings, &current_key, &suppress, bind.trigger);
        }
        Trigger::TouchEdge { edge, zone } => {
            add_edge_row(&row, settings, &current_key, &suppress, edge, zone);
        }
        Trigger::TouchTap { fingers } => {
            add_fingers_row(&row, settings, &current_key, &suppress, fingers);
        }
        Trigger::TouchTapHoldDrag { fingers, .. } => {
            add_fingers_row(&row, settings, &current_key, &suppress, fingers);
            add_direction_row(&row, settings, &current_key, &suppress, bind.trigger);
        }
        Trigger::TouchpadSwipe { .. }
        | Trigger::TouchpadTapHold { .. }
        | Trigger::TouchpadTapHoldDrag { .. } => {
            // Not emitted on the touchscreen page.
        }
    }

    // Action dropdown — lets user change what this gesture does
    let action_labels: Vec<&str> = ACTION_OPTIONS.iter().map(|(d, _)| *d).collect();
    let action_model = gtk::StringList::new(&action_labels);
    let action_combo = adw::ComboRow::builder()
        .title("Action")
        .model(&action_model)
        .build();

    // Set current selection
    let current_idx = ACTION_OPTIONS
        .iter()
        .position(|(_, k)| *k == bind.action_name)
        .unwrap_or(0) as u32;
    action_combo.set_selected(current_idx);

    {
        let current_key = current_key.clone();
        let settings = settings.clone();
        let row_ref = row.clone();
        action_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected() as usize;
            if idx >= ACTION_OPTIONS.len() {
                return;
            }
            let new_action = ACTION_OPTIONS[idx].1.to_string();
            let new_display = ACTION_OPTIONS[idx].0;
            let key = current_key.borrow().clone();
            if let Some(b) = settings
                .borrow_mut()
                .binds
                .iter_mut()
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
            0.1,
            5.0,
            0.1,
            0.5,
            0.0,
        ))
        .digits(1)
        .build();

    {
        let current_key = current_key.clone();
        let settings = settings.clone();
        sens_row.connect_value_notify(move |spin| {
            let key = current_key.borrow().clone();
            if let Some(b) = settings
                .borrow_mut()
                .binds
                .iter_mut()
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
        let current_key = current_key.clone();
        let settings = settings.clone();
        natural_row.connect_active_notify(move |switch| {
            let key = current_key.borrow().clone();
            if let Some(b) = settings
                .borrow_mut()
                .binds
                .iter_mut()
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
        let current_key = current_key.clone();
        let settings = settings.clone();
        tag_row.connect_changed(move |entry| {
            let text = entry.text().to_string();
            let key = current_key.borrow().clone();
            if let Some(b) = settings
                .borrow_mut()
                .binds
                .iter_mut()
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
// Trigger-editing rows (attached inside build_bind_row)
// ---------------------------------------------------------------------------

fn add_fingers_row(
    row: &adw::ExpanderRow,
    settings: &Rc<RefCell<TouchscreenSettings>>,
    current_key: &Rc<RefCell<String>>,
    suppress: &Rc<Cell<bool>>,
    initial: u8,
) {
    let fingers_row = adw::SpinRow::builder()
        .title("Fingers")
        .subtitle("Number of fingers (3–10)")
        .adjustment(&gtk::Adjustment::new(
            initial as f64,
            MIN_FINGERS as f64,
            MAX_FINGERS as f64,
            1.0,
            1.0,
            0.0,
        ))
        .build();

    {
        let settings = settings.clone();
        let current_key = current_key.clone();
        let suppress = suppress.clone();
        let row_ref = row.clone();
        let fingers_ref = fingers_row.clone();
        fingers_row.connect_value_notify(move |spin| {
            if suppress.get() {
                return;
            }
            let new_fingers = spin.value() as u8;
            let ok = try_update_trigger(&settings, &current_key, &row_ref, |t| match t {
                Trigger::TouchSwipe { direction, .. } => Trigger::TouchSwipe {
                    fingers: new_fingers,
                    direction,
                },
                Trigger::TouchPinch { direction, .. } => Trigger::TouchPinch {
                    fingers: new_fingers,
                    direction,
                },
                Trigger::TouchRotate { direction, .. } => Trigger::TouchRotate {
                    fingers: new_fingers,
                    direction,
                },
                Trigger::TouchTap { .. } => Trigger::TouchTap {
                    fingers: new_fingers,
                },
                Trigger::TouchTapHoldDrag { direction, .. } => Trigger::TouchTapHoldDrag {
                    fingers: new_fingers,
                    direction,
                },
                other => other,
            });
            if !ok {
                // Revert to the value currently in settings.
                let key = current_key.borrow().clone();
                let s = settings.borrow();
                if let Some(b) = s.binds.iter().find(|b| b.trigger.key() == key) {
                    let old = match b.trigger {
                        Trigger::TouchSwipe { fingers, .. }
                        | Trigger::TouchPinch { fingers, .. }
                        | Trigger::TouchRotate { fingers, .. } => fingers,
                        Trigger::TouchTap { fingers } => fingers,
                        Trigger::TouchTapHoldDrag { fingers, .. } => fingers,
                        _ => return,
                    };
                    suppress.set(true);
                    fingers_ref.set_value(old as f64);
                    suppress.set(false);
                }
            }
        });
    }
    row.add_row(&fingers_row);
}

fn add_direction_row(
    row: &adw::ExpanderRow,
    settings: &Rc<RefCell<TouchscreenSettings>>,
    current_key: &Rc<RefCell<String>>,
    suppress: &Rc<Cell<bool>>,
    trigger: Trigger,
) {
    // TapHoldDrag uses "Any" at index 0 followed by the 4 SwipeDir options.
    let (labels, selected): (Vec<&str>, u32) = match trigger {
        Trigger::TouchSwipe { direction, .. } => (
            SwipeDir::ALL.iter().map(|d| d.display()).collect(),
            SwipeDir::ALL
                .iter()
                .position(|d| *d == direction)
                .unwrap_or(0) as u32,
        ),
        Trigger::TouchPinch { direction, .. } => (
            PinchDir::ALL.iter().map(|d| d.display()).collect(),
            PinchDir::ALL
                .iter()
                .position(|d| *d == direction)
                .unwrap_or(0) as u32,
        ),
        Trigger::TouchRotate { direction, .. } => (
            RotateDir::ALL.iter().map(|d| d.display()).collect(),
            RotateDir::ALL
                .iter()
                .position(|d| *d == direction)
                .unwrap_or(0) as u32,
        ),
        Trigger::TouchTapHoldDrag { direction, .. } => {
            let mut l = vec!["Any"];
            l.extend(SwipeDir::ALL.iter().map(|d| d.display()));
            let sel = match direction {
                None => 0,
                Some(d) => 1 + SwipeDir::ALL.iter().position(|x| *x == d).unwrap_or(0) as u32,
            };
            (l, sel)
        }
        _ => return,
    };

    let model = gtk::StringList::new(&labels);
    let dir_combo = adw::ComboRow::builder()
        .title("Direction")
        .model(&model)
        .selected(selected)
        .build();

    {
        let settings = settings.clone();
        let current_key = current_key.clone();
        let suppress = suppress.clone();
        let row_ref = row.clone();
        let dir_ref = dir_combo.clone();
        dir_combo.connect_selected_notify(move |combo| {
            if suppress.get() {
                return;
            }
            let idx = combo.selected() as usize;
            let ok = try_update_trigger(&settings, &current_key, &row_ref, |t| match t {
                Trigger::TouchSwipe { fingers, .. } => Trigger::TouchSwipe {
                    fingers,
                    direction: SwipeDir::ALL[idx % SwipeDir::ALL.len()],
                },
                Trigger::TouchPinch { fingers, .. } => Trigger::TouchPinch {
                    fingers,
                    direction: PinchDir::ALL[idx % PinchDir::ALL.len()],
                },
                Trigger::TouchRotate { fingers, .. } => Trigger::TouchRotate {
                    fingers,
                    direction: RotateDir::ALL[idx % RotateDir::ALL.len()],
                },
                Trigger::TouchTapHoldDrag { fingers, .. } => {
                    let direction = if idx == 0 {
                        None
                    } else {
                        Some(SwipeDir::ALL[(idx - 1) % SwipeDir::ALL.len()])
                    };
                    Trigger::TouchTapHoldDrag { fingers, direction }
                }
                other => other,
            });
            if !ok {
                // Revert to the direction in settings.
                let key = current_key.borrow().clone();
                let s = settings.borrow();
                if let Some(b) = s.binds.iter().find(|b| b.trigger.key() == key) {
                    let old_idx = match b.trigger {
                        Trigger::TouchSwipe { direction, .. } => SwipeDir::ALL
                            .iter()
                            .position(|d| *d == direction)
                            .unwrap_or(0),
                        Trigger::TouchPinch { direction, .. } => PinchDir::ALL
                            .iter()
                            .position(|d| *d == direction)
                            .unwrap_or(0),
                        Trigger::TouchRotate { direction, .. } => RotateDir::ALL
                            .iter()
                            .position(|d| *d == direction)
                            .unwrap_or(0),
                        Trigger::TouchTapHoldDrag { direction, .. } => match direction {
                            None => 0,
                            Some(d) => 1 + SwipeDir::ALL.iter().position(|x| *x == d).unwrap_or(0),
                        },
                        _ => return,
                    } as u32;
                    suppress.set(true);
                    dir_ref.set_selected(old_idx);
                    suppress.set(false);
                }
            }
        });
    }
    row.add_row(&dir_combo);
}

fn add_edge_row(
    row: &adw::ExpanderRow,
    settings: &Rc<RefCell<TouchscreenSettings>>,
    current_key: &Rc<RefCell<String>>,
    suppress: &Rc<Cell<bool>>,
    initial_edge: Edge,
    initial_zone: Option<EdgeZone>,
) {
    let edge_labels: Vec<&str> = Edge::ALL.iter().map(|e| e.display()).collect();
    let edge_model = gtk::StringList::new(&edge_labels);
    let edge_combo = adw::ComboRow::builder()
        .title("Edge")
        .model(&edge_model)
        .selected(
            Edge::ALL
                .iter()
                .position(|e| *e == initial_edge)
                .unwrap_or(0) as u32,
        )
        .build();

    let zone_labels = zone_labels_for(initial_edge);
    let zone_model = gtk::StringList::new(&zone_labels);
    let zone_combo = adw::ComboRow::builder()
        .title("Zone")
        .model(&zone_model)
        .selected(match initial_zone {
            None => 0,
            Some(z) => (EdgeZone::ALL.iter().position(|zz| *zz == z).unwrap_or(0) as u32) + 1,
        })
        .build();

    // Edge change: mutate trigger, re-label zone options, preserve zone selection
    // (EdgeZone is stable across axis rotation — only the displayed label changes).
    {
        let settings = settings.clone();
        let current_key = current_key.clone();
        let suppress = suppress.clone();
        let row_ref = row.clone();
        let edge_ref = edge_combo.clone();
        let zone_ref = zone_combo.clone();
        edge_combo.connect_selected_notify(move |combo| {
            if suppress.get() {
                return;
            }
            let idx = combo.selected() as usize;
            let new_edge = Edge::ALL[idx % Edge::ALL.len()];
            let ok = try_update_trigger(&settings, &current_key, &row_ref, |t| match t {
                Trigger::TouchEdge { zone, .. } => Trigger::TouchEdge {
                    edge: new_edge,
                    zone,
                },
                other => other,
            });
            if ok {
                // Re-label zone vocab for the new axis.
                let labels = zone_labels_for(new_edge);
                suppress.set(true);
                let keep = zone_ref.selected();
                zone_ref.set_model(Some(&gtk::StringList::new(&labels)));
                zone_ref.set_selected(keep.min(labels.len() as u32 - 1));
                suppress.set(false);
            } else {
                // Revert edge combo.
                let key = current_key.borrow().clone();
                let s = settings.borrow();
                if let Some(b) = s.binds.iter().find(|b| b.trigger.key() == key) {
                    if let Trigger::TouchEdge { edge, .. } = b.trigger {
                        let old_idx = Edge::ALL.iter().position(|e| *e == edge).unwrap_or(0) as u32;
                        suppress.set(true);
                        edge_ref.set_selected(old_idx);
                        suppress.set(false);
                    }
                }
            }
        });
    }
    row.add_row(&edge_combo);

    // Zone change: mutate trigger (None for "Full", else one of Start/Center/End).
    {
        let settings = settings.clone();
        let current_key = current_key.clone();
        let suppress = suppress.clone();
        let row_ref = row.clone();
        let zone_ref = zone_combo.clone();
        zone_combo.connect_selected_notify(move |combo| {
            if suppress.get() {
                return;
            }
            let idx = combo.selected() as usize;
            let new_zone = if idx == 0 {
                None
            } else {
                Some(EdgeZone::ALL[(idx - 1) % EdgeZone::ALL.len()])
            };
            let ok = try_update_trigger(&settings, &current_key, &row_ref, |t| match t {
                Trigger::TouchEdge { edge, .. } => Trigger::TouchEdge {
                    edge,
                    zone: new_zone,
                },
                other => other,
            });
            if !ok {
                // Revert zone combo.
                let key = current_key.borrow().clone();
                let s = settings.borrow();
                if let Some(b) = s.binds.iter().find(|b| b.trigger.key() == key) {
                    if let Trigger::TouchEdge { zone, .. } = b.trigger {
                        let old_idx = match zone {
                            None => 0,
                            Some(z) => {
                                (EdgeZone::ALL.iter().position(|zz| *zz == z).unwrap_or(0) + 1)
                                    as u32
                            }
                        };
                        suppress.set(true);
                        zone_ref.set_selected(old_idx);
                        suppress.set(false);
                    }
                }
            }
        });
    }
    row.add_row(&zone_combo);
}

fn zone_labels_for(edge: Edge) -> [&'static str; 4] {
    match edge {
        Edge::Left | Edge::Right => ["Full", "Top", "Center", "Bottom"],
        Edge::Top | Edge::Bottom => ["Full", "Left", "Center", "Right"],
    }
}

// ---------------------------------------------------------------------------
// Add bind form — nested picker for the property-form trigger schema
// ---------------------------------------------------------------------------

fn build_add_form(
    settings: &Rc<RefCell<TouchscreenSettings>>,
    binds_group: &Rc<adw::PreferencesGroup>,
    tracked_rows: &Rc<RefCell<Vec<adw::ExpanderRow>>>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Add New Bind")
        .build();

    // Family selector (Swipe / Pinch / Rotate / Edge / Tap / Tap-Hold-Drag)
    let family_labels: [&str; 6] = Family::all_labels();
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
            1.0,
            1.0,
            0.0,
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

    // Sensitivity / natural scroll / tag — set at creation time, editable after.
    let sens_row = adw::SpinRow::builder()
        .title("Sensitivity")
        .subtitle("Speed multiplier (continuous actions)")
        .adjustment(&gtk::Adjustment::new(1.0, 0.1, 5.0, 0.1, 0.5, 0.0))
        .digits(1)
        .build();
    group.add(&sens_row);

    let natural_row = adw::SwitchRow::builder()
        .title("Natural Scroll")
        .subtitle("Invert gesture direction")
        .active(false)
        .build();
    group.add(&natural_row);

    let tag_row = adw::EntryRow::builder().title("Tag").build();
    group.add(&tag_row);

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
                    dir_combo.set_model(Some(&gtk::StringList::new(&[
                        "Up", "Down", "Left", "Right",
                    ])));
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
                    dir_combo.set_model(Some(&gtk::StringList::new(&[
                        "Clockwise",
                        "Counter-Clockwise",
                    ])));
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
                Family::Tap => {
                    fingers_row.set_visible(true);
                    dir_combo.set_visible(false);
                    edge_combo.set_visible(false);
                    zone_combo.set_visible(false);
                }
                Family::TapHoldDrag => {
                    fingers_row.set_visible(true);
                    dir_combo.set_visible(true);
                    dir_combo.set_model(Some(&gtk::StringList::new(&[
                        "Any", "Up", "Down", "Left", "Right",
                    ])));
                    dir_combo.set_selected(0);
                    edge_combo.set_visible(false);
                    zone_combo.set_visible(false);
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
        let tracked_rows = tracked_rows.clone();
        let family_combo = family_combo.clone();
        let fingers_row = fingers_row.clone();
        let dir_combo = dir_combo.clone();
        let edge_combo = edge_combo.clone();
        let zone_combo = zone_combo.clone();
        let action_combo = action_combo.clone();
        let spawn_entry = spawn_entry.clone();
        let sens_row = sens_row.clone();
        let natural_row = natural_row.clone();
        let tag_row = tag_row.clone();

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
                Family::Tap => Trigger::TouchTap { fingers },
                Family::TapHoldDrag => {
                    let direction = if dir_idx == 0 {
                        None
                    } else {
                        Some(SwipeDir::ALL[(dir_idx - 1) as usize % SwipeDir::ALL.len()])
                    };
                    Trigger::TouchTapHoldDrag { fingers, direction }
                }
            };

            let action_idx = action_combo.selected() as usize;
            if action_idx >= ACTION_OPTIONS.len() {
                return;
            }
            let action_name = ACTION_OPTIONS[action_idx].1.to_string();

            // Check for duplicates
            let key = trigger.key();
            if settings
                .borrow()
                .binds
                .iter()
                .any(|b| b.trigger.key() == key)
            {
                return;
            }

            let action_args = if action_name == "spawn" {
                let cmd = spawn_entry.text().to_string();
                if cmd.is_empty() {
                    return;
                }
                vec![cmd]
            } else {
                vec![]
            };

            let sensitivity = sens_row.value();
            let tag_text = tag_row.text().to_string();
            let bind = TouchBindEntry {
                trigger,
                action_name,
                action_args,
                sensitivity: if (sensitivity - 1.0).abs() < f64::EPSILON {
                    None
                } else {
                    Some(sensitivity)
                },
                natural_scroll: natural_row.is_active(),
                tag: if tag_text.is_empty() {
                    None
                } else {
                    Some(tag_text)
                },
                enabled: true,
            };

            // Add to settings
            settings.borrow_mut().binds.push(bind.clone());
            save_and_reload(&settings.borrow());

            // Add row to UI
            let row = build_bind_row(&bind, &binds_group, &settings, &tracked_rows);
            binds_group.add(&row);
            tracked_rows.borrow_mut().push(row);

            // Reset auxiliary fields for the next entry.
            sens_row.set_value(1.0);
            natural_row.set_active(false);
            tag_row.set_text("");
            if action_combo.selected() as usize >= ACTION_OPTIONS.len()
                || ACTION_OPTIONS[action_combo.selected() as usize].1 != "spawn"
            {
                spawn_entry.set_text("");
            }
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
