use adw::prelude::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

use crate::config::{
    self, SwipeDir, TouchBindEntry, TouchpadSettings, Trigger, MAX_FINGERS, MIN_FINGERS,
};

pub fn build() -> gtk::Box {
    let settings = Rc::new(RefCell::new(config::read_touchpad_settings()));

    let stack = gtk::Stack::new();
    stack.add_titled(&build_general(&settings), Some("general"), "Device");
    stack.add_titled(&build_gestures(&settings), Some("gestures"), "Gesture Binds");

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

fn save_and_reload(settings: &Rc<RefCell<TouchpadSettings>>) {
    config::write_touchpad_settings(&settings.borrow());
    config::reload_config();
}

fn build_general(settings: &Rc<RefCell<TouchpadSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    // --- Basic settings ---
    let basic_group = adw::PreferencesGroup::builder()
        .title("Device")
        .description("Writes to touchpad-gestures.kdl → replaces input { touchpad {} }")
        .build();

    // Off
    let off_row = adw::SwitchRow::builder()
        .title("Touchpad Enabled")
        .subtitle("Disable to turn off the touchpad entirely")
        .active(!settings.borrow().off)
        .build();
    {
        let s = settings.clone();
        off_row.connect_active_notify(move |row| {
            s.borrow_mut().off = !row.is_active();
            save_and_reload(&s);
        });
    }
    basic_group.add(&off_row);

    // Tap
    let tap_row = adw::SwitchRow::builder()
        .title("Tap to Click")
        .subtitle("Tap the touchpad to click")
        .active(settings.borrow().tap)
        .build();
    {
        let s = settings.clone();
        tap_row.connect_active_notify(move |row| {
            s.borrow_mut().tap = row.is_active();
            save_and_reload(&s);
        });
    }
    basic_group.add(&tap_row);

    // Natural Scroll
    let ns_row = adw::SwitchRow::builder()
        .title("Natural Scroll")
        .subtitle("Invert scroll direction (content follows finger)")
        .active(settings.borrow().natural_scroll)
        .build();
    {
        let s = settings.clone();
        ns_row.connect_active_notify(move |row| {
            s.borrow_mut().natural_scroll = row.is_active();
            save_and_reload(&s);
        });
    }
    basic_group.add(&ns_row);

    // Dwt
    let dwt_row = adw::SwitchRow::builder()
        .title("Disable While Typing")
        .subtitle("Ignore touchpad input while typing on the keyboard")
        .active(settings.borrow().dwt)
        .build();
    {
        let s = settings.clone();
        dwt_row.connect_active_notify(move |row| {
            s.borrow_mut().dwt = row.is_active();
            save_and_reload(&s);
        });
    }
    basic_group.add(&dwt_row);

    // Dwtp
    let dwtp_row = adw::SwitchRow::builder()
        .title("Disable While Trackpointing")
        .subtitle("Ignore touchpad input while using the trackpoint")
        .active(settings.borrow().dwtp)
        .build();
    {
        let s = settings.clone();
        dwtp_row.connect_active_notify(move |row| {
            s.borrow_mut().dwtp = row.is_active();
            save_and_reload(&s);
        });
    }
    basic_group.add(&dwtp_row);

    // Disabled on External Mouse
    let ext_mouse_row = adw::SwitchRow::builder()
        .title("Disable on External Mouse")
        .subtitle("Turn off touchpad when a mouse is connected")
        .active(settings.borrow().disabled_on_external_mouse)
        .build();
    {
        let s = settings.clone();
        ext_mouse_row.connect_active_notify(move |row| {
            s.borrow_mut().disabled_on_external_mouse = row.is_active();
            save_and_reload(&s);
        });
    }
    basic_group.add(&ext_mouse_row);

    page.add(&basic_group);

    // --- Scroll settings ---
    let scroll_group = adw::PreferencesGroup::builder()
        .title("Scrolling")
        .build();

    // Scroll Factor
    let sf_adj = gtk::Adjustment::new(
        settings.borrow().scroll_factor.unwrap_or(1.0),
        0.1, 10.0, 0.1, 0.5, 0.0,
    );
    let sf_row = adw::SpinRow::builder()
        .title("Scroll Factor")
        .subtitle("Speed multiplier for 2-finger scrolling")
        .adjustment(&sf_adj)
        .digits(1)
        .build();
    {
        let s = settings.clone();
        sf_row.connect_value_notify(move |row| {
            s.borrow_mut().scroll_factor = Some(row.value());
            save_and_reload(&s);
        });
    }
    scroll_group.add(&sf_row);

    // Scroll Method
    let scroll_methods = gtk::StringList::new(&["Default", "Two Finger", "Edge", "On Button Down", "No Scroll"]);
    let current_scroll = match settings.borrow().scroll_method.as_deref() {
        Some("two-finger") => 1,
        Some("edge") => 2,
        Some("on-button-down") => 3,
        Some("no-scroll") => 4,
        _ => 0,
    };
    let sm_row = adw::ComboRow::builder()
        .title("Scroll Method")
        .model(&scroll_methods)
        .selected(current_scroll)
        .build();
    {
        let s = settings.clone();
        sm_row.connect_selected_notify(move |row| {
            s.borrow_mut().scroll_method = match row.selected() {
                1 => Some("two-finger".to_string()),
                2 => Some("edge".to_string()),
                3 => Some("on-button-down".to_string()),
                4 => Some("no-scroll".to_string()),
                _ => None,
            };
            save_and_reload(&s);
        });
    }
    scroll_group.add(&sm_row);

    page.add(&scroll_group);

    // --- Acceleration ---
    let accel_group = adw::PreferencesGroup::builder()
        .title("Acceleration")
        .build();

    // Accel Speed
    let as_adj = gtk::Adjustment::new(
        settings.borrow().accel_speed,
        -1.0, 1.0, 0.1, 0.2, 0.0,
    );
    let as_row = adw::SpinRow::builder()
        .title("Acceleration Speed")
        .subtitle("Pointer speed (-1.0 slowest, 1.0 fastest)")
        .adjustment(&as_adj)
        .digits(1)
        .build();
    {
        let s = settings.clone();
        as_row.connect_value_notify(move |row| {
            s.borrow_mut().accel_speed = row.value();
            save_and_reload(&s);
        });
    }
    accel_group.add(&as_row);

    // Accel Profile
    let profiles = gtk::StringList::new(&["Default", "Adaptive", "Flat"]);
    let current_profile = match settings.borrow().accel_profile.as_deref() {
        Some("adaptive") => 1,
        Some("flat") => 2,
        _ => 0,
    };
    let ap_row = adw::ComboRow::builder()
        .title("Acceleration Profile")
        .model(&profiles)
        .selected(current_profile)
        .build();
    {
        let s = settings.clone();
        ap_row.connect_selected_notify(move |row| {
            s.borrow_mut().accel_profile = match row.selected() {
                1 => Some("adaptive".to_string()),
                2 => Some("flat".to_string()),
                _ => None,
            };
            save_and_reload(&s);
        });
    }
    accel_group.add(&ap_row);

    page.add(&accel_group);

    // --- Click & Tap ---
    let click_group = adw::PreferencesGroup::builder()
        .title("Click & Tap")
        .build();

    // Click Method
    let click_methods = gtk::StringList::new(&["Default", "Clickfinger", "Button Areas"]);
    let current_click = match settings.borrow().click_method.as_deref() {
        Some("clickfinger") => 1,
        Some("button-areas") => 2,
        _ => 0,
    };
    let cm_row = adw::ComboRow::builder()
        .title("Click Method")
        .model(&click_methods)
        .selected(current_click)
        .build();
    {
        let s = settings.clone();
        cm_row.connect_selected_notify(move |row| {
            s.borrow_mut().click_method = match row.selected() {
                1 => Some("clickfinger".to_string()),
                2 => Some("button-areas".to_string()),
                _ => None,
            };
            save_and_reload(&s);
        });
    }
    click_group.add(&cm_row);

    // Tap Button Map
    let tap_maps = gtk::StringList::new(&["Default", "Left-Right-Middle", "Left-Middle-Right"]);
    let current_tap = match settings.borrow().tap_button_map.as_deref() {
        Some("left-right-middle") => 1,
        Some("left-middle-right") => 2,
        _ => 0,
    };
    let tbm_row = adw::ComboRow::builder()
        .title("Tap Button Map")
        .subtitle("Which button each multi-finger tap triggers")
        .model(&tap_maps)
        .selected(current_tap)
        .build();
    {
        let s = settings.clone();
        tbm_row.connect_selected_notify(move |row| {
            s.borrow_mut().tap_button_map = match row.selected() {
                1 => Some("left-right-middle".to_string()),
                2 => Some("left-middle-right".to_string()),
                _ => None,
            };
            save_and_reload(&s);
        });
    }
    click_group.add(&tbm_row);

    // Middle Emulation
    let me_row = adw::SwitchRow::builder()
        .title("Middle Click Emulation")
        .subtitle("Press left and right buttons simultaneously for middle click")
        .active(settings.borrow().middle_emulation)
        .build();
    {
        let s = settings.clone();
        me_row.connect_active_notify(move |row| {
            s.borrow_mut().middle_emulation = row.is_active();
            save_and_reload(&s);
        });
    }
    click_group.add(&me_row);

    // Left Handed
    let lh_row = adw::SwitchRow::builder()
        .title("Left Handed")
        .subtitle("Swap left and right buttons")
        .active(settings.borrow().left_handed)
        .build();
    {
        let s = settings.clone();
        lh_row.connect_active_notify(move |row| {
            s.borrow_mut().left_handed = row.is_active();
            save_and_reload(&s);
        });
    }
    click_group.add(&lh_row);

    page.add(&click_group);

    // Gesture recognition threshold
    let thresh_group = adw::PreferencesGroup::builder()
        .title("Gesture Recognition")
        .description("Fine-tune how gestures are detected")
        .build();

    let threshold_row = adw::SpinRow::builder()
        .title("Swipe Trigger Distance")
        .subtitle("Libinput delta units of motion before a swipe commits")
        .adjustment(&gtk::Adjustment::new(
            settings.borrow().swipe_trigger_distance,
            4.0, 100.0, 1.0, 5.0, 0.0,
        ))
        .digits(1)
        .build();
    {
        let settings = settings.clone();
        threshold_row.connect_value_notify(move |row| {
            settings.borrow_mut().swipe_trigger_distance = row.value();
            save_and_reload(&settings);
        });
    }
    thresh_group.add(&threshold_row);
    page.add(&thresh_group);

    // IPC progress scaling
    let ipc_group = adw::PreferencesGroup::builder()
        .title("IPC Progress")
        .description("Controls progress scaling for external tools (libinput delta units).\n\
             Noop gestures: progress directly drives the external app (1:1 sync).\n\
             Compositor actions (workspace switch, etc): progress is informational only — \
             niri uses its own internal thresholds to decide when to commit.")
        .build();

    let progress_row = adw::SpinRow::builder()
        .title("Swipe Progress Distance")
        .subtitle("Libinput delta units of swipe that map to IPC progress = 1.0")
        .adjustment(&gtk::Adjustment::new(
            settings.borrow().swipe_progress_distance,
            10.0, 500.0, 5.0, 20.0, 0.0,
        ))
        .digits(0)
        .build();
    {
        let settings = settings.clone();
        progress_row.connect_value_notify(move |row| {
            settings.borrow_mut().swipe_progress_distance = row.value();
            save_and_reload(&settings);
        });
    }
    ipc_group.add(&progress_row);
    page.add(&ipc_group);

    page
}

// ---------------------------------------------------------------------------
// Touchpad gesture action list
// ---------------------------------------------------------------------------

const TOUCHPAD_ACTION_OPTIONS: &[(&str, &str)] = &[
    ("Focus Workspace Up", "focus-workspace-up"),
    ("Focus Workspace Down", "focus-workspace-down"),
    ("Focus Column Left", "focus-column-left"),
    ("Focus Column Right", "focus-column-right"),
    ("Toggle Overview", "toggle-overview"),
    ("Open Overview", "open-overview"),
    ("Close Overview", "close-overview"),
    ("Close Window", "close-window"),
    ("Fullscreen Window", "fullscreen-window"),
    ("Maximize Column", "maximize-column"),
    ("Center Column", "center-column"),
    ("Toggle Floating", "toggle-window-floating"),
    ("Screenshot", "screenshot"),
    ("Move Window to Workspace Down", "move-window-to-workspace-down"),
    ("Move Window to Workspace Up", "move-window-to-workspace-up"),
    ("Focus Monitor Left", "focus-monitor-left"),
    ("Focus Monitor Right", "focus-monitor-right"),
    ("Show Hotkey Overlay", "show-hotkey-overlay"),
    ("Spawn Command...", "spawn"),
    ("Noop (IPC only)", "noop"),
];

fn display_action_name(action_name: &str, action_args: &[String]) -> String {
    if action_name == "spawn" && !action_args.is_empty() {
        return format!("Spawn: {}", action_args.join(" "));
    }
    if let Some((display, _)) = TOUCHPAD_ACTION_OPTIONS.iter().find(|(_, k)| *k == action_name) {
        return display.to_string();
    }
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
// Gesture Binds tab
// ---------------------------------------------------------------------------

fn build_gestures(settings: &Rc<RefCell<TouchpadSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    let info = adw::PreferencesGroup::builder()
        .description(
            "Writes to touchpad-gestures.kdl → merges into binds {}\n\
             3+ finger gestures only. 2-finger scroll/pinch is handled by libinput."
        )
        .build();
    page.add(&info);

    // Tracked rows — each added row is recorded here so the search entry
    // can filter by visibility without rebuilding the list.
    let tracked_rows: Rc<RefCell<Vec<adw::ExpanderRow>>> = Rc::new(RefCell::new(Vec::new()));

    // Active binds group
    let binds_group = Rc::new(adw::PreferencesGroup::builder()
        .title("Active Binds")
        .build());

    // Add new bind form (at top for easy access)
    let add_group = build_add_form(settings, &binds_group, &tracked_rows);
    page.add(&add_group);

    // Search / filter entry — sits just above the Active Binds list
    let search_group = adw::PreferencesGroup::builder().build();
    let search_row = adw::EntryRow::builder()
        .title("Filter Binds")
        .build();
    search_row.add_prefix(&gtk::Image::from_icon_name("system-search-symbolic"));
    {
        let tracked_rows = tracked_rows.clone();
        search_row.connect_changed(move |entry| {
            let query = entry.text().to_string().to_lowercase();
            for r in tracked_rows.borrow().iter() {
                let title = r.title().to_string().to_lowercase();
                let subtitle = r.subtitle().to_string().to_lowercase();
                let visible = query.is_empty()
                    || title.contains(&query)
                    || subtitle.contains(&query);
                r.set_visible(visible);
            }
        });
    }
    search_group.add(&search_row);
    page.add(&search_group);

    let binds = settings.borrow().binds.clone();
    for bind in &binds {
        let row = build_bind_row(bind, &binds_group, settings, &tracked_rows);
        binds_group.add(&row);
        tracked_rows.borrow_mut().push(row);
    }
    page.add(&*binds_group);

    page
}

/// In-place trigger edit with collision detection (touchpad variant).
fn try_update_trigger<F>(
    settings: &Rc<RefCell<TouchpadSettings>>,
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
            return true;
        }
        if s.binds.iter().enumerate().any(|(i, b)| i != idx && b.trigger.key() == new_key) {
            return false;
        }
        s.binds[idx].trigger = new_trigger;
    }
    *current_key.borrow_mut() = new_key;
    row.set_title(&new_trigger.display_name());
    save_and_reload(settings);
    true
}

