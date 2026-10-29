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
    use std::f64::consts::FRAC_PI_2;
    match gate {
        // ---- AND family ----
        Gate::And => {
            cr.move_to(x, y);
            cr.line_to(x + 30.0, y);
            cr.arc(x + 30.0, y + 30.0, 30.0, -FRAC_PI_2, FRAC_PI_2);
            cr.line_to(x, y + 60.0);
            cr.close_path();
        }
        Gate::Nand => {
            // AND body compressed to fit the bubble: the rectangular part
            // is narrowed to 20 px, but the semicircle keeps its original
            // radius of 30 so it still spans the full 60 px height.
            // Body reaches x + 50; the bubble sits at x + 55.
            cr.move_to(x, y);
            cr.line_to(x + 20.0, y);
            cr.arc(x + 20.0, y + 30.0, 30.0, -FRAC_PI_2, FRAC_PI_2);
            cr.line_to(x, y + 60.0);
            cr.close_path();
        }

        // ---- OR family ----
        Gate::Or => {
            cr.move_to(x, y);
            cr.curve_to(x + 28.0, y + 2.0, x + 48.0, y + 14.0, x + 60.0, y + 30.0);
            cr.curve_to(x + 48.0, y + 46.0, x + 28.0, y + 58.0, x, y + 60.0);
            cr.curve_to(x + 8.0, y + 40.0, x + 8.0, y + 20.0, x, y);
            cr.close_path();
        }
        Gate::Nor => {
            // OR body compressed to fit the bubble (reaches x + 50).
            cr.move_to(x, y);
            cr.curve_to(x + 23.0, y + 2.0, x + 40.0, y + 14.0, x + 50.0, y + 30.0);
            cr.curve_to(x + 40.0, y + 46.0, x + 23.0, y + 58.0, x, y + 60.0);
            cr.curve_to(x + 7.0, y + 40.0, x + 7.0, y + 20.0, x, y);
            cr.close_path();
        }

        // ---- NOT ----
        Gate::Not => {
            cr.move_to(x, y);
            cr.line_to(x + 50.0, y + 30.0);
            cr.line_to(x, y + 60.0);
            cr.close_path();
        }

        // ---- XOR family ----
        Gate::Xor => {
            // OR body shifted right so the extra back curve fits.
            cr.move_to(x + 8.0, y);
            cr.curve_to(x + 32.0, y + 2.0, x + 48.0, y + 14.0, x + 60.0, y + 30.0);
            cr.curve_to(x + 48.0, y + 46.0, x + 32.0, y + 58.0, x + 8.0, y + 60.0);
            cr.curve_to(x + 16.0, y + 40.0, x + 16.0, y + 20.0, x + 8.0, y);
            cr.close_path();
        }
        Gate::Xnor => {
            // XOR body compressed so the bubble fits (reaches x + 50).
            cr.move_to(x + 8.0, y);
            cr.curve_to(x + 28.0, y + 2.0, x + 42.0, y + 14.0, x + 50.0, y + 30.0);
            cr.curve_to(x + 42.0, y + 46.0, x + 28.0, y + 58.0, x + 8.0, y + 60.0);
            cr.curve_to(x + 15.0, y + 40.0, x + 15.0, y + 20.0, x + 8.0, y);
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

    // Output inversion bubble (NOT / NAND / NOR / XNOR).
    // All of them have a body that ends at x + 50, so the bubble at x + 55
    // is tangent to the body — consistent across the four inverting gates.
    match gate {
        Gate::Not | Gate::Nand | Gate::Nor | Gate::Xnor => {
            cr.arc(
                x + 55.0,
                y + 30.0,
                BUBBLE_RADIUS,
                0.0,
                2.0 * std::f64::consts::PI,
            );
            fill_and_stroke(cr);
        }
        _ => {}
    }

    // Detail strokes specific to OR-family and XOR-family gates.
    set_color(cr, INK);
    cr.set_line_width(2.0);
    match gate {
        Gate::Or | Gate::Nor => {
            // Short stubs so the pins at x = 0 reach the concave back.
            for py in [20.0, 40.0] {
                cr.move_to(x, y + py);
                cr.line_to(x + 5.0, y + py);
                let _ = cr.stroke();
            }
        }
        Gate::Xor | Gate::Xnor => {
            // Extra back curve (the defining mark of the XOR family).
            let bulge = if matches!(gate, Gate::Xor) { 6.0 } else { 5.0 };
            cr.move_to(x, y + 60.0);
            cr.curve_to(x + bulge, y + 40.0, x + bulge, y + 20.0, x, y);
            let _ = cr.stroke();
            // Stubs that cross the back curve and meet the body.
            for py in [20.0, 40.0] {
                cr.move_to(x, y + py);
                cr.line_to(x + 14.0, y + py);
                let _ = cr.stroke();
            }
        }
        _ => {}
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
