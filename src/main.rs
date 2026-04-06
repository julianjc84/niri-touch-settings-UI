mod config;
mod pages;

use adw::prelude::*;

fn main() {
    std::env::set_var("GTK_A11Y", "none");
    gtk::glib::log_set_writer_func(|level, fields| match level {
        gtk::glib::LogLevel::Error | gtk::glib::LogLevel::Critical => {
            gtk::glib::log_writer_default(level, fields)
        }
        _ => gtk::glib::LogWriterOutput::Handled,
    });

    let app = adw::Application::builder()
        .application_id("com.github.niri-gesture-settings")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let style = adw::StyleManager::default();
    style.set_color_scheme(adw::ColorScheme::PreferDark);

    // Ensure config.kdl has include lines for both gesture files.
    config::ensure_includes();

    // Top-level device switcher: Touchscreen | Touchpad
    let view_stack = adw::ViewStack::new();
    view_stack.add_titled(&pages::touchscreen::build(), Some("touchscreen"), "Touchscreen");
    view_stack.add_titled(&pages::touchpad::build(), Some("touchpad"), "Touchpad");

    let switcher = adw::ViewSwitcher::new();
    switcher.set_stack(Some(&view_stack));
    switcher.set_policy(adw::ViewSwitcherPolicy::Wide);

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&switcher));

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&view_stack);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Niri Gesture Settings")
        .default_width(600)
        .default_height(700)
        .content(&content)
        .build();

    window.present();
}
