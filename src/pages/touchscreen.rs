use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{self, EdgeSwipeSettings, TouchscreenSettings};
use super::widgets;

pub fn build() -> gtk::Box {
    let settings = Rc::new(RefCell::new(config::read_touchscreen_settings()));

    let stack = gtk::Stack::new();
    stack.add_titled(&build_general(&settings), Some("general"), "General");
    stack.add_titled(&build_gestures(&settings), Some("gestures"), "Gestures");
    stack.add_titled(&build_edge_swipes(&settings), Some("edge-swipes"), "Edge Swipes");

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

    // Recognition threshold — applies to both gestures and edge swipes.
    let settings_clone = settings.clone();
    page.add(&widgets::build_threshold_row(
        settings.borrow().recognition_threshold,
        move |val| {
            settings_clone.borrow_mut().recognition_threshold = val;
            save_and_reload(&settings_clone.borrow());
        },
    ));

    // Pinch detection settings
    {
        let pinch_group = adw::PreferencesGroup::builder()
            .title("Pinch Detection")
            .description("Controls how pinch-in/out gestures are recognized vs swipes")
            .build();

        // Pinch threshold (min spread change)
        let pinch_threshold_row = adw::SpinRow::builder()
            .title("Pinch Threshold")
            .subtitle("Minimum finger spread change in pixels to detect a pinch")
            .adjustment(&gtk::Adjustment::new(
                settings.borrow().pinch_threshold,
                5.0,
                100.0,
                1.0,
                5.0,
                0.0,
            ))
            .build();
        {
            let settings = settings.clone();
            pinch_threshold_row.connect_value_notify(move |row| {
                settings.borrow_mut().pinch_threshold = row.value();
                save_and_reload(&settings.borrow());
            });
        }
        pinch_group.add(&pinch_threshold_row);

        // Pinch ratio
        let pinch_ratio_row = adw::SpinRow::builder()
            .title("Pinch Ratio")
            .subtitle("Spread change must exceed swipe distance by this factor")
            .adjustment(&gtk::Adjustment::new(
                settings.borrow().pinch_ratio,
                1.0,
                5.0,
                0.1,
                0.5,
                0.0,
            ))
            .digits(1)
            .build();
        {
            let settings = settings.clone();
            pinch_ratio_row.connect_value_notify(move |row| {
                settings.borrow_mut().pinch_ratio = row.value();
                save_and_reload(&settings.borrow());
            });
        }
        pinch_group.add(&pinch_ratio_row);

        // Pinch sensitivity
        let pinch_sensitivity_row = adw::SpinRow::builder()
            .title("Pinch Sensitivity")
            .subtitle("How fast the overview animation tracks your pinch")
            .adjustment(&gtk::Adjustment::new(
                settings.borrow().pinch_sensitivity,
                0.001,
                0.1,
                0.001,
                0.01,
                0.0,
            ))
            .digits(3)
            .build();
        {
            let settings = settings.clone();
            pinch_sensitivity_row.connect_value_notify(move |row| {
                settings.borrow_mut().pinch_sensitivity = row.value();
                save_and_reload(&settings.borrow());
            });
        }
        pinch_group.add(&pinch_sensitivity_row);

        // Finger threshold scale
        let finger_scale_row = adw::SpinRow::builder()
            .title("Finger Threshold Scale")
            .subtitle("Multiplier per extra finger above 3 (e.g. 1.5 = 5 fingers need 2× threshold)")
            .adjustment(&gtk::Adjustment::new(
                settings.borrow().finger_threshold_scale,
                1.0,
                3.0,
                0.1,
                0.5,
                0.0,
            ))
            .digits(1)
            .build();
        {
            let settings = settings.clone();
            finger_scale_row.connect_value_notify(move |row| {
                settings.borrow_mut().finger_threshold_scale = row.value();
                save_and_reload(&settings.borrow());
            });
        }
        pinch_group.add(&finger_scale_row);

        page.add(&pinch_group);
    }

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
        save,
        ov_action,
        true,
    ));

    page
}

