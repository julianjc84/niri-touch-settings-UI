use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{self, TouchSettings};

pub fn build(settings: &Rc<RefCell<TouchSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .title("Advanced")
        .build();

    let group = adw::PreferencesGroup::builder()
        .title("Gesture Recognition")
        .description("Fine-tune how gestures are detected")
        .build();

    // Recognition threshold.
    let threshold_adj = gtk::Adjustment::new(
        settings.borrow().recognition_threshold,
        4.0,   // min
        100.0, // max
        1.0,   // step
        5.0,   // page increment
        0.0,   // page size
    );
    let threshold_row = adw::SpinRow::builder()
        .title("Recognition Threshold")
        .subtitle("Distance in pixels before gesture direction locks")
        .adjustment(&threshold_adj)
        .digits(1)
        .build();

    {
        let settings = settings.clone();
        threshold_row.connect_value_notify(move |row| {
            settings.borrow_mut().recognition_threshold = row.value();
            config::write_settings(&settings.borrow());
            config::reload_config();
        });
    }
    group.add(&threshold_row);

    page.add(&group);
    page
}
