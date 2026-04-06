use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{self, TouchpadSettings};
use super::widgets;

pub fn build() -> gtk::Box {
    let settings = Rc::new(RefCell::new(config::read_touchpad_settings()));

    let stack = gtk::Stack::new();
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
        "Workspace Switch",
        "Vertical swipe to switch workspaces",
        &settings.borrow().workspace_switch,
        save.clone(),
        ws_action,
    ));

    page.add(&widgets::build_gesture_group(
        "View Scroll",
        "3-finger horizontal swipe to scroll between columns",
        &settings.borrow().view_scroll,
        save.clone(),
        vs_action,
    ));

    page.add(&widgets::build_gesture_group(
        "Overview Toggle",
        "4-finger vertical swipe to open/close overview",
        &settings.borrow().overview_toggle,
        save.clone(),
        ov_action,
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