fn build_edge_swipes(settings: &Rc<RefCell<TouchscreenSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder().build();

    let info = adw::PreferencesGroup::builder()
        .description("Edge swipes trigger when swiping from a screen edge. Assign each edge an action or leave empty to disable.")
        .build();
    page.add(&info);

    // Edge threshold
    {
        let threshold_group = adw::PreferencesGroup::builder()
            .title("Edge Zone")
            .build();

        let threshold_row = adw::SpinRow::builder()
            .title("Edge Threshold")
            .subtitle("Pixels from screen edge that count as the edge zone")
            .adjustment(&gtk::Adjustment::new(
                settings.borrow().edge_threshold,
                5.0,
                100.0,
                1.0,
                5.0,
                0.0,
            ))
            .build();

        {
            let settings = settings.clone();
            threshold_row.connect_value_notify(move |row| {
                settings.borrow_mut().edge_threshold = row.value();
                save_and_reload(&settings.borrow());
            });
        }

        threshold_group.add(&threshold_row);
        page.add(&threshold_group);
    }

    // One group per edge
    let edges = [
        ("Left Edge", "edge_swipe_left", "Swipe right from the left edge"),
        ("Right Edge", "edge_swipe_right", "Swipe left from the right edge"),
        ("Top Edge", "edge_swipe_top", "Swipe down from the top edge"),
        ("Bottom Edge", "edge_swipe_bottom", "Swipe up from the bottom edge"),
    ];

    for (title, field, description) in edges {
        let edge_settings = Rc::new(RefCell::new(get_edge_field(&settings.borrow(), field)));
        let group = build_edge_swipe_group(
            title,
            description,
            &edge_settings,
            settings.clone(),
            field.to_string(),
        );
        page.add(&group);
    }

    page
}

fn get_edge_field(settings: &TouchscreenSettings, field: &str) -> EdgeSwipeSettings {
    match field {
        "edge_swipe_left" => settings.edge_swipe_left.clone(),
        "edge_swipe_right" => settings.edge_swipe_right.clone(),
        "edge_swipe_top" => settings.edge_swipe_top.clone(),
        "edge_swipe_bottom" => settings.edge_swipe_bottom.clone(),
        _ => EdgeSwipeSettings::default(),
    }
}

fn set_edge_field(settings: &mut TouchscreenSettings, field: &str, edge: &EdgeSwipeSettings) {
    match field {
        "edge_swipe_left" => settings.edge_swipe_left = edge.clone(),
        "edge_swipe_right" => settings.edge_swipe_right = edge.clone(),
        "edge_swipe_top" => settings.edge_swipe_top = edge.clone(),
        "edge_swipe_bottom" => settings.edge_swipe_bottom = edge.clone(),
        _ => {}
    }
}

fn build_edge_swipe_group(
    title: &str,
    description: &str,
    edge: &Rc<RefCell<EdgeSwipeSettings>>,
    settings: Rc<RefCell<TouchscreenSettings>>,
    field: String,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description(description)
        .build();

    let actions = ["", "view-scroll", "workspace-switch", "overview-toggle"];
    let labels = ["Disabled", "View Scroll", "Workspace Switch", "Overview Toggle"];

    let current_action = edge.borrow().action.clone();
    let active_idx = actions.iter().position(|a| *a == current_action).unwrap_or(0) as u32;

    let string_list = gtk::StringList::new(&labels);
    let action_row = adw::ComboRow::builder()
        .title("Action")
        .model(&string_list)
        .selected(active_idx)
        .build();

    {
        let edge = edge.clone();
        let settings = settings.clone();
        let field = field.clone();
        action_row.connect_selected_notify(move |row| {
            let idx = row.selected() as usize;
            let action = actions[idx].to_string();
            let mut e = edge.borrow_mut();
            e.action = action.clone();
            e.enabled = !action.is_empty();
            set_edge_field(&mut settings.borrow_mut(), &field, &e);
            save_and_reload(&settings.borrow());
        });
    }
    group.add(&action_row);

    // Sensitivity
    let sens_row = adw::SpinRow::builder()
        .title("Sensitivity")
        .subtitle("Speed multiplier for the gesture")
        .adjustment(&gtk::Adjustment::new(
            edge.borrow().sensitivity,
            0.1,
            5.0,
            0.1,
            0.5,
            0.0,
        ))
        .digits(1)
        .build();

    {
        let edge = edge.clone();
        let settings = settings.clone();
        let field = field.clone();
        sens_row.connect_value_notify(move |row| {
            let mut e = edge.borrow_mut();
            e.sensitivity = row.value();
            set_edge_field(&mut settings.borrow_mut(), &field, &e);
            save_and_reload(&settings.borrow());
        });
    }
    group.add(&sens_row);

    group
}

fn save_and_reload(settings: &TouchscreenSettings) {
    config::write_touchscreen_settings(settings);
    config::reload_config();
}

