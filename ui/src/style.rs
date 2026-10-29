//! A single, cross-platform style for the whole app.
//!
//! GTK3 widgets follow the system theme by default, so the same binary looks
//! different on GNOME, Windows and macOS. A CSS provider applied at
//! application priority overrides that: colours, borders, paddings and fonts
//! are fixed here, and the widgets are drawn from them. The canvas is drawn
//! entirely with cairo, so it is identical everywhere; only the chrome around
//! it needs this.

use gtk::prelude::*;

const CSS: &str = r#"
window {
    background: #f4f4f8;
    color: #202125;
}

menubar, menu {
    background: #eceef2;
    color: #202125;
    padding: 2px;
}

menuitem {
    padding: 4px 12px;
}

menuitem:hover {
    background: #2a6fd6;
    color: #ffffff;
}

/* ---------- Sidebar ---------- */
.tool-sidebar {
    background: #f8f9fb;
    border-right: 1px solid #d0d2d8;
}

.section-title {
    font-size: 11pt;
    font-weight: bold;
    color: #5a5e66;
    margin-top: 12px;
    margin-bottom: 4px;
    margin-left: 4px;
}

.tool-button, .action-button {
    background-image: none;
    background: transparent;
    border: none;
    border-radius: 6px;
    padding: 6px 12px;
    color: #202125;
    text-shadow: none;
    box-shadow: none;
    -gtk-icon-shadow: none;
    outline: none;
    transition: background 0.15s ease;
}

.tool-button:hover, .action-button:hover {
    background: #e4e6ec;
}

.tool-button:active, .tool-button:checked {
    background: #2a6fd6;
    color: #ffffff;
    border: none;
}

.tool-button:active label, .tool-button:checked label {
    color: #ffffff;
}

/* Regular buttons (for completeness, e.g. dialog buttons) */
button {
    background-image: none;
    background: #ffffff;
    border: 1px solid #c4c6ce;
    border-radius: 4px;
    padding: 4px 12px;
    color: #202125;
    text-shadow: none;
    box-shadow: none;
    -gtk-icon-shadow: none;
    outline: none;
}

button:hover {
    background: #e9ebf0;
}

button:active, button:checked {
    background: #2a6fd6;
    border-color: #245cbf;
    color: #ffffff;
}

label {
    color: #202125;
}

.truth-table {
    font-family: monospace;
    font-size: 11pt;
    color: #202125;
    background: #ffffff;
}

scrolledwindow {
    background: #ffffff;
}
"#;

pub fn apply() {
    let provider = gtk::CssProvider::new();
    provider
        .load_from_data(CSS.as_bytes())
        .expect("embedded CSS is valid");
    if let Some(screen) = gdk::Screen::default() {
        gtk::StyleContext::add_provider_for_screen(
            &screen,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