fn build_bind_row(
    bind: &TouchBindEntry,
    group: &Rc<adw::PreferencesGroup>,
    settings: &Rc<RefCell<TouchpadSettings>>,
    tracked_rows: &Rc<RefCell<Vec<adw::ExpanderRow>>>,
) -> adw::ExpanderRow {
    let gesture_display = bind.trigger.display_name();
    let action_display = display_action_name(&bind.action_name, &bind.action_args);

    let current_key = Rc::new(RefCell::new(bind.trigger.key()));
    let suppress = Rc::new(Cell::new(false));

    let row = adw::ExpanderRow::builder()
        .title(&gesture_display)
        .subtitle(&action_display)
        .build();

    // Enable/disable toggle in suffix
    let enable_switch = gtk::Switch::builder()
        .valign(gtk::Align::Center)
        .active(bind.enabled)
        .build();

    {
        let settings = settings.clone();
        let current_key = current_key.clone();
        enable_switch.connect_active_notify(move |switch| {
            let key = current_key.borrow().clone();
            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.enabled = switch.is_active();
            }
            save_and_reload(&settings);
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
                    settings.borrow_mut().binds.retain(|b| b.trigger.key() != key);
                    save_and_reload(&settings);
                    group.remove(&row_clone);
                    tracked_rows.borrow_mut().retain(|r| r != &row_clone);
                }
            });

            dialog.present();
        });
    }
    row.add_suffix(&delete_btn);

    // -----------------------------------------------------------------
    // Editable trigger rows — touchpad: Swipe / TapHold / TapHoldDrag.
    // -----------------------------------------------------------------
    match bind.trigger {
        Trigger::TouchpadSwipe { fingers, direction } => {
            add_fingers_row(&row, settings, &current_key, &suppress, fingers);
            add_direction_row(&row, settings, &current_key, &suppress, direction);
        }
        Trigger::TouchpadTapHold { fingers }
        | Trigger::TouchpadTapHoldDrag { fingers } => {
            add_fingers_row(&row, settings, &current_key, &suppress, fingers);
        }
        _ => {
            // Not emitted on the touchpad page.
        }
    }

    // Action dropdown
    let action_labels: Vec<&str> = TOUCHPAD_ACTION_OPTIONS.iter().map(|(d, _)| *d).collect();
    let action_model = gtk::StringList::new(&action_labels);
    let action_combo = adw::ComboRow::builder()
        .title("Action")
        .model(&action_model)
        .build();

    let current_idx = TOUCHPAD_ACTION_OPTIONS.iter()
        .position(|(_, k)| *k == bind.action_name)
        .unwrap_or(0) as u32;
    action_combo.set_selected(current_idx);

    {
        let current_key = current_key.clone();
        let settings = settings.clone();
        let row_ref = row.clone();
        action_combo.connect_selected_notify(move |combo| {
            let idx = combo.selected() as usize;
            if idx >= TOUCHPAD_ACTION_OPTIONS.len() { return; }
            let new_action = TOUCHPAD_ACTION_OPTIONS[idx].1.to_string();
            let new_display = TOUCHPAD_ACTION_OPTIONS[idx].0;
            let key = current_key.borrow().clone();
            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.action_name = new_action;
                b.action_args.clear();
            }
            row_ref.set_subtitle(new_display);
            save_and_reload(&settings);
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
        let current_key = current_key.clone();
        let settings = settings.clone();
        sens_row.connect_value_notify(move |spin| {
            let key = current_key.borrow().clone();
            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.sensitivity = Some(spin.value());
            }
            save_and_reload(&settings);
        });
    }
    row.add_row(&sens_row);

    // Tag — IPC event identifier
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
            if let Some(b) = settings.borrow_mut().binds.iter_mut()
                .find(|b| b.trigger.key() == key)
            {
                b.tag = if text.is_empty() { None } else { Some(text) };
            }
            save_and_reload(&settings);
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
    settings: &Rc<RefCell<TouchpadSettings>>,
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
            1.0, 1.0, 0.0,
        ))
        .build();

    {
        let settings = settings.clone();
        let current_key = current_key.clone();
        let suppress = suppress.clone();
        let row_ref = row.clone();
        let fingers_ref = fingers_row.clone();
        fingers_row.connect_value_notify(move |spin| {
            if suppress.get() { return; }
            let new_fingers = spin.value() as u8;
            let ok = try_update_trigger(&settings, &current_key, &row_ref, |t| match t {
                Trigger::TouchpadSwipe { direction, .. } => {
                    Trigger::TouchpadSwipe { fingers: new_fingers, direction }
                }
                Trigger::TouchpadTapHold { .. } => {
                    Trigger::TouchpadTapHold { fingers: new_fingers }
                }
                Trigger::TouchpadTapHoldDrag { .. } => {
                    Trigger::TouchpadTapHoldDrag { fingers: new_fingers }
                }
                other => other,
            });
            if !ok {
                let key = current_key.borrow().clone();
                let s = settings.borrow();
                if let Some(b) = s.binds.iter().find(|b| b.trigger.key() == key) {
                    let old = match b.trigger {
                        Trigger::TouchpadSwipe { fingers, .. }
                        | Trigger::TouchpadTapHold { fingers }
                        | Trigger::TouchpadTapHoldDrag { fingers } => fingers,
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
    settings: &Rc<RefCell<TouchpadSettings>>,
    current_key: &Rc<RefCell<String>>,
    suppress: &Rc<Cell<bool>>,
    initial: SwipeDir,
) {
    let labels: Vec<&str> = SwipeDir::ALL.iter().map(|d| d.display()).collect();
    let model = gtk::StringList::new(&labels);
    let dir_combo = adw::ComboRow::builder()
        .title("Direction")
        .model(&model)
        .selected(SwipeDir::ALL.iter().position(|d| *d == initial).unwrap_or(0) as u32)
        .build();

    {
        let settings = settings.clone();
        let current_key = current_key.clone();
        let suppress = suppress.clone();
        let row_ref = row.clone();
        let dir_ref = dir_combo.clone();
        dir_combo.connect_selected_notify(move |combo| {
            if suppress.get() { return; }
            let idx = combo.selected() as usize;
            let ok = try_update_trigger(&settings, &current_key, &row_ref, |t| match t {
                Trigger::TouchpadSwipe { fingers, .. } => Trigger::TouchpadSwipe {
                    fingers,
                    direction: SwipeDir::ALL[idx % SwipeDir::ALL.len()],
                },
                other => other,
            });
            if !ok {
                let key = current_key.borrow().clone();
                let s = settings.borrow();
                if let Some(b) = s.binds.iter().find(|b| b.trigger.key() == key) {
                    if let Trigger::TouchpadSwipe { direction, .. } = b.trigger {
                        let old_idx = SwipeDir::ALL.iter()
                            .position(|d| *d == direction)
                            .unwrap_or(0) as u32;
                        suppress.set(true);
                        dir_ref.set_selected(old_idx);
                        suppress.set(false);
                    }
                }
            }
        });
    }
    row.add_row(&dir_combo);
}

// ---------------------------------------------------------------------------
// Add bind form — family picker (Swipe / TapHold / TapHoldDrag)
// ---------------------------------------------------------------------------

fn build_add_form(
    settings: &Rc<RefCell<TouchpadSettings>>,
    binds_group: &Rc<adw::PreferencesGroup>,
    tracked_rows: &Rc<RefCell<Vec<adw::ExpanderRow>>>,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Add New Bind")
        .build();

    // Family selector: Swipe (has direction) / Tap-Hold / Tap-Hold-Drag
    let family_labels = ["Swipe", "Tap-Hold", "Tap-Hold-Drag"];
    let family_model = gtk::StringList::new(&family_labels);
    let family_combo = adw::ComboRow::builder()
        .title("Gesture Family")
        .model(&family_model)
        .selected(0)
        .build();
    group.add(&family_combo);

    let fingers_row = adw::SpinRow::builder()
        .title("Fingers")
        .subtitle("Number of fingers required (3–10)")
        .adjustment(&gtk::Adjustment::new(
            3.0,
            MIN_FINGERS as f64,
            MAX_FINGERS as f64,
            1.0, 1.0, 0.0,
        ))
        .build();
    group.add(&fingers_row);

    let dir_labels = ["Up", "Down", "Left", "Right"];
    let dir_model = gtk::StringList::new(&dir_labels);
    let dir_combo = adw::ComboRow::builder()
        .title("Direction")
        .model(&dir_model)
        .selected(0)
        .build();
    group.add(&dir_combo);

    // Direction row only applies to Swipe — hide for tap variants.
    {
        let dir_combo = dir_combo.clone();
        family_combo.connect_selected_notify(move |combo| {
            dir_combo.set_visible(combo.selected() == 0);
        });
    }

    // Action dropdown
    let action_labels: Vec<&str> = TOUCHPAD_ACTION_OPTIONS.iter().map(|(d, _)| *d).collect();
    let action_model = gtk::StringList::new(&action_labels);
    let action_combo = adw::ComboRow::builder()
        .title("Action")
        .model(&action_model)
        .selected(0)
        .build();
    group.add(&action_combo);

    // Spawn command entry
    let spawn_entry = adw::EntryRow::builder()
        .title("Command")
        .visible(false)
        .build();
    group.add(&spawn_entry);

    {
        let spawn_entry = spawn_entry.clone();
        action_combo.connect_selected_notify(move |row| {
            let idx = row.selected() as usize;
            if idx < TOUCHPAD_ACTION_OPTIONS.len() {
                spawn_entry.set_visible(TOUCHPAD_ACTION_OPTIONS[idx].1 == "spawn");
            }
        });
    }

    // Sensitivity / tag — set at creation time, editable after.
    let sens_row = adw::SpinRow::builder()
        .title("Sensitivity")
        .subtitle("Speed multiplier (continuous actions)")
        .adjustment(&gtk::Adjustment::new(1.0, 0.1, 5.0, 0.1, 0.5, 0.0))
        .digits(1)
        .build();
    group.add(&sens_row);

    let tag_row = adw::EntryRow::builder()
        .title("Tag")
        .build();
    group.add(&tag_row);

    // Add button
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
        let action_combo = action_combo.clone();
        let spawn_entry = spawn_entry.clone();
        let sens_row = sens_row.clone();
        let tag_row = tag_row.clone();

        add_row.connect_activated(move |_| {
            let fingers = fingers_row.value() as u8;
            let family_idx = family_combo.selected();
            let trigger = match family_idx {
                0 => {
                    let dir_idx = dir_combo.selected() as usize;
                    if dir_idx >= SwipeDir::ALL.len() { return }
                    Trigger::TouchpadSwipe { fingers, direction: SwipeDir::ALL[dir_idx] }
                }
                1 => Trigger::TouchpadTapHold { fingers },
                _ => Trigger::TouchpadTapHoldDrag { fingers },
            };

            let action_idx = action_combo.selected() as usize;
            if action_idx >= TOUCHPAD_ACTION_OPTIONS.len() { return }
            let action_name = TOUCHPAD_ACTION_OPTIONS[action_idx].1.to_string();

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
                natural_scroll: false,
                tag: if tag_text.is_empty() { None } else { Some(tag_text) },
                enabled: true,
            };

            settings.borrow_mut().binds.push(bind.clone());
            save_and_reload(&settings);

            let row = build_bind_row(&bind, &binds_group, &settings, &tracked_rows);
            binds_group.add(&row);
            tracked_rows.borrow_mut().push(row);

            // Reset auxiliary fields for the next entry.
            sens_row.set_value(1.0);
            tag_row.set_text("");
            if action_combo.selected() as usize >= TOUCHPAD_ACTION_OPTIONS.len()
                || TOUCHPAD_ACTION_OPTIONS[action_combo.selected() as usize].1 != "spawn"
            {
                spawn_entry.set_text("");
            }
        });
    }
    group.add(&add_row);

    group
}

