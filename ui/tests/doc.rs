//! Integration tests for the editable board. They drive `Doc` through its
//! public API only: `add`, `toggle`, `draw_wire`, `move_node`, `analyze`,
//! `node`, `count`, `wires`. No private items are touched.

use logic::{Gate, Kind, Value};
use ui::doc::{Doc, Id, Look};

/// A(0,0) -> and.in0, B(0,60) -> and.in1, AND at (100,20).
fn and_board() -> (Doc, Id, Id, Id) {
    let mut d = Doc::new();
    let a = d.add(Kind::Input, 0, 0); // out (50,30)
    let b = d.add(Kind::Input, 0, 60); // out (50,90)
    let g = d.add(Kind::Gate(Gate::And), 100, 20); // in (100,40) (100,60), out (160,50)
    d.draw_wire((50, 30), (80, 40), true);
    d.draw_wire((80, 40), (100, 40), true);
    d.draw_wire((50, 90), (80, 60), true);
    d.draw_wire((80, 60), (100, 60), true);
    (d, a, b, g)
}

#[test]
fn signal_flows_through_free_wires() {
    let (mut d, a, b, g) = and_board();
    assert_eq!(d.analyze().values[g], Value::Low);
    d.toggle(a);
    d.toggle(b);
    let an = d.analyze();
    assert_eq!(an.values[g], Value::High);
    assert_eq!(an.look(d.node(g).output()), Look::Value(Value::High));
}

#[test]
fn moving_along_the_wires_keeps_connections() {
    let (mut d, a, b, g) = and_board();
    d.toggle(a);
    d.toggle(b);
    d.move_node(g, 120, 20, true);
    assert_eq!(d.analyze().values[g], Value::High);
    d.move_node(g, 140, 20, false); // Shift: wires stay behind
    assert_eq!(d.analyze().values[g], Value::Unknown);
}

#[test]
fn two_outputs_on_one_net_conflict() {
    let mut d = Doc::new();
    d.add(Kind::Input, 0, 0);
    d.add(Kind::Input, 0, 60);
    d.draw_wire((50, 30), (50, 90), true);
    assert_eq!(d.analyze().look((50, 30)), Look::Conflict);
}

#[test]
fn a_dropped_pin_connects_to_a_wire_it_lands_on() {
    let mut d = Doc::new();
    let a = d.add(Kind::Input, 0, 0);
    d.toggle(a);
    d.draw_wire((50, 30), (150, 30), true);
    let n = d.add(Kind::Gate(Gate::Not), 100, 0); // input pin (100,30) is mid-wire
    assert_eq!(d.analyze().values[n], Value::Low);
}

#[test]
fn feedback_is_flagged() {
    let mut d = Doc::new();
    let n = d.add(Kind::Gate(Gate::Not), 0, 0); // in (0,30), out (60,30)
    d.draw_wire((60, 30), (80, 30), true);
    d.draw_wire((80, 30), (-20, 60), false);
    d.draw_wire((-20, 60), (-20, 30), true);
    d.draw_wire((-20, 30), (0, 30), true);
    let _ = n;
    assert_eq!(d.analyze().look((60, 30)), Look::Cycle);
}
