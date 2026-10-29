//! A single, cross-platform **dark** style for the whole app.
//!
//! Palette (per the brief): warm black / warm off-white / neutral browns,
//! with muted green and red accents. No rounded corners anywhere; buttons get
//! L-shaped corner brackets instead of a full border.

use gtk::prelude::*;

const CSS: &str = r#"
/* ---------- hard rule: nothing is round ---------- */
* {
    border-radius: 0;
}

window {
    background: #14120f;
    color: #e8e0d4;
}

/* ---------- Menu bar ---------- */
menubar {
    background: #1c1916;
    border-bottom: 1px solid #2e2820;
    padding: 2px;
}

menu {
    background: #1c1916;
    border: 1px solid #2e2820;
    padding: 4px;
}

menuitem {
    padding: 4px 12px;
    color: #e8e0d4;
    border-radius: 0;
}

menuitem:hover {
    background: #3a332b;
    color: #ffffff;
}

/* ---------- Sidebar ---------- */
.tool-sidebar {
    background: #1a1713;
    border-right: 1px solid #2e2820;
}

.section-title {
    font-size: 11pt;
    font-weight: bold;
    color: #a89880;
    margin-top: 12px;
    margin-bottom: 4px;
    margin-left: 4px;
}

/* ---------- Tool & action buttons: L-shaped corner brackets ---------- */
.tool-button, .action-button {
    background-color: transparent;
    background-image:
        /* top-left   horizontal + vertical */
        linear-gradient(to right,  #a89880 10px, transparent 10px),
        linear-gradient(to bottom, #a89880 10px, transparent 10px),
        /* top-right  horizontal + vertical */
        linear-gradient(to left,   #a89880 10px, transparent 10px),
        linear-gradient(to bottom, #a89880 10px, transparent 10px),
        /* bottom-left  horizontal + vertical */
        linear-gradient(to right,  #a89880 10px, transparent 10px),
        linear-gradient(to top,    #a89880 10px, transparent 10px),
        /* bottom-right horizontal + vertical */
        linear-gradient(to left,   #a89880 10px, transparent 10px),
        linear-gradient(to top,    #a89880 10px, transparent 10px);
    background-size:
        10px 2px, 2px 10px,
        10px 2px, 2px 10px,
        10px 2px, 2px 10px,
        10px 2px, 2px 10px;
    background-position:
        left top,     left top,
        right top,    right top,
        left bottom,  left bottom,
        right bottom, right bottom;
    background-repeat: no-repeat;

    border: none;
    padding: 8px 14px;
    color: #e8e0d4;
    text-shadow: none;
    box-shadow: none;
    -gtk-icon-shadow: none;
    outline: none;
}

.tool-button:hover, .action-button:hover {
    background-color: #262119;
    color: #ffffff;
}

.tool-button:active, .tool-button:checked {
    background-color: #2e2820;
    color: #ffffff;
}

/* Hide the little radio / check indicator inside tool buttons. */
.tool-button check, .tool-button radio {
    -gtk-icon-source: none;
    background: none;
    border: none;
    min-width: 0;
    min-height: 0;
    padding: 0;
    margin: 0;
}

/* ---------- Regular buttons (dialogs etc.) ---------- */
button {
    background-image: none;
    background: #1f1c18;
    border: none;
    border-radius: 0;
    padding: 4px 12px;
    color: #e8e0d4;
    text-shadow: none;
    box-shadow: none;
    -gtk-icon-shadow: none;
    outline: none;
}

button:hover {
    background: #2a251e;
}

button:active, button:checked {
    background: #3a332b;
    color: #ffffff;
}

/* ---------- Labels, scrolling, paned handle ---------- */
label {
    color: #e8e0d4;
}

scrolledwindow {
    background: transparent;
    border: none;
}

scrolledwindow viewport {
    background: transparent;
}

paned > separator {
    background: #2e2820;
    min-width: 1px;
    min-height: 1px;
}

/* ---------- Truth table window ---------- */
.truth-table {
    font-family: monospace;
    font-size: 11pt;
    color: #e8e0d4;
    background: #14120f;
}

/* ---------- Dialogs ---------- */
dialog, messagedialog {
    background: #1a1713;
    color: #e8e0d4;
}

dialog label, messagedialog label {
    color: #e8e0d4;
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
