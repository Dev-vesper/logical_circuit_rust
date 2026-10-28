//! The geometry and drawing of gate symbols.

use cairo::Context;
use logic::{Gate, Kind, Value};

pub const CELL: f64 = 20.0;
pub const SIZE: f64 = 60.0;
pub const INPUT_RADIUS: f64 = 16.0;
pub const BUBBLE_RADIUS: f64 = 4.5;
pub const PORT_RADIUS: f64 = 4.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub const INPUT_CENTER: Point = Point { x: 30.0, y: 30.0 };

pub fn input_ports(kind: &Kind) -> Vec<Point> {
    match kind {
        Kind::Input => vec![],
        Kind::Gate(Gate::And) | Kind::Gate(Gate::Or) => vec![
            Point { x: 0.0, y: 15.0 },
            Point { x: 0.0, y: 45.0 },
        ],
        Kind::Gate(Gate::Not) => vec![Point { x: 0.0, y: 30.0 }],
    }
}

pub fn output_port(kind: &Kind) -> Point {
    match kind {
        Kind::Input => Point { x: 46.0, y: 30.0 },
        Kind::Gate(_) => Point { x: 60.0, y: 30.0 },
    }
}

pub fn snap(v: f64) -> f64 {
    (v / CELL).round() * CELL
}

pub fn route(sx: f64, sy: f64, dx: f64, dy: f64) -> Vec<Point> {
    if dx == sx || (dy == sy && dx > sx) {
        vec![]
    } else if dx >= sx + 2.0 * CELL {
        let mid = (sx + CELL).max((dx - CELL).min(snap((sx + dx) / 2.0)));
        vec![
            Point { x: mid, y: sy },
            Point { x: mid, y: dy },
        ]
    } else {
        vec![
            Point { x: dx - CELL, y: sy },
            Point { x: dx - CELL, y: dy },
        ]
    }
}

pub const INK: (f64, f64, f64) = (0.13, 0.15, 0.19);
pub const PAPER: (f64, f64, f64) = (1.0, 1.0, 1.0);
pub const GRID: (f64, f64, f64) = (0.84, 0.85, 0.88);
pub const SELECTED: (f64, f64, f64) = (0.16, 0.44, 0.86);
pub const HIGH: (f64, f64, f64) = (0.16, 0.62, 0.31);
pub const LOW: (f64, f64, f64) = (0.45, 0.47, 0.52);
pub const UNKNOWN: (f64, f64, f64) = (0.88, 0.72, 0.18);
pub const CYCLE: (f64, f64, f64) = (0.85, 0.24, 0.20);

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
    cr.select_font_face(
        "sans",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );
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
    cr.fill_preserve();
    set_color(cr, INK);
    cr.set_line_width(2.0);
    cr.stroke();
}

pub fn draw_gate(cr: &Context, x: f64, y: f64, gate: Gate) {
    path(cr, x, y, gate);
    fill_and_stroke(cr);
    if gate == Gate::Not {
        cr.arc(
            x + 55.0,
            y + 30.0,
            BUBBLE_RADIUS,
            0.0,
            2.0 * std::f64::consts::PI,
        );
        fill_and_stroke(cr);
    }
    centered_text(
        cr,
        x + 30.0,
        y + 74.0,
        11.0,
        (0.42, 0.45, 0.51),
        gate.name(),
    );
}

pub fn draw_input(cr: &Context, x: f64, y: f64, label: &str, value: Value) {
    cr.arc(
        x + INPUT_CENTER.x,
        y + INPUT_CENTER.y,
        INPUT_RADIUS,
        0.0,
        2.0 * std::f64::consts::PI,
    );
    set_color(cr, color_of_value(value));
    cr.fill_preserve();
    set_color(cr, INK);
    cr.set_line_width(2.0);
    cr.stroke();
    centered_text(
        cr,
        x + INPUT_CENTER.x,
        y + INPUT_CENTER.y,
        14.0,
        (1.0, 1.0, 1.0),
        label,
    );
}
