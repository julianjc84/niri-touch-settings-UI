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
    let ts_page = view_stack.add_titled(
        &pages::touchscreen::build(),
        Some("touchscreen"),
        "Touchscreen",
    );
    ts_page.set_icon_name(Some("input-touchscreen-symbolic"));
    let tp_page = view_stack.add_titled(&pages::touchpad::build(), Some("touchpad"), "Touchpad");
    tp_page.set_icon_name(Some("input-touchpad-symbolic"));

    let switcher = adw::ViewSwitcher::new();
    switcher.set_stack(Some(&view_stack));
    switcher.set_policy(adw::ViewSwitcherPolicy::Wide);

    let header = adw::HeaderBar::new();
    header.set_title_widget(Some(&switcher));

    // Dev/preview info banner showing file layout
    let info_bar = gtk::Box::new(gtk::Orientation::Vertical, 4);
    info_bar.set_margin_start(12);
    info_bar.set_margin_end(12);
    info_bar.set_margin_top(8);
    info_bar.set_margin_bottom(4);

    let info_label = gtk::Label::new(Some(
        "Development GUI — writes to KDL include files:\n\
         \n\
         Detection / Device → touchscreen-gestures.kdl / touchpad-gestures.kdl\n\
         \u{2003}Replaces input { touchscreen {} } or input { touchpad {} }\n\
         \n\
         Gesture Binds → same include files, binds {} block\n\
         \u{2003}Merges Touch*/Touchpad* binds into config.kdl binds {}",
    ));
    info_label.set_wrap(true);
    info_label.set_xalign(0.0);
    info_label.add_css_class("dim-label");
    info_label.add_css_class("caption");
    info_bar.append(&info_label);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.append(&header);
    content.append(&info_bar);
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
