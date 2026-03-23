use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{self, GestureAction, TouchSettings};

pub fn build(settings: &Rc<RefCell<TouchSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .title("Gestures")
        .build();

    page.add(&build_gesture_group(
        "Workspace Switch",
        "Vertical swipe to switch workspaces",
        settings,
        |s| &mut s.workspace_switch,
    ));

    page.add(&build_gesture_group(
        "View Scroll",
        "Horizontal swipe to scroll between columns",
        settings,
        |s| &mut s.view_scroll,
    ));

    page.add(&build_gesture_group(
        "Overview Toggle",
        "Vertical swipe to open/close overview",
        settings,
        |s| &mut s.overview_toggle,
    ));

    page
}

fn build_gesture_group(
    title: &str,
    description: &str,
    settings: &Rc<RefCell<TouchSettings>>,
    accessor: fn(&mut TouchSettings) -> &mut GestureAction,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description(description)
        .build();

    let current = accessor(&mut settings.borrow_mut()).clone();

    // Enable/disable switch — only controls this gesture, normal touch still works.
    let enable_row = adw::SwitchRow::builder()
        .title("Enabled")
        .subtitle("Normal touch input is not affected")
        .active(current.enabled)
        .build();

    {
        let settings = settings.clone();
        enable_row.connect_active_notify(move |row| {
            accessor(&mut settings.borrow_mut()).enabled = row.is_active();
            config::write_settings(&settings.borrow());
            config::reload_config();
        });
    }
    group.add(&enable_row);

    // Finger count spin row.
    let finger_adj = gtk::Adjustment::new(
        current.finger_count as f64,
        2.0,  // min
        5.0,  // max
        1.0,  // step
        1.0,  // page increment
        0.0,  // page size
    );
    let finger_row = adw::SpinRow::builder()
        .title("Finger Count")
        .subtitle("Number of fingers to trigger gesture")
        .adjustment(&finger_adj)
        .build();

    {
        let settings = settings.clone();
        finger_row.connect_value_notify(move |row| {
            accessor(&mut settings.borrow_mut()).finger_count = row.value() as u8;
            config::write_settings(&settings.borrow());
            config::reload_config();
        });
    }
    group.add(&finger_row);

    // Sensitivity spin row.
    let sens_adj = gtk::Adjustment::new(
        current.sensitivity,
        0.1,  // min
        5.0,  // max
        0.1,  // step
        0.5,  // page increment
        0.0,  // page size
    );
    let sens_row = adw::SpinRow::builder()
        .title("Sensitivity")
        .subtitle("Speed multiplier for this gesture")
        .adjustment(&sens_adj)
        .digits(1)
        .build();

    {
        let settings = settings.clone();
        sens_row.connect_value_notify(move |row| {
            accessor(&mut settings.borrow_mut()).sensitivity = row.value();
            config::write_settings(&settings.borrow());
            config::reload_config();
        });
    }
    group.add(&sens_row);

    // Natural scroll switch.
    let natural_row = adw::SwitchRow::builder()
        .title("Natural Scroll")
        .subtitle("Invert direction for this gesture")
        .active(current.natural_scroll)
        .build();

    {
        let settings = settings.clone();
        natural_row.connect_active_notify(move |row| {
            accessor(&mut settings.borrow_mut()).natural_scroll = row.is_active();
            config::write_settings(&settings.borrow());
            config::reload_config();
        });
    }
    group.add(&natural_row);

    group
}
