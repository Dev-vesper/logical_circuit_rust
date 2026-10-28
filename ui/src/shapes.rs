//! The drawing of gate symbols and the Logisim wire palette.
//!
//! Pin geometry lives in `layout.rs`; the numbers below (60x60 symbol, pins
//! on the 10 px grid) must agree with it.

use cairo::Context;
use logic::{Gate, Value};

pub const INPUT_RADIUS: f64 = 20.0;
pub const BUBBLE_RADIUS: f64 = 4.5;

pub const INPUT_CENTER: (f64, f64) = (30.0, 30.0);

pub const INK: (f64, f64, f64) = (0.13, 0.15, 0.19);
pub const PAPER: (f64, f64, f64) = (1.0, 1.0, 1.0);
pub const GRID: (f64, f64, f64) = (0.74, 0.76, 0.80);
pub const SELECTED: (f64, f64, f64) = (0.55, 0.70, 1.0);
/// The small circle drawn where a wire can be started.
pub const HINT: (f64, f64, f64) = (0.0, 0.70, 0.0);

// Logisim wire colours.
/// Gray: the wire touches no component pin.
pub const NIL: (f64, f64, f64) = (0.56, 0.56, 0.56);
/// Dark green: 0.
pub const LOW: (f64, f64, f64) = (0.0, 0.39, 0.0);
/// Bright green: 1.
pub const HIGH: (f64, f64, f64) = (0.0, 0.82, 0.0);
/// Blue: carrying a value that is not known (floating / not yet computed).
pub const UNKNOWN: (f64, f64, f64) = (0.16, 0.16, 1.0);
/// Red: conflicting drivers, or a feedback cycle.
pub const ERROR: (f64, f64, f64) = (0.75, 0.0, 0.0);

pub fn color_of_value(v: Value) -> (f64, f64, f64) {
    match v {
        Value::High => HIGH,
        Value::Low => LOW,
        Value::Unknown => UNKNOWN,
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
    centered_text(cr, x + 30.0, y + 74.0, 11.0, (0.42, 0.45, 0.51), gate.name());
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
    centered_text(cr, x + INPUT_CENTER.0, y + INPUT_CENTER.1, 14.0, (1.0, 1.0, 1.0), label);
}
