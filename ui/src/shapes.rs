//! The drawing of gate symbols and the dark-theme wire palette.
//!
//! Pin geometry lives in `layout.rs`; the numbers below (60x60 symbol, pins
//! on the 10 px grid) must agree with it.

use cairo::Context;
use logic::{Gate, Value};

pub const INPUT_RADIUS: f64 = 20.0;
pub const BUBBLE_RADIUS: f64 = 4.5;

pub const INPUT_CENTER: (f64, f64) = (30.0, 30.0);

// ---------- dark-theme palette ----------
// Only warm blacks / off-whites / neutral browns, plus muted green and red.

/// Symbol outlines and text – warm off-white.
pub const INK: (f64, f64, f64) = (0.90, 0.87, 0.82);
/// Background / gate fill – warm near-black.
pub const PAPER: (f64, f64, f64) = (0.09, 0.08, 0.07);
/// The dot grid – dark warm brown.
pub const GRID: (f64, f64, f64) = (0.22, 0.19, 0.16);
/// Selection highlight – light warm brown.
pub const SELECTED: (f64, f64, f64) = (0.72, 0.66, 0.55);
/// The little circle where a wire can be started – muted green.
pub const HINT: (f64, f64, f64) = (0.40, 0.65, 0.45);

// Logisim wire colours, dark-theme variants.
/// Warm grey/brown: the wire touches no component pin.
pub const NIL: (f64, f64, f64) = (0.42, 0.38, 0.33);
/// Dark muted green: 0.
pub const LOW: (f64, f64, f64) = (0.10, 0.28, 0.14);
/// Bright muted green: 1.
pub const HIGH: (f64, f64, f64) = (0.40, 0.72, 0.48);
/// Warm brown: carrying a value that is not known (floating).
pub const UNKNOWN: (f64, f64, f64) = (0.58, 0.42, 0.24);
/// Muted red: conflicting drivers, or a feedback cycle.
pub const ERROR: (f64, f64, f64) = (0.72, 0.30, 0.28);

pub fn color_of_value(v: Value) -> (f64, f64, f64) {
    match v {
        Value::High => HIGH,
        Value::Low => LOW,
        Value::Unknown => UNKNOWN,
    }
}

/// Text colour drawn on top of an input circle.
fn input_label_color(v: Value) -> (f64, f64, f64) {
    match v {
        // Light text on the dark-green "0" fill.
        Value::Low => (0.90, 0.87, 0.82),
        // Dark text on the brighter green "1" fill and the brown "unknown" fill.
        _ => (0.09, 0.08, 0.07),
    }
}

pub fn set_color(cr: &Context, c: (f64, f64, f64)) {
    cr.set_source_rgb(c.0, c.1, c.2);
}

pub fn centered_text(
    cr: &Context,
    cx: f64,
    cy: f64,
    font_size: f64,
    color: (f64, f64, f64),
    text: &str,
) {
    cr.select_font_face("sans", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(font_size);
    if let Ok(extents) = cr.text_extents(text) {
        set_color(cr, color);
        cr.move_to(cx - extents.width() / 2.0, cy + font_size / 3.0);
        let _ = cr.show_text(text);
        cr.new_path();
    }
}

fn path(cr: &Context, x: f64, y: f64, gate: Gate) {
    match gate {
        Gate::And => {
            cr.move_to(x, y);
            cr.line_to(x + 30.0, y);
            cr.arc(
                x + 30.0,
                y + 30.0,
                30.0,
                -std::f64::consts::FRAC_PI_2,
                std::f64::consts::FRAC_PI_2,
            );
            cr.line_to(x, y + 60.0);
            cr.close_path();
        }
        Gate::Or => {
            cr.move_to(x, y);
            cr.curve_to(x + 28.0, y + 2.0, x + 48.0, y + 14.0, x + 60.0, y + 30.0);
            cr.curve_to(x + 48.0, y + 46.0, x + 28.0, y + 58.0, x, y + 60.0);
            cr.curve_to(x + 8.0, y + 40.0, x + 8.0, y + 20.0, x, y);
            cr.close_path();
        }
        Gate::Not => {
            cr.move_to(x, y);
            cr.line_to(x + 50.0, y + 30.0);
            cr.line_to(x, y + 60.0);
            cr.close_path();
        }
    }
}

fn fill_and_stroke(cr: &Context) {
    set_color(cr, PAPER);
    let _ = cr.fill_preserve();
    set_color(cr, INK);
    cr.set_line_width(2.0);
    let _ = cr.stroke();
}

pub fn draw_gate(cr: &Context, x: f64, y: f64, gate: Gate) {
    path(cr, x, y, gate);
    fill_and_stroke(cr);
    match gate {
        Gate::Not => {
            cr.arc(x + 55.0, y + 30.0, BUBBLE_RADIUS, 0.0, 2.0 * std::f64::consts::PI);
            fill_and_stroke(cr);
        }
        Gate::Or => {
            // short stubs so the pins at x = 0 reach the concave back
            set_color(cr, INK);
            cr.set_line_width(2.0);
            for py in [20.0, 40.0] {
                cr.move_to(x, y + py);
                cr.line_to(x + 5.0, y + py);
                let _ = cr.stroke();
            }
        }
        Gate::And => {}
    }
    centered_text(
        cr,
        x + 30.0,
        y + 74.0,
        11.0,
        (0.55, 0.50, 0.44),
        gate.name(),
    );
}

pub fn draw_input(cr: &Context, x: f64, y: f64, label: &str, value: Value) {
    cr.arc(
        x + INPUT_CENTER.0,
        y + INPUT_CENTER.1,
        INPUT_RADIUS,
        0.0,
        2.0 * std::f64::consts::PI,
    );
    set_color(cr, color_of_value(value));
    let _ = cr.fill_preserve();
    set_color(cr, INK);
    cr.set_line_width(2.0);
    let _ = cr.stroke();
    centered_text(
        cr,
        x + INPUT_CENTER.0,
        y + INPUT_CENTER.1,
        14.0,
        input_label_color(value),
        label,
    );
}
