use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{self, TouchscreenSettings};
use super::widgets;

pub fn build() -> gtk::Box {
    let settings = Rc::new(RefCell::new(config::read_touchscreen_settings()));

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

fn build_general(settings: &Rc<RefCell<TouchscreenSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    let group = adw::PreferencesGroup::builder()
        .title("Touchscreen Input")
        .description("General touchscreen settings")
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
            config::write_touchscreen_settings(&settings.borrow());
            config::reload_config();
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
            config::write_touchscreen_settings(&settings.borrow());
            config::reload_config();
        });
    }
    group.add(&output_row);

    page.add(&group);
    page
}

fn build_gestures(settings: &Rc<RefCell<TouchscreenSettings>>) -> adw::PreferencesPage {
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
            config::write_touchscreen_settings(&s);
            config::reload_config();
        })
    };

    let info = adw::PreferencesGroup::builder()
        .description("Gestures require 3+ fingers. 2-finger events (pinch-to-zoom, etc.) are passed directly to apps.")
        .build();
    page.add(&info);

    page.add(&widgets::build_gesture_group(
        "View Scroll",
        "Horizontal swipe to scroll between columns",
        &settings.borrow().view_scroll,
        save.clone(),
        vs_action,
        true,
    ));

    page.add(&widgets::build_gesture_group(
        "Workspace Switch",
        "Vertical swipe to switch workspaces",
        &settings.borrow().workspace_switch,
        save.clone(),
        ws_action,
        true,
    ));

    page.add(&widgets::build_gesture_group(
        "Overview Toggle",
        "Vertical swipe to open/close overview",
        &settings.borrow().overview_toggle,
        save.clone(),
        ov_action,
        true,
    ));

    page
}

fn build_advanced(settings: &Rc<RefCell<TouchscreenSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    let settings_clone = settings.clone();
    page.add(&widgets::build_threshold_row(
        settings.borrow().recognition_threshold,
        move |val| {
            settings_clone.borrow_mut().recognition_threshold = val;
            config::write_touchscreen_settings(&settings_clone.borrow());
            config::reload_config();
        },
    ));

    page
}
