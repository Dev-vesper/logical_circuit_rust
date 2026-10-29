//! Pure geometry shared by the model and the renderer (no cairo / gtk here,
//! so the whole wiring model can be unit-tested headless).
//!
//! Like Logisim, every pin and every wire end lies on one integer grid.

use logic::{Gate, Kind};

/// Grid pitch in pixels (Logisim uses 10).
pub const CELL: i32 = 10;
/// Side of a gate / input symbol.
pub const SIZE: i32 = 60;

/// Input pin offsets from the node's top-left corner.
pub fn input_offsets(kind: &Kind) -> Vec<(i32, i32)> {
    match kind {
        Kind::Input => vec![],
        Kind::Gate(Gate::Not) => vec![(0, 30)],
        Kind::Gate(Gate::And)
        | Kind::Gate(Gate::Or)
        | Kind::Gate(Gate::Xor)
        | Kind::Gate(Gate::Nand)
        | Kind::Gate(Gate::Nor)
        | Kind::Gate(Gate::Xnor) => vec![(0, 20), (0, 40)],
    }
}

/// Output pin offset from the node's top-left corner.
pub fn output_offset(kind: &Kind) -> (i32, i32) {
    match kind {
        Kind::Input => (50, 30),
        Kind::Gate(_) => (60, 30),
    }
}

/// Nearest grid line.
pub fn snap_f(v: f64) -> i32 {
    ((v / CELL as f64).round() as i32) * CELL
}
