use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{self, TouchpadSettings};
use super::widgets;

pub fn build() -> gtk::Box {
    let settings = Rc::new(RefCell::new(config::read_touchpad_settings()));

    let stack = gtk::Stack::new();
    stack.add_titled(&build_general(&settings), Some("general"), "General");
    stack.add_titled(&build_gestures(&settings), Some("gestures"), "Gestures");
    stack.add_titled(&build_advanced(&settings), Some("advanced"), "Advanced");

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
        .description("Basic touchpad settings applied via libinput")
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

    page
}

fn build_gestures(settings: &Rc<RefCell<TouchpadSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    let ws_action = Rc::new(RefCell::new(settings.borrow().workspace_switch.clone()));
    let vs_action = Rc::new(RefCell::new(settings.borrow().view_scroll.clone()));
    let ov_action = Rc::new(RefCell::new(settings.borrow().overview_toggle.clone()));

    let save = {
        let settings = settings.clone();
        let ws = ws_action.clone();
        let vs = vs_action.clone();
        let ov = ov_action.clone();
        Rc::new(move || {
            let mut s = settings.borrow_mut();
            s.workspace_switch = ws.borrow().clone();
            s.view_scroll = vs.borrow().clone();
            s.overview_toggle = ov.borrow().clone();
            config::write_touchpad_settings(&s);
            config::reload_config();
        })
    };

    let info = adw::PreferencesGroup::builder()
        .description("Gestures require 3+ fingers. 2-finger events (scroll, pinch-to-zoom) are handled by libinput and passed directly to apps.")
        .build();
    page.add(&info);

    page.add(&widgets::build_gesture_group(
        "View Scroll",
        "Horizontal swipe to scroll between columns",
        &settings.borrow().view_scroll,
        save.clone(),
        vs_action,
        false,
    ));

    page.add(&widgets::build_gesture_group(
        "Workspace Switch",
        "Vertical swipe to switch workspaces",
        &settings.borrow().workspace_switch,
        save.clone(),
        ws_action,
        false,
    ));

    page.add(&widgets::build_gesture_group(
        "Overview Toggle",
        "Vertical swipe to open/close overview",
        &settings.borrow().overview_toggle,
        save.clone(),
        ov_action,
        false,
    ));

    page
}

fn build_advanced(settings: &Rc<RefCell<TouchpadSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    let settings_clone = settings.clone();
    page.add(&widgets::build_threshold_row(
        settings.borrow().recognition_threshold,
        move |val| {
            settings_clone.borrow_mut().recognition_threshold = val;
            config::write_touchpad_settings(&settings_clone.borrow());
            config::reload_config();
        },
    ));

    page
}
