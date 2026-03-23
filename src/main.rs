mod config;
mod pages;

use adw::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    // Suppress noisy GTK/Adwaita warnings that aren't actionable.
    std::env::set_var("GTK_A11Y", "none");
    gtk::glib::log_set_writer_func(|level, fields| {
            match level {
                gtk::glib::LogLevel::Error | gtk::glib::LogLevel::Critical => {
                    gtk::glib::log_writer_default(level, fields)
                }
                _ => gtk::glib::LogWriterOutput::Handled,
            }
        });

    let app = adw::Application::builder()
        .application_id("com.github.niri-touch-settings")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    // Use AdwStyleManager for dark theme instead of deprecated GtkSettings.
    let style = adw::StyleManager::default();
    style.set_color_scheme(adw::ColorScheme::PreferDark);

    // Ensure the main config.kdl includes our touch-gestures.kdl file.
    config::ensure_include();

    // Read settings from the separate touch-gestures.kdl file.
    let settings = Rc::new(RefCell::new(config::read_settings()));

    // Build pages.
    let general_page = pages::general::build(&settings);
    let gestures_page = pages::gestures::build(&settings);
    let advanced_page = pages::advanced::build(&settings);

    // Stack for page switching (no icons needed).
    let stack = gtk::Stack::new();
    stack.add_titled(&general_page, Some("general"), "General");
    stack.add_titled(&gestures_page, Some("gestures"), "Gestures");
    stack.add_titled(&advanced_page, Some("advanced"), "Advanced");

    let switcher = gtk::StackSwitcher::new();
    switcher.set_stack(Some(&stack));

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&switcher));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&stack);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Niri Touch Settings")
        .default_width(600)
        .default_height(700)
        .content(&content)
        .build();

    window.present();
}
