use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::GestureAction;

/// Build a preferences group for a single gesture action.
/// `on_change` is called whenever any setting changes.
pub fn build_gesture_group(
    title: &str,
    description: &str,
    action: &GestureAction,
    on_change: Rc<dyn Fn()>,
    action_ref: Rc<RefCell<GestureAction>>,
    show_natural_scroll: bool,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title(title)
        .description(description)
        .build();

    // Enable/disable switch.
    let enable_row = adw::SwitchRow::builder()
        .title("Enabled")
        .subtitle("Normal input is not affected")
        .active(action.enabled)
        .build();

    {
        let action_ref = action_ref.clone();
        let on_change = on_change.clone();
        enable_row.connect_active_notify(move |row| {
            action_ref.borrow_mut().enabled = row.is_active();
            on_change();
        });
    }
    group.add(&enable_row);

    // Finger count.
    let finger_adj = gtk::Adjustment::new(
        action.finger_count as f64,
        3.0, 5.0, 1.0, 1.0, 0.0,
    );
    let finger_row = adw::SpinRow::builder()
        .title("Finger Count")
        .subtitle("Number of fingers to trigger gesture")
        .adjustment(&finger_adj)
        .build();

    {
        let action_ref = action_ref.clone();
        let on_change = on_change.clone();
        finger_row.connect_value_notify(move |row| {
            action_ref.borrow_mut().finger_count = row.value() as u8;
            on_change();
        });
    }
    group.add(&finger_row);

    // Sensitivity.
    let sens_adj = gtk::Adjustment::new(
        action.sensitivity,
        0.1, 5.0, 0.1, 0.5, 0.0,
    );
    let sens_row = adw::SpinRow::builder()
        .title("Sensitivity")
        .subtitle("Speed multiplier for this gesture")
        .adjustment(&sens_adj)
        .digits(1)
        .build();

    {
        let action_ref = action_ref.clone();
        let on_change = on_change.clone();
        sens_row.connect_value_notify(move |row| {
            action_ref.borrow_mut().sensitivity = row.value();
            on_change();
        });
    }
    group.add(&sens_row);

    if show_natural_scroll {
        let natural_row = adw::SwitchRow::builder()
            .title("Natural Scroll")
            .subtitle("Invert direction for this gesture")
            .active(action.natural_scroll)
            .build();

        {
            let action_ref = action_ref.clone();
            let on_change = on_change.clone();
            natural_row.connect_active_notify(move |row| {
                action_ref.borrow_mut().natural_scroll = row.is_active();
                on_change();
            });
        }
        group.add(&natural_row);
    }

    group
}

/// Build a recognition threshold row.
pub fn build_threshold_row(
    threshold: f64,
    on_change: impl Fn(f64) + 'static,
) -> adw::PreferencesGroup {
    let group = adw::PreferencesGroup::builder()
        .title("Gesture Recognition")
        .description("Fine-tune how gestures are detected")
        .build();

    let threshold_adj = gtk::Adjustment::new(
        threshold,
        4.0, 100.0, 1.0, 5.0, 0.0,
    );
    let threshold_row = adw::SpinRow::builder()
        .title("Recognition Threshold")
        .subtitle("Distance in pixels before gesture direction locks")
        .adjustment(&threshold_adj)
        .digits(1)
        .build();

    threshold_row.connect_value_notify(move |row| {
        on_change(row.value());
    });

    group.add(&threshold_row);
    group
}
