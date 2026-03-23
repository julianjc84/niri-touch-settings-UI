use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

use crate::config::{self, TouchSettings};

pub fn build(settings: &Rc<RefCell<TouchSettings>>) -> adw::PreferencesPage {
    let page = adw::PreferencesPage::builder()
        .title("General")
        .build();

    let group = adw::PreferencesGroup::builder()
        .title("Touch Input")
        .description("General touchscreen settings")
        .build();

    // Touch on/off (inverted: off=true means disabled).
    // This disables ALL touch at the libinput level — no taps, no gestures, nothing.
    let enable_row = adw::SwitchRow::builder()
        .title("Touchscreen")
        .subtitle("Disables all touch input, not just gestures")
        .active(!settings.borrow().off)
        .build();

    {
        let settings = settings.clone();
        enable_row.connect_active_notify(move |row| {
            settings.borrow_mut().off = !row.is_active();
            config::write_settings(&settings.borrow());
            config::reload_config();
        });
    }
    group.add(&enable_row);

    // Map to output.
    let output_row = adw::EntryRow::builder()
        .title("Map to Output")
        .text(settings.borrow().map_to_output.as_deref().unwrap_or(""))
        .build();

    {
        let settings = settings.clone();
        output_row.connect_changed(move |row| {
            let text = row.text().to_string();
            settings.borrow_mut().map_to_output = if text.is_empty() { None } else { Some(text) };
            config::write_settings(&settings.borrow());
            config::reload_config();
        });
    }
    group.add(&output_row);

    page.add(&group);
    page
}
